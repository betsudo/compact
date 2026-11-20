// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use base::{ledger_contract as b, runtime as r};
use controls::ledger_contract as c;
use midnight_base_crypto::data_provider::{FetchMode, MidnightDataProvider, OutputMode};
use midnight_ledger::{
    construct::ContractCallExt,
    dust::{DUST_EXPECTED_FILES, DustResolver},
    prove::Resolver as LedgerResolver,
    semantics::TransactionResult,
    structure::{
        ContractDeploy, Intent, ProofPreimageMarker, ProofPreimageVersioned, ProofVersioned,
        Signature, Transaction,
    },
    test_utilities::{TestState, tx_prove},
    verify::WellFormedStrictness,
};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf, IrBuf,
};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_storage::{arena::Sp, storage::HashMap};
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    curve::Fr,
    proofs::{Proof, ProofPreimage, ProvingKeyMaterial, VerifierKey, Zkir},
};
use midnight_zkir::IrSource;
use midnight_zswap::prove::ZswapResolver;
use r::{
    CompactError, Secp256r1Base, Secp256r1Scalar,
    context::{CircuitResult, ConstructorContext},
    fab::AlignedValue,
    ledger::{ContractAddress, DefaultDB, StateValue},
    public_state::PublicStateSource,
    recording::RecordedCircuitResult,
    transaction::{Observation, ObservedCallError, ObservedContractState, RecordedCall},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json::{Value, json};
use std::{env, error::Error, fmt::Debug, fs, io, path::Path, sync::Arc};

type TestResult<T> = Result<T, Box<dyn Error>>;

fn check_recording<Output: PartialEq + Debug>(
    native: &CircuitResult<(), Output>,
    recorded: RecordedCircuitResult<(), Output>,
    witnesses: usize,
) -> TestResult<()> {
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.public_state(), recorded.public_state());
    assert_eq!(
        native.private_transcript_outputs,
        recorded.execution.private_transcript_outputs
    );
    assert_eq!(native.private_transcript_outputs.len(), witnesses);
    let (initial, program) = recorded.public.into_parts();
    assert!(!program.is_empty());
    let replay = initial.query(&program, None, &recorded.execution.context.cost_model)?;
    assert_eq!(replay.context.state.get_ref(), native.public_state());
    Ok(())
}

#[derive(Clone, Copy)]
struct Case {
    circuit: &'static str,
    family: &'static str,
    k: u8,
    index: usize,
    wrong_verifier: &'static str,
}

