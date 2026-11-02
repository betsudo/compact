// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]
use proofs::runtime as r;
use r::{BoundedUint, Field, FixedVector, WideUint};
use serde_json::{Value, json};

fn row(id: &str) -> Value {
    let ts: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_STATEFUL_LOCAL_VECTOR_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(ts["status"], "passed");
    assert_eq!(ts["format"], "stateful-local-vector-ts/v1");
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
macro_rules! pure_case {
 ($id:ident,$module:ident,[$($arg:expr),*])=>{#[test] fn $id(){check($module::pure_circuits::value($($arg),*),&row(stringify!($id)));}};
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
                    if row["noMutation"] == true {assert_eq!(out.context.query.state,held);}
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

stateful_case!(
    alias_after,
    alias_after,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    checked_helper_after,
    checked_helper_after,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    discarded_checked,
    discarded_checked,
    run,
    [],
    [true, FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    empty_after,
    empty_after,
    run,
    [],
    [FixedVector::<BoundedUint<255>, 0>::new([])]
);

stateful_case!(
    field_after,
    field_after,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    helper_after,
    helper_after,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(literal_control, literal_control, run, [], []);

stateful_case!(
    multiple_bindings,
    multiple_bindings,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    nested_after,
    nested_after,
    run,
    [],
    [FixedVector::new([
        FixedVector::new([u(0), u(255)]),
        FixedVector::new([u(1), u(2)])
    ])]
);

pure_case!(
    pure_control,
    pure_control,
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    read_only_value,
    read_only_value,
    run,
    [FixedVector::new([u(0), u(255)])],
    []
);

stateful_case!(
    returned_value_control,
    returned_value_control,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    scalar_alias,
    scalar_alias,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    scalar_result,
    scalar_result,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    stateful_helper,
    stateful_helper,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    stateful_helper_value,
    stateful_helper_value,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    struct_projection,
    struct_projection,
    run,
    [],
    [struct_projection::types::Input {
        value: FixedVector::new([u(0), u(255)])
    }]
);

stateful_case!(
    unit_result,
    unit_result,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    unsigned_after,
    unsigned_after,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    unsigned_before,
    unsigned_before,
    run,
    [],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    wide_local,
    wide_local,
    run,
    [],
    [FixedVector::new([wide_value(9, 1), wide_value(0, 0)])]
);

stateful_case!(
    checked_guard_first,
    checked_helper_after,
    run,
    [],
    [false, FixedVector::new([u(400), u(300)])]
);

stateful_case!(
    checked_first,
    checked_helper_after,
    run,
    [],
    [true, FixedVector::new([u(1), u(300)])]
);

stateful_case!(
    checked_second,
    checked_helper_after,
    run,
    [],
    [true, FixedVector::new([u(300), u(1)])]
);

stateful_case!(
    checked_both,
    checked_helper_after,
    run,
    [],
    [true, FixedVector::new([u(400), u(300)])]
);

stateful_case!(
    discarded_guard_first,
    discarded_checked,
    run,
    [],
    [false, FixedVector::new([u(400), u(300)])]
);

stateful_case!(
    discarded_first,
    discarded_checked,
    run,
    [],
    [true, FixedVector::new([u(1), u(300)])]
);

stateful_case!(
    discarded_second,
    discarded_checked,
    run,
    [],
    [true, FixedVector::new([u(300), u(1)])]
);

stateful_case!(
    discarded_both,
    discarded_checked,
    run,
    [],
    [true, FixedVector::new([u(400), u(300)])]
);

stateful_case!(
    proof_branch_0,
    proofs,
    run,
    [Field::from(0_u64)],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    proof_branch_1,
    proofs,
    run,
    [Field::from(1_u64)],
    [FixedVector::new([u(0), u(255)])]
);

stateful_case!(
    proof_field,
    proofs,
    run_field,
    [Field::from(0_u64)],
    [FixedVector::new([u(0), u(255)])]
);

struct VectorWitness {
    expected_seen: u64,
    calls: std::cell::RefCell<Vec<(u64, Vec<u128>, Value)>>,
}
impl VectorWitness {
    fn record(&self, before: u64, seen: Field) -> (u64, FixedVector<BoundedUint<255>, 2>) {
        assert_eq!(
            seen,
            Field::from(self.expected_seen),
            "initializer witness order"
        );
        let values = [u128::from(before - 4), u128::from(before - 3)];
        self.calls
            .borrow_mut()
            .push((before, values.to_vec(), encode(seen.into())));
        (before + 1, FixedVector::new(values.map(u)))
    }
}
macro_rules! vector_witness {
    ($module:ident) => {
        impl $module::ledger_contract::Witnesses<u64> for VectorWitness {
            fn vector_value(
                &self,
                ctx: r::context::WitnessContext<'_, u64, $module::ledger_contract::LedgerView<'_>>,
            ) -> (u64, FixedVector<BoundedUint<255>, 2>) {
                self.record(*ctx.private_state, ctx.ledger.seen().unwrap())
            }
        }
    };
}
vector_witness!(witness_before);
vector_witness!(witness_after);
macro_rules! witness_case {
    ($module:ident) => {
        #[test]
        fn $module() {
            use $module::ledger_contract as api;
            let row = row(stringify!($module));
            let mut observed = None;
            for lane in 0..4 {
                let witness = VectorWitness { expected_seen: row["witnessSeen"].as_str().unwrap().parse().unwrap(), calls: Default::default() };
                let ctx = api::initial_state(r::context::ConstructorContext::new(7_u64)).unwrap().into_circuit_context(Default::default());
                let held = ctx.query.state.clone();
                let out = match lane {
                    0 => api::run(ctx, &witness),
                    1 => api::recorded::run(ctx, &witness).map(replay),
                    2 => api::functional::run::Program::new().execute_native(ctx, &witness),
                    3 => api::functional::run::Program::new().execute_recorded(ctx, &witness).map(replay),
                    _ => unreachable!(),
                }.unwrap();
                check::<FixedVector<BoundedUint<65535>,2>>(Ok(out.result), &row);
                assert_eq!(out.context.private_state, 8);
                assert_eq!(out.private_transcript_outputs.into_iter().map(encode).collect::<Vec<_>>(), row["privateOutputs"].as_array().unwrap().clone());
                assert_eq!(witness.calls.borrow().iter().map(|(state,value,seen)|json!({"privateState":state,"value":value.iter().map(|x|x.to_string()).collect::<Vec<_>>(),"seen":seen})).collect::<Vec<_>>(), row["observedWitnesses"].as_array().unwrap().clone());
                assert_eq!($module::ledger_slots::seen.inspect(out.context.query.state.get_ref()).unwrap(), Field::from(1_u64));
                assert_eq!($module::ledger_slots::seen.inspect(held.get_ref()).unwrap(), Field::from(0_u64));
                let state = (out.context.query.state, out.context.query.effects, out.gas_cost);
                if let Some(before) = &observed { assert_eq!(&state,before); } else { observed = Some(state); }
            }
        }
    };
}
witness_case!(witness_before);
witness_case!(witness_after);
