// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]
use direct_branch_proofs::runtime as r;
use r::{BoundedUint, Field, FixedVector, WideUint};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_DIRECT_STATEFUL_VECTOR_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "direct-stateful-vector-literal-ts/v1");
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
fn wide_value(low: u128, high: u8) -> WideUint<1, { u128::MAX }> {
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
    ($id:ident,$module:ident,$entry:ident,[$($ctor:expr),*],[$($arg:expr),*])=>{
        #[test] fn $id(){
            use $module::ledger_contract as api;
            let row=row(stringify!($id));
            let mut observed = None;
            for lane in 0..4 {
                let ctx=api::initial_state(r::context::ConstructorContext::new(7_u64),$($ctor),*).unwrap().into_circuit_context(Default::default());
                let held=ctx.query.state.clone();
                let result=match lane {
                    0=>api::$entry(ctx,$($arg),*),
                    1=>api::recorded::$entry(ctx,$($arg),*).map(replay),
                    2=>api::functional::$entry::Program::new($($arg),*).execute_native(ctx),
                    3=>api::functional::$entry::Program::new($($arg),*).execute_recorded(ctx).map(replay),
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
stateful_case!(alias, alias, run, [], [FixedVector::new([u(0), u(255)])]);
stateful_case!(
    alias_identity,
    alias_identity,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(empty, empty, run, [], [u(255)]);
stateful_case!(
    heterogeneous,
    heterogeneous,
    run,
    [],
    [input!(heterogeneous, 255, 65535)]
);
stateful_case!(local, local, run, [], [FixedVector::new([u(0), u(255)])]);
stateful_case!(
    nested,
    nested,
    run,
    [],
    [FixedVector::new([
        FixedVector::new([u(0), u(255)]),
        FixedVector::new([u(1), u(254)])
    ])]
);
stateful_case!(
    state_condition,
    state_condition,
    run,
    [Field::from(0_u64)],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    struct_explicit,
    struct_explicit,
    run,
    [],
    [input!(struct_explicit, 0, 255)]
);
stateful_case!(
    struct_field,
    struct_field,
    run,
    [],
    [input!(struct_field, 0, 255)]
);
stateful_case!(
    struct_identity,
    struct_identity,
    run,
    [],
    [input!(struct_identity, 0, 255)]
);
stateful_case!(
    struct_unsigned,
    struct_unsigned,
    run,
    [],
    [input!(struct_unsigned, 0, 255)]
);
stateful_case!(
    struct_unsigned_small,
    struct_unsigned,
    run,
    [],
    [input!(struct_unsigned, 2, 3)]
);
stateful_case!(
    vector_explicit,
    vector_explicit,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    vector_field,
    vector_field,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    vector_identity,
    vector_identity,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    vector_thirteen,
    vector_thirteen,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    vector_unsigned,
    vector_unsigned,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    vector_unsigned_small,
    vector_unsigned,
    run,
    [],
    [FixedVector::new([u(2), u(3)])]
);
stateful_case!(
    wide_struct,
    wide_struct,
    run,
    [],
    [wide_struct::types::Input {
        left: wide_value(0, 1),
        right: wide_value(u128::MAX, 1)
    }]
);
stateful_case!(
    wide_vector,
    wide_vector,
    run,
    [],
    [FixedVector::new([
        wide_value(0, 1),
        wide_value(u128::MAX, 1)
    ])]
);
stateful_case!(
    direct_proofs_run,
    direct_proofs,
    run,
    [],
    [input!(direct_proofs, 0, 255)]
);
stateful_case!(
    direct_proofs_run_field,
    direct_proofs,
    run_field,
    [],
    [input!(direct_proofs, 0, 255)]
);
stateful_case!(
    helper_control_run,
    helper_control,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    helper_control_run_field,
    helper_control,
    run_field,
    [],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    branch_0,
    direct_branch_proofs,
    run,
    [Field::from(0_u64)],
    [input!(direct_branch_proofs, 0, 255)]
);
stateful_case!(
    branch_1,
    direct_branch_proofs,
    run,
    [Field::from(1_u64)],
    [input!(direct_branch_proofs, 0, 255)]
);
stateful_case!(
    branch_field,
    direct_branch_proofs,
    run_field,
    [Field::from(0_u64)],
    [input!(direct_branch_proofs, 0, 255)]
);
stateful_case!(
    conditional_true,
    conditional_disclosed,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    conditional_false,
    conditional_disclosed,
    run,
    [],
    [false, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    guarded_before_ok,
    guarded_before,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    guarded_before_denied,
    guarded_before,
    run,
    [],
    [false, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    guarded_after_ok,
    guarded_after,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    guarded_after_denied,
    guarded_after,
    run,
    [],
    [false, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    source_ok,
    source_checked,
    run,
    [],
    [
        true,
        true,
        FixedVector::new([u(2), u(3)]),
        FixedVector::new([u(4), u(5)])
    ]
);
stateful_case!(
    first_denied,
    source_checked,
    run,
    [],
    [
        false,
        true,
        FixedVector::new([u(2), u(3)]),
        FixedVector::new([u(4), u(5)])
    ]
);
stateful_case!(
    second_denied,
    source_checked,
    run,
    [],
    [
        true,
        false,
        FixedVector::new([u(2), u(3)]),
        FixedVector::new([u(4), u(5)])
    ]
);
stateful_case!(
    both_denied,
    source_checked,
    run,
    [],
    [
        false,
        false,
        FixedVector::new([u(2), u(3)]),
        FixedVector::new([u(4), u(5)])
    ]
);
stateful_case!(
    checked_ok,
    checked,
    run,
    [],
    [FixedVector::new([u(1), u(2)])]
);
stateful_case!(
    checked_first,
    checked,
    run,
    [],
    [FixedVector::new([u(1), u(300)])]
);
stateful_case!(
    checked_second,
    checked,
    run,
    [],
    [FixedVector::new([u(300), u(1)])]
);
stateful_case!(
    checked_both,
    checked,
    run,
    [],
    [FixedVector::new([u(400), u(300)])]
);

stateful_case!(
    nonmutating,
    nonmutating,
    run,
    [Field::from(0_u64)],
    [FixedVector::new([u(0), u(255)])]
);

#[derive(Default)]
struct SequenceWitness(std::cell::RefCell<Vec<(u64, u128)>>);
impl witness_literal::ledger_contract::Witnesses<u64> for SequenceWitness {
    fn next(
        &self,
        context: r::context::WitnessContext<
            '_,
            u64,
            witness_literal::ledger_contract::LedgerView<'_>,
        >,
    ) -> (u64, BoundedUint<255>) {
        let state = *context.private_state;
        let value = u128::from(state - 4);
        self.0.borrow_mut().push((state, value));
        (state + 1, u(value))
    }
}
#[test]
fn witness_literal() {
    use witness_literal::ledger_contract as api;
    let oracle = row("witness_literal");
    let mut observed = None;
    for lane in 0..4 {
        let witnesses = SequenceWitness::default();
        let ctx = api::initial_state(r::context::ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(Default::default());
        let held = ctx.query.state.clone();
        let out = match lane {
            0 => api::run(ctx, &witnesses),
            1 => api::recorded::run(ctx, &witnesses).map(replay),
            2 => api::functional::run::Program::new().execute_native(ctx, &witnesses),
            3 => api::functional::run::Program::new()
                .execute_recorded(ctx, &witnesses)
                .map(replay),
            _ => unreachable!(),
        }
        .unwrap();
        check::<FixedVector<BoundedUint<65535>, 2>>(Ok(out.result), &oracle);
        assert_eq!(
            out.context.private_state,
            oracle["privateState"].as_u64().unwrap()
        );
        assert_eq!(
            out.private_transcript_outputs
                .into_iter()
                .map(encode)
                .collect::<Vec<_>>(),
            oracle["privateOutputs"].as_array().unwrap().clone()
        );
        let calls=witnesses.0.borrow().iter().map(|(private_state,value)|json!({"privateState":private_state,"value":value.to_string()})).collect::<Vec<_>>();
        assert_eq!(
            calls,
            oracle["observedWitnesses"].as_array().unwrap().clone()
        );
        assert_eq!(out.context.query.state, held);
        assert_eq!(
            witness_literal::ledger_slots::seen
                .inspect(held.get_ref())
                .unwrap(),
            Field::from(0_u64)
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

stateful_case!(
    nonmutating_else,
    nonmutating,
    run,
    [Field::from(1_u64)],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    state_condition_else,
    state_condition,
    run,
    [Field::from(1_u64)],
    [FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    mixed_helper_true,
    mixed_helper,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    mixed_helper_false,
    mixed_helper,
    run,
    [],
    [false, FixedVector::new([u(0), u(255)])]
);
stateful_case!(
    mixed_parameter_true,
    mixed_parameter,
    run,
    [],
    [
        true,
        FixedVector::new([u(7), u(60000)]),
        FixedVector::new([u(0), u(255)])
    ]
);
stateful_case!(
    mixed_parameter_false,
    mixed_parameter,
    run,
    [],
    [
        false,
        FixedVector::new([u(7), u(60000)]),
        FixedVector::new([u(0), u(255)])
    ]
);
stateful_case!(
    ledger_read_literal,
    ledger_read_literal,
    run,
    [Field::from(5_u64)],
    []
);
