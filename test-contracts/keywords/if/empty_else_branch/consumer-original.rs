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

use empty_else_branch::runtime::context::ConstructorContext;
use empty_else_branch::runtime::ledger::ContractAddress;
use empty_else_branch::runtime::public_state::PublicStateSource;
use empty_else_branch::runtime::Field;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

fn fresh() -> empty_else_branch::runtime::context::CircuitContext<()> {
    empty_else_branch::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}

#[test]
fn original_true_and_false_results_with_recorded_replay() {
    for (input, expected) in [(true, 1u128), (false, 0u128)] {
        let ordinary = empty_else_branch::ledger_contract::if_empty_else_branch(fresh(), input)
            .unwrap();
        assert_eq!(ordinary.result, Field::from(expected));
        assert_eq!(
            empty_else_branch::ledger_contract::PublicStateView::from(&ordinary)
                .count()
                .unwrap(),
            Field::from(expected)
        );
        let recorded =
            empty_else_branch::ledger_contract::recorded::if_empty_else_branch(fresh(), input)
                .unwrap();
        assert_eq!(recorded.execution.result, Field::from(expected));
        assert_eq!(recorded.execution.public_state(), ordinary.public_state());
        assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
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
    }
}
