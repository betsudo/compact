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