fn funded<Output, F>(
    root: &Path,
    out: &Path,
    case: Case,
    initial_state: StateValue<DefaultDB>,
    native: &CircuitResult<(), Output>,
    call: F,
) -> TestResult<()>
where
    Output: Into<AlignedValue> + PartialEq + Debug,
    F: for<'a> Fn(&'a ObservedContractState) -> Result<RecordedCall<'a, (), Output>, CompactError>,
{
    let name = case.circuit;
    let expected_state = native.public_state();
    let key_dir = root.join("slow-proof").join(name);
    let prover_bytes = fs::read(key_dir.join("statement.prover"))?;
    let verifier_bytes = fs::read(key_dir.join("statement.verifier"))?;
    let verifier: VerifierKey = tagged_deserialize(verifier_bytes.as_slice())?;
    let wrong: VerifierKey = tagged_deserialize(fs::File::open(
        root.join("slow-proof")
            .join(case.wrong_verifier)
            .join("statement.verifier"),
    )?)?;
    assert_ne!(verifier, wrong);
    let standalone: Proof = tagged_deserialize(fs::File::open(key_dir.join("statement.proof"))?)?;
    let raw = root
        .join("family-certified")
        .join(case.family)
        .join("ts-rust/.build/zkir")
        .join(format!("{name}.zkir"));
    let ir = IrSource::load(fs::File::open(&raw)?)?;
    assert_eq!(ir.k(), case.k);
    let mut binary = Vec::new();
    tagged_serialize(&ir, &mut binary)?;
    assert_eq!(binary, fs::read(raw.with_extension("bzkir"))?);
    let op = ContractOperation::new(
        Some(verifier.clone()),
        Some(Sp::new(IrBuf::from(binary.clone()))),
    );
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let initial_contract = ContractState::new(
        initial_state,
        HashMap::new().insert(entry.clone(), op.clone()),
        ContractMaintenanceAuthority::default(),
    );
    let mut rng = StdRng::from_rng(rand::rngs::OsRng)?;
    let deploy = ContractDeploy::new(&mut rng, initial_contract);
    let address = deploy.address();
    env::var("MIDNIGHT_LEDGER_TEST_STATIC_DIR")?;
    let mut state: TestState<DefaultDB> = TestState::new(&mut rng);
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(state.give_fee_token(&mut rng, 1));
    state.ledger.contract = state
        .ledger
        .contract
        .insert(address, deploy.initial_state.clone());
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state,
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let randomness: Fr = rand::rngs::OsRng.r#gen();
    assert!(matches!(call(&observed)?.prepare(wrong, randomness),
        Err(ObservedCallError::VerifierMismatch(ref entry)) if entry == name));
    let recorded = call(&observed)?;
    assert_eq!(recorded.recorded().execution.result, native.result);
    assert_eq!(
        recorded.recorded().execution.private_transcript_outputs,
        native.private_transcript_outputs
    );
    assert_eq!(
        recorded
            .recorded()
            .execution
            .private_transcript_outputs
            .len(),
        0
    );
    assert_eq!(recorded.recorded().execution.public_state(), expected_state);
    let prepared = recorded.prepare(verifier, randomness)?;
    assert_eq!(prepared.address, address);
    assert_eq!(prepared.entry_point, entry);
    assert_eq!(prepared.op, op);
    let typescript: ProofPreimage = tagged_deserialize(fs::File::open(
        root.join("preimages")
            .join(format!("{name}-{}.preimage", case.index)),
    )?)?;
    let commitment = typescript
        .communications_commitment
        .ok_or("missing commitment")?
        .0;
    let ProofPreimageVersioned::V2(rust) = ProofPreimage::construct_proof(&prepared, commitment)
    else {
        return Err("expected current ledger preimage".into());
    };
    assert_eq!(rust.inputs, typescript.inputs);
    assert_eq!(rust.private_transcript, typescript.private_transcript);
    assert_eq!(
        rust.public_transcript_inputs,
        typescript.public_transcript_inputs
    );
    assert_eq!(
        rust.public_transcript_outputs,
        typescript.public_transcript_outputs
    );
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, state.time + state.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1u16, intent));
    let materials = Arc::new((prover_bytes, verifier_bytes, binary));
    let zswap = ZswapResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        midnight_zswap::ZSWAP_EXPECTED_FILES.to_owned(),
    )?);
    let dust = DustResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        DUST_EXPECTED_FILES.to_owned(),
    )?);
    let resolver = LedgerResolver::new(
        zswap,
        dust,
        Box::new(move |location| {
            let materials = materials.clone();
            Box::pin(async move {
                Ok::<_, io::Error>((location.0.as_ref() == name).then(|| ProvingKeyMaterial {
                    prover_key: materials.0.clone(),
                    verifier_key: materials.1.clone(),
                    ir_source: materials.2.clone(),
                }))
            })
        }),
    );
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("secp256k1 funded transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, applied) = calls.next().ok_or("missing funded call")?;
    let ProofVersioned::V3(transaction_proof) = &applied.proof else {
        return Err("expected v3 proof".into());
    };
    assert!(calls.next().is_none());
    assert_ne!(transaction_proof, &standalone);
    let dust_spends = sealed
        .intents()
        .filter_map(|(_, intent)| intent.dust_actions)
        .map(|actions| {
            actions
                .spends
                .iter()
                .filter(|spend| !spend.proof.0.is_empty() && spend.v_fee > 0)
                .count()
        })
        .sum::<usize>();
    assert_eq!(dust_spends, 1);
    let verified =
        sealed.well_formed(&state.ledger, WellFormedStrictness::default(), state.time)?;
    let mut ctx = state.context();
    ctx.block_context.last_block_time = state.time;
    let (updated, result) = state.ledger.apply(&verified, &ctx);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("missing applied state")?;
    assert_eq!(actual.data.get_ref(), expected_state);
    assert_eq!(actual.operations.get(&entry).as_deref(), Some(&op));
    ctx.ref_state = updated.clone();
    let (replayed, result) = updated.apply(&verified, &ctx);
    assert!(
        matches!(
            result,
            TransactionResult::Failure(
                midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                    midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
                )
            )
        ),
        "{result:?}"
    );
    assert_eq!(replayed, updated);
    fs::create_dir(out)?;
    let (mut after, mut replay, mut tx_bytes) = (Vec::new(), Vec::new(), Vec::new());
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    tagged_serialize(&sealed, &mut tx_bytes)?;
    assert_eq!(after, replay);
    fs::write(out.join("applied-transaction.bin"), &tx_bytes)?;
    fs::write(out.join("transaction-bound.proof"), &transaction_proof.0)?;
    fs::write(out.join("ledger-after.bin"), &after)?;
    fs::write(out.join("ledger-replay.bin"), &replay)?;
    println!(
        "{}",
        json!({"status":"passed","circuit":name,
        "ledger_matches_native":true,"case_index":case.index,"k":ir.k(),
        "transaction_proof_bytes":transaction_proof.0.len(),"standalone_proof_bytes":standalone.0.len(),
        "transaction_bytes":tx_bytes.len(),"positive_fee_dust_spends":dust_spends,
        "default_strict":true,"typescript_preimage_fields_match":true,"wrong_verifier_refused":true,
        "replay_refusal":"IntentAlreadyExists","replay_unchanged":true})
    );
    Ok(())
}

