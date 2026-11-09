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

use midnight_compact_runtime as r;
use serde_json::Value;
fn field(v: u64) -> r::Field {
    r::Field::from(v)
}
fn number(v: &Value) -> u64 {
    v.as_u64().unwrap()
}
fn bytes(v: &Value) -> r::FixedBytes<2> {
    r::FixedBytes([
        u8::try_from(number(&v[0])).unwrap(),
        u8::try_from(number(&v[1])).unwrap(),
    ])
}
fn tuple(v: &Value) -> (r::Field, r::FixedBytes<2>) {
    (field(number(&v[0])), bytes(&v[1]))
}
fn vector(v: &Value) -> r::FixedVector<r::Field, 2> {
    r::FixedVector([field(number(&v[0])), field(number(&v[1]))])
}
fn replay(out: &r::recording::RecordedCircuitResult<u64, r::Field>, row: &Value) {
    assert_eq!(
        serde_json::to_value(out.public.verify_ops()).unwrap(),
        row["publicTranscript"]
    );
    let observed = out
        .public
        .initial()
        .query(
            out.public.verify_ops(),
            None,
            &out.execution.context.cost_model,
        )
        .unwrap();
    assert_eq!(observed.context.state, out.execution.context.query.state);
    assert_eq!(
        observed.context.effects,
        out.execution.context.query.effects
    );
}
#[test]
fn constructors_match_independent_typescript_and_literal_branch_results() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("cases.json")).unwrap();
    let rows: Vec<Value> = serde_json::from_slice(
        &std::fs::read(std::env::var("COMPACT_CONSTRUCTOR_EQUALITY_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(cases.len(), 30);
    assert_eq!(rows.len(), cases.len());
    let mut ids = std::collections::BTreeSet::new();
    let mut calls = 0;
    let mut replays = 0;
    for (case, row) in cases.iter().zip(&rows) {
        let role = case["role"].as_str().unwrap();
        assert!(ids.insert((role, case["case"].as_str().unwrap())));
        assert_eq!(row["role"], case["role"]);
        assert_eq!(row["case"], case["case"]);
        let expected = number(&case["expected"]);
        assert!([7, 11].contains(&expected));
        assert_eq!(row["initial"], expected.to_string());
        macro_rules! check {
            ($sdk:ident, $values:expr, $right:expr) => {{
                use $sdk::{ledger_contract as c, ledger_slots as slots};
                let values = $values;
                let right = $right;
                let before = (values.clone(), right.clone());
                // Constructor arguments are consumed by the actual public API.
                // Callers explicitly clone when they intend to reuse owned input.
                let constructor = || {
                    c::initial_state(
                        r::context::ConstructorContext::new(42u64),
                        values.clone(),
                        right.clone(),
                    )
                    .unwrap()
                };
                let initial = constructor();
                assert_eq!(
                    slots::stored
                        .inspect(initial.ledger_state.get_ref())
                        .unwrap(),
                    field(expected)
                );
                assert_eq!(initial.private_state, 42);
                let context = || constructor().into_circuit_context(Default::default());
                macro_rules! entry {
                    ($name:ident, $increment:expr, $args:expr) => {{
                        let expected_after = expected + $increment;
                        let observed = &row["entries"][stringify!($name)];
                        assert_eq!(observed["result"], expected_after.to_string());
                        let native = c::$name(context(), $args).unwrap();
                        let recorded = c::recorded::$name(context(), $args).unwrap();
                        let functional = c::functional::$name::Program::new($args)
                            .execute_native(context())
                            .unwrap();
                        let functional_recorded = c::functional::$name::Program::new($args)
                            .execute_recorded(context())
                            .unwrap();
                        for actual in [
                            &native,
                            &recorded.execution,
                            &functional,
                            &functional_recorded.execution,
                        ] {
                            assert_eq!(actual.result, field(expected_after));
                            assert_eq!(
                                slots::stored
                                    .inspect(actual.context.query.state.get_ref())
                                    .unwrap(),
                                field(expected_after)
                            );
                            assert_eq!(actual.context.query.state, native.context.query.state);
                            assert_eq!(actual.context.query.effects, native.context.query.effects);
                            assert_eq!(actual.gas_cost, native.gas_cost);
                            assert_eq!(actual.context.private_state, 42);
                            assert!(actual.private_transcript_outputs.is_empty());
                            calls += 1;
                        }
                        replay(&recorded, observed);
                        replay(&functional_recorded, observed);
                        replays += 2;
                        assert_eq!(
                            recorded.public.verify_ops(),
                            functional_recorded.public.verify_ops()
                        );
                    }};
                }
                // Read has no argument: the dedicated arm preserves that API.
                let native = c::read(context()).unwrap();
                let recorded = c::recorded::read(context()).unwrap();
                let functional = c::functional::read::Program::new()
                    .execute_native(context())
                    .unwrap();
                let functional_recorded = c::functional::read::Program::new()
                    .execute_recorded(context())
                    .unwrap();
                assert_eq!(row["entries"]["read"]["result"], expected.to_string());
                for actual in [
                    &native,
                    &recorded.execution,
                    &functional,
                    &functional_recorded.execution,
                ] {
                    assert_eq!(actual.result, field(expected));
                    assert_eq!(
                        slots::stored
                            .inspect(actual.context.query.state.get_ref())
                            .unwrap(),
                        field(expected)
                    );
                    assert_eq!(actual.context.query.state, native.context.query.state);
                    assert_eq!(actual.context.query.effects, native.context.query.effects);
                    assert_eq!(actual.gas_cost, native.gas_cost);
                    assert_eq!(actual.context.private_state, 42);
                    assert!(actual.private_transcript_outputs.is_empty());
                    calls += 1;
                }
                replay(&recorded, &row["entries"]["read"]);
                replay(&functional_recorded, &row["entries"]["read"]);
                replays += 2;
                entry!(advance, 4, field(4));
                entry!(advance_other, 5, field(4));
                assert_eq!((values, right), before, "caller-owned input reuse");
            }};
        }
        let values = &case["args"][0];
        let right = &case["args"][1];
        macro_rules! mixed {
            ($sdk:ident) => {{
                check!(
                    $sdk,
                    r::FixedVector([
                        r::BoundedUint::<255>::new(number(&values[0]).into()).unwrap(),
                        r::BoundedUint::<255>::new(number(&values[1]).into()).unwrap()
                    ]),
                    r::BoundedUint::<65535>::new(number(right).into()).unwrap()
                );
            }};
        }
        macro_rules! tuples {
            ($sdk:ident) => {{
                check!(
                    $sdk,
                    r::FixedVector([tuple(&values[0]), tuple(&values[1])]),
                    tuple(right)
                );
            }};
        }
        macro_rules! vectors {
            ($sdk:ident) => {{
                check!(
                    $sdk,
                    r::FixedVector([vector(&values[0]), vector(&values[1])]),
                    vector(right)
                );
            }};
        }
        macro_rules! structs {
            ($sdk:ident) => {{
                let item = |v: &Value| $sdk::types::Item {
                    value: field(number(&v["value"])),
                    tag: bytes(&v["tag"]),
                };
                check!(
                    $sdk,
                    r::FixedVector([item(&values[0]), item(&values[1])]),
                    item(right)
                );
            }};
        }
        match role {
            "mixed_uint_eq" => mixed!(mixed_uint_eq),
            "mixed_uint_ne" => mixed!(mixed_uint_ne),
            "tuple_eq" => tuples!(tuple_eq),
            "tuple_ne" => tuples!(tuple_ne),
            "vector_eq" => vectors!(vector_eq),
            "vector_ne" => vectors!(vector_ne),
            "struct_eq" => structs!(struct_eq),
            "struct_ne" => structs!(struct_ne),
            other => panic!("unknown case {other}"),
        }
    }
    assert_eq!(calls, 360);
    assert_eq!(replays, 180);
    println!("boundary-cases:30 calls:{calls} replays:{replays}");
}
