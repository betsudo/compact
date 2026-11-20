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

//! Complete Aggregate literal projection helper SDK, two selected proofs and separate strict transactions.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::aggregate_literal_projection_fixture::{ledger_contract as c, ledger_slots as slots};
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;
const IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/aggregate_literal_projection_widening/goldens/helper/source-ir.json"
);
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/aggregate_literal_projection_widening/ledger9-goldens/helper/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/aggregate_literal_projection_widening/ledger9-goldens/helper/checked-programs/program-1.json"
        ),
    ),
];
const ARTIFACTS: &str = include_str!(
    "../../compact-rust-backend/tests/aggregate_literal_projection_widening/ledger9-goldens/helper/artifacts.json"
);

fn vector<const N: usize>(values: [u128; N]) -> FixedVector<BoundedUint<255>, N> {
    FixedVector::new(values.map(|x| BoundedUint::new(x).unwrap()))
}
fn result_vector(values: [u128; 2]) -> FixedVector<BoundedUint<65535>, 2> {
    FixedVector::new(values.map(|x| BoundedUint::new(x).unwrap()))
}
fn field_result(values: [u128; 2]) -> FixedVector<Field, 2> {
    FixedVector::new(values.map(Field::from))
}
fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join("contract/compact-rust-ir.json"))? != IR {
        return Err("aggregate-literal-projection complete IR bytes differ".into());
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
        return Err("aggregate-literal-projection complete Program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(directory.join(name))? != *bytes {
            return Err("aggregate-literal-projection complete Program bytes differ".into());
        }
    }
    Ok(())
}

