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

//! Constructor-derived state, later-entrypoint proof and strict application.
//! Constructor-state insertion and fee funding are privileged fixture setup.
use contract as generated;
use contract::{ledger_contract as c, runtime as r};
use midnight_base_crypto::data_provider::{FetchMode, MidnightDataProvider, OutputMode};
use midnight_compact_testkit::ProofLab;
use midnight_ledger::{
    construct::{ContractCallExt, ContractCallPrototype},
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
use midnight_serialize::{Deserializable, Tagged, tagged_deserialize, tagged_serialize};
use midnight_storage::arena::Sp;
use midnight_storage::storage::HashMap;
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    curve::Fr,
    fab::AlignedValueExt,
    hash::transient_commit,
    proofs::{
        KeyLocation, PARAMS_VERIFIER, Proof, ProofPreimage, ProverKey, ProvingKeyMaterial,
        Resolver, VerifierKey, Zkir,
    },
};
use midnight_zkir::IrSource;
use midnight_zswap::prove::ZswapResolver;
use r::{
    Field,
    context::{ConstructorContext, ConstructorResult},
    fab::AlignedValue,
    ledger::DefaultDB,
    transaction::{
        EncodedSizeLimit, Observation, ObservedCallError, ObservedContractState, PrepareCallError,
        RecordedCall,
    },
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{
    cell::RefCell,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};

const NAME: &str = "advance";
struct Artifacts(PathBuf);
impl Resolver for Artifacts {
    async fn resolve_key(&self, location: KeyLocation) -> io::Result<Option<ProvingKeyMaterial>> {
        if location.0.as_ref() != NAME {
            return Ok(None);
        }
        Ok(Some(ProvingKeyMaterial {
            prover_key: fs::read(self.0.join("keys/advance.prover"))?,
            verifier_key: fs::read(self.0.join("keys/advance.verifier"))?,
            ir_source: fs::read(self.0.join("zkir/advance.bzkir"))?,
        }))
    }
}
fn fee_resolver(root: &Path) -> Result<LedgerResolver, Box<dyn Error>> {
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
    let root = root.to_owned();
    Ok(LedgerResolver::new(
        zswap,
        dust,
        Box::new(move |location| {
            let a = Artifacts(root.clone());
            Box::pin(async move { a.resolve_key(location).await })
        }),
    ))
}
fn call_preimage(call: &ContractCallPrototype<DefaultDB>) -> Result<ProofPreimage, Box<dyn Error>> {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    let commitment = transient_commit(&fields[..], call.communication_commitment_rand);
    match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
        ProofPreimageVersioned::V2(value) => Ok(value.as_ref().clone()),
        _ => Err("unsupported proof version".into()),
    }
}
fn read_tagged<T: Deserializable + Tagged>(path: &Path) -> Result<T, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let mut reader = bytes.as_slice();
    let value = tagged_deserialize(&mut reader)?;
    if !reader.is_empty() {
        return Err(format!("trailing bytes in {}", path.display()).into());
    }
    Ok(value)
}

fn verify_standalone(root: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let verifier: VerifierKey = read_tagged(&root.join("keys/advance.verifier"))?;
    let other: VerifierKey = read_tagged(&root.join("keys/advance_other.verifier"))?;
    assert_ne!(verifier, other);
    let proof = Proof(fs::read(out.join("standalone.proof"))?);
    let inputs: Vec<Fr> = read_tagged(&out.join("standalone.inputs"))?;
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    assert!(
        other
            .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
            .is_err()
    );
    let mut corrupted = proof.clone();
    *corrupted.0.first_mut().ok_or("empty standalone proof")? ^= 1;
    assert!(
        verifier
            .verify(&PARAMS_VERIFIER, &corrupted, inputs.iter().copied())
            .is_err()
    );
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    Ok(())
}
fn assert_same_call(
    actual: &ContractCallPrototype<DefaultDB>,
    expected: &ContractCallPrototype<DefaultDB>,
) {
    // An exhaustive field pattern makes a new upstream field require review.
    let ContractCallPrototype {
        address,
        entry_point,
        op,
        guaranteed_public_transcript,
        fallible_public_transcript,
        private_transcript_outputs,
        input,
        output,
        communication_commitment_rand,
        key_location,
    } = actual;
    assert_eq!(address, &expected.address);
    assert_eq!(entry_point, &expected.entry_point);
    assert_eq!(op, &expected.op);
    assert_eq!(
        guaranteed_public_transcript,
        &expected.guaranteed_public_transcript
    );
    assert_eq!(
        fallible_public_transcript,
        &expected.fallible_public_transcript
    );
    assert_eq!(
        private_transcript_outputs,
        &expected.private_transcript_outputs
    );
    assert_eq!(input, &expected.input);
    assert_eq!(output, &expected.output);
    assert_eq!(
        communication_commitment_rand,
        &expected.communication_commitment_rand
    );
    assert_eq!(key_location, &expected.key_location);
}

