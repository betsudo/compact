// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use constant::{ledger_contract as contract, runtime as r};
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
    Curve25519Base, Curve25519Scalar, Secp256k1Base, Secp256k1Scalar, Secp256r1Base,
    Secp256r1Scalar,
    context::ConstructorContext,
    ledger::{ContractAddress, DefaultDB},
    public_state::PublicStateSource,
    transaction::{Observation, ObservedCallError, ObservedContractState},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json::json;
use std::{env, error::Error, fs, io, path::Path, sync::Arc};

#[derive(Clone, Copy)]
enum Case {
    K1Constant,
    K1Selected,
    R1Constant,
    R1Selected,
    Zero,
}

impl Case {
    fn id(self) -> &'static str {
        match self {
            Self::K1Constant => "castK1-false",
            Self::K1Selected => "castK1-true",
            Self::R1Constant => "castR1-false",
            Self::R1Selected => "castR1-true",
            Self::Zero => "castZero",
        }
    }

    fn circuit(self) -> &'static str {
        match self {
            Self::K1Constant | Self::K1Selected => "castK1",
            Self::R1Constant | Self::R1Selected => "castR1",
            Self::Zero => "castZero",
        }
    }
}

fn small(value: u8) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[0] = value;
    bytes
}

fn constant() -> [u8; 32] {
    let mut bytes = small(5);
    bytes[31] = 1;
    bytes
}

