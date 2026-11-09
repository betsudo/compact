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
fn small(v: &Value) -> r::BoundedUint<255> {
    r::BoundedUint::new(number(v).into()).unwrap()
}
fn large(v: &Value) -> r::BoundedUint<65535> {
    r::BoundedUint::new(number(v).into()).unwrap()
}
fn vector(v: &Value) -> r::FixedVector<r::Field, 2> {
    r::FixedVector([field(number(&v[0])), field(number(&v[1]))])
}
fn small_vector(v: &Value) -> r::FixedVector<r::BoundedUint<255>, 2> {
    r::FixedVector([small(&v[0]), small(&v[1])])
}
fn large_vector(v: &Value) -> r::FixedVector<r::BoundedUint<65535>, 2> {
    r::FixedVector([large(&v[0]), large(&v[1])])
}
fn small_pair(v: &Value) -> (r::BoundedUint<255>, r::BoundedUint<255>) {
    (small(&v[0]), small(&v[1]))
}
fn large_pair(v: &Value) -> (r::BoundedUint<65535>, r::BoundedUint<65535>) {
    (large(&v[0]), large(&v[1]))
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
// Keep each complete typed API case on a bounded test-worker frame.
macro_rules! check {
    ($sdk:ident, $case:ident, $row:ident, $values:expr, $right:expr $(, $extra:ident)?) => {{
        use $sdk::{ledger_contract as c, ledger_slots as slots};
        let expected = number(&$case["expected"]);
        let mut calls = 0;
        let mut replays = 0;
        let values = $values;
        let right = $right;
        let before = (values.clone(), right.clone() $(, $extra.clone())?);
        // Constructor arguments are consumed by the actual public API.
        // Callers explicitly clone when they intend to reuse owned input.
        let constructor = || {
            c::initial_state(
                r::context::ConstructorContext::new(42u64),
                values.clone(),
                right.clone(),
                $( $extra.clone(), )?
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
                let observed = &$row["entries"][stringify!($name)];
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
        assert_eq!($row["entries"]["read"]["result"], expected.to_string());
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
        replay(&recorded, &$row["entries"]["read"]);
        replay(&functional_recorded, &$row["entries"]["read"]);
        replays += 2;
        entry!(advance, 4, field(4));
        entry!(advance_other, 5, field(4));
        assert_eq!((values, right $(, $extra)?), before, "caller-owned input reuse");
    (calls, replays)
    }};
}

#[inline(never)]
fn run_aliased_vector_variable(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        aliased_vector_variable,
        case,
        row,
        r::FixedVector([
            small_vector(&case["args"][0][0]),
            small_vector(&case["args"][0][1])
        ]),
        large_vector(&case["args"][1])
    )
}
#[inline(never)]
fn run_asymmetric_eq(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        asymmetric_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        (small(&case["args"][1][0]), large(&case["args"][1][1]))
    )
}
#[inline(never)]
fn run_direct_tuple_variable(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        direct_tuple_variable,
        case,
        row,
        r::FixedVector([
            small_pair(&case["args"][0][0]),
            small_pair(&case["args"][0][1])
        ]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_direct_vector_variable(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        direct_vector_variable,
        case,
        row,
        r::FixedVector([
            small_vector(&case["args"][0][0]),
            small_vector(&case["args"][0][1])
        ]),
        large_vector(&case["args"][1])
    )
}
#[inline(never)]
fn run_flat_eq(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        flat_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_flat_ne(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        flat_ne,
        case,
        row,
        small_vector(&case["args"][0]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_hash_tuple_argument(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        hash_tuple_argument,
        case,
        row,
        small_vector(&case["args"][0]),
        r::FixedBytes::<32>(std::array::from_fn(|i| u8::try_from(number(
            &case["args"][1][i]
        ))
        .unwrap()))
    )
}
#[inline(never)]
fn run_mixed_literal_eq(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        mixed_literal_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_nested_eq(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        nested_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        (
            large(&case["args"][1][0]),
            (large(&case["args"][1][1][0]), large(&case["args"][1][1][1]))
        )
    )
}
#[inline(never)]
fn run_owned_capture_eq(case: &Value, row: &Value) -> (usize, usize) {
    let extra = vector(&case["args"][2]);
    check!(
        owned_capture_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        (large(&case["args"][1][0]), vector(&case["args"][1][1])),
        extra
    )
}
#[inline(never)]
fn run_reversed_eq(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        reversed_eq,
        case,
        row,
        small_vector(&case["args"][0]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_struct_literal_field(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        struct_literal_field,
        case,
        row,
        small_vector(&case["args"][0]),
        struct_literal_field::types::Wide {
            value: large(&case["args"][1]["value"])
        }
    )
}
#[inline(never)]
fn run_tuple_to_vector(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        tuple_to_vector,
        case,
        row,
        r::FixedVector([
            small_pair(&case["args"][0][0]),
            small_pair(&case["args"][0][1])
        ]),
        large_vector(&case["args"][1])
    )
}
#[inline(never)]
fn run_uint_to_field_tuple(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        uint_to_field_tuple,
        case,
        row,
        small_vector(&case["args"][0]),
        (
            field(number(&case["args"][1][0])),
            field(number(&case["args"][1][1]))
        )
    )
}
#[inline(never)]
fn run_vector_to_tuple(case: &Value, row: &Value) -> (usize, usize) {
    check!(
        vector_to_tuple,
        case,
        row,
        r::FixedVector([
            small_vector(&case["args"][0][0]),
            small_vector(&case["args"][0][1])
        ]),
        large_pair(&case["args"][1])
    )
}
#[inline(never)]
fn run_widened_vector_capture(case: &Value, row: &Value) -> (usize, usize) {
    let extra = small_vector(&case["args"][2]);
    check!(
        widened_vector_capture,
        case,
        row,
        small_vector(&case["args"][0]),
        (
            large(&case["args"][1][0]),
            large_vector(&case["args"][1][1])
        ),
        extra
    )
}

#[test]
fn constructors_match_independent_typescript_and_literal_branch_results() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("cases.json")).unwrap();
    let rows: Vec<Value> = serde_json::from_slice(
        &std::fs::read(std::env::var("COMPACT_CONSTRUCTOR_EQUALITY_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(cases.len(), 105);
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
        let (case_calls, case_replays) = match role {
            "aliased_vector_variable" => run_aliased_vector_variable(case, row),
            "asymmetric_eq" => run_asymmetric_eq(case, row),
            "direct_tuple_variable" => run_direct_tuple_variable(case, row),
            "direct_vector_variable" => run_direct_vector_variable(case, row),
            "flat_eq" => run_flat_eq(case, row),
            "flat_ne" => run_flat_ne(case, row),
            "hash_tuple_argument" => run_hash_tuple_argument(case, row),
            "mixed_literal_eq" => run_mixed_literal_eq(case, row),
            "nested_eq" => run_nested_eq(case, row),
            "owned_capture_eq" => run_owned_capture_eq(case, row),
            "reversed_eq" => run_reversed_eq(case, row),
            "struct_literal_field" => run_struct_literal_field(case, row),
            "tuple_to_vector" => run_tuple_to_vector(case, row),
            "uint_to_field_tuple" => run_uint_to_field_tuple(case, row),
            "vector_to_tuple" => run_vector_to_tuple(case, row),
            "widened_vector_capture" => run_widened_vector_capture(case, row),
            other => panic!("unknown case {other}"),
        };
        assert_eq!((case_calls, case_replays), (12, 6));
        calls += case_calls;
        replays += case_replays;
    }
    assert_eq!(calls, 1260);
    assert_eq!(replays, 630);
    println!("tuple-guard-cases:105 calls:{calls} replays:{replays}");
}
