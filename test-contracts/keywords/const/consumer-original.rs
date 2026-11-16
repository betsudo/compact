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

use empty_tuple_pattern::runtime::context::ConstructorContext;
use empty_tuple_pattern::runtime::ledger::ContractAddress;
use empty_tuple_pattern::runtime::{Field, FixedVector};
use empty_tuple_pattern::runtime::public_state::PublicStateSource;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

#[test]
fn original_pure_const_results() {
    assert_eq!(empty_tuple_pattern::pure_circuits::const_empty_tuple_pattern().unwrap(), ());
    assert_eq!(
        generic_type_annotation::pure_circuits::const_generic_type_annotation().unwrap(),
        FixedVector::new([Field::from(0u128); 3]),
    );
    assert_eq!(in_braced_bodies::pure_circuits::const_in_braced_bodies(true).unwrap(), Field::from(3u128));
    assert_eq!(in_braced_bodies::pure_circuits::const_in_braced_bodies(false).unwrap(), Field::from(12u128));
    assert_eq!(later_binding_sees_earlier::pure_circuits::const_later_binding_sees_earlier().unwrap(), Field::from(2u128));
    assert_eq!(multiple_bindings::pure_circuits::const_multiple_bindings().unwrap(),
               (true, Field::from(1u128), Field::from(2u128)));
    assert_eq!(pattern_trailing_comma::pure_circuits::const_pattern_trailing_comma().unwrap(), Field::from(1u128));
    assert_eq!(shadowing_in_blocks::pure_circuits::const_shadowing_in_blocks().unwrap(), Field::from(1u128));
    assert_eq!(struct_destructuring::pure_circuits::const_struct_destructuring().unwrap(),
               (Field::from(1u128), Field::from(2u128)));
    assert_eq!(tuple_destructuring::pure_circuits::const_tuple_destructuring().unwrap(),
               (Field::from(1u128), Field::from(2u128), Field::from(10u128), Field::from(20u128)));
    assert_eq!(underscore_binding::pure_circuits::const_underscore_binding().unwrap(), Field::from(2u128));
}

#[test]
fn original_const_initializer_order_and_recorded_replay() {
    let fresh = || initializer_side_effects::ledger_contract::initial_state(
        ConstructorContext::new(())
    ).unwrap().into_circuit_context(ContractAddress::default());
    let ordinary = initializer_side_effects::ledger_contract::const_initializer_side_effects(fresh()).unwrap();
    assert_eq!((ordinary.result.0.value(), ordinary.result.1.value()), (1, 2));
    let recorded = initializer_side_effects::ledger_contract::recorded::const_initializer_side_effects(fresh()).unwrap();
    assert_eq!((recorded.execution.result.0.value(), recorded.execution.result.1.value()), (1, 2));
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert!(!recorded.public.verify_ops().is_empty());
    let replay = recorded.public.initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL).unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
}
