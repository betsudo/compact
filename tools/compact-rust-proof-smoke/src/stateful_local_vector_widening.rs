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

//! Complete Stateful local Vector helper SDK, three selected proofs and separate strict transactions.
use super::*;
use crate::stateful_local_vector_fixture::{ledger_contract as c, ledger_slots as slots};
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};
const IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/stateful_local_vector_widening/goldens/proofs/source-ir.json"
);
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/stateful_local_vector_widening/goldens/proofs/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/stateful_local_vector_widening/goldens/proofs/checked-programs/program-1.json"
        ),
    ),
];

fn input(values: [u128; 2]) -> FixedVector<BoundedUint<255>, 2> {
    FixedVector::new(values.map(|value| BoundedUint::new(value).unwrap()))
}
fn result_vector(values: [u128; 2]) -> FixedVector<BoundedUint<65535>, 2> {
    FixedVector::new(values.map(|x| BoundedUint::new(x).unwrap()))
}
fn field_result(values: [u128; 2]) -> FixedVector<Field, 2> {
    FixedVector::new(values.map(Field::from))
}
fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join("contract/compact-rust-ir.json"))? != IR {
        return Err("stateful-local-vector complete IR bytes differ".into());
    }
    let directory = root.join("contract/checked-programs");
    let mut names = std::fs::read_dir(&directory)?
        .map(|row| row.map(|row| row.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut expected = PROGRAMS
        .iter()
        .map(|(name, _)| std::ffi::OsString::from(name))
        .collect::<Vec<_>>();
    expected.sort();
    if names != expected {
        return Err("stateful-local-vector complete Program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(directory.join(name))? != *bytes {
            return Err("stateful-local-vector complete Program bytes differ".into());
        }
    }
    Ok(())
}

/// Observe every complete corpus export, including unselected or over-cap models.
/// Python binds the entire producer directory before and after this observation.
pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    let definitions: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../compact-rust-backend/tests/stateful_local_vector_widening/sources.json"
    ))?;
    let mut rows = Vec::new();
    for source in definitions.as_array().ok_or("invalid source roster")? {
        let name = source["source"].as_str().ok_or("missing source name")?;
        let entries = source["entries"].as_array().ok_or("missing exports")?;
        let proof_required = source["proof_required"]
            .as_bool()
            .ok_or("missing proof applicability")?;
        if !proof_required {
            if root.join(name).join("zkir").exists() {
                return Err("proof-not-applicable source acquired ZKIR".into());
            }
            for entry in entries {
                rows.push(serde_json::json!({"source":name,"circuit":entry,"k":null,
                    "proof_required":false,"selected_for_proof":false}));
            }
            continue;
        }
        let mut actual = std::fs::read_dir(root.join(name).join("zkir"))?
            .map(|row| row.map(|row| row.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        actual.sort();
        let mut expected = entries
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(|entry| std::ffi::OsString::from(format!("{entry}.zkir")))
                    .ok_or("invalid export name")
            })
            .collect::<Result<Vec<_>, _>>()?;
        expected.sort();
        if actual != expected {
            return Err("complete model membership differs".into());
        }
        for entry in entries {
            let entry = entry.as_str().ok_or("invalid export name")?;
            let ir = IrSource::load(BufReader::new(File::open(
                root.join(name).join(format!("zkir/{entry}.zkir")),
            )?))?;
            rows.push(
                serde_json::json!({"source":name,"circuit":entry,"k":ir.model().k(),"proof_required":true,
                "selected_for_proof":name == "proofs"}),
            );
        }
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-stateful-local-vector-model-inventory/v1",
        "circuits":rows,"scope":"complete model observation, no key generation or all-source proof acceptance"})
    );
    Ok(())
}

