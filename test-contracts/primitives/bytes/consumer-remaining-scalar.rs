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

use default_semantics::runtime::context::ConstructorContext;
use default_semantics::runtime::ledger::ContractAddress;
use default_semantics::runtime::public_state::PublicStateSource;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

macro_rules! stateful_original {
    ($module:ident, $circuit:ident, $field:ident, |$result:ident| $check:expr) => {{
        let initial = $module::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();
        let ordinary = $module::ledger_contract::$circuit(
            initial.into_circuit_context(ContractAddress::default()),
        )
        .unwrap();
        let $result = &ordinary.result;
        $check;
        let view_result = $module::ledger_contract::PublicStateView::from(&ordinary)
            .$field()
            .unwrap();
        let $result = &view_result;
        $check;

        let initial = $module::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();
        let recorded = $module::ledger_contract::recorded::$circuit(
            initial.into_circuit_context(ContractAddress::default()),
        )
        .unwrap();
        assert_eq!(recorded.execution.result, ordinary.result);
        assert_eq!(recorded.execution.public_state(), ordinary.public_state());
        assert!(ordinary.private_transcript_outputs.is_empty());
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
    }};
}

#[test]
fn original_default_semantics() {
    stateful_original!(
        default_semantics,
        bytes_default_semantics,
        default_result,
        |result| assert_eq!(result.clone().into_array(), [0u8; 128])
    );
}

#[test]
fn original_equality_empty() {
    stateful_original!(
        equality_empty,
        bytes_equality_empty,
        equality_result,
        |result| assert!(*result)
    );
}

#[test]
fn original_equality_semantics() {
    assert!(equality_semantics::pure_circuits::bytes_equality_semantics().unwrap());
}

#[test]
fn original_indexing_basic() {
    stateful_original!(
        indexing_basic,
        bytes_indexing_basic,
        indexing_result,
        |result| assert_eq!(
            (result.0.value(), result.1.value(), result.2.value()),
            (10, 50, 80)
        )
    );
}

#[test]
fn original_pad_semantics() {
    assert_eq!(
        pad_semantics::pure_circuits::bytes_pad_semantics()
            .unwrap()
            .into_array(),
        [97, 98, 0, 0],
    );
}

#[test]
fn original_slice_basic() {
    stateful_original!(
        slice_basic,
        bytes_slice_basic,
        slice_result,
        |result| assert_eq!(result.clone().into_array(), [5])
    );
}

#[test]
fn original_slice_empty() {
    stateful_original!(
        slice_empty,
        bytes_slice_empty,
        slice_result,
        |result| assert_eq!(result.clone().into_array(), [0u8; 0])
    );
}

#[test]
fn original_spread_basic() {
    stateful_original!(
        spread_basic,
        bytes_spread_basic,
        spread_result,
        |result| assert_eq!(
            result.clone().into_array(),
            [0, 255, 255, 255, 128, 0, 255, 255, 255, 128]
        )
    );
}

#[test]
fn original_spread_empty() {
    stateful_original!(
        spread_empty,
        bytes_spread_empty,
        spread_result,
        |result| assert_eq!(result.clone().into_array(), [0u8; 0])
    );
}

#[test]
fn original_spread_multiple() {
    stateful_original!(
        spread_multiple,
        bytes_spread_multiple,
        spread_result,
        |result| assert_eq!(
            result.clone().into_array(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        )
    );
}

#[test]
fn original_spread_nested() {
    stateful_original!(
        spread_nested,
        bytes_spread_nested,
        spread_result,
        |result| assert_eq!(
            result.clone().into_array(),
            [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3]
        )
    );
}
