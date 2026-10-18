// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Real generated requests, not handwritten substitutes for their run bodies.
use runtime::context::{CircuitResult, ConstructorContext, WitnessContext};
use runtime::functional::ledger8::{NativeLedger8, RecordedLedger8};
use runtime::ledger::ContractAddress;
use runtime::recording::RecordedCircuitResult;
use runtime::{CompactError, Field};
use std::{cell::RefCell, fmt::Debug};

fn outcomes<T: Debug + PartialEq>(
    native: Result<CircuitResult<u64, T>, CompactError>,
    recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    functional_native: Result<CircuitResult<u64, T>, CompactError>,
    functional_recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    expected_error: Option<&str>,
) -> Option<CircuitResult<u64, T>> {
    if let Some(expected) = expected_error {
        for error in [
            native.err(),
            recorded.err(),
            functional_native.err(),
            functional_recorded.err(),
        ] {
            assert_eq!(
                error.expect("expected original first failure").to_string(),
                expected
            );
        }
        println!("FUNCTIONAL_SCENARIO_CHECKED");
        return None;
    }
    let (native, recorded, functional_native, functional_recorded) = (
        native.unwrap(),
        recorded.unwrap(),
        functional_native.unwrap(),
        functional_recorded.unwrap(),
    );
    for candidate in [
        &recorded.execution,
        &functional_native,
        &functional_recorded.execution,
    ] {
        assert_eq!(candidate.result, native.result);
        assert_eq!(candidate.context.query.state, native.context.query.state);
        assert_eq!(
            candidate.context.query.effects,
            native.context.query.effects
        );
        assert_eq!(
            candidate.context.private_state,
            native.context.private_state
        );
        assert_eq!(
            candidate.private_transcript_outputs,
            native.private_transcript_outputs
        );
        assert_eq!(candidate.gas_cost, native.gas_cost);
    }
    assert_eq!(
        serde_json::to_value(recorded.public.verify_ops()).unwrap(),
        serde_json::to_value(functional_recorded.public.verify_ops()).unwrap(),
    );
    let replay = |value: &RecordedCircuitResult<u64, T>| {
        value
            .public
            .initial()
            .query(value.public.verify_ops(), None, &native.context.cost_model)
            .unwrap()
    };
    let original = replay(&recorded);
    let functional = replay(&functional_recorded);
    assert_eq!(functional.context.state, native.context.query.state);
    assert_eq!(functional.context.effects, native.context.query.effects);
    // Combined replay is compared with combined replay, not query-summed gas.
    assert_eq!(functional.gas_cost, original.gas_cost);
    println!("FUNCTIONAL_SCENARIO_CHECKED");
    Some(native)
}

#[derive(Default)]
struct Echo(RefCell<Vec<(u64, bool, Field)>>);
macro_rules! echo {
    ($fixture:ident) => {
        impl $fixture::ledger_contract::Witnesses<u64> for Echo {
            fn echo(
                &self,
                context: WitnessContext<'_, u64, $fixture::ledger_contract::LedgerView<'_>>,
                flag: bool,
            ) -> (u64, bool) {
                let value = context.ledger.cell().unwrap();
                self.0
                    .borrow_mut()
                    .push((*context.private_state, flag, value));
                (*context.private_state + 1, flag)
            }
        }
    };
}
echo!(assert_fixture);
echo!(guarded_fixture);

macro_rules! witnessed {
    ($fixture:ident, $entry:ident, [$($argument:expr),*], $error:expr, $events:expr) => {{
        use $fixture::ledger_contract as c;
        use c::functional::$entry as program;
        let context = || c::initial_state(ConstructorContext::new(7_u64)).unwrap()
            .into_circuit_context(ContractAddress::default());
        let witnesses: [Echo; 4] = std::array::from_fn(|_| Echo::default());
        let result = outcomes(
            c::$entry(context(), &witnesses[0], $($argument),*),
            c::recorded::$entry(context(), &witnesses[1], $($argument),*),
            program::Program::new($($argument),*).execute_native(context(), &witnesses[2]),
            program::Program::new($($argument),*).execute_recorded(context(), &witnesses[3]),
            $error,
        );
        let expected: Vec<(u64, bool, Field)> = $events;
        for witness in &witnesses { assert_eq!(*witness.0.borrow(), expected); }
        result
    }};
}

