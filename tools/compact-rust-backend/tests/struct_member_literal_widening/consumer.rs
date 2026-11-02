// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]
use helper::{ledger_contract as api, runtime as r};
use r::{BoundedUint, Field, FixedVector, WideUint};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_STRUCT_MEMBER_LITERAL_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "struct-member-literal-widening-ts/v1");
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
            "assert" => r::CompactError::AssertionFailed(error["message"].as_str().unwrap().into()),
            _ => panic!("unknown independently declared error"),
        };
        assert_eq!(result.err().expect("expected callback refusal"), expected);
    }
}
fn u<const M: u128>(value: u128) -> BoundedUint<M> {
    BoundedUint::new(value).unwrap()
}
fn wide(low: u128, high: u8) -> WideUint<1, { u128::MAX }> {
    let mut bytes = low.to_le_bytes().to_vec();
    bytes.push(high);
    WideUint::from_le_bytes(&bytes).unwrap()
}
macro_rules! input {
    ($module:ident,$left:expr,$right:expr) => {
        $module::types::Input {
            left: u($left),
            right: u($right),
        }
    };
}
macro_rules! pure_case {
    ($id:ident,$source:ident,[$($arg:expr),*])=>{
        #[test] fn $id(){check($source::pure_circuits::value($($arg),*),&row(stringify!($id)));}
    };
}
pure_case!(
    small_unsigned,
    small_unsigned,
    [input!(small_unsigned, 2, 3)]
);
pure_case!(
    unsigned_bounds,
    small_unsigned,
    [input!(small_unsigned, 0, 255)]
);
pure_case!(small_field, small_field, [input!(small_field, 0, 255)]);
pure_case!(
    wide,
    wide,
    [wide::types::Input {
        left: wide(0, 1),
        right: wide(u128::MAX, 1)
    }]
);
pure_case!(product, product, [input!(product, 0, 255)]);
pure_case!(
    nested,
    nested,
    [nested::types::Envelope {
        inner: input!(nested, 0, 255)
    }]
);
pure_case!(alias, alias, [input!(alias, 0, 255)]);
pure_case!(
    nested_literal,
    nested_literal,
    [input!(nested_literal, 0, 255)]
);
pure_case!(
    conditional_true,
    conditional,
    [true, input!(conditional, 0, 255), input!(conditional, 2, 3)]
);
pure_case!(
    conditional_false,
    conditional,
    [
        false,
        input!(conditional, 0, 255),
        input!(conditional, 2, 3)
    ]
);
pure_case!(local, local, [input!(local, 0, 255)]);
pure_case!(
    source_ok,
    source_checked,
    [
        true,
        true,
        input!(source_checked, 2, 3),
        input!(source_checked, 4, 5)
    ]
);
pure_case!(
    first_denied,
    source_checked,
    [
        false,
        true,
        input!(source_checked, 2, 3),
        input!(source_checked, 4, 5)
    ]
);
pure_case!(
    second_denied,
    source_checked,
    [
        true,
        false,
        input!(source_checked, 2, 3),
        input!(source_checked, 4, 5)
    ]
);
pure_case!(
    both_denied,
    source_checked,
    [
        false,
        false,
        input!(source_checked, 2, 3),
        input!(source_checked, 4, 5)
    ]
);
pure_case!(guarded, guarded, [true, input!(guarded, 0, 255)]);
pure_case!(guarded_denied, guarded, [false, input!(guarded, 0, 255)]);
pure_case!(empty_mixed, empty_mixed, [input!(empty_mixed, 0, 255)]);
pure_case!(
    narrow_checked,
    narrow_checked,
    [input!(narrow_checked, 1, 2)]
);
pure_case!(
    narrow_first,
    narrow_checked,
    [input!(narrow_checked, 1, 300)]
);
pure_case!(
    narrow_second,
    narrow_checked,
    [input!(narrow_checked, 300, 1)]
);
pure_case!(
    narrow_both,
    narrow_checked,
    [input!(narrow_checked, 400, 300)]
);
pure_case!(
    scalar_control,
    scalar_control,
    [input!(scalar_control, 0, 255)]
);
pure_case!(
    explicit_control,
    explicit_control,
    [input!(explicit_control, 0, 255)]
);
pure_case!(
    struct_literal_projection,
    struct_literal_projection,
    [FixedVector::new([u(0), u(255)])]
);
pure_case!(
    struct_literal_variable,
    struct_literal_variable,
    [u(0), u(255)]
);

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
stateful_case!(run, run, [input!(helper, 0, 255)]);
stateful_case!(run_field, run_field, [input!(helper, 0, 255)]);
