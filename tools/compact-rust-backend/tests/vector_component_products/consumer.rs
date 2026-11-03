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
use proof_candidate::runtime as r;
use r::{BoundedUint, Field, FixedVector};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_VECTOR_COMPONENT_PRODUCTS_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "vector-component-products-ts/v1");
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
macro_rules! pure_case {
 ($id:ident,$module:ident,[$($arg:expr),*],$expected:ty)=>{#[test] fn $id(){let result:Result<$expected,r::CompactError>=$module::pure_circuits::value($($arg),*);check(result,&row(stringify!($id)));}};
}
macro_rules! stateful_case {
    ($id:ident,$module:ident,$entry:ident,[$($ctor:expr),*],[$($arg:expr),*],$expected:ty)=>{
        #[test] fn $id(){
            use $module::ledger_contract as api;
            let row=row(stringify!($id));
            let mut observed = None;
            for lane in 0..4 {
                let ctx=api::initial_state(r::context::ConstructorContext::new(7_u64),$($ctor),*).unwrap().into_circuit_context(Default::default());
                let held=ctx.query.state.clone();
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
                assert_eq!($module::ledger_slots::seen.inspect(held.get_ref()).unwrap(),Field::from(row["initialSeen"].as_str().unwrap().parse::<u64>().unwrap()));
            }
        }
    };
}

type Narrow = FixedVector<
    (
        FixedVector<(BoundedUint<255>, BoundedUint<255>), 2>,
        BoundedUint<65535>,
    ),
    2,
>;
fn pair<const M: u128>(value: u128) -> (BoundedUint<M>, BoundedUint<M>) {
    (u(value), u((value + 17) % 256))
}
fn input() -> Narrow {
    FixedVector::new([
        (FixedVector::new([pair(3), pair(5)]), u(31)),
        (FixedVector::new([pair(4), pair(6)]), u(41)),
    ])
}
fn second() -> Narrow {
    FixedVector::new([
        (FixedVector::new([pair(7), pair(9)]), u(71)),
        (FixedVector::new([pair(8), pair(10)]), u(81)),
    ])
}
stateful_case!(aliases, aliases, run, [], [input()], Wide);
stateful_case!(
    boolean,
    boolean,
    run,
    [],
    [FixedVector::new([
        (FixedVector::new([pair(3), pair(5)]), true),
        (FixedVector::new([pair(4), pair(6)]), false)
    ])],
    FixedVector<
        (
            FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
            bool
        ),
        2,
    >
);
stateful_case!(
    field,
    field,
    run,
    [],
    [input()],
    FixedVector<(FixedVector<(Field, Field), 2>, Field), 2>
);
stateful_case!(helper, helper, run, [], [input()], Wide);
stateful_case!(
    identity_control,
    identity_control,
    run,
    [],
    [input()],
    Narrow
);
stateful_case!(local, local, run, [], [input()], Wide);
stateful_case!(
    nested_outer,
    nested_outer,
    run,
    [],
    [FixedVector::new([input(), second()])], FixedVector<Wide,2>);
stateful_case!(parameter, parameter, run, [], [input()], Wide);
stateful_case!(
    parameter_boundary,
    parameter,
    run,
    [],
    [FixedVector::new([
        (FixedVector::new([pair(255), pair(0)]), u(65535)),
        (FixedVector::new([pair(1), pair(254)]), u(65534))
    ])],
    Wide
);
stateful_case!(
    prior_product_control,
    prior_product_control,
    run,
    [],
    [FixedVector::new([(u(3), u(31)), (u(4), u(41))])],
    FixedVector<(BoundedUint<65535>, BoundedUint<4294967295>), 2>
);
stateful_case!(
    proof_keep,
    proof_candidate,
    run,
    [Field::from(0_u64)],
    [input()],
    Wide
);
stateful_case!(
    proof_flip,
    proof_candidate,
    run,
    [Field::from(1_u64)],
    [input()],
    Wide
);
stateful_case!(
    proof_field,
    proof_candidate,
    run_field,
    [Field::from(0_u64)],
    [input()],
    FixedVector<(FixedVector<(Field, Field), 2>, Field), 2>
);
pure_case!(pure_control, pure_control, [input()], Wide);
stateful_case!(component_alias, component_alias, run, [], [input()], Wide);
stateful_case!(
    component_2d,
    component_2d,
    run,
    [],
    [FixedVector::new([
        (
            FixedVector::new([
                FixedVector::new([pair(3), pair(5)]),
                FixedVector::new([pair(7), pair(9)])
            ]),
            u(31)
        ),
        (
            FixedVector::new([
                FixedVector::new([pair(4), pair(6)]),
                FixedVector::new([pair(8), pair(10)])
            ]),
            u(41)
        )
    ])],
    FixedVector<
        (
            FixedVector<FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>, 2>,
            BoundedUint<4294967295>
        ),
        2,
    >
);
stateful_case!(
    component_boolean,
    component_boolean,
    run,
    [],
    [FixedVector::new([
        (FixedVector::new([(true, false), (false, true)]), u(3)),
        (FixedVector::new([(false, true), (true, false)]), u(4))
    ])],
    FixedVector<(FixedVector<(bool, bool), 2>, BoundedUint<65535>), 2>
);
stateful_case!(
    component_field,
    component_field,
    run,
    [],
    [FixedVector::new([
        (
            FixedVector::new([
                (Field::from(3_u64), Field::from(20_u64)),
                (Field::from(5_u64), Field::from(22_u64))
            ]),
            u(3)
        ),
        (
            FixedVector::new([
                (Field::from(4_u64), Field::from(21_u64)),
                (Field::from(6_u64), Field::from(23_u64))
            ]),
            u(4)
        )
    ])],
    FixedVector<(FixedVector<(Field, Field), 2>, BoundedUint<65535>), 2>
);

