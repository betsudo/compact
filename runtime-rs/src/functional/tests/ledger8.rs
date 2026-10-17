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

use super::super::ledger8::{CellBinding, Ledger8Bindings, NativeLedger8, RecordedLedger8};
use super::super::{Cell, CellRead, CellWrite, Circuit, Executor, Rule, SlotId};
use crate::{
    CompactError,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_cell, contract_state},
};
use midnight_onchain_vm::{cost_model::INITIAL_COST_MODEL, ops::Op};
use midnight_serialize::tagged_serialize;

const VALUE: Cell<bool> = Cell::new(SlotId::new(17));
const DECLARATIONS: [CellBinding; 1] = [CellBinding::new(VALUE.id(), &[0])];

// One immutable argument description and exactly one generic execution body.
struct SetIf {
    expected: bool,
    next: bool,
    enabled: bool,
}
impl SetIf {
    fn run<E: Executor + CellRead<bool> + CellWrite<bool>>(
        &self,
        executor: E,
    ) -> Result<E::Outcome<bool>, CompactError> {
        Circuit::begin(executor)
            .read(VALUE)
            .bind(|before, circuit| {
                if self.enabled {
                    circuit
                        .require(Rule::new(before == self.expected, "unexpected old value"))
                        .write(VALUE, self.next)
                        .read(VALUE)
                } else {
                    circuit.returning(before)
                }
            })
            .finish()
    }
}
fn context() -> CircuitContext<(), DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(()),
        contract_state(vec![constructor_cell::<bool, DefaultDB>(false)]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut bytes = Vec::new();
    tagged_serialize(state, &mut bytes).unwrap();
    bytes
}

#[test]
fn one_immutable_program_has_native_recorded_and_replay_state_gas_parity() {
    let program = SetIf {
        expected: false,
        next: true,
        enabled: true,
    };
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    let native = program
        .run(NativeLedger8::new(context(), bindings))
        .unwrap();
    let recorded = program
        .run(RecordedLedger8::new(context(), bindings))
        .unwrap();
    assert!(native.result && recorded.execution.result);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_ne!(native.gas_cost, RunningCost::ZERO);
    assert_eq!(
        native.private_transcript_outputs,
        recorded.execution.private_transcript_outputs
    );
    assert!(native.private_transcript_outputs.is_empty());
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
    assert_eq!(
        native.context.query.effects,
        recorded.execution.context.query.effects
    );
    assert!(!recorded.public.verify_ops().is_empty());
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
    let mut bad = recorded.public.verify_ops().to_vec();
    *bad.last_mut().unwrap() = Op::Popeq {
        cached: false,
        result: crate::fab::AlignedValue::from(false),
    };
    assert!(
        recorded
            .public
            .initial()
            .query(&bad, None, &INITIAL_COST_MODEL)
            .is_err()
    );
    // Reuse does not retain previous frame state or the previous execution's value.
    let again = program
        .run(RecordedLedger8::new(context(), bindings))
        .unwrap();
    assert_eq!(again.execution.result, recorded.execution.result);
    assert_eq!(again.execution.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(again.execution.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
}

#[test]
fn untaken_branch_skips_assertion_and_write_and_failed_guard_has_no_result() {
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    let skipped = SetIf {
        expected: true,
        next: true,
        enabled: false,
    };
    let native = skipped
        .run(NativeLedger8::new(context(), bindings))
        .unwrap();
    let recorded = skipped
        .run(RecordedLedger8::new(context(), bindings))
        .unwrap();
    assert!(!native.result && !recorded.execution.result);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        bytes(context().query.state.get_ref()),
        bytes(native.context.query.state.get_ref())
    );
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
    let failed = SetIf {
        expected: true,
        next: true,
        enabled: true,
    };
    let native = failed
        .run(NativeLedger8::new(context(), bindings))
        .err()
        .unwrap();
    let recorded = failed
        .run(RecordedLedger8::new(context(), bindings))
        .err()
        .unwrap();
    assert_eq!(
        native,
        CompactError::AssertionFailed("unexpected old value".into())
    );
    assert_eq!(native, recorded);
}

#[test]
fn gas_failures_and_unbound_or_duplicate_logical_slots_refuse() {
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    let mut native = context();
    native.gas_limit = Some(RunningCost::ZERO);
    let mut recorded = context();
    recorded.gas_limit = Some(RunningCost::ZERO);
    assert!(matches!(
        Circuit::begin(NativeLedger8::new(native, bindings))
            .read(VALUE)
            .finish(),
        Err(CompactError::LedgerQueryRejected(_))
    ));
    assert!(matches!(
        Circuit::begin(RecordedLedger8::new(recorded, bindings))
            .read(VALUE)
            .finish(),
        Err(CompactError::LedgerQueryRejected(_))
    ));
    let absent = Cell::<bool>::new(SlotId::new(18));
    let native = Circuit::begin(NativeLedger8::new(context(), bindings))
        .read(absent)
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(), bindings))
        .read(absent)
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, recorded);
    assert!(matches!(native, CompactError::InvalidLedgerCell(_)));
    let duplicate = [
        CellBinding::new(VALUE.id(), &[0]),
        CellBinding::new(VALUE.id(), &[1]),
    ];
    assert!(matches!(
        Ledger8Bindings::new(&duplicate),
        Err(CompactError::InvalidLedgerCell(_))
    ));
}
