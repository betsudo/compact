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

use super::super::ledger8::{Ledger8Bindings, NativeLedger8, RecordedLedger8, WitnessHandler};
use super::super::{Circuit, Rule, Witness, WitnessId};
use crate::{
    CompactError, Field, FixedBytes,
    context::{CircuitContext, ConstructorContext, ConstructorResult, WitnessReadMeter},
    fab::AlignedValue,
    ledger::{ContractAddress, DB, DefaultDB, constructor_cell, contract_state},
};
use std::{cell::RefCell, rc::Rc};

const ECHO: WitnessId = WitnessId::new(11);
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
        if id != ECHO {
            return Err(CompactError::AssertionFailed(
                "unknown typed witness".into(),
            ));
        }
        self.calls.borrow_mut().push(context.private_state);
        if !meter.read_cell::<bool>(&[0])? || self.fail {
            return Err(CompactError::AssertionFailed(
                "typed witness refused".into(),
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
fn roundtrip<T: Clone + std::fmt::Debug + PartialEq>(value: T)
where
    AlignedValue: From<T>,
{
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let native_handler = handler(false);
    let recorded_handler = handler(false);
    let witness = Witness::<T, T>::new(ECHO);
    let native =
        Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
            .witness(witness, value.clone())
            .finish()
            .unwrap();
    let recorded =
        Circuit::begin(RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler))
            .witness(witness, value.clone())
            .finish()
            .unwrap();
    assert_eq!(native.result, value);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.context.private_state, 1);
    assert_eq!(recorded.execution.context.private_state, 1);
    assert_eq!(*native_handler.calls.borrow(), [0]);
    assert_eq!(
        *native_handler.calls.borrow(),
        *recorded_handler.calls.borrow()
    );
    let expected = vec![AlignedValue::from(value)];
    assert_eq!(native.private_transcript_outputs, expected);
    assert_eq!(recorded.execution.private_transcript_outputs, expected);
    let meter_context = context();
    let meter = WitnessReadMeter::new(&meter_context);
    assert!(meter.read_cell::<bool>(&[0]).unwrap());
    assert_eq!(native.gas_cost, meter.gas_cost());
    assert_eq!(recorded.execution.gas_cost, meter.gas_cost());
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
fn field_witness_uses_existing_aligned_output_metering_and_replay() {
    roundtrip(Field::from(42_u64));
}
#[test]
fn fixed_bytes_witness_preserves_zero_and_high_bytes() {
    roundtrip(FixedBytes::new([0, 255, 1, 0]));
    roundtrip(FixedBytes::new([]));
}
#[test]
fn product_witness_preserves_nested_alignment_and_element_order() {
    roundtrip((Field::from(7_u64), (FixedBytes::new([0, 255]), true)));
}
#[test]
fn typed_witness_failure_and_prior_guard_stop_later_callbacks() {
    let bindings = Ledger8Bindings::new(&[]).unwrap();
    let witness = Witness::<Field, Field>::new(ECHO);
    for (fail, enabled, calls, message) in [
        (true, true, 1, "typed witness refused"),
        (false, false, 0, "guard refused"),
    ] {
        let native_handler = handler(fail);
        let recorded_handler = handler(fail);
        let native =
            Circuit::begin(NativeLedger8::new(context(), bindings).with_witnesses(&native_handler))
                .require(Rule::new(enabled, "guard refused"))
                .witness(witness, Field::from(1_u64))
                .bind(|_, circuit| circuit.witness(witness, Field::from(2_u64)))
                .finish()
                .err()
                .unwrap();
        let recorded = Circuit::begin(
            RecordedLedger8::new(context(), bindings).with_witnesses(&recorded_handler),
        )
        .require(Rule::new(enabled, "guard refused"))
        .witness(witness, Field::from(1_u64))
        .bind(|_, circuit| circuit.witness(witness, Field::from(2_u64)))
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
