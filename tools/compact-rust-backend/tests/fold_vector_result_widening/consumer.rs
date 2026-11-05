// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]
use helper::{ledger_contract as api, runtime as r};
use r::{BoundedUint, Field, FixedVector, WideUint};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_FOLD_VECTOR_RESULT_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "fold-vector-result-widening-ts/v1");
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
        let error = if row["nativeError"].is_null() {
            &row["error"]
        } else {
            &row["nativeError"]
        };
        let expected = match error["kind"].as_str().unwrap() {
            "unsigned" => r::CompactError::UnsignedOutOfRange {
                value: error["value"].as_str().unwrap().parse().unwrap(),
                max: error["max"].as_str().unwrap().parse().unwrap(),
            },
            "assert" => r::CompactError::AssertionFailed(error["message"].as_str().unwrap().into()),
            _ => panic!("unknown independently declared error"),
        };
        assert_eq!(result.err().expect("expected callback refusal"), expected);
    }
}
fn u<const M: u128>(value: u128) -> BoundedUint<M> {
    BoundedUint::new(value).unwrap()
}
fn vector<const N: usize>(values: [u128; N]) -> FixedVector<BoundedUint<255>, N> {
    FixedVector::new(values.map(u))
}
fn wide_values() -> FixedVector<WideUint<1, { u128::MAX }>, 2> {
    let mut low = [0_u8; 17];
    low[16] = 1;
    FixedVector::new([
        WideUint::from_le_bytes(&low).unwrap(),
        WideUint::from_le_bytes(&[255_u8; 16].into_iter().chain([1]).collect::<Vec<_>>()).unwrap(),
    ])
}
macro_rules! pure_case {
    ($id:ident,$source:ident,[$($arg:expr),*])=>{
        #[test] fn $id(){check($source::pure_circuits::value($($arg),*),&row(stringify!($id)));}
    };
}
pure_case!(small, small, [vector([2, 3]), vector([9, 8])]);
pure_case!(small_bounds, small, [vector([0, 255]), vector([254, 253])]);
pure_case!(zero, zero, [vector([]), vector([0, 255])]);
pure_case!(single, single, [vector([7]), vector([1, 2])]);
pure_case!(field, field, [vector([0, 255]), vector([254, 253])]);
pure_case!(wide, wide, [vector([0, 255]), vector([254, 253])]);
pure_case!(
    wide_to_wide,
    wide_to_wide,
    [
        wide_values(),
        r::FixedVector::new([
            r::WideUint::from_le_bytes(&[0]).unwrap(),
            r::WideUint::from_le_bytes(&[1]).unwrap()
        ])
    ]
);
pure_case!(checked, checked, [vector([1, 2]), vector([0, 0])]);
pure_case!(checked_first, checked, [vector([255, 1]), vector([1, 0])]);
pure_case!(checked_second, checked, [vector([255, 1]), vector([0, 0])]);
pure_case!(
    nested,
    nested,
    [
        vector([2, 3]),
        FixedVector::new([vector([9, 8]), vector([7, 6])])
    ]
);
pure_case!(
    product,
    product,
    [
        vector([2, 3]),
        FixedVector::new([(u(9), u(65535)), (u(8), u(42))])
    ]
);
pure_case!(aliased, aliased, [vector([2, 3]), vector([9, 8])]);
pure_case!(
    alias_bool,
    alias_bool,
    [
        FixedVector::new([true, false]),
        FixedVector::new([false, false])
    ]
);
pure_case!(local, local_control, [vector([2, 3]), vector([9, 8])]);
pure_case!(
    selected_true,
    selected_control,
    [true, vector([2, 3]), vector([9, 8])]
);
pure_case!(
    selected_false,
    selected_control,
    [false, vector([2, 3]), vector([9, 8])]
);
pure_case!(scalar, scalar_control, [vector([2, 3])]);
pure_case!(scalar_overflow, scalar_control, [vector([255, 1])]);
pure_case!(
    source_ok,
    source_checked,
    [true, true, vector([2, 3]), vector([9, 8])]
);
pure_case!(
    seed_denied,
    source_checked,
    [false, true, vector([7, 7]), vector([9, 8])]
);
pure_case!(
    items_denied,
    source_checked,
    [true, false, vector([7, 7]), vector([9, 8])]
);
pure_case!(
    both_denied,
    source_checked,
    [false, false, vector([7, 7]), vector([9, 8])]
);
pure_case!(
    first_item_denied,
    source_checked,
    [true, true, vector([7, 2]), vector([9, 8])]
);
pure_case!(
    second_item_denied,
    source_checked,
    [true, true, vector([2, 7]), vector([9, 8])]
);
pure_case!(
    order_control_ok,
    source_checked_control,
    [true, true, vector([2, 3]), vector([9, 8])]
);
pure_case!(
    order_control_both_denied,
    source_checked_control,
    [false, false, vector([7, 7]), vector([9, 8])]
);

pure_case!(alias_cast, alias_cast, [vector([0, 255])]);

fn replay<T>(
    out: r::recording::RecordedCircuitResult<u64, T>,
) -> r::context::CircuitResult<u64, T> {
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
    ($id:ident,$entry:ident,[$($arg:expr),*])=>{
        #[test] fn $id(){
            let row=row(stringify!($id));
            for lane in 0..4 {
                let ctx=api::initial_state(r::context::ConstructorContext::new(7_u64)).unwrap().into_circuit_context(Default::default());
                let held=ctx.query.state.clone();
                let result=match lane {
                    0=>api::$entry(ctx,$($arg),*),
                    1=>api::recorded::$entry(ctx,$($arg),*).map(replay),
                    2=>api::functional::$entry::Program::new($($arg),*).execute_native(ctx),
                    3=>api::functional::$entry::Program::new($($arg),*).execute_recorded(ctx).map(replay),
                    _=>unreachable!()
                };
                check(result.map(|out|{
                    assert_eq!(helper::ledger_slots::seen.inspect(out.context.query.state.get_ref()).unwrap(),Field::from(row["seen"].as_str().unwrap().parse::<u64>().unwrap()));
                    assert_eq!(out.context.private_state,7);
                    assert!(out.private_transcript_outputs.is_empty());
                    assert_eq!(out.context.query.effects,Default::default());
                    out.result
                }),&row);
                assert_eq!(helper::ledger_slots::seen.inspect(held.get_ref()).unwrap(),Field::from(0_u64));
            }
        }
    };
}
stateful_case!(run, run, [vector([2, 3]), vector([9, 8])]);
stateful_case!(run_bounds, run, [vector([0, 255]), vector([254, 253])]);
stateful_case!(run_checked, run_checked, [vector([1, 2]), vector([0, 0])]);
stateful_case!(
    run_checked_first,
    run_checked,
    [vector([255, 1]), vector([1, 0])]
);
stateful_case!(
    run_checked_second,
    run_checked,
    [vector([255, 1]), vector([0, 0])]
);

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
