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

//! One complete, unedited13export family SDK at its own crate root.
//! Four selected proofs do not prove its other exports or constructor/callbacks.
use super::{local_v3_application as v3, *};
use compact_rust_nested_component_products_sdk as family;
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;

const SELECTED: &[&str] = &[
    "helper",
    "witness_before",
    "witness_after",
    "field_identity",
];
fn selected(source: &str, circuit: &str) -> bool {
    source == "family" && SELECTED.contains(&circuit)
}
/// Observe every complete corpus export, including unselected or over-cap models.
/// Python binds the entire producer directory before and after this observation.
pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    let definitions: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../compact-rust-backend/tests/nested_component_products/sources.json"
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
            let model = ir.model();
            rows.push(
                serde_json::json!({"source":name,"circuit":entry,"k":model.k(),"rows":model.rows(),"proof_required":true,
                "selected_for_proof":selected(name,entry)}),
            );
        }
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-nested-component-products-model-inventory/v2",
        "circuits":rows,"proof_execution":false,"scope":"complete model observation, no key generation or all-source proof acceptance"})
    );
    Ok(())
}

const IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/nested_component_products/proof-goldens/family/source-ir.json"
);
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-1.json"
        ),
    ),
    (
        "program-2.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-2.json"
        ),
    ),
    (
        "program-3.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-3.json"
        ),
    ),
    (
        "program-4.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-4.json"
        ),
    ),
    (
        "program-5.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-5.json"
        ),
    ),
    (
        "program-6.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-6.json"
        ),
    ),
    (
        "program-7.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-7.json"
        ),
    ),
    (
        "program-8.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-8.json"
        ),
    ),
    (
        "program-9.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-9.json"
        ),
    ),
    (
        "program-10.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-10.json"
        ),
    ),
    (
        "program-11.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-11.json"
        ),
    ),
    (
        "program-12.json",
        include_bytes!(
            "../../compact-rust-backend/tests/nested_component_products/ledger9-proof-goldens/family/checked-programs/program-12.json"
        ),
    ),
];
fn validate_bindings(pool: &Path) -> Result<(), Box<dyn Error>> {
    let root = pool.join("family/contract");
    if std::fs::read(root.join("compact-rust-ir.json"))? != IR {
        return Err("nested-component-products family complete IR bytes differ".into());
    }
    let folder = root.join("checked-programs");
    let mut names = std::fs::read_dir(&folder)?
        .map(|r| r.map(|r| r.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut expected = PROGRAMS
        .iter()
        .map(|(name, _)| std::ffi::OsString::from(name))
        .collect::<Vec<_>>();
    names.sort();
    expected.sort();
    if names != expected {
        return Err("nested-component-products family complete Program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(folder.join(name))? != *bytes {
            return Err("nested-component-products family complete Program bytes differ".into());
        }
    }
    Ok(())
}
fn selected_models(pool: &Path) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    validate_bindings(pool)?;
    let root = pool.join("family/zkir");
    SELECTED
        .iter()
        .map(|name| {
            let ir = IrSourceV3::load(File::open(root.join(format!("{name}.zkir")))?)?;
            let model = ir.model();
            Ok(serde_json::json!({
                "source": "family",
                "circuit": name,
                "k": model.k(),
                "rows": model.rows(),
                "selected_for_proof": true,
            }))
        })
        .collect()
}
fn tiny_bound(rows: &[serde_json::Value]) -> Result<(), Box<dyn Error>> {
    if rows.len() != 4 || rows.iter().any(|r| r["k"].as_u64().is_none_or(|k| k > 8)) {
        return Err("nested-component-products model exceeds tiny automatic k<=8".into());
    }
    Ok(())
}
pub(super) fn models(pool: &Path) -> Result<(), Box<dyn Error>> {
    let rows = selected_models(pool)?;
    println!(
        "{}",
        serde_json::json!({"format":"compact-nested-component-products-models/v2","circuits":rows,"proof_execution":false})
    );
    tiny_bound(&rows)
}
type U8 = BoundedUint<255>;
type U16 = BoundedUint<65535>;
type U32 = BoundedUint<4294967295>;
type Narrow = FixedVector<(FixedVector<((U8, U16), U8), 1>, U16), 1>;
type Wide = FixedVector<(FixedVector<((U16, U32), U16), 1>, U32), 1>;
type FieldInput = FixedVector<(FixedVector<((Field, U8), U8), 1>, U16), 1>;
type FieldOutput = FixedVector<(FixedVector<((Field, U16), U16), 1>, U32), 1>;
fn u<const M: u128>(n: u128) -> BoundedUint<M> {
    BoundedUint::new(n).unwrap()
}
fn narrow() -> Narrow {
    FixedVector::new([(FixedVector::new([((u(3), u(60000)), u(255))]), u(65535))])
}
fn wide() -> Wide {
    FixedVector::new([(FixedVector::new([((u(3), u(60000)), u(255))]), u(65535))])
}
fn large() -> Field {
    Field::from(1_u128 << 127) * Field::from(8_u64) + Field::from(21_u64)
}
fn field_input() -> FieldInput {
    FixedVector::new([(FixedVector::new([((large(), u(255)), u(17))]), u(65535))])
}
fn field_output() -> FieldOutput {
    FixedVector::new([(FixedVector::new([((large(), u(255)), u(17))]), u(65535))])
}
fn changed_output(name: &str) -> Result<AlignedValue, Box<dyn Error>> {
    Ok(match name {
        "helper" | "witness_before" | "witness_after" => {
            let value: Wide =
                FixedVector::new([(FixedVector::new([((u(4), u(60000)), u(255))]), u(65535))]);
            value.into()
        }
        "field_identity" => {
            let value: FieldOutput = FixedVector::new([(
                FixedVector::new([((large() + Field::from(1_u64), u(255)), u(17))]),
                u(65535),
            )]);
            value.into()
        }
        _ => return Err("unknown selected nested result carrier".into()),
    })
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
    fn check(&self) {
        assert_eq!(*self.calls.borrow(), [(7, Field::from(self.expected_seen))]);
    }
}
impl family::ledger_contract::Witnesses<u64> for Witness {
    fn vector_value(
        &self,
        ctx: midnight_compact_runtime::context::WitnessContext<
            '_,
            u64,
            family::ledger_contract::LedgerView<'_>,
        >,
    ) -> (u64, Narrow) {
        assert_eq!(*ctx.private_state, 7);
        let seen = ctx.ledger.seen().unwrap();
        assert_eq!(seen, Field::from(self.expected_seen));
        self.calls.borrow_mut().push((*ctx.private_state, seen));
        (8, narrow())
    }
}
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
            return Err("nested-component-products typed four-route output/state/effects/private/gas mismatch".into());
        }
    }
    if recorded.public.verify_ops() != frecorded.public.verify_ops() {
        return Err("recorded public transcripts differ".into());
    }
    Ok(())
}