#[test]
fn read_write_return_matches_original_execution_and_replay() {
    use c::functional::replace as program;
    use let_fixture::ledger_contract as c;
    for next in [0_u64, 99, 1000] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let next = Field::from(next);
        let result = outcomes(
            c::replace(context(), next),
            c::recorded::replace(context(), next),
            program::Program::new(next).run(NativeLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            program::Program::new(next).run(RecordedLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            None,
        )
        .unwrap();
        assert_eq!(result.result, Field::from(0_u64));
        assert_eq!(
            let_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            next
        );
    }
}

#[test]
fn witness_assertions_preserve_first_failure_private_outputs_and_metered_reads() {
    for first in [false, true] {
        for second in [false, true] {
            let error = if !first {
                Some("failed assert: first witness failed")
            } else if !second {
                Some("failed assert: second witness failed")
            } else {
                None
            };
            let mut events = vec![(7, first, Field::from(0_u64))];
            if first {
                events.push((8, second, Field::from(0_u64)));
            }
            let result = witnessed!(
                assert_fixture,
                checked_value,
                [first, second, Field::from(42_u64)],
                error,
                events
            );
            if let Some(result) = result {
                assert_eq!(result.result, Field::from(42_u64));
            }
        }
    }
    for flag in [false, true] {
        let error = if flag {
            None
        } else {
            Some("failed assert: write denied")
        };
        let result = witnessed!(
            assert_fixture,
            checked_write,
            [flag, Field::from(42_u64)],
            error,
            vec![(7, flag, Field::from(0_u64))]
        );
        if let Some(result) = result {
            assert_eq!(
                assert_fixture::ledger_slots::cell
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                Field::from(42_u64)
            );
        }
    }
}

#[test]
fn inactive_branch_skips_witnesses_and_branch_refusals_stay_ordered() {
    for selected in [false, true] {
        for flag in [false, true] {
            let error = if flag {
                None
            } else if selected {
                Some("failed assert: selected denied")
            } else {
                Some("failed assert: inactive denied")
            };
            let events = if selected {
                vec![(7, flag, Field::from(0_u64))]
            } else {
                vec![]
            };
            let result = witnessed!(
                guarded_fixture,
                choose,
                [selected, flag, Field::from(55_u64)],
                error,
                events
            );
            if let Some(result) = result {
                assert_eq!(result.result, Field::from(55_u64));
            }
        }
    }
    for result in [
        witnessed!(
            guarded_fixture,
            LedgerView,
            [true, Field::from(56_u64)],
            None,
            vec![(7, true, Field::from(0_u64))]
        ),
        witnessed!(
            guarded_fixture,
            TryWitnesses,
            [true, Field::from(56_u64)],
            None,
            vec![(7, true, Field::from(0_u64))]
        ),
        witnessed!(
            guarded_fixture,
            functional,
            [true, Field::from(56_u64)],
            None,
            vec![(7, true, Field::from(0_u64))]
        ),
    ] {
        assert_eq!(result.unwrap().result, Field::from(56_u64));
    }
    let metadata: serde_json::Value =
        serde_json::from_str(guarded_fixture::ledger_contract::functional::choose::CHECKED_PROGRAM)
            .unwrap();
    assert_eq!(metadata["purpose"], "review_only_not_execution_authority");
    assert_eq!(metadata["guards"].as_array().unwrap().len(), 3);
    assert_eq!(metadata["root_origin"]["consumed_source_bytes"], "unknown");
}

#[test]
fn counter_runtime_named_entry_preserves_increment_read_and_gas() {
    use c::functional::runtime as program;
    use counter_fixture::ledger_contract as c;
    for amount in [0_u64, 1, 65535] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let amount = runtime::BoundedUint::<65535>::new(amount as u128).unwrap();
        let result = outcomes(
            c::runtime(context(), amount),
            c::recorded::runtime(context(), amount),
            program::Program::new(amount).run(NativeLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            program::Program::new(amount).run(RecordedLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            None,
        )
        .unwrap();
        assert_eq!(result.result.value(), amount.value());
    }
}

#[test]
fn normalized_parameter_names_remain_distinct() {
    use c::functional::write as program;
    use collision_fixture::ledger_contract as c;
    let context = || {
        c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let (first, second, third) = (Field::from(1_u64), Field::from(2_u64), Field::from(9_u64));
    let result = outcomes(
        c::write(context(), first, second, third),
        c::recorded::write(context(), first, second, third),
        program::Program::new(first, second, third).run(NativeLedger8::new(
            context(),
            program::ledger8_bindings().unwrap(),
        )),
        program::Program::new(first, second, third).run(RecordedLedger8::new(
            context(),
            program::ledger8_bindings().unwrap(),
        )),
        None,
    )
    .unwrap();
    assert_eq!(result.result, third);
    assert_eq!(
        collision_fixture::ledger_slots::cell
            .inspect(result.context.query.state.get_ref())
            .unwrap(),
        second
    );
    let result = outcomes(
        c::many(context(), true, true, true, true, true, true, true, true),
        c::recorded::many(context(), true, true, true, true, true, true, true, true),
        c::functional::many::Program::new(true, true, true, true, true, true, true, true)
            .execute_native(context()),
        c::functional::many::Program::new(true, true, true, true, true, true, true, true)
            .execute_recorded(context()),
        None,
    )
    .unwrap();
    assert!(result.result);
}

#[test]
fn chunked_physical_paths_preserve_sibling_cells() {
    use c::functional::shift as program;
    use chunked_fixture::ledger_contract as c;
    for next in [3_u64, 17] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let next = Field::from(next);
        let result = outcomes(
            c::shift(context(), next),
            c::recorded::shift(context(), next),
            program::Program::new(next).run(NativeLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            program::Program::new(next).run(RecordedLedger8::new(
                context(),
                program::ledger8_bindings().unwrap(),
            )),
            None,
        )
        .unwrap();
        assert_eq!(result.result, Field::from(0_u64));
        assert_eq!(
            chunked_fixture::ledger_slots::field16
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            next
        );
        assert_eq!(
            chunked_fixture::ledger_slots::field1
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            Field::from(0_u64)
        );
    }
}

struct MeasuredExecution {
    execution: CircuitResult<u64, Field>,
    public: Option<runtime::recording::PublicTrace>,
}

impl MeasuredExecution {
    fn native(execution: CircuitResult<u64, Field>) -> Self {
        Self {
            execution,
            public: None,
        }
    }

    fn recorded(recorded: RecordedCircuitResult<u64, Field>) -> Self {
        Self {
            execution: recorded.execution,
            public: Some(recorded.public),
        }
    }

    fn execution(&self) -> &CircuitResult<u64, Field> {
        &self.execution
    }
}

type MeasuredRoute =
    fn(runtime::context::CircuitContext<u64>, Field) -> Result<MeasuredExecution, CompactError>;

#[test]
fn read_write_runtime_cost_samples_keep_checks_outside_the_interval() {
    use let_fixture::ledger_contract as c;
    const ITERATIONS: usize = 256;
    let routes: [(&str, MeasuredRoute); 4] = [
        ("default_native", |context, value| {
            c::replace(context, value).map(MeasuredExecution::native)
        }),
        ("default_recorded", |context, value| {
            c::recorded::replace(context, value).map(MeasuredExecution::recorded)
        }),
        ("functional_native", |context, value| {
            c::functional::replace::Program::new(value)
                .execute_native(context)
                .map(MeasuredExecution::native)
        }),
        ("functional_recorded", |context, value| {
            c::functional::replace::Program::new(value)
                .execute_recorded(context)
                .map(MeasuredExecution::recorded)
        }),
    ];
    let initial = c::initial_state(ConstructorContext::new(7_u64))
        .unwrap()
        .ledger_state;
    let initial_state = initial.get_ref().clone();
    let fresh_context = || {
        runtime::context::ConstructorResult::new(ConstructorContext::new(7_u64), initial.clone())
            .into_circuit_context(ContractAddress::default())
    };
    let next = Field::from(99_u64);
    let expected = c::replace(fresh_context(), next).unwrap();
    let expected_recording = c::recorded::replace(fresh_context(), next).unwrap();
    let expected_ops = serde_json::to_value(expected_recording.public.verify_ops()).unwrap();
    for lap in 0..4 {
        let order = if lap % 2 == 0 {
            [0, 1, 2, 3]
        } else {
            [3, 2, 1, 0]
        };
        for (position, route) in order.into_iter().enumerate() {
            // Context construction, state clones and result-buffer allocation are
            // outside each interval. Every call receives its own fresh context.
            let contexts = (0..ITERATIONS).map(|_| fresh_context()).collect::<Vec<_>>();
            let mut results = Vec::with_capacity(ITERATIONS);
            let (route_name, run) = routes[route];
            let started = std::time::Instant::now();
            for context in contexts {
                results.push(run(context, next));
            }
            let elapsed = started.elapsed();
            // Full results (including recorded traces) remain alive until after
            // timing. Assertions and result teardown do not inflate elapsed time.
            for result in &results {
                let result = result.as_ref().unwrap();
                let execution = result.execution();
                assert_eq!(execution.result, Field::from(0_u64));
                assert_eq!(execution.context.query.state, expected.context.query.state);
                assert_eq!(
                    execution.context.query.effects,
                    expected.context.query.effects
                );
                assert_eq!(execution.context.private_state, 7);
                assert!(execution.private_transcript_outputs.is_empty());
                assert_eq!(execution.gas_cost, expected.gas_cost);
                assert_eq!(
                    let_fixture::ledger_slots::stored
                        .inspect(execution.context.query.state.get_ref())
                        .unwrap(),
                    next
                );
                if let Some(public) = &result.public {
                    assert_eq!(
                        serde_json::to_value(public.verify_ops()).unwrap(),
                        expected_ops
                    );
                }
            }
            println!(
                "FUNCTIONAL_RUNTIME_SAMPLE {}",
                serde_json::json!({
                    "route": route_name, "lap": lap, "position": position,
                    "iterations": ITERATIONS, "elapsed_ns": elapsed.as_nanos(),
                    "scope": "call_and_preallocated_result_retention_excludes_context_setup_checks_and_result_teardown",
                    "allocation_measurement": "not_performed",
                })
            );
        }
    }
    assert_eq!(initial.get_ref(), &initial_state);
}

#[test]
fn pure_values_literals_defaults_enums_and_equality_match_original() {
    use pure_fixture::ledger_contract as c;
    let seed = c::initial_state(ConstructorContext::new(7_u64)).unwrap();
    let state = seed.ledger_state.get_ref();
    assert_eq!(
        pure_fixture::ledger_slots::bytes.inspect(state).unwrap(),
        runtime::FixedBytes::new(*b"xyz")
    );
    assert_eq!(
        pure_fixture::ledger_slots::small
            .inspect(state)
            .unwrap()
            .value(),
        9
    );
    assert_eq!(
        pure_fixture::ledger_slots::wide
            .inspect(state)
            .unwrap()
            .value(),
        9
    );
    assert_eq!(
        pure_fixture::ledger_slots::choice.inspect(state).unwrap(),
        pure_fixture::types::Choice::third
    );
    for literals in [false, true] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let result = outcomes(
            c::choose(context(), literals),
            c::recorded::choose(context(), literals),
            c::functional::choose::Program::new(literals).execute_native(context()),
            c::functional::choose::Program::new(literals).execute_recorded(context()),
            None,
        )
        .unwrap();
        assert_eq!(result.result, literals);
        let state = result.context.query.state.get_ref();
        assert_eq!(
            pure_fixture::ledger_slots::bytes.inspect(state).unwrap(),
            runtime::FixedBytes::new(if literals { *b"abc" } else { [0; 3] })
        );
        assert_eq!(
            pure_fixture::ledger_slots::small
                .inspect(state)
                .unwrap()
                .value(),
            if literals { 257 } else { 0 }
        );
        assert_eq!(
            pure_fixture::ledger_slots::wide
                .inspect(state)
                .unwrap()
                .value(),
            if literals { 1_u128 << 64 } else { 0 }
        );
        assert_eq!(
            pure_fixture::ledger_slots::choice.inspect(state).unwrap(),
            if literals {
                pure_fixture::types::Choice::second
            } else {
                pure_fixture::types::Choice::first
            }
        );
    }
}

#[derive(Default)]
struct TypedWitnesses(RefCell<Vec<(u64, &'static str)>>);

impl TypedWitnesses {
    fn event(&self, private: &u64, name: &'static str) -> u64 {
        self.0.borrow_mut().push((*private, name));
        *private + 1
    }
}

impl typed_fixture::ledger_contract::Witnesses<u64> for TypedWitnesses {
    fn gate(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        allow: bool,
    ) -> (u64, bool) {
        (self.event(context.private_state, "gate"), allow)
    }
    fn zeroUnit(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
    ) -> (u64, ()) {
        (self.event(context.private_state, "zeroUnit"), ())
    }
    fn zeroField(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
    ) -> (u64, Field) {
        (
            self.event(context.private_state, "zeroField"),
            Field::from(1_u64),
        )
    }
    fn oneUnit(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        _value: (),
    ) -> (u64, Field) {
        (
            self.event(context.private_state, "oneUnit"),
            Field::from(2_u64),
        )
    }
    fn first(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        assert_eq!(value, Field::from(10_u64));
        (
            self.event(context.private_state, "first"),
            Field::from(11_u64),
        )
    }
    fn second(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        assert_eq!(value, Field::from(11_u64));
        assert_eq!(context.ledger.fieldCell().unwrap(), value);
        (
            self.event(context.private_state, "second"),
            Field::from(22_u64),
        )
    }
    fn byteEcho(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        value: runtime::FixedBytes<3>,
    ) -> (u64, runtime::FixedBytes<3>) {
        assert_eq!(value, runtime::FixedBytes::new(*b"abc"));
        (
            self.event(context.private_state, "byteEcho"),
            runtime::FixedBytes::new(*b"def"),
        )
    }
    fn mixed(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
        bytes: runtime::FixedBytes<3>,
    ) -> (u64, typed_fixture::types::Packet) {
        assert_eq!(value, Field::from(10_u64));
        assert_eq!(bytes, runtime::FixedBytes::new(*b"abc"));
        (
            self.event(context.private_state, "mixed"),
            typed_fixture::types::Packet {
                value: Field::from(20_u64),
                bytes: runtime::FixedBytes::new(*b"def"),
            },
        )
    }
    fn product(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        packet: typed_fixture::types::Packet,
    ) -> (u64, typed_fixture::types::Packet) {
        assert_eq!(packet.value, Field::from(30_u64));
        assert_eq!(packet.bytes, runtime::FixedBytes::new(*b"xyz"));
        (self.event(context.private_state, "product"), packet)
    }
    fn tupleMixed(
        &self,
        context: WitnessContext<'_, u64, typed_fixture::ledger_contract::LedgerView<'_>>,
        pair: (Field, runtime::FixedBytes<3>),
    ) -> (u64, typed_fixture::types::Packet) {
        assert_eq!(
            pair,
            (Field::from(40_u64), runtime::FixedBytes::new(*b"uvw"))
        );
        (
            self.event(context.private_state, "tupleMixed"),
            typed_fixture::types::Packet {
                value: Field::from(40_u64),
                bytes: runtime::FixedBytes::new(*b"uvw"),
            },
        )
    }
}

#[test]
fn typed_witnesses_preserve_identity_arity_values_order_and_guarding() {
    use typed_fixture::ledger_contract as c;
    for (selected, allow) in [(false, false), (true, false), (true, true)] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [TypedWitnesses; 4] = std::array::from_fn(|_| TypedWitnesses::default());
        let value = Field::from(10_u64);
        let bytes = runtime::FixedBytes::new(*b"abc");
        let packet = || typed_fixture::types::Packet {
            value: Field::from(30_u64),
            bytes: runtime::FixedBytes::new(*b"xyz"),
        };
        let pair = (Field::from(40_u64), runtime::FixedBytes::new(*b"uvw"));
        let result = outcomes(
            c::typed(
                context(),
                &witnesses[0],
                selected,
                allow,
                value,
                bytes,
                packet(),
                pair,
            ),
            c::recorded::typed(
                context(),
                &witnesses[1],
                selected,
                allow,
                value,
                bytes,
                packet(),
                pair,
            ),
            c::functional::typed::Program::new(selected, allow, value, bytes, packet(), pair)
                .execute_native(context(), &witnesses[2]),
            c::functional::typed::Program::new(selected, allow, value, bytes, packet(), pair)
                .execute_recorded(context(), &witnesses[3]),
            (selected && !allow).then_some("failed assert: typed denied"),
        );
        let names: &[&str] = if !selected {
            &[]
        } else if !allow {
            &["gate"]
        } else {
            &[
                "gate",
                "zeroUnit",
                "zeroField",
                "oneUnit",
                "first",
                "second",
                "byteEcho",
                "mixed",
                "product",
                "tupleMixed",
            ]
        };
        let events = names
            .iter()
            .enumerate()
            .map(|(index, name)| (7 + index as u64, *name))
            .collect::<Vec<_>>();
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(
                result.result,
                Field::from(if selected { 22_u64 } else { 10_u64 })
            );
            assert_eq!(result.context.private_state, 7 + names.len() as u64);
            let state = result.context.query.state.get_ref();
            assert_eq!(
                typed_fixture::ledger_slots::fieldCell
                    .inspect(state)
                    .unwrap(),
                result.result
            );
            if selected {
                assert_eq!(
                    typed_fixture::ledger_slots::byteCell
                        .inspect(state)
                        .unwrap(),
                    runtime::FixedBytes::new(*b"def")
                );
                assert_eq!(
                    typed_fixture::ledger_slots::packetCell
                        .inspect(state)
                        .unwrap(),
                    typed_fixture::types::Packet {
                        value: Field::from(40_u64),
                        bytes: runtime::FixedBytes::new(*b"uvw")
                    }
                );
                let expected: Vec<runtime::fab::AlignedValue> = vec![
                    true.into(),
                    ().into(),
                    Field::from(1_u64).into(),
                    Field::from(2_u64).into(),
                    Field::from(11_u64).into(),
                    Field::from(22_u64).into(),
                    runtime::FixedBytes::new(*b"def").into(),
                    typed_fixture::types::Packet {
                        value: Field::from(20_u64),
                        bytes: runtime::FixedBytes::new(*b"def"),
                    }
                    .into(),
                    packet().into(),
                    typed_fixture::types::Packet {
                        value: Field::from(40_u64),
                        bytes: runtime::FixedBytes::new(*b"uvw"),
                    }
                    .into(),
                ];
                assert_eq!(result.private_transcript_outputs, expected);
            } else {
                assert!(result.private_transcript_outputs.is_empty());
            }
        }
    }
}