/// Validate exact bindings and inspect selected tiny models before key generation.
pub(super) fn models(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    let mut rows = Vec::new();
    for name in ["run", "run_field"] {
        let ir = IrSource::load(BufReader::new(File::open(
            root.join(format!("zkir/{name}.zkir")),
        )?))?;
        let k = ir.model().k();
        rows.push(serde_json::json!({"circuit":name,"k":k}));
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-stateful-local-vector-models/v1","circuits":rows,
            "selected_for_proof":["run","run_field"],"unproved":[]})
    );
    if rows
        .iter()
        .any(|row| row["k"].as_u64().is_none_or(|k| k > 8))
    {
        return Err(
            "stateful-local-vector model exceeds tiny automatic k<=8; full roster retained above"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    if root
        .join("stateful-local-vector-proof-result.json")
        .exists()
    {
        return Err("Stateful local Vector proof already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (name, initial_seen, input_values, values, seen)) in [
        ("run", 0_u64, [0_u128, 255], [0_u128, 255], 1_u64),
        ("run", 1, [0, 255], [255, 0], 3),
        ("run_field", 0, [0, 255], [0, 255], 2),
    ]
    .into_iter()
    .enumerate()
    {
        let mut rng = StdRng::seed_from_u64(0x0172_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64), Field::from(initial_seen))?;
        let deploy = make_deploy(root, name, initial.ledger_state.get_ref().clone(), &mut rng)?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        // Each complete generated API is exercised at its genuine Rust type.
        // Only the upstream prepared call is shared after typed comparisons.
        macro_rules! check_routes {
            ($entry:ident, $call:ident, $expected:expr) => {{
                let expected = $expected;
                let native = c::$entry(observed.circuit_context(7), input(input_values))?;
                let recorded = c::recorded::$entry(observed.circuit_context(7), input(input_values))?;
                let functional_native = c::functional::$entry::Program::new(input(input_values))
                    .execute_native(observed.circuit_context(7))?;
                let functional = c::functional::$entry::Program::new(input(input_values))
                    .execute_recorded(observed.circuit_context(7))?;
                for actual in [&native, &recorded.execution, &functional_native, &functional.execution] {
                    if actual.result != expected
                        || actual.context.private_state != 7
                        || actual.context.query.state != native.context.query.state
                        || actual.context.query.effects != native.context.query.effects
                        || actual.gas_cost != native.gas_cost
                        || !actual.private_transcript_outputs.is_empty()
                    {
                        return Err("Stateful local Vector four-route output/state/effects/private/gas mismatch".into());
                    }
                }
                if recorded.public.verify_ops() != functional.public.verify_ops()
                    || slots::seen.inspect(native.context.query.state.get_ref())? != Field::from(seen)
                {
                    return Err("Stateful local Vector recording or seen Cell differs".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = check_generated_trace(root, name, functional, AlignedValue::from(input(input_values)))?;
                let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
                    root.join(format!("keys/{name}.verifier")),
                )?))?;
                let facade = c::Contract::default();
                let ordinary = facade.recording().$call(&observed, 7, input(input_values))?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err("Stateful local Vector observed call differs from functional preparation".into());
                }
                (prepared, state, verifier)
            }};
        }
        let (prepared, state, verifier) = match name {
            "run" => check_routes!(run, run_call, result_vector(values)),
            "run_field" => check_routes!(run_field, run_field_call, field_result(values)),
            _ => return Err("unknown selected export".into()),
        };
        let proof_bytes = prove(root, name, index, &prepared, &verifier)?;
        let transaction_bytes = apply(root, name, index, prepared, deploy, &state, rng)?;
        rows.push(serde_json::json!({"circuit":name,"initial_seen":initial_seen,"input":input_values,"expected":values,"seen":seen,
            "result_type":if name == "run" {"Vector<2,Uint<16>>"} else {"Vector<2,Field>"},"witness_count":0,"proof_bytes":proof_bytes,"transaction_bytes":transaction_bytes,
            "standalone_proof":format!("stateful-local-vector-{index}.proof"),"applied_transaction":format!("stateful-local-vector-{index}-transaction.bin"),
            "transaction_proof_relationship":"independently proved","corrupt_proof_rejected":true,"changed_binding_rejected":true,
            "changed_result_rejected":true,"changed_result_same_geometry":true,"wrong_verifier_rejected":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-stateful-local-vector-proof/v1","status":"passed",
        "scope":"both local-binding branch outcomes and Field export, fresh proofs and independently proved funded default-strict transactions",
        "constructor_execution_proved":false,"all_exports_proved":false,"selected_source_exports_proved":["run","run_field"],"unproved_selected_source_exports":[],"calls":rows});
    write_new(
        root.join("stateful-local-vector-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
fn write_new(path: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)
}

fn commitment(call: &ContractCallPrototype<DefaultDB>) -> Fr {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    transient_commit(&fields[..], call.communication_commitment_rand)
}

fn prove(
    root: &Path,
    name: &str,
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<usize, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected stateful-local-vector proof version".into()),
        };
    let ir: IrSource = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("zkir/{name}.bzkir")),
    )?))?;
    let pk: ProverKey<IrSource> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
    )?))?;
    let checked = preimage.check(&ir)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, skips) = futures_executor::block_on(ir.prove(
        ChaCha20Rng::from_seed([0x74 + index as u8; 32]),
        &params,
        pk,
        &preimage,
    ))?;
    if skips != checked
        || inputs.first() != Some(&preimage.binding_input)
        || inputs.get(1) != Some(&commitment)
    {
        return Err(
            "stateful-local-vector proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("stateful-local-vector proof is empty".into());
    }
    let mut corrupt = proof.clone();
    corrupt.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupt, inputs.iter().copied())
        .is_ok()
    {
        return Err("stateful-local-vector accepted corrupted proof bytes".into());
    }
    // A genuinely different source relation, not merely a second key for the same erased circuit.
    let other = if name == "run" { "run_field" } else { "run" };
    let wrong_path = root.join(format!("keys/{other}.verifier"));
    if std::fs::read(&wrong_path)? == std::fs::read(root.join(format!("keys/{name}.verifier")))? {
        return Err("wrong-verifier control has identical key bytes".into());
    }
    let wrong: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(wrong_path)?))?;
    if wrong
        .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("stateful-local-vector accepted wrong immutable verifier".into());
    }
    let mut changed_binding = inputs.clone();
    changed_binding[0] = if inputs[0] == Fr::from(0_u64) {
        Fr::from(1_u64)
    } else {
        Fr::from(0_u64)
    };
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_binding.into_iter())
        .is_ok()
    {
        return Err("stateful-local-vector proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = match name {
        "run" => AlignedValue::from(result_vector([999, 998])),
        "run_field" => AlignedValue::from(field_result([999, 998])),
        _ => return Err("unknown selected result carrier".into()),
    };
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed stateful-local-vector result geometry differs".into());
    }
    let changed_commitment = self::commitment(&changed_call);
    if changed_commitment == commitment {
        return Err("changed result did not change commitment".into());
    }
    let mut changed_result = inputs.clone();
    changed_result[1] = changed_commitment;
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_result.into_iter())
        .is_ok()
    {
        return Err("stateful-local-vector proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed stateful-local-vector proof version".into()),
    };
    if changed_preimage.check(&ir).is_ok() {
        return Err("stateful-local-vector IR accepted changed result".into());
    }
    write_new(
        root.join(format!("stateful-local-vector-{index}.proof")),
        &proof.0,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(
        root.join(format!("stateful-local-vector-{index}.preimage")),
        &bytes,
    )?;
    println!(
        "stateful-local-vector {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof.0.len())
}

fn apply(
    root: &Path,
    name: &'static str,
    index: usize,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    expected: &StateValue<DefaultDB>,
    mut rng: StdRng,
) -> Result<usize, Box<dyn Error>> {
    let address = deploy.address();
    let mut funded = qualified_coin_funding::fee_funded_state(&mut rng)?;
    funded.ledger.contract = funded.ledger.contract.insert(address, deploy.initial_state);
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, Timestamp::from_secs(funded.time.to_secs() + 3600))
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let resolver = qualified_coin_funding::fee_resolver(root, name)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let provider = LocalProvingProvider {
        rng: StdRng::seed_from_u64(0x0172_5052 + index as u64),
        resolver: &resolver,
        params: &params,
    };
    let proven = futures_executor::block_on(
        tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
    )?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0172_5345 + index as u64));
    let sealed = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(funded.balance_tx(rng, sealed, &resolver))?;
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    let verified =
        sealed.well_formed(&funded.ledger, WellFormedStrictness::default(), funded.time)?;
    let context = |state: LedgerState<DefaultDB>| TransactionContext {
        ref_state: state,
        block_context: BlockContext {
            tblock: funded.time,
            last_block_time: funded.time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, result) = funded
        .ledger
        .apply(&verified, &context(funded.ledger.clone()));
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("stateful-local-vector strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("stateful-local-vector contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("stateful-local-vector applied state differs".into());
    }
    let (replayed, result) = updated.apply(&verified, &context(updated.clone()));
    if !matches!(
        result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != updated
    {
        return Err(format!(
            "stateful-local-vector replay refused incorrectly or mutated ledger: {result:?}"
        )
        .into());
    }
    write_new(
        root.join(format!("stateful-local-vector-{index}-transaction.bin")),
        &transaction_bytes,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(
        root.join(format!("stateful-local-vector-{index}-state.bin")),
        &bytes,
    )?;
    println!(
        "stateful-local-vector {index}: funded default-strict apply; replay refused without mutation"
    );
    Ok(transaction_bytes.len())
}
