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

//! Complete nested-product SDKs, each compiled at its own crate root.
//! Model observation generates no keys, proofs or funded transactions.
use super::{local_v3_application as v3, *};
use compact_rust_vector_component_products_inner_product_single_sdk as single;
use compact_rust_vector_component_products_witness_after_sdk as after;
use compact_rust_vector_component_products_witness_before_sdk as before;
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;

const SELECTED: [(&str, u8); 3] = [
    ("witness_before", 8),
    ("witness_after", 8),
    ("inner_product_single", 8),
];

fn selected(source: &str, circuit: &str) -> bool {
    circuit == "run" && SELECTED.iter().any(|(name, _)| *name == source)
}
/// Observe every complete corpus export, including unselected or over-cap models.
/// Python binds the entire producer directory before and after this observation.
pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    let definitions: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../compact-rust-backend/tests/vector_component_products/sources.json"
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
                rows.push(
                    serde_json::json!({"source":name,"circuit":entry,"k":null,"rows":null,
                    "proof_required":false,"selected_for_proof":false}),
                );
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
                serde_json::json!({"source":name,"circuit":entry,"k":ir.model().k(),"rows":ir.model().rows(),"proof_required":true,
                "selected_for_proof":selected(name,entry)}),
            );
        }
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-vector-component-products-model-inventory/v2",
        "circuits":rows,"proof_execution":false,"scope":"complete model observation, no key generation or all-source proof acceptance"})
    );
    Ok(())
}

