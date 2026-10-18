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

//! Both checked tree kinds use existing slot semantics and full physical paths.
use super::super::ledger8::{
    CellBinding, Ledger8Bindings, MerkleBinding, NativeLedger8, RecordedLedger8,
};
use super::super::{
    Circuit, Executor, Merkle, MerkleCheckRoot, MerkleInsert, MerkleInsertHash, MerkleInspect,
    MerkleReset, MerkleResetHistory, Rule, SlotId,
};
use crate::{
    BoundedUint, CompactError, Field, FixedBytes,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{
        ContractAddress, DefaultDB, StateValue, constructor_historic_merkle_tree,
        constructor_merkle_tree, contract_state,
    },
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const ID: SlotId = SlotId::new(71);
const PLAIN: [MerkleBinding; 1] = [MerkleBinding::new(ID, &[0, 0], 3, false)];
const HISTORIC: [MerkleBinding; 1] = [MerkleBinding::new(ID, &[0, 0], 3, true)];
fn context(historic: bool) -> CircuitContext<u64, DefaultDB> {
    let tree = if historic {
        constructor_historic_merkle_tree(3)
    } else {
        constructor_merkle_tree(3)
    };
    ConstructorResult::new(
        ConstructorContext::new(101),
        contract_state(vec![StateValue::Array(vec![tree].into())]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bindings(historic: bool) -> Ledger8Bindings<'static> {
    Ledger8Bindings::new(&[])
        .unwrap()
        .bind_merkle(if historic { &HISTORIC } else { &PLAIN })
        .unwrap()
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut bytes = vec![];
    tagged_serialize(state, &mut bytes).unwrap();
    bytes
}
fn run<E, const H: bool>(
    executor: E,
    tree: Merkle<Field, 3, H>,
    root: (Field,),
) -> Result<E::Outcome<bool>, CompactError>
where
    E: Executor
        + MerkleInspect<Field, 3, H>
        + MerkleCheckRoot<Field, (Field,), 3, H>
        + MerkleInsert<Field, 3, H>
        + MerkleInsertHash<Field, 3, H>
        + MerkleReset<Field, 3, H>,
{
    let (executor, valid) = executor.merkle_check_root(tree, root)?;
    assert!(valid);
    let (executor, full) = executor.merkle_is_full(tree)?;
    assert!(!full);
    let executor = executor.merkle_insert(tree, Field::from(7_u64))?;
    let executor =
        executor.merkle_insert_index(tree, Field::from(9_u64), BoundedUint::new(1).unwrap())?;
    let executor = executor.merkle_insert_default(tree, BoundedUint::new(2).unwrap())?;
    let executor = executor.merkle_reset(tree)?;
    let (executor, valid) = executor.merkle_check_root(tree, root)?;
    assert!(valid);
    let executor = executor.merkle_insert_hash(tree, FixedBytes::new([1; 32]))?;
    let executor = executor.merkle_insert_hash_index(
        tree,
        FixedBytes::new([2; 32]),
        BoundedUint::new(1).unwrap(),
    )?;
    let (executor, full) = executor.merkle_is_full(tree)?;
    assert!(!full);
    let executor = executor.merkle_reset(tree)?;
    let (executor, valid) = executor.merkle_check_root(tree, root)?;
    assert!(valid);
    Ok(executor.finish(valid))
}
macro_rules! parity_test {
    ($name:ident,$historic:literal) => {
        #[test]
        fn $name() {
            let tree: Merkle<Field, 3, $historic> = Merkle::new(ID);
            let original = context($historic);
            let root = crate::slots::MerkleSlot::<Field, 3, $historic>::new(&[0, 0])
                .inspect(original.query.state.get_ref())
                .unwrap()
                .root()
                .unwrap();
            let native = run(
                NativeLedger8::new(context($historic), bindings($historic)),
                tree,
                (root.0,),
            )
            .unwrap();
            let recorded = run(
                RecordedLedger8::new(context($historic), bindings($historic)),
                tree,
                (root.0,),
            )
            .unwrap();
            assert!(native.result);
            assert_eq!(native.result, recorded.execution.result);
            assert_eq!(native.gas_cost, recorded.execution.gas_cost);
            assert_eq!(native.context.private_state, 101);
            assert_eq!(recorded.execution.context.private_state, 101);
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
    };
}
parity_test!(plain_all_execution_forms_keep_state_gas_and_replay, false);
parity_test!(
    historic_all_common_execution_forms_keep_state_gas_and_replay,
    true
);
#[test]
fn historic_history_reset_delegates_existing_slots() {
    let tree: Merkle<Field, 3, true> = Merkle::new(ID);
    let slot = crate::slots::MerkleSlot::<Field, 3, true>::new(&[0, 0]);
    let native = NativeLedger8::new(context(true), bindings(true))
        .merkle_insert(tree, Field::from(7_u64))
        .unwrap()
        .merkle_reset_history(tree)
        .unwrap()
        .finish(());
    let recorded = RecordedLedger8::new(context(true), bindings(true))
        .merkle_insert(tree, Field::from(7_u64))
        .unwrap()
        .merkle_reset_history(tree)
        .unwrap()
        .finish(());
    let reference = slot.insert(context(true), Field::from(7_u64)).unwrap();
    let reference = slot.reset_history(reference.context).unwrap();
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(reference.context.query.state.get_ref())
    );
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    assert_eq!(
        bytes(replay.context.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}
#[test]
fn wrong_depth_kind_identity_and_paths_refuse_before_slot_execution() {
    let wrong_depth: Merkle<Field, 4, false> = Merkle::new(ID);
    let wrong_kind: Merkle<Field, 3, true> = Merkle::new(ID);
    assert!(matches!(
        NativeLedger8::new(context(false), bindings(false)).merkle_is_full(wrong_depth),
        Err(CompactError::InvalidLedgerCell(_))
    ));
    assert!(matches!(
        RecordedLedger8::new(context(false), bindings(false)).merkle_is_full(wrong_kind),
        Err(CompactError::InvalidLedgerCell(_))
    ));
    let unbound: Merkle<Field, 3, false> = Merkle::new(SlotId::new(72));
    assert!(matches!(
        NativeLedger8::new(context(false), bindings(false)).merkle_is_full(unbound),
        Err(CompactError::InvalidLedgerCell(_))
    ));
    for depth in [0, 1, 33, u8::MAX] {
        let declarations = [MerkleBinding::new(ID, &[0, 0], depth, false)];
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_merkle(&declarations)
                .is_err()
        );
    }
    for (id, path) in [
        (71, &[1][..]),
        (72, &[0][..]),
        (72, &[0, 0][..]),
        (72, &[0, 0, 1][..]),
    ] {
        let cells = [CellBinding::new(SlotId::new(id), path)];
        assert!(
            Ledger8Bindings::new(&cells)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
    }
    assert!(bindings(false).bind_merkle(&PLAIN).is_err());
}
#[test]
fn merkle_prior_guards_and_gas_failures_stop_later_producers() {
    let tree: Merkle<Field, 3, false> = Merkle::new(ID);
    let empty = Ledger8Bindings::new(&[]).unwrap();
    let native = Circuit::begin(NativeLedger8::new(context(false), empty))
        .require(Rule::new(false, "merkle guard"))
        .merkle_is_full(tree)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(false), empty))
        .require(Rule::new(false, "merkle guard"))
        .merkle_is_full(tree)
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::AssertionFailed("merkle guard".into()));
    assert_eq!(native, recorded);
    let mut first = context(false);
    first.gas_limit = Some(RunningCost::ZERO);
    let mut second = context(false);
    second.gas_limit = Some(RunningCost::ZERO);
    let calls = std::cell::Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(first, bindings(false)))
        .merkle_is_full(tree)
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(second, bindings(false)))
        .merkle_is_full(tree)
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
fn filling_all_eight_leaves_sets_full_before_any_reset() {
    fn fill<E: Executor + MerkleInsert<Field, 3, false> + MerkleInspect<Field, 3, false>>(
        mut executor: E,
    ) -> Result<E::Outcome<bool>, CompactError> {
        let tree = Merkle::new(ID);
        for value in 0..8_u64 {
            executor = executor.merkle_insert(tree, Field::from(value + 1))?;
        }
        let (executor, full) = executor.merkle_is_full(tree)?;
        assert!(full, "eight inserts fill a depth-three tree");
        Ok(executor.finish(full))
    }
    let native = fill(NativeLedger8::new(context(false), bindings(false))).unwrap();
    let recorded = fill(RecordedLedger8::new(context(false), bindings(false))).unwrap();
    assert!(native.result);
    assert_eq!(native.result, recorded.execution.result);
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
        bytes(recorded.execution.context.query.state.get_ref())
    );
    assert_eq!(
        replay.context.effects,
        recorded.execution.context.query.effects
    );
}

