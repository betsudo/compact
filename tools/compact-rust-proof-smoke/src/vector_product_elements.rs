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

//! Complete Vector product elements helper SDK, five selected full-source proofs and separate strict transactions.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::vector_product_elements_fixture::{ledger_contract as c, ledger_slots as slots};
use crate::vector_product_elements_helper_fixture::{
    ledger_contract as h, ledger_slots as helper_slots,
};
use compact_rust_vector_product_elements_proof_checked::{
    ledger_contract as checked, ledger_slots as checked_slots,
};
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;
const IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/stateful_vector_product_elements/goldens/proof_candidate/source-ir.json"
);
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/proof_candidate/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/proof_candidate/checked-programs/program-1.json"
        ),
    ),
];

const HELPER_IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/stateful_vector_product_elements/goldens/helper/source-ir.json"
);
const HELPER_PROGRAMS: &[(&str, &[u8])] = &[(
    "program-0.json",
    include_bytes!(
        "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/helper/checked-programs/program-0.json"
    ),
)];
const CHECKED_IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/checked_helper/source-ir.json"
);
const CHECKED_PROGRAMS: &[(&str, &[u8])] = &[(
    "program-0.json",
    include_bytes!(
        "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/checked_helper/checked-programs/program-0.json"
    ),
)];
const ARTIFACT_PINS: [(&str, &str); 3] = [
    (
        "helper",
        include_str!(
            "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/helper/artifacts.json"
        ),
    ),
    (
        "proof_candidate",
        include_str!(
            "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/proof_candidate/artifacts.json"
        ),
    ),
    (
        "checked_helper",
        include_str!(
            "../../compact-rust-backend/tests/stateful_vector_product_elements/ledger9-goldens/checked_helper/artifacts.json"
        ),
    ),
];
const SELECTED: [(&str, &str); 4] = [
    ("helper", "run"),
    ("proof_candidate", "run_field"),
    ("proof_candidate", "run"),
    ("checked_helper", "run"),
];
fn selected(source: &str, circuit: &str) -> bool {
    SELECTED.contains(&(source, circuit))
}
type InputProducts = FixedVector<(FixedVector<BoundedUint<255>, 2>, BoundedUint<65535>), 2>;
type ResultProducts = FixedVector<(FixedVector<BoundedUint<65535>, 2>, BoundedUint<4294967295>), 2>;
fn input(values: [([u128; 2], u128); 2]) -> InputProducts {
    FixedVector::new(values.map(|(inner, scalar)| {
        (
            FixedVector::new(inner.map(|n| BoundedUint::new(n).unwrap())),
            BoundedUint::new(scalar).unwrap(),
        )
    }))
}
fn result_vector(values: [([u128; 2], u128); 2]) -> ResultProducts {
    FixedVector::new(values.map(|(inner, scalar)| {
        (
            FixedVector::new(inner.map(|n| BoundedUint::new(n).unwrap())),
            BoundedUint::new(scalar).unwrap(),
        )
    }))
}
fn field_result(values: [([u128; 2], u128); 2]) -> FixedVector<(FixedVector<Field, 2>, Field), 2> {
    FixedVector::new(values.map(|(inner, scalar)| {
        (
            FixedVector::new(inner.map(Field::from)),
            Field::from(scalar),
        )
    }))
}
fn validate_bindings(pool: &Path) -> Result<(), Box<dyn Error>> {
    for (source, ir, programs) in [
        ("proof_candidate", IR, PROGRAMS),
        ("helper", HELPER_IR, HELPER_PROGRAMS),
        ("checked_helper", CHECKED_IR, CHECKED_PROGRAMS),
    ] {
        let root = pool.join(source);
        if std::fs::read(root.join("contract/compact-rust-ir.json"))? != ir {
            return Err(
                format!("vector-product-elements {source} complete IR bytes differ").into(),
            );
        }
        let directory = root.join("contract/checked-programs");
        let mut names = std::fs::read_dir(&directory)?
            .map(|row| row.map(|row| row.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        names.sort();
        let mut expected = programs
            .iter()
            .map(|(name, _)| std::ffi::OsString::from(name))
            .collect::<Vec<_>>();
        expected.sort();
        if names != expected {
            return Err(format!(
                "vector-product-elements {source} complete Program membership differs"
            )
            .into());
        }
        for (name, bytes) in programs {
            if std::fs::read(directory.join(name))? != *bytes {
                return Err(format!(
                    "vector-product-elements {source} complete Program bytes differ"
                )
                .into());
            }
        }
    }
    Ok(())
}

/// Observe every complete corpus export, including unselected or over-cap models.
/// Python binds the entire producer directory before and after this observation.
pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    let definitions: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../compact-rust-backend/tests/stateful_vector_product_elements/sources.json"
    ))?;
    let mut rows = Vec::new();
    for source in definitions.as_array().ok_or("invalid source roster")? {
        let name = source["source"].as_str().ok_or("missing source name")?;
        let entries = source["entries"].as_array().ok_or("missing exports")?;
        let proof_required = source["stateful"]
            .as_bool()
            .ok_or("missing proof applicability")?;
        let capabilities: serde_json::Value = serde_json::from_slice(&std::fs::read(
            root.join(name).join("contract/rust-capabilities.json"),
        )?)?;
        let caps = capabilities["circuits"]
            .as_array()
            .ok_or("missing capabilities")?;
        if proof_required {
            if caps.len() != entries.len()
                || entries.iter().any(|entry| {
                    caps.iter()
                        .filter(|c| {
                            c["name"] == *entry
                                && c["proof_required"] == true
                                && c["recorded"] == true
                                && c["observed_call"] == true
                        })
                        .count()
                        != 1
                })
            {
                return Err("model applicability differs from emitted capabilities".into());
            }
        } else if !caps.is_empty() {
            return Err("pure source acquired stateful capability rows".into());
        }
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
            let ir = IrSourceV3::load(BufReader::new(File::open(
                root.join(name).join(format!("zkir/{entry}.zkir")),
            )?))?;
            rows.push(
                serde_json::json!({"source":name,"circuit":entry,"k":ir.k(),"proof_required":true,
                "selected_for_proof":selected(name,entry)}),
            );
        }
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-vector-product-elements-model-inventory/v2",
        "circuits":rows,"proof_execution":false,"scope":"complete model observation, no key generation or all-source proof acceptance"})
    );
    Ok(())
}

