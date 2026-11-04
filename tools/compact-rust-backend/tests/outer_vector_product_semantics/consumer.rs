// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(test)]
use family::runtime as r;
use r::{BoundedUint, Field, FixedVector};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_OUTER_VECTOR_PRODUCT_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "outer-vector-product-widening-ts/v1");
    ts["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap()
        .clone()
}
fn encode(value: r::fab::AlignedValue) -> Value {
    json!({"valueAtoms":value.value.0.iter().map(|x|&x.0).collect::<Vec<_>>(),"alignment":value.alignment})
}
fn check<T: Into<r::fab::AlignedValue>>(result: Result<T, r::CompactError>, row: &Value) {
    if row["error"].is_null() {
        assert_eq!(encode(result.unwrap().into()), row["result"]);
    } else {
        let error = &row["error"];
        let expected = match error["kind"].as_str().unwrap() {
            "unsigned" => r::CompactError::UnsignedOutOfRange {
                value: error["value"].as_str().unwrap().parse().unwrap(),
                max: error["max"].as_str().unwrap().parse().unwrap(),
            },
            "assert" | "witness" => {
                r::CompactError::AssertionFailed(error["message"].as_str().unwrap().into())
            }
            _ => panic!("unknown independently declared error"),
        };
        assert_eq!(
            result.err().expect("expected actual execution refusal"),
            expected
        );
    }
}
fn u<const M: u128>(value: u128) -> BoundedUint<M> {
    BoundedUint::new(value).unwrap()
}
fn replay<T>(
    out: r::recording::RecordedCircuitResult<u64, T>,
    row: &Value,
) -> r::context::CircuitResult<u64, T> {
    assert_eq!(
        serde_json::to_value(out.public.verify_ops()).unwrap(),
        row["publicTranscript"],
        "complete public transcript differs from TS"
    );
    let replay = out
        .public
        .initial()
        .query(
            out.public.verify_ops(),
            None,
            &out.execution.context.cost_model,
        )
        .unwrap();
    assert_eq!(replay.context.state, out.execution.context.query.state);
    assert_eq!(replay.context.effects, out.execution.context.query.effects);
    out.execution
}
macro_rules! stateful_case {
    ($id:ident,$module:ident,$entry:ident,[$($ctor:expr),*],[$($arg:expr),*],$expected:ty)=>{
        #[test] fn $id(){
            use $module::ledger_contract as api;
            let row=row(stringify!($id));
            let mut observed = None;
            for lane in 0..4 {
                let ctx=api::initial_state(r::context::ConstructorContext::new(7_u64),$($ctor),*).unwrap().into_circuit_context(Default::default());
                assert_eq!($module::ledger_slots::seen.inspect(ctx.query.state.get_ref()).unwrap(),Field::from(row["initialSeen"].as_str().unwrap().parse::<u64>().unwrap()));
                let supplied:Vec<Value>=vec![$(encode(r::fab::AlignedValue::from($arg))),*];
                assert_eq!(supplied,row["argumentValues"].as_array().unwrap().clone(),"Rust inputs differ from declared TS arguments");
                let result: Result<r::context::CircuitResult<u64,$expected>,r::CompactError>=match lane {
                    0=>api::$entry(ctx,$($arg),*),
                    1=>api::recorded::$entry(ctx,$($arg),*).map(|out|replay(out,&row)),
                    2=>api::functional::$entry::Program::new($($arg),*).execute_native(ctx),
                    3=>api::functional::$entry::Program::new($($arg),*).execute_recorded(ctx).map(|out|replay(out,&row)),
                    _=>unreachable!()
                };
                check(result.map(|out|{
                    assert_eq!($module::ledger_slots::seen.inspect(out.context.query.state.get_ref()).unwrap(),Field::from(row["seen"].as_str().unwrap().parse::<u64>().unwrap()));
                    assert_eq!(out.context.private_state,7);
                    assert!(out.private_transcript_outputs.is_empty());
                    let state=(out.context.query.state.clone(),out.context.query.effects.clone(),out.gas_cost);
                    if let Some(before)=&observed {assert_eq!(&state,before);} else {observed=Some(state);}
                    out.result
                }),&row);
            }
        }
    };
}

