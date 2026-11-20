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

use empty_iteration::runtime::context::{ConstructorContext, WitnessContext};
use empty_iteration::runtime::ledger::ContractAddress;
use empty_iteration::runtime::public_state::PublicStateSource;
use empty_iteration::runtime::{Field, FixedBytes};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
use std::cell::RefCell;

macro_rules! assert_original_loop {
    ($contract:ident, $circuit:ident, $expected:expr) => {{
        let fresh = || {
            $contract::ledger_contract::initial_state(ConstructorContext::new(()))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let before = fresh();
        let mut before_bytes = Vec::new();
        tagged_serialize(before.public_state(), &mut before_bytes).unwrap();
        let ordinary = $contract::ledger_contract::$circuit(before).unwrap();
        assert_eq!(ordinary.result, Field::from($expected));
        assert_eq!(
            $contract::ledger_contract::PublicStateView::from(&ordinary)
                .loop_result()
                .unwrap(),
            Field::from($expected)
        );
        let recorded = $contract::ledger_contract::recorded::$circuit(fresh()).unwrap();
        assert_eq!(recorded.execution.result, ordinary.result);
        assert_eq!(recorded.execution.public_state(), ordinary.public_state());
        assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
        let replay = recorded
            .public
            .initial()
            .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
            .unwrap();
        let mut replay_bytes = Vec::new();
        tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
        let mut ordinary_bytes = Vec::new();
        tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
        assert_eq!(replay_bytes, ordinary_bytes);
        if $expected == 0u128 {
            assert_eq!(ordinary_bytes, before_bytes);
        }
    }};
}

#[test]
fn empty_iteration_original_result_and_replay() {
    assert_original_loop!(empty_iteration, bytes_loop_empty_iteration, 0u128);
}

#[test]
fn spread_iteration_original_result_and_replay() {
    assert_original_loop!(spread_iteration, bytes_loop_spread_iteration, 0u128);
}

#[test]
fn iteration_original_result_and_replay() {
    assert_original_loop!(iteration, bytes_loop_iteration, 120u128);
}

struct OrderWitness {
    calls: RefCell<Vec<&'static str>>,
}

impl zero_operand_order::ledger_contract::Witnesses<u64> for OrderWitness {
    fn initial_operand(
        &self,
        context: WitnessContext<'_, u64, zero_operand_order::ledger_contract::LedgerView<'_>>,
    ) -> (u64, Field) {
        self.calls.borrow_mut().push("initial");
        (*context.private_state + 1, Field::from(2u128))
    }

    fn empty_operand(
        &self,
        context: WitnessContext<'_, u64, zero_operand_order::ledger_contract::LedgerView<'_>>,
    ) -> (u64, FixedBytes<0>) {
        self.calls.borrow_mut().push("source");
        (*context.private_state + 10, FixedBytes::new([]))
    }

    fn unreachable_reducer(
        &self,
        _: WitnessContext<'_, u64, zero_operand_order::ledger_contract::LedgerView<'_>>,
    ) -> (u64, Field) {
        panic!("zero-length reducer executed")
    }
}

#[test]
fn zero_length_fold_evaluates_operands_in_order_and_skips_reducer() {
    let fresh = || {
        zero_operand_order::ledger_contract::initial_state(ConstructorContext::new(0_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let witness = OrderWitness {
        calls: RefCell::new(Vec::new()),
    };
    let ordinary = zero_operand_order::ledger_contract::zero_operand_order(fresh(), &witness)
        .unwrap();
    assert_eq!(ordinary.result, Field::from(0u128));
    assert_eq!(ordinary.context.private_state, 11);
    assert_eq!(witness.calls.into_inner(), vec!["initial", "source"]);
    assert_eq!(ordinary.private_transcript_outputs.len(), 2);

    let recorded_witness = OrderWitness {
        calls: RefCell::new(Vec::new()),
    };
    let recorded = zero_operand_order::ledger_contract::recorded::zero_operand_order(
        fresh(), &recorded_witness,
    )
    .unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.context.private_state, 11);
    assert_eq!(recorded_witness.calls.into_inner(), vec!["initial", "source"]);
    assert_eq!(recorded.execution.private_transcript_outputs.len(), 2);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
}
