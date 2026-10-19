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

//! Real Set parity through both existing interpreters, without coin allocation.
use super::super::ledger8::{
    CellBinding, CounterBinding, Ledger8Bindings, MapBinding, NativeLedger8, RecordedLedger8,
    SetBinding,
};
use super::super::{Circuit, Executor, Rule, Set, SetRead, SetWrite, SlotId};
use crate::{
    CompactError, Field, OpaqueString,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_set, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const SET: Set<Field> = Set::new(SlotId::new(41));
const DECLARATIONS: [SetBinding; 1] = [SetBinding::new(SET.id(), &[0, 0])];
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(91),
        contract_state(vec![StateValue::Array(vec![constructor_set()].into())]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bindings() -> Ledger8Bindings<'static> {
    Ledger8Bindings::new(&[])
        .unwrap()
        .bind_sets(&DECLARATIONS)
        .unwrap()
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut bytes = vec![];
    tagged_serialize(state, &mut bytes).unwrap();
    bytes
}
fn run<E: Executor + SetRead<Field> + SetWrite<Field>>(
    executor: E,
) -> Result<E::Outcome<bool>, CompactError> {
    let (executor, empty) = executor.set_is_empty(SET)?;
    assert!(empty);
    let executor = executor.set_insert(SET, Field::from(42_u64))?;
    let (executor, present) = executor.set_member(SET, Field::from(42_u64))?;
    assert!(present);
    let (executor, size) = executor.set_size(SET)?;
    assert_eq!(size, 1);
    let executor = executor.set_remove(SET, Field::from(42_u64))?;
    let (executor, present) = executor.set_member(SET, Field::from(42_u64))?;
    assert!(!present);
    let (executor, size) = executor.set_size(SET)?;
    assert_eq!(size, 0);
    let executor = executor.set_insert(SET, Field::from(9_u64))?;
    let executor = executor.set_reset(SET)?;
    let (executor, empty) = executor.set_is_empty(SET)?;
    assert!(empty);
    Ok(executor.finish(empty))
}
#[test]
fn six_set_operations_preserve_results_state_gas_and_replay() {
    let native = run(NativeLedger8::new(context(), bindings())).unwrap();
    let recorded = run(RecordedLedger8::new(context(), bindings())).unwrap();
    assert!(native.result);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.context.private_state, 91);
    assert_eq!(recorded.execution.context.private_state, 91);
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
fn set_bindings_reject_aliases_across_all_existing_kinds() {
    for (id, path) in [
        (41, &[1][..]),
        (42, &[0][..]),
        (42, &[0, 0][..]),
        (42, &[0, 0, 1][..]),
    ] {
        let cells = [CellBinding::new(SlotId::new(id), path)];
        let counters = [CounterBinding::new(SlotId::new(id), path)];
        let maps = [MapBinding::new(SlotId::new(id), path)];
        assert!(
            Ledger8Bindings::new(&cells)
                .unwrap()
                .bind_sets(&DECLARATIONS)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_counters(&[], &counters)
                .unwrap()
                .bind_sets(&DECLARATIONS)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_maps(&[], &[], &maps)
                .unwrap()
                .bind_sets(&DECLARATIONS)
                .is_err()
        );
        let sets = [DECLARATIONS[0], SetBinding::new(SlotId::new(id), path)];
        assert!(Ledger8Bindings::new(&[]).unwrap().bind_sets(&sets).is_err());
    }
    let cells = [CellBinding::new(SlotId::new(42), &[0, 1])];
    assert!(
        Ledger8Bindings::new(&cells)
            .unwrap()
            .bind_sets(&DECLARATIONS)
            .is_ok()
    );
    assert!(bindings().bind_sets(&DECLARATIONS).is_err());
}
#[test]
fn noncopy_set_values_delegate_existing_codecs() {
    let set: Set<OpaqueString> = Set::new(SET.id());
    let value = || OpaqueString("owned-set-value".into());
    let native = NativeLedger8::new(context(), bindings())
        .set_insert(set, value())
        .unwrap();
    let recorded = RecordedLedger8::new(context(), bindings())
        .set_insert(set, value())
        .unwrap();
    let (native, present) = native.set_member(set, value()).unwrap();
    let (recorded, recorded_present) = recorded.set_member(set, value()).unwrap();
    assert!(present);
    assert_eq!(present, recorded_present);
    let native = native.finish(present);
    let recorded = recorded.finish(recorded_present);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}
#[test]
fn unbound_sets_and_prior_guards_keep_original_refusals() {
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = NativeLedger8::new(context(), empty)
        .set_member(SET, Field::from(1_u64))
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context(), empty)
        .set_member(SET, Field::from(1_u64))
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let native = Circuit::begin(NativeLedger8::new(context(), empty))
        .require(Rule::new(false, "set guard"))
        .set_member(SET, Field::from(1_u64))
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(), empty))
        .require(Rule::new(false, "set guard"))
        .set_member(SET, Field::from(1_u64))
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::AssertionFailed("set guard".into()));
    assert_eq!(native, recorded);
}
#[test]
fn zero_gas_set_reads_suppress_later_producers() {
    let mut first = context();
    first.gas_limit = Some(RunningCost::ZERO);
    let mut second = context();
    second.gas_limit = Some(RunningCost::ZERO);
    let calls = std::cell::Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(first, bindings()))
        .set_member(SET, Field::from(1_u64))
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(second, bindings()))
        .set_member(SET, Field::from(1_u64))
        .bind(|_, circuit| {
            circuit.try_returning(|| {
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