pub(super) fn run(pool: &Path) -> Result<(), Box<dyn Error>> {
    tiny_bound(&selected_models(pool)?)?;
    if pool
        .join("nested-component-products-proof-result.json")
        .exists()
    {
        return Err("proof result already exists".into());
    }
    let root = pool.join("family");
    let mut rows = Vec::new();
    for (index, &name) in SELECTED.iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(0x0185_0000 + index as u64);
        let initial = family::ledger_contract::initial_state(ConstructorContext::new(7_u64))?;
        let initial_state = initial.ledger_state.get_ref().clone();
        if family::ledger_slots::seen.inspect(&initial_state)? != Field::from(0_u64) {
            return Err("initial Cell differs".into());
        }
        let k = match name {
            "helper" | "field_identity" => 8,
            "witness_before" | "witness_after" => 7,
            _ => unreachable!(),
        };
        let (ir, verifier, operation) = v3::operation(&root, name, k)?;
        let deploy = v3::deploy(initial_state, name, operation, &mut rng);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        macro_rules! scalar_call {
            ($entry:ident,$call:ident,$input:expr,$expected:expr,$output:ty,$seen:expr) => {{
                use family::ledger_contract as api;
                let native: midnight_compact_runtime::context::CircuitResult<u64, $output> =
                    api::$entry(observed.circuit_context(7), $input)?;
                let recorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    $output,
                > = api::recorded::$entry(observed.circuit_context(7), $input)?;
                let fnative: midnight_compact_runtime::context::CircuitResult<u64, $output> =
                    api::functional::$entry::Program::new($input)
                        .execute_native(observed.circuit_context(7))?;
                let frecorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    $output,
                > = api::functional::$entry::Program::new($input)
                    .execute_recorded(observed.circuit_context(7))?;
                check_routes(&native, &recorded, &fnative, &frecorded, &$expected, 7, &[])?;
                if family::ledger_slots::seen.inspect(native.context.query.state.get_ref())?
                    != Field::from($seen as u64)
                {
                    return Err("scalar selected Cell mismatch".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = midnight_compact_runtime::transaction::RecordedCall::new(
                    &observed, frecorded, name, $input,
                )
                .prepare(verifier.clone(), Fr::from(0_u64))?;
                let ordinary = api::Contract::default()
                    .recording()
                    .$call(&observed, 7, $input)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err("ordinary/functional selected preparation differs".into());
                }
                (prepared, state)
            }};
        }
        macro_rules! witnessed {
            ($entry:ident,$call:ident,$before:expr,$seen:expr) => {{
                use family::ledger_contract as api;
                let wn = Witness::new($before);
                let wr = Witness::new($before);
                let wfn = Witness::new($before);
                let wfr = Witness::new($before);
                let native: midnight_compact_runtime::context::CircuitResult<u64, Wide> =
                    api::$entry(observed.circuit_context(7), &wn)?;
                let recorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    Wide,
                > = api::recorded::$entry(observed.circuit_context(7), &wr)?;
                let fnative: midnight_compact_runtime::context::CircuitResult<u64, Wide> =
                    api::functional::$entry::Program::new()
                        .execute_native(observed.circuit_context(7), &wfn)?;
                let frecorded: midnight_compact_runtime::recording::RecordedCircuitResult<
                    u64,
                    Wide,
                > = api::functional::$entry::Program::new()
                    .execute_recorded(observed.circuit_context(7), &wfr)?;
                check_routes(
                    &native,
                    &recorded,
                    &fnative,
                    &frecorded,
                    &wide(),
                    8,
                    &[narrow().into()],
                )?;
                for w in [&wn, &wr, &wfn, &wfr] {
                    w.check();
                }
                if family::ledger_slots::seen.inspect(native.context.query.state.get_ref())?
                    != Field::from($seen as u64)
                {
                    return Err("witness selected Cell mismatch".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = midnight_compact_runtime::transaction::RecordedCall::new(
                    &observed,
                    frecorded,
                    name,
                    (),
                )
                .prepare(verifier.clone(), Fr::from(0_u64))?;
                let ow = Witness::new($before);
                let journal = ow.calls.clone();
                let facade = api::Contract::from(ow);
                let ordinary = facade
                    .recording()
                    .$call(&observed, 7)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if *journal.borrow() != [(7, Field::from($before as u64))] {
                    return Err("ordinary observed witness count/order differs".into());
                }
                if format!("{prepared:?}") != format!("{ordinary:?}") {
                    return Err("ordinary/functional selected preparation differs".into());
                }
                (prepared, state)
            }};
        }
        let (prepared, state) = match name {
            "helper" => scalar_call!(helper, helper_call, narrow(), wide(), Wide, 2),
            "witness_before" => witnessed!(witness_before, witness_before_call, 0, 5),
            "witness_after" => witnessed!(witness_after, witness_after_call, 6, 6),
            "field_identity" => scalar_call!(
                field_identity,
                field_identity_call,
                field_input(),
                field_output(),
                FieldOutput,
                14
            ),
            _ => unreachable!(),
        };
        let stem = format!("nested-component-products-{index}");
        let proof = prove(&root, name, &stem, &prepared, &verifier, &ir)?;
        let transaction_proof_bytes =
            v3::apply(&root, name, &stem, prepared, deploy, &state, &proof)?;
        let transaction_bytes =
            std::fs::metadata(root.join(format!("{stem}-transaction.bin")))?.len();
        rows.push(serde_json::json!({"source":"family","circuit":name,"initial_seen":0,"seen":([2,5,6,14][index]),"private_state":if index==1||index==2{8}else{7},"witness_count":if index==1||index==2{1}else{0},"witness_seen":if index==1{Some(0)}else if index==2{Some(6)}else{None},"result_type":if index==3{"Vector<1,[Vector<1,[[Field,Uint<16>],Uint<16>]>,Uint<32>]>"}else{"Vector<1,[Vector<1,[[Uint<16>,Uint<32>],Uint<16>]>,Uint<32>]>"},"proof_bytes":proof.0.len(),"transaction_bytes":transaction_bytes,
            "transaction_proof_bytes":transaction_proof_bytes,"transaction_proof_version":3,"positive_fee_dust_spends":1,
            "standalone_proof":format!("family/nested-component-products-{index}.proof"),"applied_transaction":format!("family/nested-component-products-{index}-transaction.bin"),"transaction_proof_relationship":"independently proved","changed_binding_rejected":true,"changed_result_rejected":true,"changed_result_same_geometry":true,"changed_preimage_rejected":true,"wrong_verifier_rejected":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-nested-component-products-proof/v2","status":"passed","scope":"four selected exports of the complete 13-export family; ten of 14 required source exports remain unproved; constructor and callback implementation outside proof scope","constructor_execution_proved":false,"all_exports_proved":false,"all_source_exports_deployed":false,"selected_source_exports_proved":SELECTED.iter().map(|name|["family",name]).collect::<Vec<_>>(),"unproved_selected_source_exports":[],"calls":rows});
    write_new(
        pool.join("nested-component-products-proof-result.json"),
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
    stem: &str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    ir: &IrSourceV3,
) -> Result<Proof, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected nested-component-products proof version".into()),
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
            "nested-component-products proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("nested-component-products proof is empty".into());
    }
    // A genuinely different source relation, not merely a second key for the same erased circuit.
    let other = match name {
        "helper" | "witness_before" | "witness_after" => "field_identity",
        "field_identity" => "helper",
        _ => return Err("unknown source/circuit verifier pair".into()),
    };
    let wrong_path = root.join(format!("keys/{other}.verifier"));
    if std::fs::read(&wrong_path)? == std::fs::read(root.join(format!("keys/{name}.verifier")))? {
        return Err("wrong-verifier control has identical key bytes".into());
    }
    let wrong: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(wrong_path)?))?;
    if wrong
        .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("nested-component-products accepted wrong immutable verifier".into());
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
        return Err("nested-component-products proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = changed_output(name)?;
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed nested-component-products result geometry differs".into());
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
        return Err("nested-component-products proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed nested-component-products proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("nested-component-products IR accepted changed result".into());
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
    let root = pool.join("family");
    for (index, &name) in SELECTED.iter().enumerate() {
        let stem = format!("nested-component-products-{index}");
        let verifier = v3::read_verifier(&root, name)?;
        v3::verify_standalone(&root, &stem, &verifier)?;
        println!("family/{name}: standalone v3 proof verified and corruption refused");
    }
    Ok(())
}
