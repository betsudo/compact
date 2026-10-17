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

use super::super::ledger8::{
    CellBinding, Ledger8Bindings, NativeLedger8, RecordedLedger8, WitnessHandler,
};
use super::super::{
    Cell, CellWrite, Circuit, Executor, Rule, SlotId, Witness, WitnessCall, WitnessId,
};
use crate::{
    CompactError,
    context::{
        CircuitContext, ConstructorContext, ConstructorResult, RunningCost, WitnessReadMeter,
    },
    fab::AlignedValue,
    ledger::{ContractAddress, DB, DefaultDB, constructor_cell, contract_state},
};
use std::{cell::RefCell, rc::Rc};
const VALUE: Cell<bool> = Cell::new(SlotId::new(0));
const ECHO: Witness<bool, bool> = Witness::new(WitnessId::new(5));
const DECLARATIONS: [CellBinding; 1] = [CellBinding::new(VALUE.id(), &[0])];
struct Echo {
    seen: Rc<RefCell<Vec<(bool, u64)>>>,
    fail_at: Option<u64>,
}
impl<D: DB> WitnessHandler<u64, D, bool, bool> for Echo {
    fn call(
        &self,
        id: WitnessId,
        argument: bool,
        context: &CircuitContext<u64, D>,
        meter: &WitnessReadMeter<'_, D>,
    ) -> Result<(u64, bool), CompactError> {
        if id != ECHO.id() {
            return Err(CompactError::AssertionFailed("unknown witness".into()));
        }
        self.seen
            .borrow_mut()
            .push((argument, context.private_state));
        let held: bool = meter.read_cell(&[0])?;
        if self.fail_at == Some(context.private_state) {
            return Err(CompactError::AssertionFailed("witness refused".into()));
        }
        Ok((context.private_state + 1, argument && held))
    }
}
struct Guarded {
    first: bool,
    second: bool,
}
impl Guarded {
    fn run<E: Executor + WitnessCall<bool, bool> + CellWrite<bool>>(
        &self,
        executor: E,
    ) -> Result<E::Outcome<()>, CompactError> {
        Circuit::begin(executor)
            .witness(ECHO, self.first)
            .bind(|first, circuit| {
                circuit
                    .require(Rule::new(first, "first denied"))
                    .bind(|(), circuit| {
                        circuit.witness(ECHO, self.second).bind(|second, circuit| {
                            circuit
                                .require(Rule::new(second, "second denied"))
                                .bind(|(), circuit| circuit.write(VALUE, false))
                        })
                    })
            })
            .finish()
    }
}
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(0),
        contract_state(vec![constructor_cell::<bool, DefaultDB>(true)]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn handler(fail_at: Option<u64>) -> Echo {
    Echo {
        seen: Rc::new(RefCell::new(vec![])),
        fail_at,
    }
}
#[test]
fn metered_witnesses_preserve_order_private_adoption_outputs_and_cost() {
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    let native_handler = handler(None);
    let recorded_handler = handler(None);
    let program = Guarded {
        first: true,
        second: true,
    };
    let native = program
        .run(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
        .unwrap();
    let recorded = program
        .run(RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler))
        .unwrap();
    assert_eq!(*native_handler.seen.borrow(), [(true, 0), (true, 1)]);
    assert_eq!(
        *native_handler.seen.borrow(),
        *recorded_handler.seen.borrow()
    );
    assert_eq!(native.context.private_state, 2);
    assert_eq!(recorded.execution.context.private_state, 2);
    let expected = vec![AlignedValue::from(true), AlignedValue::from(true)];
    assert_eq!(native.private_transcript_outputs, expected);
    assert_eq!(recorded.execution.private_transcript_outputs, expected);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_ne!(native.gas_cost, RunningCost::ZERO);
    // Witness ledger projections charge gas without adding public VM reads.
    assert_eq!(recorded.public.verify_ops().len(), 3);
    let replay = recorded
        .public
        .initial()
        .query(
            recorded.public.verify_ops(),
            None,
            &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL,
        )
        .unwrap();
    let state_bytes = |state: &crate::ledger::StateValue<DefaultDB>| {
        let mut bytes = Vec::new();
        midnight_serialize::tagged_serialize(state, &mut bytes).unwrap();
        bytes
    };
    assert_eq!(
        state_bytes(replay.context.state.get_ref()),
        state_bytes(native.context.query.state.get_ref())
    );
    assert_eq!(
        state_bytes(replay.context.state.get_ref()),
        state_bytes(recorded.execution.context.query.state.get_ref())
    );
    assert_eq!(replay.context.effects, native.context.query.effects);
    assert_eq!(
        replay.context.effects,
        recorded.execution.context.query.effects
    );
    let meter_context = context();
    let meter = WitnessReadMeter::new(&meter_context);
    assert!(meter.read_cell::<bool>(&[0]).unwrap());
    let write_only = crate::slots::CellSlot::<bool>::new(&[0])
        .write(context(), false)
        .unwrap();
    assert_eq!(
        native.gas_cost,
        meter.gas_cost() + meter.gas_cost() + write_only.gas_cost
    );
    let again = program
        .run(NativeLedger8::new(context(), bindings).with_witnesses(handler(None)))
        .unwrap();
    assert_eq!(again.context.private_state, 2);
    assert_eq!(again.private_transcript_outputs, expected);
}
#[test]
fn assertion_and_witness_failures_stop_later_callbacks_and_public_write() {
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    for (first, second, fail_at, expected_calls, message) in [
        (false, true, None, 1, "first denied"),
        (true, false, None, 2, "second denied"),
        (true, true, Some(0), 1, "witness refused"),
        (true, true, Some(1), 2, "witness refused"),
    ] {
        let native_handler = handler(fail_at);
        let recorded_handler = handler(fail_at);
        let program = Guarded { first, second };
        let native = program
            .run(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
            .err()
            .unwrap();
        let recorded = program
            .run(RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler))
            .err()
            .unwrap();
        assert_eq!(native, CompactError::AssertionFailed(message.into()));
        assert_eq!(native, recorded);
        assert_eq!(native_handler.seen.borrow().len(), expected_calls);
        assert_eq!(
            *native_handler.seen.borrow(),
            *recorded_handler.seen.borrow()
        );
    }
}
#[test]
fn witness_projected_read_limit_and_unknown_identity_refuse() {
    let bindings = Ledger8Bindings::new(&DECLARATIONS).unwrap();
    let mut native = context();
    native.gas_limit = Some(RunningCost::ZERO);
    let mut recorded = context();
    recorded.gas_limit = Some(RunningCost::ZERO);
    let native = Guarded {
        first: true,
        second: true,
    }
    .run(NativeLedger8::new(native, bindings).with_witnesses(handler(None)));
    let recorded = Guarded {
        first: true,
        second: true,
    }
    .run(RecordedLedger8::new(recorded, bindings).with_witnesses(handler(None)));
    assert!(matches!(native, Err(CompactError::LedgerQueryRejected(_))));
    assert!(matches!(
        recorded,
        Err(CompactError::LedgerQueryRejected(_))
    ));
    let unknown = Witness::<bool, bool>::new(WitnessId::new(6));
    let native =
        Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(handler(None)))
            .witness(unknown, true)
            .finish()
            .err()
            .unwrap();
    let recorded =
        Circuit::begin(RecordedLedger8::new(context(), bindings).with_witnesses(handler(None)))
            .witness(unknown, true)
            .finish()
            .err()
            .unwrap();
    assert_eq!(native, recorded);
    assert_eq!(
        native,
        CompactError::AssertionFailed("unknown witness".into())
    );
}