/// Inspect the selected complete circuits before key generation.
pub(super) fn models(pool: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(pool)?;
    let mut rows = Vec::new();
    for (source, name) in SELECTED {
        let ir = IrSourceV3::load(BufReader::new(File::open(
            pool.join(source).join(format!("zkir/{name}.zkir")),
        )?))?;
        rows.push(serde_json::json!({"source":source,"circuit":name,"k":ir.k(),"selected_for_proof":selected(source,name)}));
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-vector-product-elements-models/v3","circuits":rows,
        "selected_for_proof":SELECTED,"unselected":[],"proof_execution":false})
    );
    if rows
        .iter()
        .filter(|r| r["selected_for_proof"] == true)
        .any(|r| r["k"].as_u64().is_none_or(|k| k > 8))
    {
        return Err(
            "vector-product-elements model exceeds tiny automatic k<=8; full roster retained above"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    for (source, pins) in ARTIFACT_PINS {
        v3::validate_pinned_artifacts(&root.join(source), pins)?;
    }
    if root
        .join("vector-product-elements-proof-result.json")
        .exists()
    {
        return Err("Vector product elements proof already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (source, name, initial_seen, input_values, values, seen)) in [
        (
            "helper",
            "run",
            0_u64,
            [([3_u128, 5], 31), ([4, 6], 41)],
            [([3_u128, 5], 31), ([4, 6], 41)],
            1_u64,
        ),
        (
            "proof_candidate",
            "run_field",
            0,
            [([3, 5], 31), ([4, 6], 41)],
            [([3, 5], 31), ([4, 6], 41)],
            2,
        ),
        (
            "proof_candidate",
            "run",
            0,
            [([3, 5], 31), ([4, 6], 41)],
            [([3, 5], 31), ([4, 6], 41)],
            1,
        ),
        (
            "proof_candidate",
            "run",
            1,
            [([3, 5], 31), ([4, 6], 41)],
            [([4, 6], 41), ([3, 5], 31)],
            3,
        ),
        (
            "checked_helper",
            "run",
            0,
            [([3, 5], 31), ([4, 6], 41)],
            [([4, 6], 41), ([3, 5], 31)],
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut rng = StdRng::seed_from_u64(0x0180_0000 + index as u64);
        let pool = root;
        let source_root = pool.join(source);
        let root = source_root.as_path();
        let initial = match (source, name) {
            ("helper", "run") => h::initial_state(ConstructorContext::new(7_u64))?,
            ("proof_candidate", "run" | "run_field") => {
                c::initial_state(ConstructorContext::new(7_u64), Field::from(initial_seen))?
            }
            ("checked_helper", "run") => checked::initial_state(ConstructorContext::new(7_u64))?,
            _ => return Err("unknown selected source/circuit constructor".into()),
        };
        let actual_initial_seen = match (source, name) {
            ("helper", "run") => helper_slots::seen.inspect(initial.ledger_state.get_ref())?,
            ("proof_candidate", "run" | "run_field") => {
                slots::seen.inspect(initial.ledger_state.get_ref())?
            }
            ("checked_helper", "run") => {
                checked_slots::seen.inspect(initial.ledger_state.get_ref())?
            }
            _ => return Err("unknown selected source/circuit slot".into()),
        };
        if actual_initial_seen != Field::from(initial_seen) {
            return Err("source-specific initial seen value differs from scenario identity".into());
        }
        let (ir, verifier, operation) = v3::operation(root, name, 8)?;
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
            ($api:ident, $slots:ident, $entry:ident, $call:ident, $expected:expr, $aligned:expr; $($arg:expr),+) => {{
                let expected = $expected;
                let native = $api::$entry(observed.circuit_context(7), $($arg),+)?;
                let recorded = $api::recorded::$entry(observed.circuit_context(7), $($arg),+)?;
                let functional_native = $api::functional::$entry::Program::new($($arg),+)
                    .execute_native(observed.circuit_context(7))?;
                let functional = $api::functional::$entry::Program::new($($arg),+)
                    .execute_recorded(observed.circuit_context(7))?;
                for actual in [&native, &recorded.execution, &functional_native, &functional.execution] {
                    if actual.result != expected
                        || actual.context.private_state != 7
                        || actual.context.query.state != native.context.query.state
                        || actual.context.query.effects != native.context.query.effects
                        || actual.gas_cost != native.gas_cost
                        || !actual.private_transcript_outputs.is_empty()
                    {
                        return Err("Vector product elements four-route output/state/effects/private/gas mismatch".into());
                    }
                }
                if recorded.public.verify_ops() != functional.public.verify_ops()
                    || $slots::seen.inspect(native.context.query.state.get_ref())? != Field::from(seen)
                {
                    return Err("Vector product elements recording or seen Cell differs".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = positional_transaction::check_generated_trace(root, name, functional, $aligned)?;
                let facade = $api::Contract::default();
                let ordinary = facade.recording().$call(&observed, 7, $($arg),+)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err("Vector product elements observed call differs from functional preparation".into());
                }
                (prepared, state)
            }};
        }
        let (prepared, state) = match (source, name) {
            ("helper", "run") => {
                check_routes!(h, helper_slots, run, run_call, result_vector(values),
                AlignedValue::from(input(input_values)); input(input_values))
            }
            ("proof_candidate", "run_field") => {
                check_routes!(c, slots, run_field, run_field_call, field_result(values),
                AlignedValue::from(input(input_values)); input(input_values))
            }
            ("proof_candidate", "run") => {
                check_routes!(c, slots, run, run_call, result_vector(values),
                AlignedValue::from(input(input_values)); input(input_values))
            }
            ("checked_helper", "run") => {
                check_routes!(checked, checked_slots, run, run_call, result_vector(values),
                AlignedValue::from((true, result_vector(input_values))); true, result_vector(input_values))
            }
            _ => return Err("unknown selected export".into()),
        };
        let stem = format!("vector-product-elements-{index}");
        let reordered = [values[1], values[0]];
        let changed_output = if name == "run_field" {
            AlignedValue::from(field_result(reordered))
        } else {
            AlignedValue::from(result_vector(reordered))
        };
        let wrong_path = if source == "helper" {
            pool.join("proof_candidate/keys/run_field.verifier")
        } else {
            pool.join("helper/keys/run.verifier")
        };
        let proof = prove(
            root,
            name,
            &stem,
            &prepared,
            &ir,
            changed_output,
            &wrong_path,
        )?;
        positional_transaction::verify_separately(root, name, &stem)?;
        let transaction_proof_bytes =
            v3::apply(root, name, &stem, prepared, deploy, &state, &proof)?;
        let transaction_bytes =
            std::fs::metadata(root.join(format!("{stem}-transaction.bin")))?.len();
        rows.push(serde_json::json!({"source":source,"circuit":name,"initial_seen":initial_seen,"input":input_values,"guard":(source == "checked_helper").then_some(true),"expected":values,"seen":seen,
            "result_type":if name == "run" {"Vector<2,[Vector<2,Uint<16>>,Uint<32>]>"} else {"Vector<2,[Vector<2,Field>,Field]>"},"witness_count":0,"proof_bytes":proof.0.len(),"transaction_bytes":transaction_bytes,
            "transaction_proof_bytes":transaction_proof_bytes,"transaction_proof_version":3,"positive_fee_dust_spends":1,"standalone_verified_separately":true,
            "standalone_proof":format!("{source}/vector-product-elements-{index}.proof"),"applied_transaction":format!("{source}/vector-product-elements-{index}-transaction.bin"),
            "transaction_proof_relationship":"independently proved","corrupt_proof_rejected":true,"changed_binding_rejected":true,
            "changed_result_rejected":true,"changed_result_same_geometry":true,"wrong_verifier_rejected":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-vector-product-elements-proof/v3","status":"passed",
        "scope":"complete helpers, both conditional Uint branches and the Field export; fresh proofs and independently proved funded default-strict transactions",
        "constructor_execution_proved":false,"all_exports_proved":false,"all_source_exports_deployed":false,"selected_source_exports_proved":SELECTED,"unproved_selected_source_exports":[],"calls":rows});
    v3::write_new(
        root.join("vector-product-elements-proof-result.json"),
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
    ir: &IrSourceV3,
    changed_output: AlignedValue,
    wrong_path: &Path,
) -> Result<Proof, Box<dyn Error>> {
    let verifier = v3::read_verifier(root, name)?;
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected vector-product-elements proof version".into()),
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
            "vector-product-elements proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("vector-product-elements proof is empty".into());
    }
    let mut corrupt = proof.clone();
    corrupt.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupt, inputs.iter().copied())
        .is_ok()
    {
        return Err("vector-product-elements accepted corrupted proof bytes".into());
    }
    // A genuinely different source relation, not merely a second key for the same erased circuit.
    if std::fs::read(wrong_path)? == std::fs::read(root.join(format!("keys/{name}.verifier")))? {
        return Err("wrong-verifier control has identical key bytes".into());
    }
    let wrong: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(wrong_path)?))?;
    if wrong
        .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("vector-product-elements accepted wrong immutable verifier".into());
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
        return Err("vector-product-elements proof accepted changed public binding".into());
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
        return Err("changed vector-product-elements result geometry differs".into());
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
        return Err("vector-product-elements proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed vector-product-elements proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("vector-product-elements IR accepted changed result".into());
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
