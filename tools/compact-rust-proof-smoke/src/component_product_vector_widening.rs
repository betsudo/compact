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

//! Fourteen actual in-cap calls from two complete, unedited source SDKs.
//! The two observed k9 exports and constructors remain unproved.
use super::*;
use compact_rust_component_product_vector_family_sdk as family;
use compact_rust_component_product_vector_original_sdk as original;
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};

const SELECTED: &[(&str, &str, &str, u64)] = &[
    ("original_nested", "run", "original_nested", 1),
    ("family", "parameter", "parameter", 1),
    ("family", "helper", "helper", 2),
    ("family", "local", "local", 3),
    ("family", "checked_helper", "checked_valid", 4),
    ("family", "witness_before", "witness_before", 5),
    ("family", "witness_after", "witness_after", 6),
    ("family", "deep", "deep", 9),
    ("family", "boolean_field", "boolean_field", 10),
    ("family", "empty", "empty", 11),
    ("family", "tuple_empty", "tuple_empty", 12),
    ("family", "tuple_single", "tuple_single", 13),
    ("family", "field_identity", "field_identity", 14),
    ("family", "empty_leaf", "empty_leaf", 15),
];
const FAMILY_IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/source-ir.json"
);
const FAMILY_PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-1.json"
        ),
    ),
    (
        "program-2.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-2.json"
        ),
    ),
    (
        "program-3.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-3.json"
        ),
    ),
    (
        "program-4.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-4.json"
        ),
    ),
    (
        "program-5.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-5.json"
        ),
    ),
    (
        "program-6.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-6.json"
        ),
    ),
    (
        "program-7.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-7.json"
        ),
    ),
    (
        "program-8.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-8.json"
        ),
    ),
    (
        "program-9.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-9.json"
        ),
    ),
    (
        "program-10.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-10.json"
        ),
    ),
    (
        "program-11.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-11.json"
        ),
    ),
    (
        "program-12.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-12.json"
        ),
    ),
    (
        "program-13.json",
        include_bytes!(
            "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/family/checked-programs/program-13.json"
        ),
    ),
];
const ORIGINAL_IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/original_nested/source-ir.json"
);
const ORIGINAL_PROGRAMS: &[(&str, &[u8])] = &[(
    "program-0.json",
    include_bytes!(
        "../../compact-rust-backend/tests/component_product_vector_widening/proof-goldens/original_nested/checked-programs/program-0.json"
    ),
)];
fn validate_bindings(pool: &Path) -> Result<(), Box<dyn Error>> {
    for (source, ir, programs) in [
        ("family", FAMILY_IR, FAMILY_PROGRAMS),
        ("original_nested", ORIGINAL_IR, ORIGINAL_PROGRAMS),
    ] {
        let root = pool.join(source).join("contract");
        if std::fs::read(root.join("compact-rust-ir.json"))? != ir {
            return Err(
                format!("component-product-vector {source} complete IR bytes differ").into(),
            );
        }
        let folder = root.join("checked-programs");
        let mut names = std::fs::read_dir(&folder)?
            .map(|r| r.map(|r| r.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut expected = programs
            .iter()
            .map(|(n, _)| std::ffi::OsString::from(n))
            .collect::<Vec<_>>();
        names.sort();
        expected.sort();
        if names != expected {
            return Err(format!(
                "component-product-vector {source} complete Program membership differs"
            )
            .into());
        }
        for (name, bytes) in programs {
            if std::fs::read(folder.join(name))? != *bytes {
                return Err(format!(
                    "component-product-vector {source} complete Program bytes differ"
                )
                .into());
            }
        }
    }
    Ok(())
}
fn selected_models(pool: &Path) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    validate_bindings(pool)?;
    SELECTED.iter().map(|(source,name,_,_)|{
  let ir=IrSource::load(BufReader::new(File::open(pool.join(source).join(format!("zkir/{name}.zkir")))?))?;
  Ok(serde_json::json!({"source":source,"circuit":name,"k":ir.model().k(),"selected_for_proof":true}))
 }).collect()
}
fn tiny_bound(rows: &[serde_json::Value]) -> Result<(), Box<dyn Error>> {
    if rows.len() != 14 || rows.iter().any(|r| r["k"].as_u64().is_none_or(|k| k > 8)) {
        return Err("component-product-vector model exceeds tiny automatic k<=8".into());
    }
    Ok(())
}
pub(super) fn models(pool: &Path) -> Result<(), Box<dyn Error>> {
    let rows = selected_models(pool)?;
    println!(
        "{}",
        serde_json::json!({"format":"compact-component-product-vector-models/v1","circuits":rows,"proof_execution":false})
    );
    tiny_bound(&rows)
}
type U8 = BoundedUint<255>;
type U16 = BoundedUint<65535>;
type U32 = BoundedUint<4294967295>;
type NarrowProduct = (FixedVector<(FixedVector<U8, 2>, U8), 1>, U16);
type WideProduct = (FixedVector<(FixedVector<U16, 2>, U16), 1>, U32);
type Narrow = FixedVector<NarrowProduct, 1>;
type Wide = FixedVector<WideProduct, 1>;
type OriginalNestedInput = FixedVector<(FixedVector<((FixedVector<U8, 1>,), U8), 1>, U8), 1>;
type OriginalNestedOutput = FixedVector<(FixedVector<((FixedVector<U16, 1>,), U16), 1>, U16), 1>;
type DeepNarrowChild = FixedVector<(FixedVector<U8, 2>, U16), 1>;
type DeepWideChild = FixedVector<(FixedVector<U16, 2>, U32), 1>;
type DeepInput = FixedVector<(FixedVector<((DeepNarrowChild, U8), U16), 1>, U8), 1>;
type DeepOutput = FixedVector<(FixedVector<((DeepWideChild, U16), U32), 1>, U16), 1>;
type BoolInput = FixedVector<(FixedVector<(FixedVector<bool, 2>, U8), 1>, bool), 1>;
type BoolOutput = FixedVector<(FixedVector<(FixedVector<bool, 2>, Field), 1>, bool), 1>;
type EmptyTupleInput = FixedVector<(FixedVector<(FixedVector<(), 2>, U8), 1>, U16), 1>;
type EmptyTupleOutput = FixedVector<(FixedVector<(FixedVector<(), 2>, U16), 1>, U32), 1>;
type SingleTupleInput = FixedVector<(FixedVector<((FixedVector<U8, 1>,), U16), 1>, U16), 1>;
type SingleTupleOutput = FixedVector<(FixedVector<((FixedVector<U16, 1>,), U32), 1>, U32), 1>;
type EmptyLeafInput = FixedVector<(FixedVector<(FixedVector<U8, 0>, U8), 1>, U16), 1>;
type EmptyLeafOutput = FixedVector<(FixedVector<(FixedVector<U16, 0>, U16), 1>, U32), 1>;
type FieldIdentityInput = FixedVector<(FixedVector<(FixedVector<Field, 2>, U8), 1>, U16), 1>;
type FieldIdentityOutput = FixedVector<(FixedVector<(FixedVector<Field, 2>, U16), 1>, U32), 1>;
fn u<const M: u128>(value: u128) -> BoundedUint<M> {
    BoundedUint::new(value).unwrap()
}
fn narrow(a: u128, b: u128, c: u128, d: u128) -> Narrow {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([u(a), u(b)]), u(c))]),
        u(d),
    )])
}
fn wide(a: u128, b: u128, c: u128, d: u128) -> Wide {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([u(a), u(b)]), u(c))]),
        u(d),
    )])
}
fn original_nested_input() -> OriginalNestedInput {
    FixedVector::new([(
        FixedVector::new([((FixedVector::new([u(3)]),), u(255))]),
        u(219),
    )])
}
fn deep_input() -> DeepInput {
    let child = FixedVector::new([(FixedVector::new([u(3), u(128)]), u(60000))]);
    FixedVector::new([(FixedVector::new([((child, u(255)), u(65535))]), u(219))])
}
fn boolean_input(a: bool, b: bool, c: u128, d: bool) -> BoolInput {
    FixedVector::new([(FixedVector::new([(FixedVector::new([a, b]), u(c))]), d)])
}
fn empty_tuple_input() -> EmptyTupleInput {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([(), ()]), u(255))]),
        u(65535),
    )])
}
fn single_tuple_input() -> SingleTupleInput {
    FixedVector::new([(
        FixedVector::new([((FixedVector::new([u(3)]),), u(60000))]),
        u(65535),
    )])
}
fn empty_leaf_input() -> EmptyLeafInput {
    FixedVector::new([(FixedVector::new([(FixedVector::new([]), u(255))]), u(65535))])
}
fn field_identity_input() -> FieldIdentityInput {
    let large = Field::from(1_u128 << 127) * Field::from(8_u64) + Field::from(21_u64);
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([large, Field::from(42_u64)]), u(17))]),
        u(65535),
    )])
}
fn original_nested_output(value: u128) -> OriginalNestedOutput {
    FixedVector::new([(
        FixedVector::new([((FixedVector::new([u(value)]),), u(255))]),
        u(219),
    )])
}
fn deep_output(value: u128) -> DeepOutput {
    let child = FixedVector::new([(FixedVector::new([u(value), u(128)]), u(60000))]);
    FixedVector::new([(FixedVector::new([((child, u(255)), u(65535))]), u(219))])
}
fn single_tuple_output(value: u128) -> SingleTupleOutput {
    FixedVector::new([(
        FixedVector::new([((FixedVector::new([u(value)]),), u(60000))]),
        u(65535),
    )])
}
fn empty_tuple_output(middle: u128) -> EmptyTupleOutput {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([(), ()]), u(middle))]),
        u(65535),
    )])
}
fn empty_leaf_output(middle: u128) -> EmptyLeafOutput {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([]), u(middle))]),
        u(65535),
    )])
}
fn boolean_output(middle: u64) -> BoolOutput {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([true, false]), Field::from(middle))]),
        false,
    )])
}
fn field_output(delta: u64) -> FieldIdentityOutput {
    let large = Field::from(1_u128 << 127) * Field::from(8_u64) + Field::from(21_u64 + delta);
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([large, Field::from(42_u64)]), u(17))]),
        u(65535),
    )])
}
fn changed_output(source: &str, name: &str) -> Result<Option<AlignedValue>, Box<dyn Error>> {
    Ok(Some(match (source, name) {
        ("original_nested", "run") => original_nested_output(4).into(),
        (
            "family",
            "parameter" | "helper" | "local" | "checked_helper" | "witness_before"
            | "witness_after",
        ) => wide(4, 128, 255, 65535).into(),
        ("family", "deep") => deep_output(4).into(),
        ("family", "boolean_field") => boolean_output(254).into(),
        ("family", "tuple_empty") => empty_tuple_output(254).into(),
        ("family", "tuple_single") => single_tuple_output(4).into(),
        ("family", "field_identity") => field_output(1).into(),
        ("family", "empty_leaf") => empty_leaf_output(254).into(),
        ("family", "empty") => return Ok(None),
        _ => return Err("unknown typed result carrier".into()),
    }))
}
fn encode(value: AlignedValue) -> serde_json::Value {
    serde_json::json!({"valueAtoms":value.value.0.iter().map(|x|&x.0).collect::<Vec<_>>(),"alignment":value.alignment})
}
fn check_ts(
    row: &serde_json::Value,
    args: Vec<AlignedValue>,
    output: AlignedValue,
) -> Result<(), Box<dyn Error>> {
    if row["argumentValues"] != serde_json::json!(args.into_iter().map(encode).collect::<Vec<_>>())
        || row["result"] != encode(output)
        || !row["error"].is_null()
    {
        return Err("typed proof arguments/results differ from current independent TS".into());
    }
    Ok(())
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
        (8, narrow(3, 128, 255, 65535))
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
            return Err("component-product-vector typed four-route output/state/effects/private/gas mismatch".into());
        }
    }
    if recorded.public.verify_ops() != frecorded.public.verify_ops() {
        return Err("recorded public transcripts differ".into());
    }
    Ok(())
}

