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

use midnight_compact_runtime::FixedBytes;
use midnight_compact_runtime::context::{
    CircuitContext, ConstructorContext, ConstructorResult, RunningCost,
};
use midnight_compact_runtime::ledger::{
    ContractAddress, DefaultDB, HashOutput, contract_state, read_cell,
};
use midnight_compact_runtime::recording::RecordingFrame;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_onchain_vm::ops::{LogEventType, Op};

fn context() -> CircuitContext<()> {
    let state = contract_state::<DefaultDB>(Vec::new());
    ConstructorResult::new(ConstructorContext::new(()), state)
        .into_circuit_context(ContractAddress(HashOutput([42; 32])))
}

#[test]
fn standalone_events_follow_vm_order_without_changing_public_state() {
    let first = FixedBytes::new([0x07; 32]);
    let second = FixedBytes::new([0x42; 32]);

    let native_first = context().emit_shielded_spend(first).unwrap();
    let native_second = native_first.context.emit_shielded_spend(second).unwrap();
    let recorded = RecordingFrame::new(context())
        .emit_shielded_spend(first)
        .unwrap()
        .emit_shielded_spend(second)
        .unwrap()
        .finish(());
    assert_eq!(
        recorded.execution.gas_cost,
        native_first.gas_cost + native_second.gas_cost
    );
    assert_eq!(
        recorded.execution.context.query.state,
        context().query.state
    );
    assert_eq!(
        recorded.execution.context.query.effects,
        context().query.effects
    );
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    assert!(matches!(
        recorded.public.verify_ops(),
        [
            Op::Push { storage: false, .. },
            Op::Log,
            Op::Push { storage: false, .. },
            Op::Log,
        ]
    ));

    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    assert_eq!(replay.context.state, context().query.state);
    assert_eq!(replay.context.effects, context().query.effects);
    let events = recorded.execution.context.events();
    assert!(events == native_second.context.events());
    assert_eq!(events.len(), 2);
    assert_eq!(replay.events.len(), 2);
    for ((event, log), value) in events.iter().zip(replay.events).zip([first, second]) {
        assert_eq!(event.address, context().query.address);
        assert_eq!(event.log, log);
        assert_eq!(log.version, 1);
        assert_eq!(log.event_type, LogEventType::ShieldedSpend);
        assert_eq!(read_cell::<FixedBytes<32>, _>(&log.data).unwrap(), value);
    }
}

#[test]
fn mint_events_replay_in_order_with_the_exact_payload_and_gas() {
    let mut bytes = [0; 81];
    bytes[..32].fill(0x42);
    bytes[32..64].fill(0xa5);
    let absent = FixedBytes::new(bytes);
    bytes[64] = 1;
    bytes[65..].copy_from_slice(&u128::MAX.to_le_bytes());
    let present = FixedBytes::new(bytes);

    let native_first = context().emit_shielded_mint(present).unwrap();
    let native_second = native_first.context.emit_shielded_mint(absent).unwrap();
    let recorded = RecordingFrame::new(context())
        .emit_shielded_mint(present)
        .unwrap()
        .emit_shielded_mint(absent)
        .unwrap()
        .finish(());
    assert_eq!(
        recorded.execution.gas_cost,
        native_first.gas_cost + native_second.gas_cost
    );
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let events = recorded.execution.context.events();
    assert_eq!(events.len(), 2);
    assert!(events == native_second.context.events());
    assert_eq!(replay.events.len(), 2);
    assert_eq!(replay.context.state, context().query.state);
    for ((event, log), value) in events.iter().zip(replay.events).zip([present, absent]) {
        assert_eq!(event.address, context().query.address);
        assert_eq!(event.log, log);
        assert_eq!(log.version, 1);
        assert_eq!(log.event_type, LogEventType::ShieldedMint);
        assert_eq!(read_cell::<FixedBytes<81>, _>(&log.data).unwrap(), value);
    }
}

