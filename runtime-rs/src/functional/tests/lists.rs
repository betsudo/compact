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

//! Existing List codecs, head defaults and empty-pop policy remain authoritative.
use super::super::ledger8::{
    CellBinding, Ledger8Bindings, ListBinding, NativeLedger8, RecordedLedger8,
};
use super::super::{Circuit, Executor, List, ListHead, ListRead, ListWrite, Rule, SlotId};
use crate::{
    CompactError, Field,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_list, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const LIST: List<Field> = List::new(SlotId::new(51));
const DECLARATIONS: [ListBinding; 1] = [ListBinding::new(LIST.id(), &[0, 0])];
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(93),
        contract_state(vec![StateValue::Array(vec![constructor_list()].into())]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bindings() -> Ledger8Bindings<'static> {
    Ledger8Bindings::new(&[])
        .unwrap()
        .bind_lists(&DECLARATIONS)
        .unwrap()
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut out = vec![];
    tagged_serialize(state, &mut out).unwrap();
    out
}
fn run<E: Executor + ListRead<Field> + ListHead<Field, (bool, Field)> + ListWrite<Field>>(
    executor: E,
) -> Result<E::Outcome<u64>, CompactError> {
    let (executor, head) = executor.list_head(LIST)?;
    assert_eq!(head, (false, Field::from(0_u64)));
    let (executor, empty) = executor.list_is_empty(LIST)?;
    assert!(empty);
    let (executor, length) = executor.list_length(LIST)?;
    assert_eq!(length, 0);
    let executor = executor.list_push_front(LIST, Field::from(7_u64))?;
    let executor = executor.list_push_front(LIST, Field::from(8_u64))?;
    let (executor, head) = executor.list_head(LIST)?;
    assert_eq!(head, (true, Field::from(8_u64)));
    let (executor, length) = executor.list_length(LIST)?;
    assert_eq!(length, 2);
    let executor = executor.list_pop_front(LIST)?;
    let (executor, head) = executor.list_head(LIST)?;
    assert_eq!(head, (true, Field::from(7_u64)));
    let (executor, length) = executor.list_length(LIST)?;
    assert_eq!(length, 1);
    let executor = executor.list_reset(LIST)?;
    let (executor, head) = executor.list_head(LIST)?;
    assert_eq!(head, (false, Field::from(0_u64)));
    let (executor, length) = executor.list_length(LIST)?;
    assert_eq!(length, 0);
    Ok(executor.finish(length))
}
#[test]
fn six_list_operations_keep_head_order_defaults_and_replay() {
    let native = run(NativeLedger8::new(context(), bindings())).unwrap();
    let recorded = run(RecordedLedger8::new(context(), bindings())).unwrap();
    assert_eq!(native.result, 0);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.context.private_state, 93);
    assert_eq!(recorded.execution.context.private_state, 93);
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
#[test]
fn list_bindings_and_prior_guards_preserve_authority() {
    let cells = [CellBinding::new(SlotId::new(52), &[0])];
    assert!(
        Ledger8Bindings::new(&cells)
            .unwrap()
            .bind_lists(&DECLARATIONS)
            .is_err()
    );
    let same = [CellBinding::new(LIST.id(), &[1])];
    assert!(
        Ledger8Bindings::new(&same)
            .unwrap()
            .bind_lists(&DECLARATIONS)
            .is_err()
    );
    let duplicate = [DECLARATIONS[0], ListBinding::new(SlotId::new(52), &[0, 0])];
    assert!(
        Ledger8Bindings::new(&[])
            .unwrap()
            .bind_lists(&duplicate)
            .is_err()
    );
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = NativeLedger8::new(context(), empty)
        .list_length(LIST)
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context(), empty)
        .list_length(LIST)
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let native = Circuit::begin(NativeLedger8::new(context(), empty))
        .require(Rule::new(false, "list guard"))
        .list_length(LIST)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(), empty))
        .require(Rule::new(false, "list guard"))
        .list_length(LIST)
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::AssertionFailed("list guard".into()));
    assert_eq!(native, recorded);
}
#[test]
fn list_read_gas_failure_stops_later_producers() {
    let mut first = context();
    first.gas_limit = Some(RunningCost::ZERO);
    let mut second = context();
    second.gas_limit = Some(RunningCost::ZERO);
    let calls = std::cell::Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(first, bindings()))
        .list_length(LIST)
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(second, bindings()))
        .list_length(LIST)
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
#[test]
fn empty_pop_matches_existing_slots_without_new_policy() {
    let slot = crate::slots::ListSlot::<Field>::new(&[0, 0]);
    let reference = slot.pop_front(context());
    let candidate = NativeLedger8::new(context(), bindings()).list_pop_front(LIST);
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
        _ => panic!("native empty-pop policy changed"),
    }
    let reference = slot.record_pop_front(crate::recording::RecordingFrame::new(context()));
    let candidate = RecordedLedger8::new(context(), bindings()).list_pop_front(LIST);
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
        _ => panic!("recorded empty-pop policy changed"),
    }
}