#[test]
fn out_of_tree_index_keeps_existing_native_and_recorded_slot_outcomes() {
    let tree: Merkle<Field, 3, false> = Merkle::new(ID);
    let slot = crate::slots::MerkleSlot::<Field, 3, false>::new(&[0, 0]);
    for position in [8_u128, u64::MAX as u128] {
        let index = BoundedUint::new(position).unwrap();
        let reference = slot.insert_index(context(false), Field::from(7_u64), index);
        let candidate = NativeLedger8::new(context(false), bindings(false)).merkle_insert_index(
            tree,
            Field::from(7_u64),
            index,
        );
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
            _ => panic!("native indexed insertion policy changed"),
        }
        let reference = slot.record_insert_index(
            crate::recording::RecordingFrame::new(context(false)),
            Field::from(7_u64),
            index,
        );
        let candidate = RecordedLedger8::new(context(false), bindings(false)).merkle_insert_index(
            tree,
            Field::from(7_u64),
            index,
        );
        match (reference, candidate) {
            (Ok(reference), Ok(candidate)) => {
                let reference = reference.finish(());
                let candidate = candidate.finish(());
                assert_eq!(
                    bytes(reference.execution.context.query.state.get_ref()),
                    bytes(candidate.execution.context.query.state.get_ref())
                );
                assert_eq!(reference.execution.gas_cost, candidate.execution.gas_cost);
                assert_eq!(
                    serde_json::to_value(reference.public.verify_ops()).unwrap(),
                    serde_json::to_value(candidate.public.verify_ops()).unwrap()
                );
            }
            (Err(reference), Err(candidate)) => assert_eq!(reference, candidate),
            _ => panic!("recorded indexed insertion policy changed"),
        }
    }
}