pub fn inputs() -> r::FixedVector<Field, 2> {
    r::FixedVector([Field::from(2u64), Field::from(3u64)])
}
fn encode(value: Field) -> Value {
    let value = r::fab::AlignedValue::from(value);
    json!({"valueAtoms":value.value.0.iter().map(|x| &x.0).collect::<Vec<_>>(), "alignment":value.alignment})
}
pub struct Log {
    trace: RefCell<Vec<Value>>,
    failure: Option<String>,
}
impl Log {
    pub fn observe(&self, name: &str, private: u64, seen: Field) -> u64 {
        self.trace
            .borrow_mut()
            .push(json!({"name":name,"privateState":private,"seen":encode(seen)}));
        private + 1
    }
    pub fn accept(&self, name: &str) -> Result<(), r::CompactError> {
        if self.failure.as_deref() == Some(name) {
            Err(r::CompactError::AssertionFailed(
                "constructor-witness-stop".into(),
            ))
        } else {
            Ok(())
        }
    }
}
include!("constructor.rs");

fn check_program_bindings(root: &Path) {
    let mut assets = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        assert!(path.is_file() && path.extension().unwrap() == "json");
        let text = std::fs::read_to_string(path).unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        let name = value["root"].as_str().unwrap().to_owned();
        assert!(
            assets.insert(name, text).is_none(),
            "duplicate Program root"
        );
    }
    assert_eq!(assets.len(), 3);
    for (name, exported) in [
        ("read", c::functional::read::CHECKED_PROGRAM),
        ("advance", c::functional::advance::CHECKED_PROGRAM),
        (
            "advance_other",
            c::functional::advance_other::CHECKED_PROGRAM,
        ),
    ] {
        let value: Value = serde_json::from_str(exported).unwrap();
        assert_eq!(value["root"], name, "exported Program root");
        assert_eq!(value["purpose"], "review_only_not_execution_authority");
        assert_eq!(exported, assets[name], "exported Program asset bytes");
    }
}

