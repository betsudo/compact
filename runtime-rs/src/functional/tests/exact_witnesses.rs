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
    ExactWitnessOutput, Ledger8Bindings, NativeLedger8, RecordedLedger8, WitnessHandler,
};
use super::super::{Circuit, Rule, Witness, WitnessId};
use crate::{
    CompactError, Field, HashInputValue, OpaqueBytes, OpaqueString,
    context::{CircuitContext, ConstructorContext, ConstructorResult, WitnessReadMeter},
    fab::AlignedValue,
    ledger::{ContractAddress, DB, DefaultDB, constructor_cell, contract_state},
};
use std::{cell::RefCell, rc::Rc};
const ID: WitnessId = WitnessId::new(19);
struct Echo {
    calls: Rc<RefCell<Vec<u64>>>,
    fail: bool,
}
impl<D: DB, T> WitnessHandler<u64, D, T, T> for Echo {
    fn call(
        &self,
        id: WitnessId,
        argument: T,
        context: &CircuitContext<u64, D>,
        meter: &WitnessReadMeter<'_, D>,
    ) -> Result<(u64, T), CompactError> {
        if id != ID {
            return Err(CompactError::AssertionFailed(
                "unknown exact witness".into(),
            ));
        }
        self.calls.borrow_mut().push(context.private_state);
        if !meter.read_cell::<bool>(&[0])? || self.fail {
            return Err(CompactError::AssertionFailed(
                "exact witness refused".into(),
            ));
        }
        Ok((context.private_state + 1, argument))
    }
}
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(0),
        contract_state(vec![constructor_cell::<bool, DefaultDB>(true)]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn handler(fail: bool) -> Echo {
    Echo {
        calls: Rc::new(RefCell::new(vec![])),
        fail,
    }
}
fn roundtrip<T: Clone + HashInputValue + std::fmt::Debug + PartialEq>(value: T) {
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let native_handler = handler(false);
    let recorded_handler = handler(false);
    let exact = Witness::<T, T, ExactWitnessOutput>::with_mode(ID);
    // Both legacy constructor inference and the two-argument method turbofish
    // remain valid even though these interpreters now implement both strategies.
    let native =
        Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
            .witness::<Field, Field>(Witness::new(ID), Field::from(7_u64))
            .bind(|before, circuit| {
                circuit
                    .witness_in(exact, value.clone())
                    .bind(|exact_value, circuit| {
                        circuit
                            .witness(Witness::new(ID), true)
                            .bind(|after, circuit| circuit.returning((before, exact_value, after)))
                    })
            })
            .finish()
            .unwrap();
    let recorded =
        Circuit::begin(RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler))
            .witness::<Field, Field>(Witness::new(ID), Field::from(7_u64))
            .bind(|before, circuit| {
                circuit
                    .witness_in(exact, value.clone())
                    .bind(|exact_value, circuit| {
                        circuit
                            .witness(Witness::new(ID), true)
                            .bind(|after, circuit| circuit.returning((before, exact_value, after)))
                    })
            })
            .finish()
            .unwrap();
    assert_eq!(native.result, (Field::from(7_u64), value.clone(), true));
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.context.private_state, 3);
    assert_eq!(recorded.execution.context.private_state, 3);
    assert_eq!(*native_handler.calls.borrow(), [0, 1, 2]);
    assert_eq!(
        *native_handler.calls.borrow(),
        *recorded_handler.calls.borrow()
    );
    let expected = vec![
        AlignedValue::from(Field::from(7_u64)),
        crate::exact_private_output(value),
        AlignedValue::from(true),
    ];
    assert_eq!(native.private_transcript_outputs, expected);
    assert_eq!(recorded.execution.private_transcript_outputs, expected);
    let meter_context = context();
    let meter = WitnessReadMeter::new(&meter_context);
    assert!(meter.read_cell::<bool>(&[0]).unwrap());
    assert_eq!(
        native.gas_cost,
        meter.gas_cost() + meter.gas_cost() + meter.gas_cost()
    );
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    let replay = recorded
        .public
        .initial()
        .query(
            recorded.public.verify_ops(),
            None,
            &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL,
        )
        .unwrap();
    let bytes = |state: &crate::ledger::StateValue<DefaultDB>| {
        let mut bytes = Vec::new();
        midnight_serialize::tagged_serialize(state, &mut bytes).unwrap();
        bytes
    };
    assert_eq!(
        bytes(replay.context.state.get_ref()),
        bytes(native.context.query.state.get_ref())
    );
    assert_eq!(
        bytes(replay.context.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
    assert_eq!(replay.context.effects, native.context.query.effects);
    assert_eq!(
        replay.context.effects,
        recorded.execution.context.query.effects
    );
}
#[test]
fn exact_string_preserves_unicode_nul_and_empty_values() {
    roundtrip(OpaqueString("λ文\0".into()));
    roundtrip(OpaqueString(String::new()));
}
#[test]
fn exact_bytes_preserve_internal_and_trailing_zeros() {
    roundtrip(OpaqueBytes(vec![0, 255, 0]));
    roundtrip(OpaqueBytes(vec![]));
}
#[test]
fn exact_nested_product_retains_leaf_order_and_old_ordinary_calls() {
    roundtrip((
        Field::from(3_u64),
        (OpaqueString("x\0".into()), OpaqueBytes(vec![1, 0])),
    ));
}
#[test]
fn exact_and_ordinary_modes_are_distinct_without_altering_ordinary_encoding() {
    let value = OpaqueBytes(vec![1, 0]);
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let ordinary =
        Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(handler(false)))
            .witness(Witness::new(ID), value.clone())
            .finish()
            .unwrap();
    let exact =
        Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(handler(false)))
            .witness_in(
                Witness::<_, _, ExactWitnessOutput>::with_mode(ID),
                value.clone(),
            )
            .finish()
            .unwrap();
    assert_eq!(
        ordinary.private_transcript_outputs,
        [AlignedValue::from(value.clone())]
    );
    assert_eq!(
        exact.private_transcript_outputs,
        [crate::exact_private_output(value)]
    );
    assert_ne!(
        ordinary.private_transcript_outputs,
        exact.private_transcript_outputs
    );
    assert_eq!(ordinary.result, exact.result);
}
#[test]
fn exact_failures_and_prior_guards_stop_later_callbacks() {
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let exact = Witness::<OpaqueString, OpaqueString, ExactWitnessOutput>::with_mode(ID);
    for (enabled, fail, calls, message) in [
        (false, false, 0, "exact guard refused"),
        (true, true, 1, "exact witness refused"),
    ] {
        let native_handler = handler(fail);
        let recorded_handler = handler(fail);
        let native =
            Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
                .require(Rule::new(enabled, "exact guard refused"))
                .witness_in(exact, OpaqueString("x\0".into()))
                .bind(|_, circuit| circuit.witness(Witness::new(ID), true))
                .finish()
                .err()
                .unwrap();
        let recorded = Circuit::begin(
            RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler),
        )
        .require(Rule::new(enabled, "exact guard refused"))
        .witness_in(exact, OpaqueString("x\0".into()))
        .bind(|_, circuit| circuit.witness(Witness::new(ID), true))
        .finish()
        .err()
        .unwrap();
        assert_eq!(native, CompactError::AssertionFailed(message.into()));
        assert_eq!(native, recorded);
        assert_eq!(native_handler.calls.borrow().len(), calls);
        assert_eq!(
            *native_handler.calls.borrow(),
            *recorded_handler.calls.borrow()
        );
    }
}

