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

//! Real scalar Map operations use one generic body and both existing frames.
use super::super::ledger8::{
    CellBinding, CounterBinding, Ledger8Bindings, MapBinding, NativeLedger8, RecordedLedger8,
};
use super::super::{Executor, Map, MapLookup, MapRead, MapRemove, MapWrite, SlotId};
use crate::{
    CompactError, Field,
    context::{CircuitContext, ConstructorContext, ConstructorResult},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_map, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const MAP: Map<bool, Field> = Map::new(SlotId::new(31));
const BINDINGS: [MapBinding; 1] = [MapBinding::new(MAP.id(), &[0, 0])];
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(73_u64),
        contract_state(vec![StateValue::Array(vec![constructor_map()].into())]),
    )
    .into_circuit_context(ContractAddress::default())
}
#[derive(Debug, PartialEq, Eq)]
struct MapObservation {
    initially_empty: bool,
    inserted_present: bool,
    stored_value: Field,
    populated_size: u64,
    finally_empty: bool,
}
fn exercise<
    E: Executor
        + MapRead<bool, Field>
        + MapLookup<bool, Field>
        + MapRemove<bool, Field>
        + MapWrite<bool, Field>,
>(
    executor: E,
) -> Result<E::Outcome<MapObservation>, CompactError> {
    let (executor, empty) = executor.map_is_empty(MAP)?;
    let executor = executor.map_insert(MAP, true, Field::from(42_u64))?;
    let (executor, present) = executor.map_member(MAP, true)?;
    let (executor, value) = executor.map_lookup(MAP, true)?;
    let (executor, size) = executor.map_size(MAP)?;
    let executor = executor.map_remove(MAP, true)?;
    let (executor, removed_present) = executor.map_member(MAP, true)?;
    let (executor, removed_size) = executor.map_size(MAP)?;
    assert!(!removed_present);
    assert_eq!(removed_size, 0);
    let executor = executor.map_insert_default(MAP, false)?;
    let (executor, default) = executor.map_lookup(MAP, false)?;
    assert_eq!(default, Field::from(0_u64));
    let executor = executor.map_reset(MAP)?;
    let (executor, final_empty) = executor.map_is_empty(MAP)?;
    Ok(executor.finish(MapObservation {
        initially_empty: empty,
        inserted_present: present,
        stored_value: value,
        populated_size: size,
        finally_empty: final_empty,
    }))
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut bytes = Vec::new();
    tagged_serialize(state, &mut bytes).unwrap();
    bytes
}
#[test]
fn all_eight_map_operations_preserve_native_recorded_and_replay_parity() {
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let native = exercise(NativeLedger8::new(context(), bindings)).unwrap();
    let recorded = exercise(RecordedLedger8::new(context(), bindings)).unwrap();
    assert_eq!(
        native.result,
        MapObservation {
            initially_empty: true,
            inserted_present: true,
            stored_value: Field::from(42_u64),
            populated_size: 1,
            finally_empty: true,
        }
    );
    assert_eq!(native.context.private_state, 73);
    assert_eq!(recorded.execution.context.private_state, 73);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        native.context.private_state,
        recorded.execution.context.private_state
    );
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
fn unbound_maps_and_missing_keys_use_existing_refusals() {
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = NativeLedger8::new(context(), empty)
        .map_lookup(MAP, true)
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context(), empty)
        .map_lookup(MAP, true)
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let native = NativeLedger8::new(context(), bindings)
        .map_lookup(MAP, true)
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context(), bindings)
        .map_lookup(MAP, true)
        .err()
        .unwrap();
    assert_eq!(native, recorded);
}
#[test]
fn map_bindings_reject_cross_kind_identity_and_path_aliases() {
    for (id, path) in [
        (31, &[1][..]),
        (32, &[0][..]),
        (32, &[0, 0][..]),
        (32, &[0, 0, 1][..]),
    ] {
        let cells = [CellBinding::new(SlotId::new(id), path)];
        let counters = [CounterBinding::new(SlotId::new(id), path)];
        assert!(Ledger8Bindings::with_maps(&cells, &[], &BINDINGS).is_err());
        assert!(Ledger8Bindings::with_maps(&[], &counters, &BINDINGS).is_err());
        let maps = [BINDINGS[0], MapBinding::new(SlotId::new(id), path)];
        assert!(Ledger8Bindings::with_maps(&[], &[], &maps).is_err());
    }
    assert!(
        Ledger8Bindings::with_maps(
            &[CellBinding::new(SlotId::new(32), &[0, 1])],
            &[],
            &BINDINGS
        )
        .is_ok()
    );
}

#[test]
fn scalar_maps_retain_owned_keys_and_values_through_existing_codecs() {
    use crate::{OpaqueBytes, OpaqueString};
    let map: Map<OpaqueString, OpaqueBytes> = Map::new(MAP.id());
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let key = || OpaqueString("owned-key".into());
    let value = || OpaqueBytes(vec![1, 2, 3]);
    let native = NativeLedger8::new(context(), bindings)
        .map_insert(map, key(), value())
        .unwrap();
    let recorded = RecordedLedger8::new(context(), bindings)
        .map_insert(map, key(), value())
        .unwrap();
    let (native, native_value) = native.map_lookup(map, key()).unwrap();
    let (recorded, recorded_value) = recorded.map_lookup(map, key()).unwrap();
    assert_eq!(native_value, value());
    assert_eq!(native_value, recorded_value);
    let native = native.finish(native_value);
    let recorded = recorded.finish(recorded_value);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}