type U8 = BoundedUint<255>;
type U16 = BoundedUint<65535>;
type U32 = BoundedUint<4294967295>;
type NarrowProduct = ((U8,), U8);
type WideProduct = ((U16,), U16);
type Narrow = FixedVector<NarrowProduct, 2>;
type Wide = FixedVector<WideProduct, 2>;
fn narrow(a: u128, b: u128, c: u128, d: u128) -> Narrow {
    FixedVector::new([((u(a),), u(b)), ((u(c),), u(d))])
}
fn wide(a: u128, b: u128, c: u128, d: u128) -> Wide {
    FixedVector::new([((u(a),), u(b)), ((u(c),), u(d))])
}
stateful_case!(original,original,run,[],[FixedVector::<NarrowProduct,1>::new([((u(3),),u(255))])],FixedVector<WideProduct,1>);
stateful_case!(
    parameter,
    family,
    parameter,
    [],
    [narrow(3, 128, 255, 219)],
    Wide
);
stateful_case!(
    parameter_boundaries,
    family,
    parameter,
    [],
    [narrow(0, 255, 255, 0)],
    Wide
);
stateful_case!(helper, family, helper, [], [narrow(3, 128, 255, 219)], Wide);
stateful_case!(local, family, local, [], [narrow(3, 128, 255, 219)], Wide);
stateful_case!(
    checked_valid,
    family,
    checked_helper,
    [],
    [true, wide(3, 128, 255, 219)],
    Wide
);
stateful_case!(
    checked_assert_first,
    family,
    checked_helper,
    [],
    [false, wide(256, 257, 258, 259)],
    Wide
);
stateful_case!(
    checked_first_leaf,
    family,
    checked_helper,
    [],
    [true, wide(256, 257, 258, 259)],
    Wide
);
stateful_case!(
    checked_second_leaf,
    family,
    checked_helper,
    [],
    [true, wide(3, 257, 258, 259)],
    Wide
);
stateful_case!(
    checked_third_leaf,
    family,
    checked_helper,
    [],
    [true, wide(3, 128, 258, 259)],
    Wide
);
stateful_case!(
    checked_fourth_leaf,
    family,
    checked_helper,
    [],
    [true, wide(3, 128, 255, 259)],
    Wide
);
stateful_case!(
    branch_left,
    family,
    branch,
    [],
    [true, narrow(3, 128, 255, 219), narrow(17, 255, 0, 254)],
    Wide
);
stateful_case!(
    branch_right,
    family,
    branch,
    [],
    [false, narrow(3, 128, 255, 219), narrow(17, 255, 0, 254)],
    Wide
);
type DeepInput = FixedVector<((FixedVector<((U8,), U8), 1>,), U16), 1>;
type DeepOutput = FixedVector<((FixedVector<((U16,), U16), 1>,), U32), 1>;
fn deep_input() -> DeepInput {
    FixedVector::new([((FixedVector::new([((u(3),), u(255))]),), u(65535))])
}
stateful_case!(deep, family, deep, [], [deep_input()], DeepOutput);
type BoolInput = FixedVector<((bool, U8), bool), 1>;
type BoolOutput = FixedVector<((bool, Field), bool), 1>;
fn bool_input(a: bool, b: u128, c: bool) -> BoolInput {
    FixedVector::new([((a, u(b)), c)])
}
stateful_case!(
    boolean_field,
    family,
    boolean_field,
    [],
    [bool_input(true, 255, false)],
    BoolOutput
);
stateful_case!(
    boolean_field_other,
    family,
    boolean_field,
    [],
    [bool_input(false, 0, true)],
    BoolOutput
);
stateful_case!(empty,family,empty,[],[FixedVector::<NarrowProduct,0>::new([])],FixedVector<WideProduct,0>);
type EmptyTupleInput = FixedVector<(((), U8), U8), 1>;
type EmptyTupleOutput = FixedVector<(((), U16), U16), 1>;
fn empty_tuple_input() -> EmptyTupleInput {
    FixedVector::new([(((), u(255)), u(219))])
}
stateful_case!(
    tuple_empty,
    family,
    tuple_empty,
    [],
    [empty_tuple_input()],
    EmptyTupleOutput
);
type SingleTupleInput = FixedVector<(((U8,),), U8), 1>;
type SingleTupleOutput = FixedVector<(((U16,),), U16), 1>;
fn single_tuple_input() -> SingleTupleInput {
    FixedVector::new([(((u(3),),), u(255))])
}
stateful_case!(
    tuple_single,
    family,
    tuple_single,
    [],
    [single_tuple_input()],
    SingleTupleOutput
);
type FieldInput = FixedVector<((Field, U8), U8), 1>;
type FieldOutput = FixedVector<((Field, U16), U16), 1>;
fn field_input() -> FieldInput {
    let large = Field::from(1_u128 << 127) * Field::from(8_u64) + Field::from(21_u64);
    FixedVector::new([((large, u(17)), u(255))])
}
stateful_case!(
    field_identity,
    family,
    field_identity,
    [],
    [field_input()],
    FieldOutput
);
type EmptyLeafInput = FixedVector<((FixedVector<U8, 0>, U8), U8), 1>;
type EmptyLeafOutput = FixedVector<((FixedVector<U16, 0>, U16), U16), 1>;
fn empty_leaf_input() -> EmptyLeafInput {
    FixedVector::new([((FixedVector::new([]), u(255)), u(219))])
}
stateful_case!(
    empty_leaf,
    family,
    empty_leaf,
    [],
    [empty_leaf_input()],
    EmptyLeafOutput
);
type LastInput = FixedVector<(U8, (U8, U8)), 1>;
type LastOutput = FixedVector<(U16, (U16, U16)), 1>;
fn last_input() -> LastInput {
    FixedVector::new([(u(3), (u(128), u(255)))])
}
stateful_case!(
    last_child,
    family,
    last_child,
    [],
    [last_input()],
    LastOutput
);
struct VectorWitness {
    expected_seen: u64,
    fail: bool,
    calls: std::cell::RefCell<Vec<Value>>,
}
impl family::ledger_contract::TryWitnesses<u64> for VectorWitness {
    fn product_value(
        &self,
        ctx: r::context::WitnessContext<'_, u64, family::ledger_contract::LedgerView<'_>>,
    ) -> Result<(u64, Narrow), r::CompactError> {
        let before = *ctx.private_state;
        let seen = ctx.ledger.seen()?;
        assert_eq!(
            seen,
            Field::from(self.expected_seen),
            "nested witness order"
        );
        if self.fail {
            self.calls
                .borrow_mut()
                .push(json!({"privateState":before,"seen":encode(seen.into())}));
            return Err(r::CompactError::AssertionFailed(
                "outer Vector product witness denied".into(),
            ));
        }
        let raw = json!([
            [[(before - 4).to_string()], (before + 121).to_string()],
            [[(before + 248).to_string()], (before + 212).to_string()]
        ]);
        self.calls
            .borrow_mut()
            .push(json!({"privateState":before,"value":raw,"seen":encode(seen.into())}));
        Ok((
            before + 1,
            narrow(
                u128::from(before - 4),
                u128::from(before + 121),
                u128::from(before + 248),
                u128::from(before + 212),
            ),
        ))
    }
}
macro_rules! witness_case {
    ($id:ident,$entry:ident,$fail:expr) => {
        #[test]
        fn $id() {
            use family::ledger_contract as api;
            let row = row(stringify!($id));
            let mut observed = None;
            for lane in 0..4 {
                let witness = VectorWitness {
                    expected_seen: row["witnessSeen"].as_str().unwrap().parse().unwrap(),
                    fail: $fail,
                    calls: Default::default(),
                };
                let ctx = api::initial_state(r::context::ConstructorContext::new(7_u64))
                    .unwrap()
                    .into_circuit_context(Default::default());
                assert_eq!(
                    family::ledger_slots::seen
                        .inspect(ctx.query.state.get_ref())
                        .unwrap(),
                    Field::from(0_u64)
                );
                assert_eq!(row["argumentValues"], json!([]));
                let result: Result<r::context::CircuitResult<u64, Wide>, r::CompactError> =
                    match lane {
                        0 => api::$entry(ctx, &witness),
                        1 => api::recorded::$entry(ctx, &witness).map(|out| replay(out, &row)),
                        2 => api::functional::$entry::Program::new().execute_native(ctx, &witness),
                        3 => api::functional::$entry::Program::new()
                            .execute_recorded(ctx, &witness)
                            .map(|out| replay(out, &row)),
                        _ => unreachable!(),
                    };
                check(
                    result.map(|out| {
                        assert_eq!(
                            out.context.private_state,
                            row["privateState"].as_u64().unwrap()
                        );
                        assert_eq!(
                            out.private_transcript_outputs
                                .into_iter()
                                .map(encode)
                                .collect::<Vec<_>>(),
                            row["privateOutputs"].as_array().unwrap().clone()
                        );
                        assert_eq!(
                            family::ledger_slots::seen
                                .inspect(out.context.query.state.get_ref())
                                .unwrap(),
                            Field::from(row["seen"].as_str().unwrap().parse::<u64>().unwrap())
                        );
                        let state = (
                            out.context.query.state,
                            out.context.query.effects,
                            out.gas_cost,
                        );
                        if let Some(before) = &observed {
                            assert_eq!(&state, before);
                        } else {
                            observed = Some(state);
                        }
                        out.result
                    }),
                    &row,
                );
                assert_eq!(
                    *witness.calls.borrow(),
                    row["observedWitnesses"].as_array().unwrap().clone()
                );
            }
        }
    };
}
witness_case!(witness_before, witness_before, false);
witness_case!(witness_after, witness_after, false);
witness_case!(witness_before_error, witness_before, true);
witness_case!(witness_after_error, witness_after, true);