#[test]
fn receive_events_replay_in_order_with_the_exact_payload_and_gas() {
    let mut bytes = [0; 578];
    bytes[..32].fill(0x42);
    bytes[33..545].fill(0x33);
    bytes[546..].fill(0xa5);
    let absent = FixedBytes::new(bytes);
    bytes[32] = 1;
    bytes[545] = 1;
    let present = FixedBytes::new(bytes);

    let native_first = context().emit_shielded_receive(present).unwrap();
    let native_second = native_first.context.emit_shielded_receive(absent).unwrap();
    let recorded = RecordingFrame::new(context())
        .emit_shielded_receive(present)
        .unwrap()
        .emit_shielded_receive(absent)
        .unwrap()
        .finish(());
    assert_eq!(
        recorded.execution.gas_cost,
        native_first.gas_cost + native_second.gas_cost
    );
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let events = recorded.execution.context.events();
    assert_eq!(events.len(), 2);
    assert!(events == native_second.context.events());
    assert_eq!(replay.events.len(), 2);
    assert_eq!(replay.context.state, context().query.state);
    for ((event, log), value) in events.iter().zip(replay.events).zip([present, absent]) {
        assert_eq!(event.address, context().query.address);
        assert_eq!(event.log, log);
        assert_eq!(log.version, 1);
        assert_eq!(log.event_type, LogEventType::ShieldedReceive);
        assert_eq!(read_cell::<FixedBytes<578>, _>(&log.data).unwrap(), value);
    }
}

#[test]
fn unshielded_spend_events_replay_in_order_with_the_exact_payload_and_gas() {
    let mut bytes = [0; 145];
    bytes[1..33].fill(0x42);
    bytes[33..65].fill(0xa5);
    bytes[65..97].fill(0x33);
    bytes[97..129].fill(0x67);
    bytes[129..].copy_from_slice(&u128::MAX.to_le_bytes());
    let right_sender = FixedBytes::new(bytes);
    bytes[0] = 1;
    let left_sender = FixedBytes::new(bytes);

    let native_first = context().emit_unshielded_spend(left_sender).unwrap();
    let native_second = native_first.context.emit_unshielded_spend(right_sender).unwrap();
    let recorded = RecordingFrame::new(context())
        .emit_unshielded_spend(left_sender)
        .unwrap()
        .emit_unshielded_spend(right_sender)
        .unwrap()
        .finish(());
    assert_eq!(
        recorded.execution.gas_cost,
        native_first.gas_cost + native_second.gas_cost
    );
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let events = recorded.execution.context.events();
    assert_eq!(events.len(), 2);
    assert!(events == native_second.context.events());
    assert_eq!(replay.events.len(), 2);
    assert_eq!(replay.context.state, context().query.state);
    for ((event, log), value) in events.iter().zip(replay.events).zip([left_sender, right_sender]) {
        assert_eq!(event.address, context().query.address);
        assert_eq!(event.log, log);
        assert_eq!(log.version, 1);
        assert_eq!(log.event_type, LogEventType::UnshieldedSpend);
        assert_eq!(read_cell::<FixedBytes<145>, _>(&log.data).unwrap(), value);
    }
}

#[test]
fn rejected_vm_log_returns_no_successor_context() {
    let mut initial = context();
    initial.gas_limit = Some(RunningCost::ZERO);
    let nullifier = FixedBytes::new([1; 32]);
    assert!(initial.emit_shielded_spend(nullifier).is_err());

    let mut initial = context();
    initial.gas_limit = Some(RunningCost::ZERO);
    assert!(
        RecordingFrame::new(initial)
            .emit_shielded_spend(nullifier)
            .is_err()
    );
    assert!(context().events().is_empty());
}

#[test]
fn audited_local_helper_cannot_add_an_unrecorded_event() {
    let nullifier = FixedBytes::new([1; 32]);
    let result =
        RecordingFrame::new(context()).call_local(|context| context.emit_shielded_spend(nullifier));
    assert!(matches!(
        result,
        Err(midnight_compact_runtime::CompactError::InvalidLedgerCell(message))
            if message.contains("local helper changed public or Zswap execution context")
    ));
}