#[test]
fn map_carrier_preserves_prior_guards_and_missing_key_failure_order() {
    use super::super::{Circuit, Rule};
    use std::cell::Cell;
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = Circuit::begin(NativeLedger8::new(context(), empty))
        .require(Rule::new(false, "map guard"))
        .map_lookup(MAP, true)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(), empty))
        .require(Rule::new(false, "map guard"))
        .map_lookup(MAP, true)
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::AssertionFailed("map guard".into()));
    assert_eq!(native, recorded);
    let calls = Cell::new(0);
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let result = Circuit::begin(NativeLedger8::new(context(), bindings))
        .map_lookup(MAP, true)
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
    let result = Circuit::begin(RecordedLedger8::new(context(), bindings))
        .map_lookup(MAP, true)
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

#[test]
fn map_reads_enforce_existing_zero_gas_limit_before_later_producers() {
    use super::super::Circuit;
    use std::cell::Cell;
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let mut native_context = context();
    native_context.gas_limit = Some(crate::context::RunningCost::ZERO);
    let mut recorded_context = context();
    recorded_context.gas_limit = Some(crate::context::RunningCost::ZERO);
    let calls = Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(native_context, bindings))
        .map_member(MAP, true)
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(recorded_context, bindings))
        .map_member(MAP, true)
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

fn nested_context() -> CircuitContext<u64, DefaultDB> {
    use crate::{
        fab::AlignedValue,
        ledger::{LedgerHashMap, constructor_cell},
    };
    let child = |value| {
        StateValue::Map(LedgerHashMap::new().insert(
            AlignedValue::from(true),
            constructor_cell::<_, DefaultDB>(Field::from(value)),
        ))
    };
    let entries = LedgerHashMap::new()
        .insert(AlignedValue::from(true), child(10_u64))
        .insert(AlignedValue::from(false), child(90_u64));
    ConstructorResult::new(
        ConstructorContext::new(73_u64),
        contract_state(vec![StateValue::Array(
            vec![StateValue::Map(entries)].into(),
        )]),
    )
    .into_circuit_context(ContractAddress::default())
}
#[test]
fn nested_maps_inspect_and_remove_without_scalar_value_codec() {
    use crate::slots::MapNode;
    let map: Map<bool, MapNode<bool, Field>> = Map::new(MAP.id());
    fn run<
        E: Executor
            + MapRead<bool, crate::slots::MapNode<bool, Field>>
            + MapRemove<bool, crate::slots::MapNode<bool, Field>>,
    >(
        executor: E,
        map: Map<bool, crate::slots::MapNode<bool, Field>>,
    ) -> Result<E::Outcome<u64>, CompactError> {
        let (executor, size) = executor.map_size(map)?;
        assert_eq!(size, 2);
        let (executor, present) = executor.map_member(map, true)?;
        assert!(present);
        let executor = executor.map_remove(map, true)?;
        let (executor, present) = executor.map_member(map, true)?;
        assert!(!present);
        // Removing the now-absent child preserves its sibling and size.
        let executor = executor.map_remove(map, true)?;
        let (executor, sibling) = executor.map_member(map, false)?;
        assert!(sibling);
        let (executor, size) = executor.map_size(map)?;
        assert_eq!(size, 1);
        let (executor, empty) = executor.map_is_empty(map)?;
        assert!(!empty);
        Ok(executor.finish(size))
    }
    let bindings = Ledger8Bindings::with_maps(&[], &[], &BINDINGS).unwrap();
    let native = run(NativeLedger8::new(nested_context(), bindings), map).unwrap();
    let recorded = run(RecordedLedger8::new(nested_context(), bindings), map).unwrap();
    assert_eq!(native.result, 1);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(native.context.private_state, 73);
    assert_eq!(recorded.execution.context.private_state, 73);
    assert!(native.private_transcript_outputs.is_empty());
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    // Compare the entire surviving child, not membership alone.
    let StateValue::Array(chunks) = native.context.query.state.get_ref() else {
        panic!("expected chunk array")
    };
    let StateValue::Array(fields) = chunks.get(0).unwrap() else {
        panic!("expected nested fields")
    };
    let StateValue::Map(entries) = fields.get(0).unwrap() else {
        panic!("expected outer Map")
    };
    let sibling = entries.get(&crate::fab::AlignedValue::from(false)).unwrap();
    let expected = StateValue::Map(crate::ledger::LedgerHashMap::new().insert(
        crate::fab::AlignedValue::from(true),
        crate::ledger::constructor_cell::<_, DefaultDB>(Field::from(90_u64)),
    ));
    assert_eq!(bytes(&sibling), bytes(&expected));
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
fn nested_map_guards_and_unbound_identity_preserve_refusals() {
    use super::super::{Circuit, Rule};
    let map: Map<bool, crate::slots::MapNode<bool, Field>> = Map::new(MAP.id());
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = NativeLedger8::new(nested_context(), empty)
        .map_member(map, true)
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(nested_context(), empty)
        .map_member(map, true)
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let native = Circuit::begin(NativeLedger8::new(nested_context(), empty))
        .require(Rule::new(false, "nested map guard"))
        .map_remove(map, true)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(nested_context(), empty))
        .require(Rule::new(false, "nested map guard"))
        .map_remove(map, true)
        .finish()
        .err()
        .unwrap();
    assert_eq!(
        native,
        CompactError::AssertionFailed("nested map guard".into())
    );
    assert_eq!(native, recorded);
}