/// Validate exact bindings and inspect selected tiny models before key generation.
pub(super) fn models(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    let mut rows = Vec::new();
    for name in ["run", "run_field"] {
        let ir = IrSourceV3::load(BufReader::new(File::open(
            root.join(format!("zkir/{name}.zkir")),
        )?))?;
        let k = ir.k();
        rows.push(serde_json::json!({"circuit":name,"k":k}));
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-aggregate-literal-projection-models/v1","circuits":rows,
            "selected_for_proof":["run","run_field"],"unproved":[]})
    );
    if rows
        .iter()
        .any(|row| row["k"].as_u64().is_none_or(|k| k > 8))
    {
        return Err(
            "aggregate-literal-projection model exceeds tiny automatic k<=8; full roster retained above"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    v3::validate_pinned_artifacts(root, ARTIFACTS)?;
    if root
        .join("aggregate-literal-projection-proof-result.json")
        .exists()
    {
        return Err("Aggregate literal projection proof already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (name, input_values, values, seen)) in [
        ("run", [0_u128, 255], [255_u128, 0], 1_u64),
        ("run_field", [0, 255], [255, 0], 2),
    ]
    .into_iter()
    .enumerate()
    {
        let mut rng = StdRng::seed_from_u64(0x0166_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64))?;
        let (ir, verifier, operation) = v3::operation(root, name, 7)?;
        let deploy = v3::deploy(
            initial.ledger_state.get_ref().clone(),
            name,
            operation,
            &mut rng,
        );
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
                let native = c::$entry(observed.circuit_context(7), vector(input_values))?;
                let recorded = c::recorded::$entry(observed.circuit_context(7), vector(input_values))?;
                let functional_native = c::functional::$entry::Program::new(vector(input_values))
                    .execute_native(observed.circuit_context(7))?;
                let functional = c::functional::$entry::Program::new(vector(input_values))
                    .execute_recorded(observed.circuit_context(7))?;
                for actual in [&native, &recorded.execution, &functional_native, &functional.execution] {
                    if actual.result != expected
                        || actual.context.private_state != 7
                        || actual.context.query.state != native.context.query.state
                        || actual.context.query.effects != native.context.query.effects
                        || actual.gas_cost != native.gas_cost
                        || !actual.private_transcript_outputs.is_empty()
                    {
                        return Err("Aggregate literal projection four-route output/state/effects/private/gas mismatch".into());
                    }
                }
                if recorded.public.verify_ops() != functional.public.verify_ops()
                    || slots::seen.inspect(native.context.query.state.get_ref())? != Field::from(seen)
                {
                    return Err("Aggregate literal projection recording or seen Cell differs".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = positional_transaction::check_generated_trace(root, name, functional, AlignedValue::from(vector(input_values)))?;
                let facade = c::Contract::default();
                let ordinary = facade.recording().$call(&observed, 7, vector(input_values))?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err("Aggregate literal projection observed call differs from functional preparation".into());
                }
                (prepared, state)
            }};
        }
        let (prepared, state) = match name {
            "run" => check_routes!(run, run_call, result_vector(values)),
            "run_field" => check_routes!(run_field, run_field_call, field_result(values)),
            _ => return Err("unknown selected helper".into()),
        };
        let stem = format!("aggregate-literal-projection-{index}");
        let changed_values = values.map(|value| if value == 255 { 254 } else { value });
        let changed_output = if name == "run" {
            AlignedValue::from(result_vector(changed_values))
        } else {
            AlignedValue::from(field_result(changed_values))
        };
        let proof = prove(root, name, &stem, &prepared, &verifier, &ir, changed_output)?;
        positional_transaction::verify_separately(root, name, &stem)?;
        let transaction_proof_bytes =
            v3::apply(root, name, &stem, prepared, deploy, &state, &proof)?;
        let transaction_bytes =
            std::fs::metadata(root.join(format!("{stem}-transaction.bin")))?.len();
        rows.push(serde_json::json!({"circuit":name,"input":input_values,"expected":values,"seen":seen,
            "result_type":if name == "run" {"Vector<2,Uint<16>>"} else {"Vector<2,Field>"},"witness_count":0,"proof_bytes":proof.0.len(),"transaction_bytes":transaction_bytes,
            "transaction_proof_bytes":transaction_proof_bytes,"transaction_proof_version":3,"positive_fee_dust_spends":1,"standalone_verified_separately":true,
            "standalone_proof":format!("aggregate-literal-projection-{index}.proof"),"applied_transaction":format!("aggregate-literal-projection-{index}-transaction.bin"),
            "transaction_proof_relationship":"independently proved","corrupt_proof_rejected":true,"changed_binding_rejected":true,
            "changed_result_rejected":true,"changed_result_same_geometry":true,"wrong_verifier_rejected":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-aggregate-literal-projection-proof/v2","status":"passed",
        "scope":"two selected Aggregate literal projection helper exports, fresh proofs and independently proved funded default-strict transactions",
        "constructor_execution_proved":false,"all_exports_proved":false,"helper_exports_proved":["run","run_field"],"unproved_helper_exports":[],"calls":rows});
    v3::write_new(
        root.join("aggregate-literal-projection-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
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
    stem: &str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    ir: &IrSourceV3,
    changed_output: AlignedValue,
) -> Result<Proof, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected aggregate-literal-projection proof version".into()),
        };
    let pk: ProverKey<IrSourceV3> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
    )?))?;
    let checked = preimage.check(ir)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, skips) = futures_executor::block_on(ir.prove(
        ChaCha20Rng::from_rng(rand::rngs::OsRng)?,
        &params,
        pk,
        &preimage,
    ))?;
    if skips != checked
        || inputs.first() != Some(&preimage.binding_input)
        || inputs.get(1) != Some(&commitment)
    {
        return Err(
            "aggregate-literal-projection proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("aggregate-literal-projection proof is empty".into());
    }
    let mut corrupt = proof.clone();
    corrupt.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupt, inputs.iter().copied())
        .is_ok()
    {
        return Err("aggregate-literal-projection accepted corrupted proof bytes".into());
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
        return Err("aggregate-literal-projection accepted wrong immutable verifier".into());
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
        return Err("aggregate-literal-projection proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = changed_output;
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
        || changed_call
            .output
            .value
            .0
            .iter()
            .zip(&call.output.value.0)
            .any(|(left, right)| left.0.len() != right.0.len())
    {
        return Err("changed aggregate-literal-projection result geometry differs".into());
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
        return Err("aggregate-literal-projection proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed aggregate-literal-projection proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("aggregate-literal-projection IR accepted changed result".into());
    }
    v3::write_new(root.join(format!("{stem}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    v3::write_new(root.join(format!("{stem}.preimage")), &bytes)?;
    bytes.clear();
    tagged_serialize(&inputs, &mut bytes)?;
    v3::write_new(root.join(format!("{stem}.inputs")), &bytes)?;
    println!(
        "{stem}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof)
}