fn initialized(case: &Value) -> Result<ConstructorResult<u64>, Box<dyn Error>> {
    let log = Log {
        trace: RefCell::new(Vec::new()),
        failure: None,
    };
    let initial = initialize(case, &log)?;
    check_extra(&initial);
    assert_eq!(
        initial.private_state,
        case["privateState"].as_u64().unwrap()
    );
    assert_eq!(
        generated::ledger_slots::stored.inspect(initial.ledger_state.get_ref())?,
        Field::from(case["expected"].as_u64().unwrap())
    );
    let trace = case["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            json!({
                "name":row["name"], "privateState":row["privateState"],
                "seen":encode(Field::from(row["seen"].as_u64().unwrap()))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(*log.trace.borrow(), trace);
    Ok(initial)
}

fn prove_case(root: &Path, out: &Path, case: &Value) -> Result<Value, Box<dyn Error>> {
    fs::create_dir(out)?;
    let verifier_bytes = fs::read(root.join("keys/advance.verifier"))?;
    let verifier: VerifierKey = read_tagged(&root.join("keys/advance.verifier"))?;
    let other_bytes = fs::read(root.join("keys/advance_other.verifier"))?;
    assert_ne!(verifier_bytes, other_bytes);
    let other: VerifierKey = read_tagged(&root.join("keys/advance_other.verifier"))?;
    assert_ne!(verifier, other);
    let ir = IrSource::load(fs::File::open(root.join("zkir/advance.zkir"))?)?;
    let model = ir.model();
    assert_eq!((model.k(), model.rows()), (7, 78));
    assert!(ir.do_communications_commitment);
    let binary_ir = fs::read(root.join("zkir/advance.bzkir"))?;
    let mut expected_ir = Vec::new();
    tagged_serialize(&ir, &mut expected_ir)?;
    assert_eq!(binary_ir, expected_ir);
    let prover: ProverKey<IrSource> = read_tagged(&root.join("keys/advance.prover"))?;
    let mut rng = StdRng::from_rng(rand::rngs::OsRng)?;
    let initial = initialized(case)?;
    let private = initial.private_state;
    let entry = EntryPointBuf(NAME.as_bytes().to_vec());
    let operation = ContractOperation::new(
        Some(verifier.clone()),
        Some(Sp::new(IrBuf::from(binary_ir))),
    );
    let operations = HashMap::new().insert(entry.clone(), operation.clone());
    let initial_contract = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        operations,
        ContractMaintenanceAuthority::default(),
    );
    let deploy = ContractDeploy::new(&mut rng, initial_contract);
    let address = deploy.address();
    // Privileged constructor-state insertion and upstream fee funding are
    // declared test setup. Neither is a constructor/deployment/wallet proof.
    std::env::var("MIDNIGHT_LEDGER_TEST_STATIC_DIR")?;
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
    let delta = Field::from(4_u64);
    let expected_value = Field::from(case["expected"].as_u64().unwrap() + 4);
    let communication_randomness: Fr = rand::rngs::OsRng.r#gen();
    let mut malformed = c::functional::advance::Program::new(delta)
        .execute_recorded(observed.circuit_context(private))?;
    malformed.execution.context.query.state = initial.ledger_state.clone();
    assert!(matches!(
        RecordedCall::new(&observed, malformed, NAME, delta)
            .prepare(verifier.clone(), communication_randomness),
        Err(ObservedCallError::Prepare(
            PrepareCallError::ReplayStateMismatch
        ))
    ));
    let recorded = c::functional::advance::Program::new(delta)
        .execute_recorded(observed.circuit_context(private))?;
    assert_eq!(recorded.execution.result, expected_value);
    assert_eq!(recorded.execution.context.private_state, private);
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    let expected = recorded.execution.context.query.state.get_ref().clone();
    assert_eq!(
        generated::ledger_slots::stored.inspect(&expected)?,
        expected_value
    );
    let prepared = RecordedCall::new(&observed, recorded, NAME, delta)
        .prepare(verifier.clone(), communication_randomness)?;
    let ordinary = c::recorded::Contract.advance_call(&observed, private, delta)?;
    assert_eq!(ordinary.recorded().execution.result, expected_value);
    let ordinary = ordinary.prepare(verifier.clone(), communication_randomness)?;
    assert_same_call(&prepared, &ordinary);
    assert_eq!(prepared.address, address);
    assert_eq!(prepared.entry_point, entry);
    assert_eq!(prepared.op, operation);
    let lab = ProofLab::new(
        Artifacts(root.to_owned()),
        MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?,
    );
    let resolved = futures_executor::block_on(lab.verifier(
        KeyLocation(NAME.into()),
        EncodedSizeLimit::new(verifier_bytes.len()),
    ))?;
    assert_eq!(resolved, verifier);
    let preimage = call_preimage(&prepared)?;
    let skips = ir.check(&preimage)?;
    assert!(
        skips.iter().all(Option::is_none),
        "no skipped public inputs admitted"
    );
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, proved_skips) =
        futures_executor::block_on(ir.prove(rand::rngs::OsRng, &params, prover, &preimage))?;
    assert_eq!(proved_skips, skips);
    assert_eq!(inputs.first(), Some(&preimage.binding_input));
    assert_eq!(
        inputs.get(1),
        Some(
            &preimage
                .communications_commitment
                .ok_or("missing commitment")?
                .0
        )
    );
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    let mut binding = inputs.clone();
    binding[0] = binding[0] + Fr::from(1_u64);
    assert!(
        verifier
            .verify(&PARAMS_VERIFIER, &proof, binding.into_iter())
            .is_err()
    );
    let mut communication_refusals = Vec::new();
    let mut preimage_refusals = Vec::new();
    for change_output in [false, true] {
        let mut changed = prepared.clone();
        let slot = if change_output {
            &mut changed.output
        } else {
            &mut changed.input
        };
        let replacement = AlignedValue::from(if change_output {
            Field::from(case["expected"].as_u64().unwrap() + 5)
        } else {
            Field::from(5_u64)
        });
        assert_eq!(slot.alignment, replacement.alignment);
        assert_eq!(slot.value.0.len(), replacement.value.0.len());
        assert_ne!(*slot, replacement);
        *slot = replacement;
        let bad = call_preimage(&changed)?;
        let mut changed_inputs = inputs.clone();
        changed_inputs[1] = bad
            .communications_commitment
            .ok_or("missing changed commitment")?
            .0;
        assert_ne!(inputs[1], changed_inputs[1]);
        let rejected = verifier
            .verify(&PARAMS_VERIFIER, &proof, changed_inputs.into_iter())
            .is_err();
        assert!(rejected);
        communication_refusals.push(rejected);
        // Unlike an unconstrained witness-return relation, advance computes
        // stored + delta: changing only its input or output must fail check.
        let rejected = ir.check(&bad).is_err();
        assert!(rejected);
        preimage_refusals.push(rejected);
    }
    let mut corrupted = proof.clone();
    assert!(!corrupted.0.is_empty());
    corrupted.0[0] ^= 1;
    assert_eq!(corrupted.0.len(), proof.0.len());
    assert!(
        verifier
            .verify(&PARAMS_VERIFIER, &corrupted, inputs.iter().copied())
            .is_err()
    );
    assert!(
        other
            .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
            .is_err()
    );
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    assert_eq!(
        fs::read(root.join("keys/advance.verifier"))?,
        verifier_bytes
    );
    assert_eq!(
        fs::read(root.join("keys/advance_other.verifier"))?,
        other_bytes
    );
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, state.time + state.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let resolver = fee_resolver(root)?;
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("constructor Vector v3 transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, call) = calls
        .next()
        .ok_or("missing funded constructor Vector call")?;
    let ProofVersioned::V3(transaction_proof) = &call.proof else {
        return Err("expected transaction-bound v3 proof".into());
    };
    assert!(calls.next().is_none());
    assert_ne!(transaction_proof, &proof);
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
    let mut context = state.context();
    context.block_context.last_block_time = state.time;
    let (updated, result) = state.ledger.apply(&verified, &context);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("missing applied contract")?;
    assert_eq!(actual.data.get_ref(), &expected);
    assert_eq!(actual.operations.get(&entry).as_deref(), Some(&operation));
    assert_eq!(
        generated::ledger_slots::stored.inspect(actual.data.get_ref())?,
        expected_value
    );
    context.ref_state = updated.clone();
    let (replayed, result) = updated.apply(&verified, &context);
    assert!(matches!(
        result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ));
    assert_eq!(replayed, updated);
    let mut updated_bytes = Vec::new();
    let mut replayed_bytes = Vec::new();
    tagged_serialize(&updated, &mut updated_bytes)?;
    tagged_serialize(&replayed, &mut replayed_bytes)?;
    assert_eq!(replayed_bytes, updated_bytes);
    let mut transaction = Vec::new();
    tagged_serialize(&sealed, &mut transaction)?;
    fs::write(out.join("standalone.proof"), &proof.0)?;
    let mut public_inputs = Vec::new();
    tagged_serialize(&inputs, &mut public_inputs)?;
    fs::write(out.join("standalone.inputs"), public_inputs)?;
    fs::write(out.join("transaction-bound.proof"), &transaction_proof.0)?;
    fs::write(out.join("applied-transaction.bin"), &transaction)?;
    fs::write(out.join("ledger-after.bin"), &updated_bytes)?;
    fs::write(out.join("ledger-replay.bin"), &replayed_bytes)?;
    let result = json!({"status":"passed","case":case["id"],"initial_stored":case["expected"],
        "delta":4,"final_stored":case["expected"].as_u64().unwrap()+4,
        "standalone_proof_bytes":proof.0.len(),"transaction_bytes":transaction.len(),
        "transaction_proof_version":3,"transaction_proof_bytes":transaction_proof.0.len(),
        "positive_fee_dust_spends":dust_spends,
        "proof_relationship":"separate standalone proof and independently proved funded transaction",
        "changed_binding_rejected":true,"changed_input_rejected":communication_refusals[0],
        "changed_output_rejected":communication_refusals[1],
        "changed_input_preimage_rejected":preimage_refusals[0],
        "changed_output_preimage_rejected":preimage_refusals[1],
        "corrupted_proof_rejected":true,"distinct_verifier_rejected":true,
        "preparation_refusal":"ReplayStateMismatch","strict_application":"default",
        "replay_refusal":"IntentAlreadyExists","replay_unchanged":true,
        "scope":"selected later entrypoint; privileged constructor state and fee funding, not constructor/deployment proof or wallet lifecycle"});
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&result)?)?;
    Ok(result)
}

