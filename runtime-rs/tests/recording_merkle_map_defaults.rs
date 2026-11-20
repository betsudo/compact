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

use midnight_compact_runtime::{
    Field,
    context::{ConstructorContext, ConstructorResult},
    fab::AlignedValue,
    ledger::{ContractAddress, DefaultDB, StateValue, constructor_cell, contract_state, read_cell},
    recording::RecordingFrame,
    slots::{MapSlot, MerkleNode},
};

#[test]
fn nested_merkle_defaults_preserve_the_sibling_and_replay() {
    check_nested_merkle_default::<false>();
    check_nested_merkle_default::<true>();
}

fn check_nested_merkle_default<const HISTORIC: bool>() {
    let slot = MapSlot::<bool, MerkleNode<Field, 32, HISTORIC>>::new(&[0]);
    let sibling = constructor_cell::<_, DefaultDB>(Field::from(37_u64));
    let mut entries = midnight_compact_runtime::ledger::LedgerHashMap::new();
    entries = entries.insert(AlignedValue::from(false), sibling.clone());
    entries = entries.insert(AlignedValue::from(true), StateValue::Null);
    let initial = contract_state(vec![StateValue::Map(entries)]);
    let context = || {
        ConstructorResult::new(ConstructorContext::new(()), initial.clone())
            .into_circuit_context(ContractAddress::default())
    };

    let native = slot.insert_default(context(), true).unwrap();
    let recorded = slot
        .record_insert_default(RecordingFrame::new(context()), true)
        .unwrap()
        .finish(());
    let replay = recorded
        .public
        .initial()
        .query(
            recorded.public.verify_ops(),
            None,
            &native.context.cost_model,
        )
        .unwrap();
    assert_eq!(native.context.query.state, replay.context.state);
    assert_eq!(
        native.context.query.state,
        recorded.execution.context.query.state
    );
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);

    let StateValue::Array(root) = native.context.query.state.get_ref() else {
        panic!("contract root");
    };
    let Some(StateValue::Map(map)) = root.get(0) else {
        panic!("map slot");
    };
    assert_eq!(
        map.get(&AlignedValue::from(false)).as_deref(),
        Some(&sibling)
    );
    let stored = map.get(&AlignedValue::from(true)).unwrap();
    let StateValue::Array(fields) = &*stored else {
        panic!("nested tree");
    };
    assert_eq!(fields.len(), if HISTORIC { 3 } else { 2 });
    let Some(StateValue::BoundedMerkleTree(tree)) = fields.get(0) else {
        panic!("bounded tree");
    };
    assert_eq!(tree.height(), 32);
    assert_eq!(read_cell::<u64, _>(fields.get(1).unwrap()).unwrap(), 0);
    if HISTORIC {
        let Some(StateValue::Map(history)) = fields.get(2) else {
            panic!("root history");
        };
        assert!(history.is_empty());
    }
    let invalid = MapSlot::<bool, MerkleNode<Field, 33, HISTORIC>>::new(&[0]);
    assert!(invalid.insert_default(context(), true).is_err());
    assert!(
        invalid
            .record_insert_default(RecordingFrame::new(context()), true)
            .is_err()
    );
}
