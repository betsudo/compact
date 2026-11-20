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

use basic::runtime::{Field, context::ConstructorContext, ledger::ContractAddress};
use basic::runtime::public_state::PublicStateSource;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

const EXPECTED: [u128; 4] = [0, 0, 0, 0];

#[test]
fn original_default_semantics() {
    let expected = EXPECTED.map(Field::from);
    let context = basic::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default());
    let ordinary = basic::ledger_contract::vector_default_semantics(context).unwrap();
    assert_eq!(ordinary.result.clone().into_array(), expected);
    assert_eq!(
        basic::ledger_contract::PublicStateView::from(&ordinary)
            .default_result().unwrap().into_array(),
        expected,
    );

    let context = basic::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default());
    let recorded = basic::ledger_contract::recorded::vector_default_semantics(context).unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert!(ordinary.private_transcript_outputs.is_empty());
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    assert!(!recorded.public.verify_ops().is_empty());
    let replay = recorded.public.initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
}