fn run() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "constructor Vector requires local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    let mut args = std::env::args_os().skip(1);
    let mode = args.next().ok_or("mode required")?;
    if mode == "--verify-standalone" {
        let material = PathBuf::from(args.next().ok_or("material required")?);
        let output = PathBuf::from(args.next().ok_or("case directory required")?);
        if args.next().is_some() {
            return Err("unexpected verifier argument".into());
        }
        verify_standalone(&material, &output)?;
        println!(
            "{}",
            json!({"status":"passed","standalone_verified":true,
            "corrupted_proof_rejected":true,"distinct_verifier_rejected":true})
        );
        return Ok(());
    }
    if mode != "--preflight" && mode != "--prove" {
        return Err("unsupported mode".into());
    }
    let sdk = PathBuf::from(args.next().ok_or("SDK required")?);
    let role = args
        .next()
        .ok_or("role required")?
        .into_string()
        .map_err(|_| "invalid role")?;
    let cases = PathBuf::from(args.next().ok_or("cases required")?);
    let material = PathBuf::from(args.next().ok_or("material required")?);
    let output = PathBuf::from(args.next().ok_or("output required")?);
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    check_program_bindings(&sdk.join("contract/checked-programs"));
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(cases)?)?;
    let cases = cases
        .into_iter()
        .filter(|v| v["role"] == role && v["error"].is_null())
        .collect::<Vec<_>>();
    if cases.is_empty() {
        return Err("source requires an executing case".into());
    }
    let mut results = Vec::new();
    if mode == "--prove" {
        fs::create_dir(&output)?;
    }
    for case in cases {
        if mode == "--preflight" {
            let initial = initialized(&case)?;
            let out = c::functional::advance::Program::new(Field::from(4_u64))
                .execute_recorded(initial.into_circuit_context(Default::default()))?;
            assert_eq!(
                out.execution.result,
                Field::from(case["expected"].as_u64().unwrap() + 4)
            );
            results.push(json!({"case":case["id"],"native_preflight":true}));
        } else {
            results.push(prove_case(
                &material,
                &output.join(case["id"].as_str().unwrap()),
                &case,
            )?);
        }
    }
    println!(
        "{}",
        json!({"status":"passed","mode":mode.to_str(),"results":results})
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Match the maintained proof-smoke worker stack, without changing limits.
    std::thread::Builder::new()
        .name("constructor-vector-proof".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| run().map_err(|e| e.to_string()))?
        .join()
        .map_err(|_| "constructor proof worker panicked")?
        .map_err(Into::into)
}
