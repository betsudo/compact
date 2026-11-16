// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use if_branches_write_ledger::runtime::context::ConstructorContext;
use if_branches_write_ledger::runtime::ledger::ContractAddress;
use if_branches_write_ledger::runtime::public_state::PublicStateSource;
use if_branches_write_ledger::runtime::Field;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

fn fresh() -> if_branches_write_ledger::runtime::context::CircuitContext<()> {
    if_branches_write_ledger::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}

fn state_bytes<S: PublicStateSource>(source: &S) -> Vec<u8> {
    let mut bytes = Vec::new();
    tagged_serialize(source.public_state(), &mut bytes).unwrap();
    bytes
}

#[test]
fn original_constructor_and_sequential_calls_with_recorded_replay() {
    let initial = fresh();
    let view = if_branches_write_ledger::ledger_contract::PublicStateView::from(&initial);
    assert_eq!(view.x().unwrap(), Field::from(3u128));
    assert!(!view.y().unwrap());
    assert!(view.set().unwrap().member(Field::from(1u128)));
    assert_eq!(view.set().unwrap().size().unwrap().value(), 1);
    assert!(view.caroline().unwrap().is_empty());

    let first = if_branches_write_ledger::ledger_contract::if_branches_write_ledger(initial, true)
        .unwrap();
    assert!(first.result);
    let first_state = state_bytes(&first);
    let first_cost = first.gas_cost;
    assert_eq!(first_cost.read_time.into_picoseconds(), 935000000);
    assert_eq!(first_cost.compute_time.into_picoseconds(), 5434381661);
    assert_eq!(first_cost.bytes_written, 1416);
    assert_eq!(first_cost.bytes_deleted, 1220);
    let second = if_branches_write_ledger::ledger_contract::if_branches_write_ledger(
        first.context, false,
    )
    .unwrap();
    assert!(!second.result);
    let second_view = if_branches_write_ledger::ledger_contract::PublicStateView::from(&second);
    assert!(!second_view.y().unwrap());
    assert_eq!(second_view.caroline().unwrap().head().unwrap(), Some(false));
    assert!(second_view.set().unwrap().member(Field::from(1u128)));
    assert_eq!(second.gas_cost.read_time.into_picoseconds(), 935000000);
    assert_eq!(second.gas_cost.compute_time.into_picoseconds(), 5434381661);
    assert_eq!(second.gas_cost.bytes_written, 1808);
    assert_eq!(second.gas_cost.bytes_deleted, 1612);

    let recorded_first =
        if_branches_write_ledger::ledger_contract::recorded::if_branches_write_ledger(
            fresh(), true,
        )
        .unwrap();
    assert!(recorded_first.execution.result);
    assert_eq!(state_bytes(&recorded_first.execution), first_state);
    assert_eq!(recorded_first.execution.gas_cost, first_cost);
    assert!(recorded_first.execution.private_transcript_outputs.is_empty());
    assert_eq!(recorded_first.public.verify_ops().len(), 34);
    let replay_first = recorded_first
        .public
        .initial()
        .query(recorded_first.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut replay_first_bytes = Vec::new();
    tagged_serialize(replay_first.context.state.get_ref(), &mut replay_first_bytes).unwrap();
    assert_eq!(replay_first_bytes, first_state);

    let recorded_second =
        if_branches_write_ledger::ledger_contract::recorded::if_branches_write_ledger(
            recorded_first.execution.context, false,
        )
        .unwrap();
    assert!(!recorded_second.execution.result);
    assert_eq!(
        state_bytes(&recorded_second.execution),
        state_bytes(&second)
    );
    assert_eq!(recorded_second.execution.gas_cost, second.gas_cost);
    assert!(recorded_second.execution.private_transcript_outputs.is_empty());
    assert_eq!(recorded_second.public.verify_ops().len(), 34);
    let replay_second = recorded_second
        .public
        .initial()
        .query(recorded_second.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut replay_second_bytes = Vec::new();
    tagged_serialize(replay_second.context.state.get_ref(), &mut replay_second_bytes).unwrap();
    assert_eq!(replay_second_bytes, state_bytes(&second));
}