#[test]
fn exact_projected_reads_enforce_the_existing_gas_limit() {
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let witness = Witness::<OpaqueBytes, OpaqueBytes, ExactWitnessOutput>::with_mode(ID);
    let mut native_context = context();
    native_context.gas_limit = Some(crate::context::RunningCost::ZERO);
    let mut recorded_context = context();
    recorded_context.gas_limit = Some(crate::context::RunningCost::ZERO);
    let native_handler = handler(false);
    let recorded_handler = handler(false);
    let native = Circuit::begin(
        NativeLedger8::new(native_context, bindings).with_witnesses(&native_handler),
    )
    .witness_in(witness, OpaqueBytes(vec![1, 0]))
    .bind(|_, circuit| circuit.witness(Witness::new(ID), true))
    .finish();
    let recorded = Circuit::begin(
        RecordedLedger8::new(recorded_context, bindings).with_witnesses(&recorded_handler),
    )
    .witness_in(witness, OpaqueBytes(vec![1, 0]))
    .bind(|_, circuit| circuit.witness(Witness::new(ID), true))
    .finish();
    assert!(matches!(native, Err(CompactError::LedgerQueryRejected(_))));
    assert!(matches!(
        recorded,
        Err(CompactError::LedgerQueryRejected(_))
    ));
    assert_eq!(*native_handler.calls.borrow(), [0]);
    assert_eq!(
        *native_handler.calls.borrow(),
        *recorded_handler.calls.borrow()
    );
}
