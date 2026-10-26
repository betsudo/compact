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

use direct::runtime as r;
use r::{
    Field, FixedVector,
    context::{CircuitContext, CircuitResult, ConstructorContext, WitnessContext},
};
use serde_json::{Value, json};
use std::cell::RefCell;
#[derive(Debug)]
struct Event {
    name: &'static str,
    private: u64,
    acc: Option<Field>,
    item: Option<Field>,
}
struct Callbacks {
    fail: Option<&'static str>,
    events: RefCell<Vec<Event>>,
}
impl Callbacks {
    fn new(fail: Option<&'static str>) -> Self {
        Self {
            fail,
            events: RefCell::new(vec![]),
        }
    }
    fn operand<T>(
        &self,
        name: &'static str,
        private: u64,
        value: T,
    ) -> Result<(u64, T), r::CompactError> {
        self.events.borrow_mut().push(Event {
            name,
            private,
            acc: None,
            item: None,
        });
        if self.fail == Some(name) {
            return Err(r::CompactError::AssertionFailed(format!("{name} refused")));
        }
        Ok((private + 1, value))
    }
}
macro_rules! operands { ($module:ident, $($name:ident : $ty:ty = $value:expr),+ ) => {
 impl $module::ledger_contract::TryWitnesses<u64> for Callbacks {
  $(fn $name(&self,ctx:WitnessContext<'_,u64,$module::ledger_contract::LedgerView<'_>>)->Result<(u64,$ty),r::CompactError>{self.operand(stringify!($name),*ctx.private_state,$value)})+
 }
}; }
operands!(ordered,seed:Field=Field::from(1u64),items:FixedVector<Field,2>=FixedVector::new([Field::from(2u64),Field::from(3u64)]));
operands!(multi_ordered,seed:Field=Field::from(1u64),left:FixedVector<Field,2>=FixedVector::new([Field::from(2u64),Field::from(3u64)]),right:FixedVector<Field,2>=FixedVector::new([Field::from(4u64),Field::from(5u64)]));
impl witness_reducer::ledger_contract::TryWitnesses<u64> for Callbacks {
    fn step(
        &self,
        ctx: WitnessContext<'_, u64, witness_reducer::ledger_contract::LedgerView<'_>>,
        acc: Field,
        item: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        self.events.borrow_mut().push(Event {
            name: "step",
            private: *ctx.private_state,
            acc: Some(acc),
            item: Some(item),
        });
        if self.fail == Some("step") && item == Field::from(7u64) {
            return Err(r::CompactError::AssertionFailed("step refused".into()));
        }
        Ok((*ctx.private_state + 1, acc * Field::from(10u64) + item))
    }
}
impl empty_ordered::ledger_contract::TryWitnesses<u64> for Callbacks {
    fn seed(
        &self,
        ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
    ) -> Result<(u64, Field), r::CompactError> {
        self.operand("seed", *ctx.private_state, Field::from(9_u64))
    }
    fn items(
        &self,
        ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
    ) -> Result<(u64, FixedVector<Field, 0>), r::CompactError> {
        self.operand("items", *ctx.private_state, FixedVector::new([]))
    }
    fn step(
        &self,
        ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
        acc: Field,
        item: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        self.events.borrow_mut().push(Event {
            name: "step",
            private: *ctx.private_state,
            acc: Some(acc),
            item: Some(item),
        });
        if self.fail == Some("step") {
            return Err(r::CompactError::AssertionFailed("step refused".into()));
        }
        Ok((*ctx.private_state + 1, acc * Field::from(10_u64) + item))
    }
}
fn encode(x: r::fab::AlignedValue) -> Value {
    json!({"valueAtoms":x.value.0.iter().map(|x|&x.0).collect::<Vec<_>>(),"alignment":x.alignment})
}
fn rows() -> Value {
    serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_DIRECT_FOLD_TS_REFERENCE").unwrap()).unwrap(),
    )
    .unwrap()
}
fn field(x: &Value) -> Field {
    Field::from(x.as_str().unwrap().parse::<u128>().unwrap())
}
fn events(actual: &Callbacks, expected: &Value) {
    let actual = actual.events.borrow();
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
        assert_eq!(a.name, b["name"].as_str().unwrap());
        assert_eq!(a.private, b["privateState"].as_u64().unwrap());
        if let Some(acc) = a.acc {
            assert_eq!(acc, field(&b["acc"]));
        }
        if let Some(item) = a.item {
            assert_eq!(item, field(&b["item"]));
        }
    }
}
fn success<T: Clone + Into<r::fab::AlignedValue>>(
    actual: &CircuitResult<u64, T>,
    expected: &Value,
) {
    assert_eq!(encode(actual.result.clone().into()), expected["result"]);
    let stored = r::slots::CellSlot::<Field>::new(&[0u8])
        .inspect(actual.context.query.state.get_ref())
        .unwrap();
    assert_eq!(stored, field(&expected["stored"]));
    assert_eq!(
        actual.context.private_state,
        expected["privateState"].as_u64().unwrap()
    );
    assert_eq!(
        json!(
            actual
                .private_transcript_outputs
                .iter()
                .cloned()
                .map(encode)
                .collect::<Vec<_>>()
        ),
        expected["privateOutputs"]
    );
    assert_eq!(actual.context.query.effects, Default::default());
    assert_eq!(
        expected["effects"],
        json!({"claimedNullifiers":[],"claimedShieldedReceives":[],"claimedShieldedSpends":[],"claimedContractCalls":[],"shieldedMints":[],"unshieldedMints":[],"unshieldedInputs":[],"unshieldedOutputs":[],"claimedUnshieldedSpends":[]})
    );
    for (key, cost) in [
        (
            "readTime",
            serde_json::to_value(actual.gas_cost.read_time).unwrap(),
        ),
        (
            "computeTime",
            serde_json::to_value(actual.gas_cost.compute_time).unwrap(),
        ),
        (
            "bytesWritten",
            serde_json::to_value(actual.gas_cost.bytes_written).unwrap(),
        ),
        (
            "bytesDeleted",
            serde_json::to_value(actual.gas_cost.bytes_deleted).unwrap(),
        ),
    ] {
        assert_eq!(
            cost,
            json!(
                expected["gas"][key]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            )
        );
    }
}