stateful_case!(
    empty_component,
    empty_component,
    run,
    [],
    [FixedVector::<
        (
            FixedVector<(BoundedUint<255>, BoundedUint<255>), 2>,
            BoundedUint<65535>
        ),
        0,
    >::new([])],
    FixedVector<
        (
            FixedVector<(BoundedUint<255>, BoundedUint<255>), 2>,
            BoundedUint<65535>
        ),
        0,
    >
);

struct VectorWitness {
    expected_seen: u64,
    calls: std::cell::RefCell<Vec<Value>>,
}
impl VectorWitness {
    fn record(&self, before: u64, seen: Field) -> (u64, Narrow) {
        assert_eq!(
            seen,
            Field::from(self.expected_seen),
            "initializer witness order"
        );
        let raw = [
            (
                [[before - 4, before + 13], [before - 2, before + 15]],
                31_u64,
            ),
            (
                [[before - 3, before + 14], [before - 1, before + 16]],
                41_u64,
            ),
        ];
        self.calls.borrow_mut().push(json!({"privateState":before,
            "value":raw.iter().map(|v|json!([v.0.map(|p|p.map(|n|n.to_string())),v.1.to_string()])).collect::<Vec<_>>(),
            "seen":encode(seen.into())}));
        (
            before + 1,
            FixedVector::new(raw.map(|v| {
                (
                    FixedVector::new(v.0.map(|p| (u(u128::from(p[0])), u(u128::from(p[1]))))),
                    u(u128::from(v.1)),
                )
            })),
        )
    }
}
macro_rules! witness_case {
    ($module:ident) => {
        impl $module::ledger_contract::Witnesses<u64> for VectorWitness {
            fn vector_value(
                &self,
                ctx: r::context::WitnessContext<'_, u64, $module::ledger_contract::LedgerView<'_>>,
            ) -> (u64, Narrow) {
                self.record(*ctx.private_state, ctx.ledger.seen().unwrap())
            }
        }
        #[test]
        fn $module() {
            use $module::ledger_contract as api;
            let row = row(stringify!($module));
            let mut observed = None;
            for lane in 0..4 {
                let witness = VectorWitness {
                    expected_seen: row["witnessSeen"].as_str().unwrap().parse().unwrap(),
                    calls: Default::default(),
                };
                let ctx = api::initial_state(r::context::ConstructorContext::new(7_u64))
                    .unwrap()
                    .into_circuit_context(Default::default());
                let held = ctx.query.state.clone();
                let out: r::context::CircuitResult<u64, Wide> = match lane {
                    0 => api::run(ctx, &witness),
                    1 => api::recorded::run(ctx, &witness).map(|out| replay(out, &row)),
                    2 => api::functional::run::Program::new().execute_native(ctx, &witness),
                    3 => api::functional::run::Program::new()
                        .execute_recorded(ctx, &witness)
                        .map(|out| replay(out, &row)),
                    _ => unreachable!(),
                }
                .unwrap();
                check(Ok(out.result), &row);
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
                    *witness.calls.borrow(),
                    row["observedWitnesses"].as_array().unwrap().clone()
                );
                assert_eq!(
                    $module::ledger_slots::seen
                        .inspect(out.context.query.state.get_ref())
                        .unwrap(),
                    Field::from(row["seen"].as_str().unwrap().parse::<u64>().unwrap())
                );
                assert_eq!(
                    $module::ledger_slots::seen.inspect(held.get_ref()).unwrap(),
                    Field::from(row["initialSeen"].as_str().unwrap().parse::<u64>().unwrap())
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
            }
        }
    };
}
witness_case!(witness_before);
witness_case!(witness_after);

// The helper narrows original wide input before returning a value widened by its caller.
// Both invalid components are supplied together to make first-error ordering observable.
type Wide = FixedVector<
    (
        FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
        BoundedUint<4294967295>,
    ),
    2,
