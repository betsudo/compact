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

use manual_update::runtime::context::{CircuitContext, ConstructorContext, WitnessContext};
use manual_update::runtime::ledger::ContractAddress;
use manual_update::runtime::public_state::PublicStateSource;
use manual_update::runtime::FixedBytes;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
use std::cell::Cell;

const EXPECTED: [u8; 4] = [1, 4, 14, 50];

#[test]
fn original_manual_update_ordinary_and_recorded() {
    let context = manual_update::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default());
    let ordinary = manual_update::ledger_contract::bytes_manual_update_semantics(context).unwrap();
    assert_eq!(ordinary.result.clone().into_array(), EXPECTED);
    assert_eq!(
        manual_update::ledger_contract::PublicStateView::from(&ordinary)
            .manual_update_result()
            .unwrap()
            .into_array(),
        EXPECTED,
    );
    let context = manual_update::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default());
    let recorded =
        manual_update::ledger_contract::recorded::bytes_manual_update_semantics(context).unwrap();
    assert_eq!(recorded.execution.result.clone().into_array(), EXPECTED);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    assert!(!recorded.public.verify_ops().is_empty());
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
}

struct Witness {
    bytes: [u8; 4],
    calls: Cell<usize>,
}

impl witness_return::ledger_contract::Witnesses<u64> for Witness {
    fn bytes_witness(
        &self,
        context: WitnessContext<'_, u64, witness_return::ledger_contract::LedgerView<'_>>,
    ) -> (u64, FixedBytes<4>) {
        self.calls.set(self.calls.get() + 1);
        (*context.private_state, FixedBytes::new(self.bytes))
    }
}

fn witness_context() -> CircuitContext<u64> {
    witness_return::ledger_contract::initial_state(ConstructorContext::new(3_u64))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}

#[test]
fn original_witness_return_ordinary_and_recorded() {
    let witness = Witness {
        bytes: [10, 20, 30, 40],
        calls: Cell::new(0),
    };
    let ordinary = witness_return::ledger_contract::bytes_witness_return_semantics(
        witness_context(),
        &witness,
    )
    .unwrap();
    assert_eq!(ordinary.result, ());
    assert_eq!(ordinary.context.private_state, 3);
    assert_eq!(ordinary.private_transcript_outputs.len(), 1);
    assert_eq!(witness.calls.get(), 1);

    let recorded = witness_return::ledger_contract::recorded::bytes_witness_return_semantics(
        witness_context(),
        &witness,
    )
    .unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.private_transcript_outputs.len(), 1);
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_eq!(witness.calls.get(), 2);
}

#[test]
fn wrong_witness_bytes_are_refused_in_both_modes() {
    for (index, value, message) in [
        (0, 11, "Witness byte 0 should be 10"),
        (1, 21, "Witness byte 1 should be 20"),
        (3, 41, "Witness byte 3 should be 40"),
    ] {
        let mut bytes = [10, 20, 30, 40];
        bytes[index] = value;
        let witness = Witness {
            bytes,
            calls: Cell::new(0),
        };
        let ordinary = witness_return::ledger_contract::bytes_witness_return_semantics(
            witness_context(),
            &witness,
        );
        assert!(ordinary.unwrap_err().to_string().contains(message));
        let recorded = witness_return::ledger_contract::recorded::bytes_witness_return_semantics(
            witness_context(),
            &witness,
        );
        assert!(recorded.unwrap_err().to_string().contains(message));
        assert_eq!(witness.calls.get(), 2);
    }
}