const BINDINGS: &[(&str, &[u8], &[u8])] = &[
    (
        "witness_before",
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/proof-goldens/witness_before/source-ir.json"
        ),
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/ledger9-proof-goldens/witness_before/checked-programs/program-0.json"
        ),
    ),
    (
        "witness_after",
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/proof-goldens/witness_after/source-ir.json"
        ),
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/ledger9-proof-goldens/witness_after/checked-programs/program-0.json"
        ),
    ),
    (
        "inner_product_single",
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/proof-goldens/inner_product_single/source-ir.json"
        ),
        include_bytes!(
            "../../compact-rust-backend/tests/vector_component_products/ledger9-proof-goldens/inner_product_single/checked-programs/program-0.json"
        ),
    ),
];
fn validate_bindings(pool: &Path) -> Result<(), Box<dyn Error>> {
    for (source, ir, program) in BINDINGS {
        let root = pool.join(source).join("contract");
        if std::fs::read(root.join("compact-rust-ir.json"))? != *ir {
            return Err(
                format!("vector-component-products {source} complete IR bytes differ").into(),
            );
        }
        let directory = root.join("checked-programs");
        let names = std::fs::read_dir(&directory)?
            .map(|row| row.map(|r| r.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        if names != [std::ffi::OsString::from("program-0.json")] {
            return Err(format!(
                "vector-component-products {source} complete Program membership differs"
            )
            .into());
        }
        if std::fs::read(directory.join("program-0.json"))? != *program {
            return Err(format!(
                "vector-component-products {source} complete Program bytes differ"
            )
            .into());
        }
    }
    Ok(())
}
pub(super) fn models(pool: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(pool)?;
    let mut rows = Vec::new();
    for (name, _, _) in BINDINGS {
        let ir = IrSourceV3::load(BufReader::new(File::open(
            pool.join(name).join("zkir/run.zkir"),
        )?))?;
        rows.push(serde_json::json!({"source":name,"circuit":"run","k":ir.model().k(),"rows":ir.model().rows(),"selected_for_proof":true}));
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-vector-component-products-models/v2","circuits":rows,"proof_execution":false})
    );
    if rows.iter().any(|r| r["k"].as_u64().is_none_or(|k| k > 8)) {
        return Err("vector-component-products model exceeds tiny automatic k<=8".into());
    }
    Ok(())
}
type Narrow = FixedVector<
    (
        FixedVector<(BoundedUint<255>, BoundedUint<255>), 2>,
        BoundedUint<65535>,
    ),
    2,
>;
type Wide = FixedVector<
    (
        FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
        BoundedUint<4294967295>,
    ),
    2,
>;
type SingleInput = FixedVector<(FixedVector<(BoundedUint<255>,), 2>, BoundedUint<255>), 2>;
type SingleOutput = FixedVector<(FixedVector<(BoundedUint<65535>,), 2>, BoundedUint<65535>), 2>;
fn u<const M: u128>(n: u128) -> BoundedUint<M> {
    BoundedUint::new(n).unwrap()
}
fn narrow() -> Narrow {
    FixedVector::new([
        (FixedVector::new([(u(3), u(20)), (u(5), u(22))]), u(31)),
        (FixedVector::new([(u(4), u(21)), (u(6), u(23))]), u(41)),
    ])
}
fn wide() -> Wide {
    FixedVector::new([
        (FixedVector::new([(u(3), u(20)), (u(5), u(22))]), u(31)),
        (FixedVector::new([(u(4), u(21)), (u(6), u(23))]), u(41)),
    ])
}
fn single_input() -> SingleInput {
    FixedVector::new([
        (FixedVector::new([(u(3),), (u(5),)]), u(7)),
        (FixedVector::new([(u(4),), (u(6),)]), u(8)),
    ])
}
fn single_output() -> SingleOutput {
    FixedVector::new([
        (FixedVector::new([(u(3),), (u(5),)]), u(7)),
        (FixedVector::new([(u(4),), (u(6),)]), u(8)),
    ])
}
struct Witness {
    expected_seen: u64,
    calls: std::rc::Rc<std::cell::RefCell<Vec<(u64, Field)>>>,
}
impl Witness {
    fn new(seen: u64) -> Self {
        Self {
            expected_seen: seen,
            calls: Default::default(),
        }
    }
    fn record(&self, private: u64, seen: Field) -> (u64, Narrow) {
        assert_eq!(private, 7);
        assert_eq!(seen, Field::from(self.expected_seen));
        self.calls.borrow_mut().push((private, seen));
        (private + 1, narrow())
    }
    fn check(&self) {
        assert_eq!(*self.calls.borrow(), [(7, Field::from(self.expected_seen))]);
    }
}
macro_rules! witness_impl {
    ($sdk:ident) => {
        impl $sdk::ledger_contract::Witnesses<u64> for Witness {
            fn vector_value(
                &self,
                ctx: midnight_compact_runtime::context::WitnessContext<
                    '_,
                    u64,
                    $sdk::ledger_contract::LedgerView<'_>,
                >,
            ) -> (u64, Narrow) {
                self.record(*ctx.private_state, ctx.ledger.seen().unwrap())
            }
        }
    };
}
witness_impl!(before);
witness_impl!(after);
fn check_routes<T: PartialEq>(
    native: &midnight_compact_runtime::context::CircuitResult<u64, T>,
    recorded: &midnight_compact_runtime::recording::RecordedCircuitResult<u64, T>,
    fnative: &midnight_compact_runtime::context::CircuitResult<u64, T>,
    frecorded: &midnight_compact_runtime::recording::RecordedCircuitResult<u64, T>,
    expected: &T,
    private: u64,
    private_outputs: &[AlignedValue],
) -> Result<(), Box<dyn Error>> {
    for out in [native, &recorded.execution, fnative, &frecorded.execution] {
        if out.result != *expected
            || out.context.private_state != private
            || out.context.query.state != native.context.query.state
            || out.context.query.effects != native.context.query.effects
            || out.gas_cost != native.gas_cost
            || out.private_transcript_outputs != private_outputs
        {
            return Err("vector-component-products typed four-route output/state/effects/private/gas mismatch".into());
        }
    }
    if recorded.public.verify_ops() != frecorded.public.verify_ops() {
        return Err("recorded public transcripts differ".into());
    }
    Ok(())
}
pub(super) fn run(pool: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(pool)?;
    if pool
        .join("vector-component-products-proof-result.json")
        .exists()
    {
        return Err("proof result already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (source, k)) in SELECTED.into_iter().enumerate() {
        let root = pool.join(source);
        let mut rng = StdRng::seed_from_u64(0x0183_0000 + index as u64);
        let initial = match source {
            "witness_before" => {
                before::ledger_contract::initial_state(ConstructorContext::new(7_u64))?
            }
            "witness_after" => {
                after::ledger_contract::initial_state(ConstructorContext::new(7_u64))?
            }
            "inner_product_single" => {
                single::ledger_contract::initial_state(ConstructorContext::new(7_u64))?
            }
            _ => unreachable!(),
        };
        let initial_state = initial.ledger_state.get_ref().clone();
        let (ir, verifier, operation) = v3::operation(&root, "run", k)?;
        let deploy = v3::deploy(initial_state.clone(), "run", operation, &mut rng);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        macro_rules! witnessed {
            ($sdk:ident,$seen:expr) => {{
                use $sdk::ledger_contract as api;
                let wn = Witness::new($seen);
                let wr = Witness::new($seen);
                let wfn = Witness::new($seen);
                let wfr = Witness::new($seen);
                let native: midnight_compact_runtime::context::CircuitResult<u64, Wide> =
                    api::run(observed.circuit_context(7), &wn)?;
                let recorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    Wide,
                > = api::recorded::run(observed.circuit_context(7), &wr)?;
                let fnative: midnight_compact_runtime::context::CircuitResult<u64, Wide> =
                    api::functional::run::Program::new()
                        .execute_native(observed.circuit_context(7), &wfn)?;
                let frecorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    Wide,
                > = api::functional::run::Program::new()
                    .execute_recorded(observed.circuit_context(7), &wfr)?;
                check_routes(
                    &native,
                    &recorded,
                    &fnative,
                    &frecorded,
                    &wide(),
                    8,
                    &[AlignedValue::from(narrow())],
                )?;
                for w in [&wn, &wr, &wfn, &wfr] {
                    w.check();
                }
                if $sdk::ledger_slots::seen.inspect(&initial_state)? != Field::from(0_u64)
                    || $sdk::ledger_slots::seen.inspect(native.context.query.state.get_ref())?
                        != Field::from(1_u64)
                {
                    return Err("witness seen Cell mismatch".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = midnight_compact_runtime::transaction::RecordedCall::new(
                    &observed,
                    frecorded,
                    "run",
                    (),
                )
                .prepare(verifier.clone(), Fr::from(0_u64))?;
                let ow = Witness::new($seen);
                let journal = ow.calls.clone();
                let facade = api::Contract::from(ow);
                let ordinary = facade
                    .recording()
                    .run_call(&observed, 7)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if *journal.borrow() != [(7, Field::from($seen as u64))] {
                    return Err("ordinary observed witness count/order differs".into());
                }
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err(
                        "ordinary observed witness call differs from functional preparation".into(),
                    );
                }
                (prepared, state)
            }};
        }
        let (prepared, state) = match source {
            "witness_before" => witnessed!(before, 0),
            "witness_after" => witnessed!(after, 1),
            "inner_product_single" => {
                use single::ledger_contract as api;
                let native: midnight_compact_runtime::context::CircuitResult<u64, SingleOutput> =
                    api::run(observed.circuit_context(7), single_input())?;
                let recorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    SingleOutput,
                > = api::recorded::run(observed.circuit_context(7), single_input())?;
                let fnative: midnight_compact_runtime::context::CircuitResult<u64, SingleOutput> =
                    api::functional::run::Program::new(single_input())
                        .execute_native(observed.circuit_context(7))?;
                let frecorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    SingleOutput,
                > = api::functional::run::Program::new(single_input())
                    .execute_recorded(observed.circuit_context(7))?;
                check_routes(
                    &native,
                    &recorded,
                    &fnative,
                    &frecorded,
                    &single_output(),
                    7,
                    &[],
                )?;
                if single::ledger_slots::seen.inspect(&initial_state)? != Field::from(0_u64)
                    || single::ledger_slots::seen.inspect(native.context.query.state.get_ref())?
                        != Field::from(1_u64)
                {
                    return Err("single-product seen Cell mismatch".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = midnight_compact_runtime::transaction::RecordedCall::new(
                    &observed,
                    frecorded,
                    "run",
                    single_input(),
                )
                .prepare(verifier.clone(), Fr::from(0_u64))?;
                let ordinary = api::Contract::default()
                    .recording()
                    .run_call(&observed, 7, single_input())?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err(
                        "ordinary observed single-product call differs from functional preparation"
                            .into(),
                    );
                }
                (prepared, state)
            }
            _ => unreachable!(),
        };
        let stem = format!("vector-component-products-{index}");
        let proof = prove(&root, source, &stem, &prepared, &verifier, &ir)?;
        let transaction_proof_bytes =
            v3::apply(&root, "run", &stem, prepared, deploy, &state, &proof)?;
        let transaction_bytes =
            std::fs::metadata(root.join(format!("{stem}-transaction.bin")))?.len();
        rows.push(serde_json::json!({"source":source,"circuit":"run","initial_seen":0,"seen":1,"private_state":if index<2{8}else{7},"witness_count":if index<2{1}else{0},"witness_seen":if index<2{Some(index)}else{None},"result_type":if index<2{"Vector<2,[Vector<2,[Uint<16>,Uint<16>]>,Uint<32>]>"}else{"Vector<2,[Vector<2,[Uint<16>]>,Uint<16>]>"},"proof_bytes":proof.0.len(),"transaction_bytes":transaction_bytes,
            "transaction_proof_bytes":transaction_proof_bytes,"transaction_proof_version":3,"positive_fee_dust_spends":1,
            "standalone_proof":format!("{source}/vector-component-products-{index}.proof"),"applied_transaction":format!("{source}/vector-component-products-{index}-transaction.bin"),"transaction_proof_relationship":"independently proved","changed_binding_rejected":true,"changed_result_rejected":true,"changed_result_same_geometry":true,"changed_preimage_rejected":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-vector-component-products-proof/v2","status":"passed","scope":"three existing complete source exports; other 21 required exports remain unproved; callbacks/constructor implementation not proved","constructor_execution_proved":false,"all_exports_proved":false,"all_source_exports_deployed":false,"selected_source_exports_proved":[["witness_before","run"],["witness_after","run"],["inner_product_single","run"]],"unproved_selected_source_exports":[],"calls":rows});
    write_new(
        pool.join("vector-component-products-proof-result.json"),
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
    source: &str,
    stem: &str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    ir: &IrSourceV3,
) -> Result<Proof, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected vector-component-products proof version".into()),
        };
    let pk: ProverKey<IrSourceV3> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/run.prover"),
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
            "vector-component-products proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("vector-component-products proof is empty".into());
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
        return Err("vector-component-products proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = match source {
        "witness_before" | "witness_after" => {
            let changed: Wide = FixedVector::new([
                (
                    FixedVector::new([(u(999), u(998)), (u(997), u(996))]),
                    u(99999),
                ),
                (
                    FixedVector::new([(u(995), u(994)), (u(993), u(992))]),
                    u(99998),
                ),
            ]);
            AlignedValue::from(changed)
        }
        "inner_product_single" => {
            let changed: SingleOutput = FixedVector::new([
                (FixedVector::new([(u(999),), (u(998),)]), u(997)),
                (FixedVector::new([(u(996),), (u(995),)]), u(994)),
            ]);
            AlignedValue::from(changed)
        }
        _ => return Err("unknown selected result carrier".into()),
    };
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed vector-component-products result geometry differs".into());
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
        return Err("vector-component-products proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed vector-component-products proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("vector-component-products IR accepted changed result".into());
    }
    write_new(root.join(format!("{stem}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(root.join(format!("{stem}.preimage")), &bytes)?;
    bytes.clear();
    tagged_serialize(&inputs, &mut bytes)?;
    write_new(root.join(format!("{stem}.inputs")), &bytes)?;
    Ok(proof)
}

pub(super) fn verify(pool: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(pool)?;
    for (index, (source, _)) in SELECTED.into_iter().enumerate() {
        let root = pool.join(source);
        let stem = format!("vector-component-products-{index}");
        let verifier = v3::read_verifier(&root, "run")?;
        v3::verify_standalone(&root, &stem, &verifier)?;
        let other = if source == "inner_product_single" {
            "witness_before"
        } else {
            "inner_product_single"
        };
        let wrong_root = pool.join(other);
        if std::fs::read(wrong_root.join("keys/run.verifier"))?
            == std::fs::read(root.join("keys/run.verifier"))?
        {
            return Err("wrong-verifier control has identical key bytes".into());
        }
        let wrong = v3::read_verifier(&wrong_root, "run")?;
        let proof = Proof(std::fs::read(root.join(format!("{stem}.proof")))?);
        let input_bytes = std::fs::read(root.join(format!("{stem}.inputs")))?;
        let mut input_reader = input_bytes.as_slice();
        let inputs: Vec<Fr> = tagged_deserialize(&mut input_reader)?;
        if !input_reader.is_empty() || wrong.verify(&PARAMS_VERIFIER, &proof, inputs).is_ok() {
            return Err("vector-component-products accepted wrong immutable verifier".into());
        }
        println!(
            "{source}/run: standalone v3 proof verified; corruption and wrong verifier refused"
        );
    }
    Ok(())
}
