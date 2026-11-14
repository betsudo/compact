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

use empty_iteration::runtime::{context::ConstructorContext, ledger::ContractAddress, Field};
use empty_iteration::runtime::public_state::PublicStateSource;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

macro_rules! check_original {
    ($contract:ident, $circuit:ident, $expected:expr) => {{
        let context = $contract::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default());
        let ordinary = $contract::ledger_contract::$circuit(context).unwrap();
        assert_eq!(ordinary.result, Field::from($expected as u128));
        assert_eq!(
            $contract::ledger_contract::PublicStateView::from(&ordinary)
                .loop_result()
                .unwrap(),
            Field::from($expected as u128),
        );
        let context = $contract::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default());
        let recorded = $contract::ledger_contract::recorded::$circuit(context).unwrap();
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
    }};
}

#[test]
fn original_empty_iteration() {
    check_original!(empty_iteration, vector_loop_empty_iteration, 0);
}

#[test]
fn original_iteration() {
    check_original!(iteration, vector_loop_iteration, 150);
}

#[test]
fn original_spread_iteration() {
    check_original!(spread_iteration, vector_loop_spread_iteration, 15);
}