pub(super) fn run(pool: &Path, ts: &Path) -> Result<(), Box<dyn Error>> {
    tiny_bound(&selected_models(pool)?)?;
    if pool
        .join("component-product-vector-proof-result.json")
        .exists()
    {
        return Err("proof result already exists".into());
    }
    let captures: serde_json::Value = serde_json::from_slice(&std::fs::read(ts)?)?;
    if captures["status"] != "passed"
        || captures["format"] != "component-product-vector-widening-ts/v1"
    {
        return Err("current TS capture not qualified".into());
    }
    let mut rows = Vec::new();
    for (index, &(source, name, row_id, seen)) in SELECTED.iter().enumerate() {
        let root = pool.join(source);
        let row = captures["rows"]
            .as_array()
            .ok_or("missing current TS rows")?
            .iter()
            .find(|r| r["source"] == source && r["entry"] == name && r["id"] == row_id)
            .ok_or("missing actual current TS proof case")?;
        let mut rng = StdRng::seed_from_u64(0x0187_0000 + index as u64);
        let initial = if source == "original_nested" {
            original::ledger_contract::initial_state(ConstructorContext::new(7_u64))?
                .ledger_state
                .get_ref()
                .clone()
        } else {
            family::ledger_contract::initial_state(ConstructorContext::new(7_u64))?
                .ledger_state
                .get_ref()
                .clone()
        };
        let initial_seen = if source == "original_nested" {
            original::ledger_slots::seen.inspect(&initial)?
        } else {
            family::ledger_slots::seen.inspect(&initial)?
        };
        if initial_seen != Field::from(0_u64) {
            return Err("initial Cell differs".into());
        }
        let deploy = make_deploy(&root, name, initial, &mut rng)?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
            root.join(format!("keys/{name}.verifier")),
        )?))?;
        macro_rules! plain {
   ($module:ident,$entry:ident,$call:ident,[$($arg:expr),*],$expected:expr,$output:ty)=>{{
    use $module::ledger_contract as api;
    let expected:$output=$expected;
    check_ts(row,vec![$(AlignedValue::from($arg)),*],expected.clone().into())?;
    let native:midnight_compact_runtime::context::CircuitResult<u64,$output>=api::$entry(observed.circuit_context(7),$($arg),*)?;
    let recorded=api::recorded::$entry(observed.circuit_context(7),$($arg),*)?;
    let fnative=api::functional::$entry::Program::new($($arg),*).execute_native(observed.circuit_context(7))?;
    let frecorded=api::functional::$entry::Program::new($($arg),*).execute_recorded(observed.circuit_context(7))?;
    check_routes(&native,&recorded,&fnative,&frecorded,&expected,7,&[])?;
    let actual_seen=$module::ledger_slots::seen.inspect(native.context.query.state.get_ref())?;
    if actual_seen!=Field::from(seen){return Err("selected source Cell mismatch".into());}
    let state=native.context.query.state.get_ref().clone();
    let prepared=check_generated_trace(&root,name,frecorded,($($arg),*))?;
    let ordinary=api::Contract::default().recording().$call(&observed,7,$($arg),*)?.prepare(verifier.clone(),Fr::from(0_u64))?;
    if format!("{prepared:?}")!=format!("{ordinary:?}"){return Err("ordinary/functional selected preparation differs".into());}
    (prepared,state)
   }};
  }
        macro_rules! witnessed {
            ($entry:ident,$call:ident,$before:expr) => {{
                use family::ledger_contract as api;
                let wn = Witness::new($before);
                let wr = Witness::new($before);
                let wfn = Witness::new($before);
                let wfr = Witness::new($before);
                let expected = wide(3, 128, 255, 65535);
                check_ts(row, vec![], expected.clone().into())?;
                let native = api::$entry(observed.circuit_context(7), &wn)?;
                let recorded = api::recorded::$entry(observed.circuit_context(7), &wr)?;
                let fnative = api::functional::$entry::Program::new()
                    .execute_native(observed.circuit_context(7), &wfn)?;
                let frecorded = api::functional::$entry::Program::new()
                    .execute_recorded(observed.circuit_context(7), &wfr)?;
                check_routes(
                    &native,
                    &recorded,
                    &fnative,
                    &frecorded,
                    &expected,
                    8,
                    &[narrow(3, 128, 255, 65535).into()],
                )?;
                for w in [&wn, &wr, &wfn, &wfr] {
                    w.check();
                }
                if family::ledger_slots::seen.inspect(native.context.query.state.get_ref())?
                    != Field::from(seen)
                {
                    return Err("witness Cell differs".into());
                }
                let state = native.context.query.state.get_ref().clone();
                let prepared = check_generated_trace(&root, name, frecorded, ())?;
                let ow = Witness::new($before);
                let journal = ow.calls.clone();
                let facade = api::Contract::from(ow);
                let ordinary = facade
                    .recording()
                    .$call(&observed, 7)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                if *journal.borrow() != [(7, Field::from($before as u64))]
                    || format!("{prepared:?}") != format!("{ordinary:?}")
                {
                    return Err("ordinary/functional observed witness preparation differs".into());
                }
                (prepared, state)
            }};
        }
        let (prepared, state) = match (source, name) {
            ("original_nested", "run") => plain!(
                original,
                run,
                run_call,
                [original_nested_input()],
                original_nested_output(3),
                OriginalNestedOutput
            ),
            ("family", "parameter") => plain!(
                family,
                parameter,
                parameter_call,
                [narrow(3, 128, 255, 65535)],
                wide(3, 128, 255, 65535),
                Wide
            ),
            ("family", "helper") => plain!(
                family,
                helper,
                helper_call,
                [narrow(3, 128, 255, 65535)],
                wide(3, 128, 255, 65535),
                Wide
            ),
            ("family", "local") => plain!(
                family,
                local,
                local_call,
                [narrow(3, 128, 255, 65535)],
                wide(3, 128, 255, 65535),
                Wide
            ),
            ("family", "checked_helper") => plain!(
                family,
                checked_helper,
                checked_helper_call,
                [true, wide(3, 128, 255, 65535)],
                wide(3, 128, 255, 65535),
                Wide
            ),
            ("family", "witness_before") => witnessed!(witness_before, witness_before_call, 0),
            ("family", "witness_after") => witnessed!(witness_after, witness_after_call, 6),
            ("family", "deep") => plain!(
                family,
                deep,
                deep_call,
                [deep_input()],
                deep_output(3),
                DeepOutput
            ),
            ("family", "boolean_field") => plain!(
                family,
                boolean_field,
                boolean_field_call,
                [boolean_input(true, false, 255, false)],
                boolean_output(255),
                BoolOutput
            ),
            ("family", "empty") => {
                plain!(family,empty,empty_call,[FixedVector::<NarrowProduct,0>::new([])],FixedVector::<WideProduct,0>::new([]),FixedVector<WideProduct,0>)
            }
            ("family", "tuple_empty") => plain!(
                family,
                tuple_empty,
                tuple_empty_call,
                [empty_tuple_input()],
                empty_tuple_output(255),
                EmptyTupleOutput
            ),
            ("family", "tuple_single") => plain!(
                family,
                tuple_single,
                tuple_single_call,
                [single_tuple_input()],
                single_tuple_output(3),
                SingleTupleOutput
            ),
            ("family", "field_identity") => plain!(
                family,
                field_identity,
                field_identity_call,
                [field_identity_input()],
                field_output(0),
                FieldIdentityOutput
            ),
            ("family", "empty_leaf") => plain!(
                family,
                empty_leaf,
                empty_leaf_call,
                [empty_leaf_input()],
                empty_leaf_output(255),
                EmptyLeafOutput
            ),
            _ => unreachable!(),
        };
        let proof_bytes = prove(
            &root,
            pool,
            (source, name, seen),
            index,
            &prepared,
            &verifier,
        )?;
        let transaction_bytes = apply(&root, name, index, prepared, deploy, &state, rng)?;
        let witnessed = matches!(name, "witness_before" | "witness_after");
        let ir: serde_json::Value = serde_json::from_slice(if source == "family" {
            FAMILY_IR
        } else {
            ORIGINAL_IR
        })?;
        let result_type = ir["stateful_circuits"]
            .as_array()
            .ok_or("missing whole source circuits")?
            .iter()
            .find(|c| c["name"] == name)
            .ok_or("missing source result")?["result"]
            .clone();
        let result_applicable = name != "empty";
        rows.push(serde_json::json!({"source":source,"circuit":name,"initial_seen":0,"seen":seen,"private_state":if witnessed{8}else{7},"witness_count":if witnessed{1}else{0},"witness_seen":if name=="witness_before"{Some(0)}else if name=="witness_after"{Some(6)}else{None},"result_type":result_type,"proof_bytes":proof_bytes,"transaction_bytes":transaction_bytes,
   "standalone_proof":format!("{source}/component-product-vector-{index}.proof"),"applied_transaction":format!("{source}/component-product-vector-{index}-transaction.bin"),"transaction_proof_relationship":"independently proved","corrupt_proof_rejected":true,"changed_binding_rejected":true,"changed_result_rejected":if result_applicable{Some(true)}else{None},"changed_result_same_geometry":if result_applicable{Some(true)}else{None},"result_negative_applicability":if result_applicable{"applicable_nonempty_result"}else{"inapplicable_unique_empty_carrier"},"changed_cell_write_preimage_rejected":true,"changed_preimage_rejected":true,"wrong_verifier_rejected":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-component-product-vector-proof/v1","status":"passed","scope":"all14in-cap exports of the complete16export corpus, one typed call each;2k9exports/constructor/callbackdeployment/wallet/allinputs unproved","constructor_execution_proved":false,"all_exports_proved":false,"all_source_exports_deployed":false,"selected_source_exports_proved":SELECTED.iter().map(|(s,n,_,_)|[s,n]).collect::<Vec<_>>(),"unproved_selected_source_exports":[],"calls":rows});
    write_new(
        pool.join("component-product-vector-proof-result.json"),
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
    pool: &Path,
    case: (&str, &str, u64),
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<usize, Box<dyn Error>> {
    let (source, name, seen) = case;
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected component-product-vector proof version".into()),
        };
    let ir: IrSource = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("zkir/{name}.bzkir")),
    )?))?;
    let pk: ProverKey<IrSource> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
    )?))?;
    let checked = preimage.check(&ir)?;
    let mut changed_statement = preimage.as_ref().clone();
    let last = changed_statement
        .public_transcript_inputs
        .last_mut()
        .ok_or("missing source CellWrite input")?;
    if *last != Fr::from(seen) {
        return Err("source CellWrite constant is not the targeted final transcript input".into());
    }
    if seen == 0 {
        return Err("changed CellWrite control requires the fixed nonzero source write".into());
    }
    *last = Fr::from(0_u64);
    if changed_statement.check(&ir).is_ok() {
        return Err("IR accepted changed source CellWrite preimage".into());
    }

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
            "component-product-vector proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("component-product-vector proof is empty".into());
    }
    let mut corrupt = proof.clone();
    corrupt.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupt, inputs.iter().copied())
        .is_ok()
    {
        return Err("component-product-vector accepted corrupted proof bytes".into());
    }
    // Distinct source relations: zero-argument CellWrite(empty,k5) versus
    // nonempty input/output helpers. Never merely a second key of one circuit.
    let wrong_path = pool
        .join("family/keys")
        .join(if source == "family" && name == "empty" {
            "helper.verifier"
        } else {
            "empty.verifier"
        });
    if std::fs::read(&wrong_path)? == std::fs::read(root.join(format!("keys/{name}.verifier")))? {
        return Err("wrong-verifier control has identical key bytes".into());
    }
    let wrong: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(wrong_path)?))?;
    if wrong
        .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("component-product-vector accepted wrong immutable verifier".into());
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
        return Err("component-product-vector proof accepted changed public binding".into());
    }
    if let Some(output) = changed_output(source, name)? {
        // Bind a genuinely different public result through the exact upstream
        // communication commitment, rather than changing an arbitrary input slot.
        let mut changed_call = call.clone();
        changed_call.output = output;
        if changed_call.output.alignment != call.output.alignment
            || changed_call.output.value.0.len() != call.output.value.0.len()
        {
            return Err("changed component-product-vector result geometry differs".into());
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
            return Err("component-product-vector proof accepted changed result commitment".into());
        }
        let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
            &changed_call,
            changed_commitment,
        ) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected changed component-product-vector proof version".into()),
        };
        if changed_preimage.check(&ir).is_ok() {
            return Err("component-product-vector IR accepted changed result".into());
        }
    } else if !call.output.value.0.is_empty() || !call.output.alignment.0.is_empty() {
        return Err("empty result negative applicability differs".into());
    }
    write_new(
        root.join(format!("component-product-vector-{index}.proof")),
        &proof.0,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(
        root.join(format!("component-product-vector-{index}.preimage")),
        &bytes,
    )?;
    println!(
        "component-product-vector {index}: {}-byte proof verified; changed binding/result refused",
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
        rng: StdRng::seed_from_u64(0x0187_5052 + index as u64),
        resolver: &resolver,
        params: &params,
    };
    let proven = futures_executor::block_on(
        tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
    )?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0187_5345 + index as u64));
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
        return Err(format!("component-product-vector strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("component-product-vector contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("component-product-vector applied state differs".into());
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
            "component-product-vector replay refused incorrectly or mutated ledger: {result:?}"
        )
        .into());
    }
    write_new(
        root.join(format!("component-product-vector-{index}-transaction.bin")),
        &transaction_bytes,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(
        root.join(format!("component-product-vector-{index}-state.bin")),
        &bytes,
    )?;
    println!(
        "component-product-vector {index}: funded default-strict apply; replay refused without mutation"
    );
    Ok(transaction_bytes.len())
}