#[test]
fn merkle_bindings_reject_cross_kind_aliases_in_both_extension_orders() {
    use super::super::ledger8::{CounterBinding, ListBinding, MapBinding, SetBinding};
    for (id, path) in [
        (71, &[1][..]),
        (72, &[0][..]),
        (72, &[0, 0][..]),
        (72, &[0, 0, 1][..]),
    ] {
        let cells = [CellBinding::new(SlotId::new(id), path)];
        let counters = [CounterBinding::new(SlotId::new(id), path)];
        let maps = [MapBinding::new(SlotId::new(id), path)];
        let sets = [SetBinding::new(SlotId::new(id), path)];
        let lists = [ListBinding::new(SlotId::new(id), path)];
        assert!(
            Ledger8Bindings::new(&cells)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_counters(&[], &counters)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
        assert!(
            Ledger8Bindings::with_maps(&[], &[], &maps)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_sets(&sets)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_lists(&lists)
                .unwrap()
                .bind_merkle(&PLAIN)
                .is_err()
        );
        assert!(bindings(false).bind_sets(&sets).is_err());
        assert!(bindings(false).bind_lists(&lists).is_err());
        let trees = [PLAIN[0], MerkleBinding::new(SlotId::new(id), path, 3, true)];
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_merkle(&trees)
                .is_err()
        );
    }
    let sets = [SetBinding::new(SlotId::new(72), &[0, 1])];
    let lists = [ListBinding::new(SlotId::new(73), &[1, 0])];
    assert!(
        bindings(false)
            .bind_sets(&sets)
            .unwrap()
            .bind_lists(&lists)
            .is_ok()
    );
    for depth in [2, 32] {
        let tree = [MerkleBinding::new(ID, &[0, 0], depth, false)];
        assert!(
            Ledger8Bindings::new(&[])
                .unwrap()
                .bind_merkle(&tree)
                .is_ok()
        );
    }
    let plain: Merkle<Field, 3, false> = Merkle::new(ID);
    let historic: Merkle<Field, 3, true> = Merkle::new(ID);
    assert!(matches!(
        NativeLedger8::new(context(false), bindings(false)).merkle_is_full(historic),
        Err(CompactError::InvalidLedgerCell(_))
    ));
    assert!(matches!(
        RecordedLedger8::new(context(true), bindings(true)).merkle_is_full(plain),
        Err(CompactError::InvalidLedgerCell(_))
    ));
    let wrong_depth: Merkle<Field, 4, true> = Merkle::new(ID);
    assert!(matches!(
        NativeLedger8::new(context(true), bindings(true)).merkle_is_full(wrong_depth),
        Err(CompactError::InvalidLedgerCell(_))
    ));
}

