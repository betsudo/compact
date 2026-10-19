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

//! Remaining Counter operations preserve official comparison and mutation policy.
use super::super::ledger8::{CounterBinding, Ledger8Bindings, NativeLedger8, RecordedLedger8};
use super::super::{
    Circuit, Counter, CounterCompare, CounterDecrement, CounterIncrement, CounterRead,
    CounterReset, Executor, Rule, SlotId,
};
use crate::{
    CompactError,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_cell, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const COUNTER: Counter = Counter::new(SlotId::new(61));
const DECLARATIONS: [CounterBinding; 1] = [CounterBinding::new(COUNTER.id(), &[0, 0])];
fn context(value: u64) -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(97),
        contract_state(vec![StateValue::Array(
            vec![constructor_cell(value)].into(),
        )]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bindings() -> Ledger8Bindings<'static> {
    Ledger8Bindings::with_counters(&[], &DECLARATIONS).unwrap()
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut bytes = vec![];
    tagged_serialize(state, &mut bytes).unwrap();
    bytes
}
fn run<
    E: Executor + CounterRead + CounterIncrement + CounterCompare + CounterDecrement + CounterReset,
>(
    executor: E,
    amount: u16,
) -> Result<E::Outcome<u64>, CompactError> {
    let (executor, less) = executor.counter_less_than(COUNTER, 0)?;
    assert!(!less);
    let (executor, less) = executor.counter_less_than(COUNTER, u64::MAX)?;
    assert!(less);
    let executor = executor.increment(COUNTER, amount)?;
    let (executor, value) = executor.read_counter(COUNTER)?;
    assert_eq!(value, amount as u64);
    let (executor, less) = executor.counter_less_than(COUNTER, amount as u64)?;
    assert!(!less);
    let (executor, less) = executor.counter_less_than(COUNTER, amount as u64 + 1)?;
    assert!(less);
    let executor = executor.decrement(COUNTER, amount)?;
    let (executor, value) = executor.read_counter(COUNTER)?;
    assert_eq!(value, 0);
    let executor = executor.increment(COUNTER, 5)?;
    let executor = executor.reset_counter(COUNTER)?;
    let (executor, value) = executor.read_counter(COUNTER)?;
    assert_eq!(value, 0);
    Ok(executor.finish(value))
}
#[test]
fn counter_boundaries_reset_comparison_and_replay_keep_existing_results() {
    for amount in [0, 1, u16::MAX] {
        let native = run(NativeLedger8::new(context(0), bindings()), amount).unwrap();
        let recorded = run(RecordedLedger8::new(context(0), bindings()), amount).unwrap();
        assert_eq!(native.result, 0);
        assert_eq!(native.result, recorded.execution.result);
        assert_eq!(native.context.private_state, 97);
        assert_eq!(recorded.execution.context.private_state, 97);
        assert_eq!(native.gas_cost, recorded.execution.gas_cost);
        assert!(native.private_transcript_outputs.is_empty());
        assert!(recorded.execution.private_transcript_outputs.is_empty());
        assert_eq!(
            bytes(native.context.query.state.get_ref()),
            bytes(recorded.execution.context.query.state.get_ref())
        );
        let replay = recorded
            .public
            .initial()
            .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
            .unwrap();
        assert_eq!(
            bytes(replay.context.state.get_ref()),
            bytes(recorded.execution.context.query.state.get_ref())
        );
        assert_eq!(
            replay.context.effects,
            recorded.execution.context.query.effects
        );
    }
}
#[test]
fn maximum_counter_threshold_is_not_truncated() {
    let native = NativeLedger8::new(context(u64::MAX), bindings())
        .counter_less_than(COUNTER, u64::MAX)
        .unwrap();
    let recorded = RecordedLedger8::new(context(u64::MAX), bindings())
        .counter_less_than(COUNTER, u64::MAX)
        .unwrap();
    assert!(!native.1);
    assert_eq!(native.1, recorded.1);
    let native = native.0.finish(native.1);
    let recorded = recorded.0.finish(recorded.1);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
}
#[test]
fn decrement_underflow_keeps_existing_slot_outcome_without_new_arithmetic() {
    let slot = crate::slots::CounterSlot::new(&[0, 0]);
    let reference = slot.decrement(context(0), 1);
    let candidate = NativeLedger8::new(context(0), bindings()).decrement(COUNTER, 1);
    match (reference, candidate) {
        (Ok(reference), Ok(candidate)) => {
            let candidate = candidate.finish(());
            assert_eq!(
                bytes(reference.context.query.state.get_ref()),
                bytes(candidate.context.query.state.get_ref())
            );
            assert_eq!(reference.gas_cost, candidate.gas_cost);
        }
        (Err(reference), Err(candidate)) => assert_eq!(reference, candidate),
        _ => panic!("native underflow policy changed"),
    }
    let reference = slot.record_decrement(crate::recording::RecordingFrame::new(context(0)), 1);
    let candidate = RecordedLedger8::new(context(0), bindings()).decrement(COUNTER, 1);
    match (reference, candidate) {
        (Ok(reference), Ok(candidate)) => {
            let reference = reference.finish(());
            let candidate = candidate.finish(());
            assert_eq!(
                bytes(reference.execution.context.query.state.get_ref()),
                bytes(candidate.execution.context.query.state.get_ref())
            );
            assert_eq!(reference.execution.gas_cost, candidate.execution.gas_cost);
        }
        (Err(reference), Err(candidate)) => assert_eq!(reference, candidate),
        _ => panic!("recorded underflow policy changed"),
    }
}
#[test]
fn prior_guards_unbound_identity_and_gas_limits_keep_failure_order() {
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = NativeLedger8::new(context(0), empty)
        .decrement(COUNTER, 0)
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context(0), empty)
        .decrement(COUNTER, 0)
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let native = Circuit::begin(NativeLedger8::new(context(0), empty))
        .require(Rule::new(false, "counter guard"))
        .reset_counter(COUNTER)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(0), empty))
        .require(Rule::new(false, "counter guard"))
        .reset_counter(COUNTER)
        .finish()
        .err()
        .unwrap();
    assert_eq!(
        native,
        CompactError::AssertionFailed("counter guard".into())
    );
    assert_eq!(native, recorded);
    let mut first = context(0);
    first.gas_limit = Some(RunningCost::ZERO);
    let mut second = context(0);
    second.gas_limit = Some(RunningCost::ZERO);
    let calls = std::cell::Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(first, bindings()))
        .counter_less_than(COUNTER, 1)
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(second, bindings()))
        .counter_less_than(COUNTER, 1)
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    assert!(matches!(native, Err(CompactError::LedgerQueryRejected(_))));
    assert!(matches!(
        recorded,
        Err(CompactError::LedgerQueryRejected(_))
    ));
    assert_eq!(calls.get(), 0);
}