// Each case executes complete unedited generated crates through four public routes.
// Recorded results independently replay their public VM operations.
fn replay<T>(out: r::recording::RecordedCircuitResult<u64, T>) -> CircuitResult<u64, T> {
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
macro_rules! execute {
    ($module:ident, $lane:expr, $context:ident, [$($arg:expr),*], [$($witness:expr),*]) => {
        match $lane {
            0 => $module::ledger_contract::run($context, $($witness,)* $($arg,)*),
            1 => $module::ledger_contract::recorded::run($context, $($witness,)* $($arg,)*).map(replay),
            2 => $module::ledger_contract::functional::run::Program::new($($arg),*)
                .execute_native($context, $($witness,)*),
            3 => $module::ledger_contract::functional::run::Program::new($($arg),*)
                .execute_recorded($context, $($witness,)*).map(replay),
            _ => unreachable!(),
        }
    };
}
macro_rules! scenario {
    ($test:ident, $id:literal, $module:ident, [$($arg:expr),*], $fail:expr, $callbacks:ident, [$($witness:expr),*]) => {
        #[test]
        fn $test() {
            let ts = rows();
            assert_eq!(ts["format"], "direct-fold-ingress-live-ts/v1");
            let row = ts["rows"].as_array().unwrap().iter().find(|row| row["id"] == $id).unwrap();
            let mut reference = None;
            for lane in 0..4 {
                let $callbacks = Callbacks::new($fail);
                let context: CircuitContext<u64> = $module::ledger_contract::initial_state(
                    ConstructorContext::new(7)).unwrap().into_circuit_context(Default::default());
                let held = context.query.state.clone();
                let actual = execute!($module, lane, context, [$($arg),*], [$($witness),*]);
                events(&$callbacks, &row["events"]);
                if let Some(message) = row["error"].as_str() {
                    assert_eq!(actual.err().expect("expected source/witness refusal"),
                        r::CompactError::AssertionFailed(message.into()));
                } else {
                    let actual = actual.unwrap();
                    success(&actual, row);
                    if let Some(state) = &reference {
                        assert_eq!(&actual.context.query.state, state);
                    } else {
                        reference = Some(actual.context.query.state.clone());
                    }
                }
                // This tests persistent ownership isolation, not callback rollback.
                assert_eq!(r::slots::CellSlot::<Field>::new(&[0u8]).inspect(held.get_ref()).unwrap(),
                    Field::from(2u64));
            }
        }
    };
}
fn vector<const N: usize>(values: [u64; N]) -> FixedVector<Field, N> {
    FixedVector::new(values.map(Field::from))
}
scenario!(
    direct_0_four_routes_match_live_ts,
    "direct_0",
    direct,
    [Field::from(0_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    direct_1_four_routes_match_live_ts,
    "direct_1",
    direct,
    [Field::from(1_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    direct_3_four_routes_match_live_ts,
    "direct_3",
    direct,
    [Field::from(3_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    empty_four_routes_match_live_ts,
    "empty",
    empty,
    [Field::from(9_u64), vector([])],
    None,
    callbacks,
    []
);
scenario!(
    helper_four_routes_match_live_ts,
    "helper",
    helper,
    [Field::from(1_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    captured_10_four_routes_match_live_ts,
    "captured_10",
    captured,
    [Field::from(1_u64), Field::from(10_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    captured_2_four_routes_match_live_ts,
    "captured_2",
    captured,
    [Field::from(1_u64), Field::from(2_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    captured_0_four_routes_match_live_ts,
    "captured_0",
    captured,
    [Field::from(1_u64), Field::from(0_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    guarded_four_routes_match_live_ts,
    "guarded",
    guarded,
    [Field::from(1_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    guarded_first_four_routes_match_live_ts,
    "guarded_first",
    guarded,
    [Field::from(1_u64), vector([7, 3])],
    None,
    callbacks,
    []
);
scenario!(
    guarded_later_four_routes_match_live_ts,
    "guarded_later",
    guarded,
    [Field::from(1_u64), vector([2, 7])],
    None,
    callbacks,
    []
);
scenario!(
    ordered_four_routes_match_live_ts,
    "ordered",
    ordered,
    [],
    None,
    callbacks,
    [&callbacks]
);
scenario!(
    ordered_seed_four_routes_match_live_ts,
    "ordered_seed",
    ordered,
    [],
    Some("seed"),
    callbacks,
    [&callbacks]
);
scenario!(
    ordered_items_four_routes_match_live_ts,
    "ordered_items",
    ordered,
    [],
    Some("items"),
    callbacks,
    [&callbacks]
);
scenario!(
    stateful_four_routes_match_live_ts,
    "stateful",
    effectful_reducer,
    [Field::from(1_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    witness_four_routes_match_live_ts,
    "witness",
    witness_reducer,
    [Field::from(1_u64), vector([2, 3])],
    None,
    callbacks,
    [&callbacks]
);
scenario!(
    witness_first_four_routes_match_live_ts,
    "witness_first",
    witness_reducer,
    [Field::from(1_u64), vector([7, 3])],
    Some("step"),
    callbacks,
    [&callbacks]
);
scenario!(
    witness_later_four_routes_match_live_ts,
    "witness_later",
    witness_reducer,
    [Field::from(1_u64), vector([2, 7])],
    Some("step"),
    callbacks,
    [&callbacks]
);
scenario!(
    multiple_four_routes_match_live_ts,
    "multiple",
    multiple_inputs,
    [Field::from(1_u64), vector([2, 3]), vector([4, 5])],
    None,
    callbacks,
    []
);
scenario!(
    inline_four_routes_match_live_ts,
    "inline",
    inline_effectful,
    [Field::from(1_u64), Field::from(10_u64), vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    owned_four_routes_match_live_ts,
    "owned",
    owned,
    [
        r::OpaqueString(String::from("held owned value")),
        vector([2, 3])
    ],
    None,
    callbacks,
    []
);
scenario!(
    unit_four_routes_match_live_ts,
    "unit",
    unit,
    [vector([2, 3])],
    None,
    callbacks,
    []
);
scenario!(
    multi_ordered_four_routes_match_live_ts,
    "multi_ordered",
    multi_ordered,
    [],
    None,
    callbacks,
    [&callbacks]
);
scenario!(
    multi_ordered_seed_four_routes_match_live_ts,
    "multi_ordered_seed",
    multi_ordered,
    [],
    Some("seed"),
    callbacks,
    [&callbacks]
);
scenario!(
    multi_ordered_left_four_routes_match_live_ts,
    "multi_ordered_left",
    multi_ordered,
    [],
    Some("left"),
    callbacks,
    [&callbacks]
);
scenario!(
    multi_ordered_right_four_routes_match_live_ts,
    "multi_ordered_right",
    multi_ordered,
    [],
    Some("right"),
    callbacks,
    [&callbacks]
);
scenario!(
    branch_true_four_routes_match_live_ts,
    "branch_true",
    inline_branch,
    [
        Field::from(1_u64),
        Field::from(10_u64),
        true,
        vector([2, 3])
    ],
    None,
    callbacks,
    []
);
scenario!(
    branch_false_four_routes_match_live_ts,
    "branch_false",
    inline_branch,
    [
        Field::from(1_u64),
        Field::from(10_u64),
        false,
        vector([2, 3])
    ],
    None,
    callbacks,
    []
);
scenario!(
    branch_nested_four_routes_match_live_ts,
    "branch_nested",
    inline_branch,
    [
        Field::from(1_u64),
        Field::from(10_u64),
        false,
        vector([7, 3])
    ],
    None,
    callbacks,
    []
);
scenario!(
    branch_first_four_routes_match_live_ts,
    "branch_first",
    inline_branch,
    [
        Field::from(1_u64),
        Field::from(10_u64),
        true,
        vector([9, 3])
    ],
    None,
    callbacks,
    []
);
scenario!(
    branch_later_four_routes_match_live_ts,
    "branch_later",
    inline_branch,
    [
        Field::from(1_u64),
        Field::from(10_u64),
        true,
        vector([2, 9])
    ],
    None,
    callbacks,
    []
);

scenario!(
    single_zero_four_routes_match_live_ts,
    "single_zero",
    single,
    [Field::from(0_u64), Field::from(10_u64), vector([2])],
    None,
    callbacks,
    []
);

scenario!(
    single_one_four_routes_match_live_ts,
    "single_one",
    single,
    [Field::from(1_u64), Field::from(10_u64), vector([2])],
    None,
    callbacks,
    []
);

scenario!(
    single_capture_zero_four_routes_match_live_ts,
    "single_capture_zero",
    single,
    [Field::from(3_u64), Field::from(0_u64), vector([2])],
    None,
    callbacks,
    []
);

scenario!(
    empty_ordered_four_routes_match_live_ts,
    "empty_ordered",
    empty_ordered,
    [],
    None,
    callbacks,
    [&callbacks]
);

scenario!(
    empty_step_unreached_four_routes_match_live_ts,
    "empty_step_unreached",
    empty_ordered,
    [],
    Some("step"),
    callbacks,
    [&callbacks]
);

scenario!(
    empty_seed_failure_four_routes_match_live_ts,
    "empty_seed_failure",
    empty_ordered,
    [],
    Some("seed"),
    callbacks,
    [&callbacks]
);

scenario!(
    empty_items_failure_four_routes_match_live_ts,
    "empty_items_failure",
    empty_ordered,
    [],
    Some("items"),
    callbacks,
    [&callbacks]
);

scenario!(
    owned_capture_0_four_routes_match_live_ts,
    "owned_capture_0",
    owned_capture,
    [
        r::OpaqueString::from("seed"),
        r::OpaqueString::from("tag"),
        FixedVector::new([r::OpaqueString::from("tag"), r::OpaqueString::from("tag")])
    ],
    None,
    callbacks,
    []
);

scenario!(
    owned_capture_1_four_routes_match_live_ts,
    "owned_capture_1",
    owned_capture,
    [
        r::OpaqueString::from("seed"),
        r::OpaqueString::from("tag"),
        FixedVector::new([r::OpaqueString::from("other"), r::OpaqueString::from("tag")])
    ],
    None,
    callbacks,
    []
);

scenario!(
    owned_capture_2_four_routes_match_live_ts,
    "owned_capture_2",
    owned_capture,
    [
        r::OpaqueString::from("seed"),
        r::OpaqueString::from("tag"),
        FixedVector::new([
            r::OpaqueString::from("other"),
            r::OpaqueString::from("seed")
        ])
    ],
    None,
    callbacks,
    []
);

scenario!(
    owned_capture_3_four_routes_match_live_ts,
    "owned_capture_3",
    owned_capture,
    [
        r::OpaqueString::from("seed"),
        r::OpaqueString::from("seed"),
        FixedVector::new([
            r::OpaqueString::from("other"),
            r::OpaqueString::from("seed")
        ])
    ],
    None,
    callbacks,
    []
);

scenario!(
    shadow_0_four_routes_match_live_ts,
    "shadow_0",
    shadow,
    [
        Field::from(1_u64),
        Field::from(20_u64),
        Field::from(100_u64),
        vector([2, 3])
    ],
    None,
    callbacks,
    []
);

scenario!(
    shadow_1_four_routes_match_live_ts,
    "shadow_1",
    shadow,
    [
        Field::from(2_u64),
        Field::from(8_u64),
        Field::from(7_u64),
        vector([4, 5])
    ],
    None,
    callbacks,
    []
);

scenario!(
    shadow_effectful_0_four_routes_match_live_ts,
    "shadow_effectful_0",
    shadow_effectful,
    [
        Field::from(1_u64),
        Field::from(20_u64),
        Field::from(100_u64),
        vector([2, 3])
    ],
    None,
    callbacks,
    []
);

scenario!(
    shadow_effectful_1_four_routes_match_live_ts,
    "shadow_effectful_1",
    shadow_effectful,
    [
        Field::from(2_u64),
        Field::from(8_u64),
        Field::from(7_u64),
        vector([4, 5])
    ],
    None,
    callbacks,
    []
);