// No Clone or Default: independent capabilities must not impose either bound.
struct UnsharedLeaf(Field);
impl crate::fab::Aligned for UnsharedLeaf {
    fn alignment() -> crate::fab::Alignment {
        <Field as crate::fab::Aligned>::alignment()
    }
}
impl From<UnsharedLeaf> for crate::fab::Value {
    fn from(value: UnsharedLeaf) -> Self {
        value.0.into()
    }
}
impl crate::ledger::CellValue for UnsharedLeaf {
    fn decode_cell_value(value: &crate::fab::ValueSlice) -> Result<Self, CompactError> {
        <Field as crate::ledger::CellValue>::decode_cell_value(value).map(Self)
    }
}
#[test]
fn inspection_hash_and_reset_do_not_require_leaf_clone_or_default() {
    let tree: Merkle<UnsharedLeaf, 3, false> = Merkle::new(ID);
    let native = NativeLedger8::new(context(false), bindings(false))
        .merkle_insert(tree, UnsharedLeaf(Field::from(7_u64)))
        .unwrap()
        .merkle_reset(tree)
        .unwrap()
        .merkle_insert_hash(tree, FixedBytes::new([1; 32]))
        .unwrap();
    let recorded = RecordedLedger8::new(context(false), bindings(false))
        .merkle_insert(tree, UnsharedLeaf(Field::from(7_u64)))
        .unwrap()
        .merkle_reset(tree)
        .unwrap()
        .merkle_insert_hash(tree, FixedBytes::new([1; 32]))
        .unwrap();
    let (native, full) = native.merkle_is_full(tree).unwrap();
    let (recorded, recorded_full) = recorded.merkle_is_full(tree).unwrap();
    assert!(!full);
    assert_eq!(full, recorded_full);
    let native = native.finish(full);
    let recorded = recorded.finish(recorded_full);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}

#[test]
fn native_root_check_does_not_require_clone_for_the_root_value() {
    let initial = context(false);
    let digest = crate::slots::MerkleSlot::<Field, 3, false>::new(&[0, 0])
        .inspect(initial.query.state.get_ref())
        .unwrap()
        .root()
        .unwrap();
    let tree: Merkle<Field, 3, false> = Merkle::new(ID);
    let (native, valid) = NativeLedger8::new(initial, bindings(false))
        .merkle_check_root(tree, UnsharedLeaf(digest.0))
        .unwrap();
    assert!(valid);
    let native = native.finish(valid);
    assert_eq!(native.context.private_state, 101);
    assert!(native.private_transcript_outputs.is_empty());
}
