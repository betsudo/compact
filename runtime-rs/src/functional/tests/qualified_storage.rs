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

//! Explicit held call-context authority mirrors existing qualified_set_paths tests.
//! This setup is not an offer allocator, wallet spend or network-authentication claim.
use super::super::ledger8::{
    CellBinding, Ledger8Bindings, NativeLedger8, QualifiedCellWrite, QualifiedSetInsert,
    RecordedLedger8, SetBinding,
};
use super::super::{Cell, Circuit, Executor, Rule, Set, SlotId};
use crate::{
    CompactError, FixedBytes,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{
        self, ContractAddress, DefaultDB, StateValue, constructor_cell, constructor_set,
        contract_state,
    },
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
type Qualified = (FixedBytes<32>, FixedBytes<32>, u128, u64);
const CELL: Cell<Qualified> = Cell::new(SlotId::new(81));
const SET: Set<Qualified> = Set::new(SlotId::new(82));
const CELLS: [CellBinding; 1] = [CellBinding::new(CELL.id(), &[0, 0])];
const SETS: [SetBinding; 1] = [SetBinding::new(SET.id(), &[0, 1])];
fn bindings() -> Ledger8Bindings<'static> {
    Ledger8Bindings::new(&CELLS)
        .unwrap()
        .bind_sets(&SETS)
        .unwrap()
}
fn coin() -> ledger::CoinInfo {
    ledger::coin_info_from_compact(FixedBytes::new([1; 32]), FixedBytes::new([2; 32]), 42)
}
fn recipient() -> ledger::CoinRecipient {
    ledger::coin_recipient_from_compact(true, FixedBytes::new([3; 32]), FixedBytes::new([0; 32]))
}
fn initial(index: Option<u64>) -> CircuitContext<u64, DefaultDB> {
    let value: Qualified = (FixedBytes::new([0; 32]), FixedBytes::new([0; 32]), 0, 0);
    let state = contract_state(vec![StateValue::Array(
        vec![constructor_cell(value), constructor_set()].into(),
    )]);
    let mut context = ConstructorResult::new(ConstructorContext::new(107), state)
        .into_circuit_context(ContractAddress::default());
    // Seed existing official call authority exactly as the retained path tests do.
    if let Some(index) = index {
        context.query.call_context.com_indices = context
            .query
            .call_context
            .com_indices
            .insert(coin().commitment(&recipient()), index);
    }
    context
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut out = vec![];
    tagged_serialize(state, &mut out).unwrap();
    out
}
fn store<E: Executor + QualifiedCellWrite<Qualified> + QualifiedSetInsert<Qualified>>(
    executor: E,
) -> Result<E::Outcome<()>, CompactError> {
    let executor = executor.write_qualified_coin(CELL, coin(), recipient())?;
    let executor = executor.insert_qualified_coin(SET, coin(), recipient())?;
    Ok(executor.finish(()))
}
#[test]
fn qualified_storage_uses_existing_index_authority_without_new_intents() {
    for index in [0, 7, u64::MAX] {
        let authority = initial(Some(index)).query.call_context.com_indices.clone();
        let native = store(NativeLedger8::new(initial(Some(index)), bindings())).unwrap();
        let recorded = store(RecordedLedger8::new(initial(Some(index)), bindings())).unwrap();
        let expected = (
            FixedBytes::new([1; 32]),
            FixedBytes::new([2; 32]),
            42_u128,
            index,
        );
        let actual = crate::slots::CellSlot::<Qualified>::new(&[0, 0])
            .inspect(native.context.query.state.get_ref())
            .unwrap();
        assert_eq!(actual, expected);
        let member = ledger::member_set(
            &native.context.query,
            &[0, 1],
            expected,
            None,
            &native.context.cost_model,
        )
        .unwrap();
        assert!(member.1);
        assert_eq!(native.context.private_state, 107);
        assert_eq!(recorded.execution.context.private_state, 107);
        assert!(native.private_transcript_outputs.is_empty());
        assert!(recorded.execution.private_transcript_outputs.is_empty());
        assert!(native.context.circuit_zswap().inputs().is_empty());
        assert!(native.context.circuit_zswap().outputs().is_empty());
        assert_eq!(
            native.context.circuit_zswap(),
            recorded.execution.context.circuit_zswap()
        );
        assert_eq!(native.context.query.call_context.com_indices, authority);
        assert_eq!(
            recorded.execution.context.query.call_context.com_indices,
            authority
        );
        assert_eq!(native.gas_cost, recorded.execution.gas_cost);
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
            bytes(native.context.query.state.get_ref())
        );
        assert_eq!(replay.context.effects, native.context.query.effects);
    }
}
#[test]
fn missing_commitment_and_wrong_recipient_keep_original_refusals() {
    for wrong in [false, true] {
        let context = || initial(if wrong { Some(7) } else { None });
        let target = || {
            if wrong {
                ledger::coin_recipient_from_compact(
                    true,
                    FixedBytes::new([4; 32]),
                    FixedBytes::new([0; 32]),
                )
            } else {
                recipient()
            }
        };
        let native = NativeLedger8::new(context(), bindings())
            .write_qualified_coin(CELL, coin(), target())
            .err()
            .unwrap();
        let recorded = RecordedLedger8::new(context(), bindings())
            .write_qualified_coin(CELL, coin(), target())
            .err()
            .unwrap();
        let reference = crate::slots::CellSlot::<Qualified>::new(&[0, 0])
            .write_coin(context(), coin(), target())
            .err()
            .unwrap();
        assert_eq!(native, reference);
        assert_eq!(native, recorded);
        assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
        let native = NativeLedger8::new(context(), bindings())
            .insert_qualified_coin(SET, coin(), target())
            .err()
            .unwrap();
        let recorded = RecordedLedger8::new(context(), bindings())
            .insert_qualified_coin(SET, coin(), target())
            .err()
            .unwrap();
        let reference = crate::slots::SetSlot::<Qualified>::new(&[0, 1])
            .insert_coin(context(), coin(), target())
            .err()
            .unwrap();
        assert_eq!(native, reference);
        assert_eq!(native, recorded);
    }
}
#[test]
fn wrong_storage_alignment_is_not_accepted_by_handwritten_tokens() {
    let cell: Cell<bool> = Cell::new(CELL.id());
    let set: Set<bool> = Set::new(SET.id());
    let native = NativeLedger8::new(initial(Some(7)), bindings())
        .write_qualified_coin(cell, coin(), recipient())
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(initial(Some(7)), bindings())
        .write_qualified_coin(cell, coin(), recipient())
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let native = NativeLedger8::new(initial(Some(7)), bindings())
        .insert_qualified_coin(set, coin(), recipient())
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(initial(Some(7)), bindings())
        .insert_qualified_coin(set, coin(), recipient())
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
}
#[test]
fn guards_and_query_gas_stop_later_qualified_storage_producers() {
    let native = Circuit::begin(NativeLedger8::new(initial(None), bindings()))
        .require(Rule::new(false, "qualified guard"))
        .write_qualified_coin(CELL, coin(), recipient())
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(initial(None), bindings()))
        .require(Rule::new(false, "qualified guard"))
        .write_qualified_coin(CELL, coin(), recipient())
        .finish()
        .err()
        .unwrap();
    assert_eq!(
        native,
        CompactError::AssertionFailed("qualified guard".into())
    );
    assert_eq!(native, recorded);
    for set in [false, true] {
        let mut first = initial(Some(7));
        first.gas_limit = Some(RunningCost::ZERO);
        let mut second = initial(Some(7));
        second.gas_limit = Some(RunningCost::ZERO);
        let calls = std::cell::Cell::new(0);
        let native = Circuit::begin(NativeLedger8::new(first, bindings()));
        let recorded = Circuit::begin(RecordedLedger8::new(second, bindings()));
        let native = if set {
            native.insert_qualified_coin(SET, coin(), recipient())
        } else {
            native.write_qualified_coin(CELL, coin(), recipient())
        };
        let recorded = if set {
            recorded.insert_qualified_coin(SET, coin(), recipient())
        } else {
            recorded.write_qualified_coin(CELL, coin(), recipient())
        };
        let native = native
            .bind(|(), c| {
                c.try_returning(|| {
                    calls.set(calls.get() + 1);
                    Ok(())
                })
            })
            .finish();
        let recorded = recorded
            .bind(|(), c| {
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
}