>;
fn checked_input(inner: u128, scalar: u128) -> Wide {
    FixedVector::new([
        (FixedVector::new([pair(3), pair(5)]), u(31)),
        (FixedVector::new([pair(inner), pair(6)]), u(scalar)),
    ])
}
stateful_case!(
    checked_helper_valid,
    checked_helper,
    run,
    [],
    [true, checked_input(4, 41)],
    Wide
);
stateful_case!(
    checked_helper_assert_first,
    checked_helper,
    run,
    [],
    [false, checked_input(300, 100000)],
    Wide
);
stateful_case!(
    checked_helper_inner_first,
    checked_helper,
    run,
    [],
    [true, checked_input(300, 100000)],
    Wide
);
stateful_case!(
    checked_helper_scalar,
    checked_helper,
    run,
    [],
    [true, checked_input(4, 100000)],
    Wide
);

// Failure consumes the context without a returned execution/proof result. This is not
// transaction rollback; the external witness journal deliberately remains observable.
struct RefusingWitness {
    calls: std::cell::RefCell<Vec<Value>>,
}
macro_rules! witness_error_case {
    ($id:ident, $module:ident) => {
        impl $module::ledger_contract::TryWitnesses<u64> for RefusingWitness {
            fn vector_value(
                &self,
                ctx: r::context::WitnessContext<'_, u64, $module::ledger_contract::LedgerView<'_>>,
            ) -> Result<(u64, Narrow), r::CompactError> {
                self.calls.borrow_mut().push(json!({
                    "privateState": *ctx.private_state,
                    "seen": encode(ctx.ledger.seen()?.into())
                }));
                Err(r::CompactError::AssertionFailed("witness refused".into()))
            }
        }
        #[test]
        fn $id() {
            use $module::ledger_contract as api;
            let row = row(stringify!($id));
            for lane in 0..4 {
                let witness = RefusingWitness { calls: Default::default() };
                let ctx = api::initial_state(r::context::ConstructorContext::new(7_u64))
                    .unwrap().into_circuit_context(Default::default());
                let held = ctx.query.state.clone();
                let result: Result<r::context::CircuitResult<u64,Wide>,r::CompactError> = match lane {
                    0 => api::run(ctx, &witness),
                    1 => api::recorded::run(ctx, &witness).map(|out| replay(out, &row)),
                    2 => api::functional::run::Program::new().execute_native(ctx, &witness),
                    3 => api::functional::run::Program::new().execute_recorded(ctx, &witness).map(|out| replay(out, &row)),
                    _ => unreachable!(),
                };
                check(result.map(|out| out.result), &row);
                let calls = witness.calls.borrow();
                assert_eq!(*calls, vec![json!({"privateState":7,"seen":encode(Field::from(row["witnessSeen"].as_str().unwrap().parse::<u64>().unwrap()).into())})]);
                assert_eq!(*calls, row["observedWitnesses"].as_array().unwrap().clone());
                assert_eq!($module::ledger_slots::seen.inspect(held.get_ref()).unwrap(), Field::from(0_u64));
            }
        }
    };
}
witness_error_case!(witness_before_error, witness_before);
witness_error_case!(witness_after_error, witness_after);

stateful_case!(
    original_regression,
    original_regression,
    run,
    [],
    [FixedVector::new([
        (FixedVector::new([pair(3), pair(5)]), u(31)),
        (FixedVector::new([pair(4), pair(6)]), u(41))
    ])],
    FixedVector<
        (
            FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
            BoundedUint<65535>
        ),
        2,
    >
);

stateful_case!(
    inner_tuple_alias,
    inner_tuple_alias,
    run,
    [],
    [input()],
    Wide
);
fn second_boundary() -> Narrow {
    FixedVector::new([
        (FixedVector::new([(u(0), u(255)), (u(255), u(0))]), u(65535)),
        (
            FixedVector::new([(u(254), u(255)), (u(1), u(254))]),
            u(65534),
        ),
    ])
}
stateful_case!(
    parameter_second_boundary,
    parameter,
    run,
    [],
    [second_boundary()],
    Wide
);
stateful_case!(
    proof_field_boundary,
    proof_candidate,
    run_field,
    [Field::from(0_u64)],
    [second_boundary()],
    FixedVector<(FixedVector<(Field, Field), 2>, Field), 2>
);
stateful_case!(
    component_field_large,
    component_field,
    run,
    [],
    [FixedVector::new([
        (
            FixedVector::new([
                (Field::from(u128::MAX), Field::from(0_u64)),
                (Field::from(1_u64), Field::from(u128::MAX - 1))
            ]),
            u(255)
        ),
        (
            FixedVector::new([
                (Field::from(2_u64), Field::from(3_u64)),
                (Field::from(4_u64), Field::from(5_u64))
            ]),
            u(254)
        )
    ])],
    FixedVector<(FixedVector<(Field, Field), 2>, BoundedUint<65535>), 2>
);
