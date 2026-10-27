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

fn merkle_context(
    state: runtime::ledger::ChargedState<runtime::ledger::DefaultDB>,
) -> runtime::context::CircuitContext<u64> {
    runtime::context::ConstructorResult::new(ConstructorContext::new(7_u64), state)
        .into_circuit_context(ContractAddress::default())
}
macro_rules! merkle_case{
    ($state:expr,$entry:ident,[$($argument:expr),*],$error:expr)=>{{
        use merkle_fixture::ledger_contract as c;
        outcomes(c::$entry(merkle_context($state.clone()),$($argument),*),c::recorded::$entry(merkle_context($state.clone()),$($argument),*),
            c::functional::$entry::Program::new($($argument),*).execute_native(merkle_context($state.clone())),c::functional::$entry::Program::new($($argument),*).execute_recorded(merkle_context($state.clone())),$error)
    }};
}
fn merkle_assert_first_free(
    state: &runtime::ledger::ChargedState<runtime::ledger::DefaultDB>,
    expected: u128,
) {
    assert!(merkle_fixture::ledger_slots::plain.path().len() > 1);
    assert!(merkle_fixture::ledger_slots::historic.path().len() > 1);
    assert_eq!(
        merkle_fixture::ledger_slots::plain
            .inspect(state.get_ref())
            .unwrap()
            .first_free()
            .unwrap()
            .value(),
        expected
    );
    assert_eq!(
        merkle_fixture::ledger_slots::historic
            .inspect(state.get_ref())
            .unwrap()
            .first_free()
            .unwrap()
            .value(),
        expected
    );
}
fn merkle_root(
    state: &runtime::ledger::ChargedState<runtime::ledger::DefaultDB>,
) -> merkle_fixture::types::MerkleTreeDigest {
    merkle_fixture::types::MerkleTreeDigest {
        field: merkle_fixture::ledger_slots::plain
            .inspect(state.get_ref())
            .unwrap()
            .root()
            .unwrap()
            .0,
    }
}
#[test]
fn merkle_nine_forms_preserve_fullness_roots_history_reset_and_replay() {
    use merkle_fixture::ledger_contract as c;
    let initial = c::initial_state(ConstructorContext::new(7_u64))
        .unwrap()
        .ledger_state;
    let blank = merkle_root(&initial);
    let leaf = runtime::BoundedUint::<255>::new(7).unwrap();
    let mut state = merkle_case!(initial, append, [leaf], None)
        .unwrap()
        .context
        .query
        .state;
    merkle_assert_first_free(&state, 1);
    let root = merkle_root(&state);
    let observed = merkle_case!(state, observe, [root.clone()], None).unwrap();
    assert!(!observed.result.plainFull);
    assert!(!observed.result.historicFull);
    assert!(observed.result.plainKnown);
    assert!(observed.result.historicKnown);
    let observed = merkle_case!(state, observe, [blank.clone()], None).unwrap();
    assert!(!observed.result.plainKnown);
    let official_blank = merkle_fixture::ledger_slots::historic
        .inspect(state.get_ref())
        .unwrap()
        .history()
        .unwrap()
        .iter()
        .any(|entry| entry.0 == blank.field);
    assert_eq!(observed.result.historicKnown, official_blank);
    state = merkle_case!(
        state,
        place,
        [
            leaf,
            runtime::BoundedUint::<{ u64::MAX as u128 }>::new(3).unwrap()
        ],
        None
    )
    .unwrap()
    .context
    .query
    .state;
    merkle_assert_first_free(&state, 4);
    let hash = runtime::FixedBytes::new([7_u8; 32]);
    state = merkle_case!(state, appendHash, [hash], None)
        .unwrap()
        .context
        .query
        .state;
    merkle_assert_first_free(&state, 5);
    state = merkle_case!(
        state,
        placeHash,
        [
            hash,
            runtime::BoundedUint::<{ u64::MAX as u128 }>::new(6).unwrap()
        ],
        None
    )
    .unwrap()
    .context
    .query
    .state;
    merkle_assert_first_free(&state, 7);
    state = merkle_case!(
        state,
        placeDefault,
        [runtime::BoundedUint::<{ u64::MAX as u128 }>::new(7).unwrap()],
        None
    )
    .unwrap()
    .context
    .query
    .state;
    merkle_assert_first_free(&state, 8);
    let root = merkle_root(&state);
    let observed = merkle_case!(state, observe, [root.clone()], None).unwrap();
    assert!(observed.result.plainFull);
    assert!(observed.result.historicFull);
    assert!(observed.result.plainKnown);
    assert!(observed.result.historicKnown);
    state = merkle_case!(state, forget, [], None)
        .unwrap()
        .context
        .query
        .state;
    merkle_assert_first_free(&state, 8);
    let retained_history = merkle_fixture::ledger_slots::historic
        .inspect(state.get_ref())
        .unwrap()
        .history()
        .unwrap();
    assert_eq!(retained_history.len(), 1);
    assert_eq!(retained_history[0].0, root.field);
    let observed = merkle_case!(state, observe, [root.clone()], None).unwrap();
    assert!(observed.result.plainKnown);
    assert!(observed.result.historicKnown);
    let old = merkle_case!(state, observe, [blank.clone()], None).unwrap();
    assert!(!old.result.plainKnown);
    assert!(!old.result.historicKnown);
    state = merkle_case!(state, reset, [], None)
        .unwrap()
        .context
        .query
        .state;
    merkle_assert_first_free(&state, 0);
    assert_eq!(state, initial);
    let observed = merkle_case!(state, observe, [blank.clone()], None).unwrap();
    assert!(!observed.result.plainFull);
    assert!(!observed.result.historicFull);
    assert!(observed.result.plainKnown);
    assert!(observed.result.historicKnown);
    assert_eq!(observed.context.private_state, 7);
    assert!(observed.private_transcript_outputs.is_empty());
}
#[test]
fn merkle_index_capacity_and_full_append_preserve_official_query_refusals() {
    use merkle_fixture::ledger_contract as c;
    let initial = c::initial_state(ConstructorContext::new(7_u64))
        .unwrap()
        .ledger_state;
    let leaf = runtime::BoundedUint::<255>::new(7).unwrap();
    let index = runtime::BoundedUint::<{ u64::MAX as u128 }>::new(8).unwrap();
    let reference = c::place(merkle_context(initial.clone()), leaf, index)
        .err()
        .expect("out-of-capacity index must refuse");
    assert!(matches!(reference, CompactError::LedgerQueryRejected(_)));
    assert!(merkle_case!(initial, place, [leaf, index], Some(&reference.to_string())).is_none());
    // Ordinary setup supplies a held full state; this is not a separate proof claim.
    let full = c::placeDefault(
        merkle_context(initial),
        runtime::BoundedUint::<{ u64::MAX as u128 }>::new(7).unwrap(),
    )
    .unwrap()
    .context
    .query
    .state;
    merkle_assert_first_free(&full, 8);
    let reference = c::append(merkle_context(full.clone()), leaf)
        .err()
        .expect("append to full tree must refuse");
    assert!(matches!(reference, CompactError::LedgerQueryRejected(_)));
    assert!(merkle_case!(full, append, [leaf], Some(&reference.to_string())).is_none());
}
