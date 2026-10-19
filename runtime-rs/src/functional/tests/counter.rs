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
    CellBinding, CounterBinding, Ledger8Bindings, NativeLedger8, RecordedLedger8,
};
use super::super::{Cell, Circuit, Counter, CounterIncrement, CounterRead, Executor, SlotId};
use crate::{
    CompactError,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_counter, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
const ROUND: Counter = Counter::new(SlotId::new(3));
const DECLARATIONS: [CounterBinding; 1] = [CounterBinding::new(ROUND.id(), &[0])];
struct Increment {
    amount: u16,
}
impl Increment {
    fn run<E: Executor + CounterRead + CounterIncrement>(
        &self,
        executor: E,
    ) -> Result<E::Outcome<u64>, CompactError> {
        Circuit::begin(executor)
            .increment(ROUND, self.amount)
            .read_counter(ROUND)
            .finish()
    }
}
fn context() -> CircuitContext<(), DefaultDB> {
    ConstructorResult::new(
        ConstructorContext::new(()),
        contract_state(vec![constructor_counter::<DefaultDB>()]),
    )
    .into_circuit_context(ContractAddress::default())
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut output = Vec::new();
    tagged_serialize(state, &mut output).unwrap();
    output
}
#[test]
fn counter_one_body_preserves_native_recorded_replay_and_gas() {
    let bindings = Ledger8Bindings::with_counters(&[], &DECLARATIONS).unwrap();
    for amount in [0, 1, u16::MAX] {
        let program = Increment { amount };
        let native = program
            .run(NativeLedger8::new(context(), bindings))
            .unwrap();
        let recorded = program
            .run(RecordedLedger8::new(context(), bindings))
            .unwrap();
        assert_eq!(native.result, u64::from(amount));
        assert_eq!(native.result, recorded.execution.result);
        assert_eq!(native.gas_cost, recorded.execution.gas_cost);
        assert_eq!(
            native.private_transcript_outputs,
            recorded.execution.private_transcript_outputs
        );
        assert_eq!(
            bytes(native.context.query.state.get_ref()),
            bytes(recorded.execution.context.query.state.get_ref())
        );
        assert_eq!(
            native.context.query.effects,
            recorded.execution.context.query.effects
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
fn counter_binding_kind_duplicates_and_query_limit_refuse() {
    let bindings = Ledger8Bindings::with_counters(&[], &DECLARATIONS).unwrap();
    let cell = Cell::<u64>::new(ROUND.id());
    assert!(
        Circuit::begin(NativeLedger8::new(context(), bindings))
            .read(cell)
            .finish()
            .is_err()
    );
    assert!(
        Circuit::begin(RecordedLedger8::new(context(), bindings))
            .read(cell)
            .finish()
            .is_err()
    );
    let duplicate_cells = [CellBinding::new(ROUND.id(), &[0])];
    assert!(Ledger8Bindings::with_counters(&duplicate_cells, &DECLARATIONS).is_err());
    let duplicate_counters = [DECLARATIONS[0], DECLARATIONS[0]];
    assert!(Ledger8Bindings::with_counters(&[], &duplicate_counters).is_err());
    let mut native = context();
    native.gas_limit = Some(RunningCost::ZERO);
    let mut recorded = context();
    recorded.gas_limit = Some(RunningCost::ZERO);
    let program = Increment { amount: 1 };
    assert!(matches!(
        program.run(NativeLedger8::new(native, bindings)),
        Err(CompactError::LedgerQueryRejected(_))
    ));
    assert!(matches!(
        program.run(RecordedLedger8::new(recorded, bindings)),
        Err(CompactError::LedgerQueryRejected(_))
    ));
}