#[test]
fn noncopy_list_heads_use_the_existing_maybe_carrier() {
    use crate::OpaqueString;
    let list: List<OpaqueString> = List::new(LIST.id());
    let value = || OpaqueString("owned λ list".into());
    let native = NativeLedger8::new(context(), bindings())
        .list_push_front(list, value())
        .unwrap();
    let recorded = RecordedLedger8::new(context(), bindings())
        .list_push_front(list, value())
        .unwrap();
    let (native, head): (_, (bool, OpaqueString)) = native.list_head(list).unwrap();
    let (recorded, recorded_head): (_, (bool, OpaqueString)) = recorded.list_head(list).unwrap();
    assert_eq!(head, (true, value()));
    assert_eq!(head, recorded_head);
    let native = native.finish(head);
    let recorded = recorded.finish(recorded_head);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}

// Deliberately no Default implementation. This proves unrelated List methods
// do not acquire the head/default requirement through a bundled capability.
struct NonDefaultElement(Field);
impl crate::fab::Aligned for NonDefaultElement {
    fn alignment() -> crate::fab::Alignment {
        <Field as crate::fab::Aligned>::alignment()
    }
}
impl From<NonDefaultElement> for crate::fab::Value {
    fn from(value: NonDefaultElement) -> Self {
        value.0.into()
    }
}
impl crate::ledger::CellValue for NonDefaultElement {
    fn decode_cell_value(value: &crate::fab::ValueSlice) -> Result<Self, CompactError> {
        <Field as crate::ledger::CellValue>::decode_cell_value(value).map(Self)
    }
}
#[test]
fn list_inspection_and_mutation_do_not_require_default() {
    let list: List<NonDefaultElement> = List::new(LIST.id());
    let native = NativeLedger8::new(context(), bindings())
        .list_push_front(list, NonDefaultElement(Field::from(5_u64)))
        .unwrap();
    let recorded = RecordedLedger8::new(context(), bindings())
        .list_push_front(list, NonDefaultElement(Field::from(5_u64)))
        .unwrap();
    let (native, length) = native.list_length(list).unwrap();
    let (recorded, recorded_length) = recorded.list_length(list).unwrap();
    assert_eq!(length, 1);
    assert_eq!(length, recorded_length);
    let (native, empty) = native.list_is_empty(list).unwrap();
    let (recorded, recorded_empty) = recorded.list_is_empty(list).unwrap();
    assert!(!empty);
    assert_eq!(empty, recorded_empty);
    let native = native
        .list_pop_front(list)
        .unwrap()
        .list_reset(list)
        .unwrap()
        .finish(());
    let recorded = recorded
        .list_pop_front(list)
        .unwrap()
        .list_reset(list)
        .unwrap()
        .finish(());
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}
#[test]
fn list_binding_aliases_cover_every_existing_kind_and_both_binding_orders() {
    use super::super::ledger8::{CounterBinding, MapBinding, SetBinding};
    for (id, path) in [
        (51, &[1][..]),
        (52, &[0][..]),
        (52, &[0, 0][..]),
        (52, &[0, 0, 1][..]),
    ] {
        let cells = [CellBinding::new(SlotId::new(id), path)];
        let counters = [CounterBinding::new(SlotId::new(id), path)];
        let maps = [MapBinding::new(SlotId::new(id), path)];
        let sets = [SetBinding::new(SlotId::new(id), path)];
        assert!(
            Ledger8Bindings::new(&cells)
                .unwrap()
                .bind_lists(&DECLARATIONS)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_counters(&[], &counters)
                .unwrap()
                .bind_lists(&DECLARATIONS)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_maps(&[], &[], &maps)
                .unwrap()
                .bind_lists(&DECLARATIONS)
                .is_err()
        );
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_sets(&sets)
                .unwrap()
                .bind_lists(&DECLARATIONS)
                .is_err()
        );
        assert!(bindings().bind_sets(&sets).is_err());
        let lists = [DECLARATIONS[0], ListBinding::new(SlotId::new(id), path)];
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_lists(&lists)
                .is_err()
        );
    }
    assert!(bindings().bind_lists(&DECLARATIONS).is_err());
    let cells = [CellBinding::new(SlotId::new(52), &[0, 1])];
    assert!(
        Ledger8Bindings::new(&cells)
            .unwrap()
            .bind_lists(&DECLARATIONS)
            .is_ok()
    );
    let sets = [SetBinding::new(SlotId::new(52), &[0, 1])];
    assert!(bindings().bind_sets(&sets).is_ok());
}