fn input_bytes(value: &Value) -> [u8; 32] {
    hex::decode(value["raw"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap()
}

fn check_atoms<T: Into<AlignedValue>>(value: T, expected: &Value) {
    let aligned: AlignedValue = value.into();
    let actual: Vec<_> = aligned
        .value
        .0
        .into_iter()
        .map(|atom| hex::encode(atom.0))
        .collect();
    let expected: Vec<_> = expected["fab"]
        .as_array()
        .unwrap()
        .iter()
        .map(|atom| atom.as_str().unwrap())
        .collect();
    assert_eq!(actual, expected);
}

fn parity(vectors: &Value) -> TestResult<()> {
    let base_context = || {
        Ok::<_, CompactError>(
            b::initial_state(ConstructorContext::new(()))?
                .into_circuit_context(ContractAddress::default()),
        )
    };
    for row in vectors["secp256r1_base_roundtrip"].as_array().unwrap() {
        let input = Secp256r1Base::from_le_bytes(input_bytes(row)).unwrap();
        let native = b::secp256r1_base_roundtrip(base_context()?, input)?;
        assert_eq!(native.result, input);
        assert_eq!(b::PublicStateView::from(&native).stored()?, input);
        check_atoms(native.result, row);
        check_recording(
            &native,
            b::recorded::secp256r1_base_roundtrip(base_context()?, input)?,
            0,
        )?;
    }
    let controls_context = || {
        Ok::<_, CompactError>(
            c::initial_state(ConstructorContext::new(()))?
                .into_circuit_context(ContractAddress::default()),
        )
    };
    for row in vectors["scalarRoundtrip"].as_array().unwrap() {
        let input = Secp256r1Scalar::from_le_bytes(input_bytes(row)).unwrap();
        let native = c::scalarRoundtrip(controls_context()?, input)?;
        assert_eq!(native.result, input);
        assert_eq!(c::PublicStateView::from(&native).scalarCell()?, input);
        check_atoms(native.result, row);
        check_recording(
            &native,
            c::recorded::scalarRoundtrip(controls_context()?, input)?,
            0,
        )?;
    }
    let seed = || {
        let scalar =
            Secp256r1Scalar::from_le_bytes(input_bytes(&vectors["scalarRoundtrip"][4])).unwrap();
        Ok::<_, CompactError>(c::scalarRoundtrip(controls_context()?, scalar)?.context)
    };
    let native = c::resetFields(seed()?)?;
    assert_eq!(
        native.result,
        (Secp256r1Base::default(), Secp256r1Scalar::default())
    );
    let state = c::PublicStateView::from(&native);
    assert_eq!(state.baseCell()?, Secp256r1Base::default());
    assert_eq!(state.scalarCell()?, Secp256r1Scalar::default());
    check_recording(&native, c::recorded::resetFields(seed()?)?, 0)?;
    println!("6 base values, 5 scalar values, and one reset matched FAB and public VM replay");
    Ok(())
}

fn main() -> TestResult<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [mode, root, out] = args.as_slice() else {
        return Err("expected parity or exact circuit, evidence root, and output directory".into());
    };
    let root = Path::new(root);
    let out = Path::new(out);
    let vectors: Value =
        serde_json::from_slice(&fs::read(root.join("preimages/field-cases.json"))?)?;
    match mode.to_str() {
        Some("parity") => parity(&vectors),
        Some("secp256r1_base_roundtrip") => {
            let case = Case {
                circuit: "secp256r1_base_roundtrip",
                family: "base/roundtrip",
                k: 11,
                index: 3,
                wrong_verifier: "scalarRoundtrip",
            };
            let input =
                Secp256r1Base::from_le_bytes(input_bytes(&vectors[case.circuit][case.index]))
                    .unwrap();
            let init = || b::initial_state(ConstructorContext::new(()));
            let native = b::secp256r1_base_roundtrip(
                init()?.into_circuit_context(ContractAddress::default()),
                input,
            )?;
            assert_eq!(native.result, input);
            let contract = b::Contract::default();
            funded(
                root,
                out,
                case,
                init()?.ledger_state.get_ref().clone(),
                &native,
                |observed| {
                    contract
                        .recording()
                        .secp256r1_base_roundtrip_call(observed, (), input)
                },
            )
        }
        Some("scalarRoundtrip") => {
            let case = Case {
                circuit: "scalarRoundtrip",
                family: "controls",
                k: 11,
                index: 4,
                wrong_verifier: "resetFields",
            };
            let input =
                Secp256r1Scalar::from_le_bytes(input_bytes(&vectors[case.circuit][case.index]))
                    .unwrap();
            let init = || c::initial_state(ConstructorContext::new(()));
            let native = c::scalarRoundtrip(
                init()?.into_circuit_context(ContractAddress::default()),
                input,
            )?;
            assert_eq!(native.result, input);
            let contract = c::Contract::default();
            funded(
                root,
                out,
                case,
                init()?.ledger_state.get_ref().clone(),
                &native,
                |observed| {
                    contract
                        .recording()
                        .scalarRoundtrip_call(observed, (), input)
                },
            )
        }
        Some("resetFields") => {
            let case = Case {
                circuit: "resetFields",
                family: "controls",
                k: 12,
                index: 0,
                wrong_verifier: "secp256r1_base_roundtrip",
            };
            let init = || c::initial_state(ConstructorContext::new(()));
            let native = c::resetFields(init()?.into_circuit_context(ContractAddress::default()))?;
            assert_eq!(
                native.result,
                (Secp256r1Base::default(), Secp256r1Scalar::default())
            );
            let contract = c::Contract::default();
            funded(
                root,
                out,
                case,
                init()?.ledger_state.get_ref().clone(),
                &native,
                |observed| contract.recording().resetFields_call(observed, ()),
            )
        }
        _ => Err("unrecognized circuit".into()),
    }
}