fn run_case(
    generated: &Path,
    capture: &Path,
    proofs: &Path,
    out: &Path,
    wrong_k11_path: &Path,
    wrong_k12_path: &Path,
    case: Case,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let name = case.circuit();
    let k1b = Secp256k1Base::from_le_bytes(small(7)).ok_or("invalid K1 base input")?;
    let k1s = Secp256k1Scalar::from_le_bytes(small(8)).ok_or("invalid K1 scalar input")?;
    let r1b = Secp256r1Base::from_le_bytes(small(7)).ok_or("invalid P-256 base input")?;
    let r1s = Secp256r1Scalar::from_le_bytes(small(8)).ok_or("invalid P-256 scalar input")?;
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let initial_state = initial.ledger_state.get_ref().clone();
    let context = initial.into_circuit_context(ContractAddress::default());
    let native = match case {
        Case::K1Constant => contract::castK1(context, false, k1b, k1s)?,
        Case::K1Selected => contract::castK1(context, true, k1b, k1s)?,
        Case::R1Constant => contract::castR1(context, false, r1b, r1s)?,
        Case::R1Selected => contract::castR1(context, true, r1b, r1s)?,
        Case::Zero => contract::castZero(context)?,
    };
    let expected_state = native.public_state();
    let view = contract::PublicStateView::from(&native);
    match case {
        Case::K1Constant => {
            assert_eq!(
                view.k1b()?,
                Secp256k1Base::from_le_bytes(constant()).ok_or("invalid K1 base literal")?
            );
            assert_eq!(
                view.k1s()?,
                Secp256k1Scalar::from_le_bytes(constant()).ok_or("invalid K1 scalar literal")?
            );
        }
        Case::K1Selected => {
            assert_eq!(view.k1b()?, k1b);
            assert_eq!(view.k1s()?, k1s);
        }
        Case::R1Constant => {
            assert_eq!(
                view.r1b()?,
                Secp256r1Base::from_le_bytes(constant()).ok_or("invalid P-256 base literal")?
            );
            assert_eq!(
                view.r1s()?,
                Secp256r1Scalar::from_le_bytes(constant()).ok_or("invalid P-256 scalar literal")?
            );
        }
        Case::R1Selected => {
            assert_eq!(view.r1b()?, r1b);
            assert_eq!(view.r1s()?, r1s);
        }
        Case::Zero => {
            assert_eq!(
                view.k1b()?,
                Secp256k1Base::from_le_bytes([0; 32]).ok_or("invalid K1 zero")?
            );
            assert_eq!(
                view.k1s()?,
                Secp256k1Scalar::from_le_bytes([0; 32]).ok_or("invalid K1 scalar zero")?
            );
            assert_eq!(
                view.r1b()?,
                Secp256r1Base::from_le_bytes([0; 32]).ok_or("invalid P-256 zero")?
            );
            assert_eq!(
                view.r1s()?,
                Secp256r1Scalar::from_le_bytes([0; 32]).ok_or("invalid P-256 scalar zero")?
            );
            assert_eq!(
                view.c25519b()?,
                Curve25519Base::from_le_bytes([0; 32]).ok_or("invalid Curve25519 zero")?
            );
            assert_eq!(
                view.c25519s()?,
                Curve25519Scalar::from_le_bytes([0; 32]).ok_or("invalid Curve25519 scalar zero")?
            );
        }
    }

    let proof_dir = proofs.join(case.id());
    let prover_bytes = fs::read(proof_dir.join("statement.prover"))?;
    let verifier_bytes = fs::read(proof_dir.join("statement.verifier"))?;
    let verifier: VerifierKey = tagged_deserialize(verifier_bytes.as_slice())?;
    let wrong_path = if matches!(case, Case::Zero) {
        wrong_k12_path
    } else {
        wrong_k11_path
    };
    let wrong: VerifierKey = tagged_deserialize(fs::File::open(wrong_path)?)?;
    assert_ne!(verifier, wrong);
    let standalone: Proof = tagged_deserialize(fs::File::open(proof_dir.join("statement.proof"))?)?;
    let raw = generated.join(format!("both/constant/zkir/{name}.zkir"));
    let ir = IrSource::load(fs::File::open(&raw)?)?;
    assert_eq!(ir.k(), if matches!(case, Case::Zero) { 12 } else { 11 });
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
    let contract = contract::Contract::default();
    let call = || match case {
        Case::K1Constant => contract
            .recording()
            .castK1_call(&observed, (), false, k1b, k1s),
        Case::K1Selected => contract
            .recording()
            .castK1_call(&observed, (), true, k1b, k1s),
        Case::R1Constant => contract
            .recording()
            .castR1_call(&observed, (), false, r1b, r1s),
        Case::R1Selected => contract
            .recording()
            .castR1_call(&observed, (), true, r1b, r1s),
        Case::Zero => contract.recording().castZero_call(&observed, ()),
    };
    let randomness: Fr = rand::rngs::OsRng.r#gen();
    assert!(matches!(call()?.prepare(wrong, randomness),
        Err(ObservedCallError::VerifierMismatch(ref entry)) if entry == name));
    let recorded = call()?;
    assert_eq!(recorded.recorded().execution.result, native.result);
    assert_eq!(
        recorded.recorded().execution.private_transcript_outputs,
        native.private_transcript_outputs
    );
    assert_eq!(recorded.recorded().execution.public_state(), expected_state);
    let prepared = recorded.prepare(verifier, randomness)?;
    assert_eq!(prepared.address, address);
    assert_eq!(prepared.entry_point, entry);
    assert_eq!(prepared.op, op);

    let typescript: ProofPreimage = tagged_deserialize(fs::File::open(
        capture.join(case.id()).join("honest.preimage"),
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
        .map_err(|error| format!("mixed-cast funded proving failed: {error:?}"))?;
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
    let out = out.join(case.id());
    fs::create_dir(&out)?;
    let (mut after, mut replay, mut tx_bytes) = (Vec::new(), Vec::new(), Vec::new());
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    tagged_serialize(&sealed, &mut tx_bytes)?;
    assert_eq!(after, replay);
    fs::write(out.join("applied-transaction.bin"), &tx_bytes)?;
    fs::write(out.join("transaction-bound.proof"), &transaction_proof.0)?;
    fs::write(out.join("ledger-after.bin"), &after)?;
    fs::write(out.join("ledger-replay.bin"), &replay)?;
    Ok(json!({
        "status":"passed","case":case.id(),"circuit":name,"ledger_matches_native":true,"k":ir.k(),
        "transaction_proof_bytes":transaction_proof.0.len(),
        "standalone_proof_bytes":standalone.0.len(),"transaction_bytes":tx_bytes.len(),
        "positive_fee_dust_spends":dust_spends,"default_strict":true,
        "typescript_preimage_fields_match":true,"wrong_verifier_refused":true,
        "replay_refusal":"IntentAlreadyExists","replay_unchanged":true
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let generated = args.next().ok_or("missing generated root")?;
    let capture = args.next().ok_or("missing capture root")?;
    let proofs = args.next().ok_or("missing proof root")?;
    let out = args.next().ok_or("missing output path")?;
    let wrong_k11 = args
        .next()
        .ok_or("missing independent k=11 wrong verifier")?;
    let wrong_k12 = args
        .next()
        .ok_or("missing independent k=12 wrong verifier")?;
    assert!(args.next().is_none());
    fs::create_dir(&out)?;
    let mut results = Vec::new();
    for case in [
        Case::K1Constant,
        Case::K1Selected,
        Case::R1Constant,
        Case::R1Selected,
        Case::Zero,
    ] {
        results.push(run_case(
            Path::new(&generated),
            Path::new(&capture),
            Path::new(&proofs),
            Path::new(&out),
            Path::new(&wrong_k11),
            Path::new(&wrong_k12),
            case,
        )?);
    }
    println!("{}", json!(results));
    Ok(())
}
