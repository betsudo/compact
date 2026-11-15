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

use boolean_reduction::runtime::context::ConstructorContext;
use boolean_reduction::runtime::ledger::ContractAddress;
use boolean_reduction::runtime::public_state::PublicStateSource;
use conditional_accumulation::runtime::Field;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

#[test]
fn pure_bytes_fold_source_normalization() {
    assert_eq!(
        pure_control::pure_circuits::bytes_fold_pure()
            .unwrap()
            .value(),
        6
    );
}

macro_rules! assert_replayed_state {
    ($recorded:expr, $ordinary:expr) => {{
        let replay = $recorded
            .public
            .initial()
            .query($recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
            .unwrap();
        let mut replay_bytes = Vec::new();
        tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
        let mut ordinary_bytes = Vec::new();
        tagged_serialize($ordinary.public_state(), &mut ordinary_bytes).unwrap();
        assert_eq!(replay_bytes, ordinary_bytes);
    }};
}

#[test]
fn boolean_reduction_original_result_and_replay() {
    let fresh = || {
        boolean_reduction::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let ordinary =
        boolean_reduction::ledger_contract::bytes_fold_boolean_reduction(fresh()).unwrap();
    assert!(ordinary.result);
    assert!(
        boolean_reduction::ledger_contract::PublicStateView::from(&ordinary)
            .fold_result()
            .unwrap()
    );
    let recorded =
        boolean_reduction::ledger_contract::recorded::bytes_fold_boolean_reduction(fresh())
            .unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_replayed_state!(recorded, ordinary);
}

#[test]
fn conditional_accumulation_original_result_and_replay() {
    let fresh = || {
        conditional_accumulation::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let ordinary =
        conditional_accumulation::ledger_contract::bytes_fold_conditional_accumulation(fresh())
            .unwrap();
    assert_eq!(ordinary.result, Field::from(15u128));
    assert_eq!(
        conditional_accumulation::ledger_contract::PublicStateView::from(&ordinary)
            .fold_result()
            .unwrap(),
        Field::from(15u128)
    );
    let recorded =
        conditional_accumulation::ledger_contract::recorded::bytes_fold_conditional_accumulation(
            fresh(),
        )
        .unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_replayed_state!(recorded, ordinary);
}

#[test]
fn summation_original_result_and_replay() {
    let fresh = || {
        summation::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let ordinary = summation::ledger_contract::bytes_fold_summation(fresh()).unwrap();
    assert_eq!(ordinary.result.value(), 15);
    assert_eq!(
        summation::ledger_contract::PublicStateView::from(&ordinary)
            .fold_result()
            .unwrap()
            .value(),
        15
    );
    let recorded = summation::ledger_contract::recorded::bytes_fold_summation(fresh()).unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_replayed_state!(recorded, ordinary);
}
