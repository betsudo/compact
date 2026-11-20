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

use midnight_compact_runtime::ledger::{
    ChargedState, ContractAddress, DefaultDB, QueryContext, StateValue, constructor_counter,
    increment_counter, read_counter,
};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf,
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
use midnight_storage::storage::HashMap;

fn envelope(value: StateValue<DefaultDB>) -> String {
    let operations = HashMap::new()
        .insert(
            EntryPointBuf(b"increment".to_vec()),
            ContractOperation::new(None, None),
        )
        .insert(
            EntryPointBuf(b"read_round".to_vec()),
            ContractOperation::new(None, None),
        );
    let state = ContractState::new(value, operations, ContractMaintenanceAuthority::default());
    let mut bytes = Vec::new();
    tagged_serialize(&state, &mut bytes).unwrap();
    hex::encode(bytes)
}

#[test]
fn counter_state_matches_ledger9_typescript_oracle() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ledger9-counter-oracle.json")).unwrap();
    let initial = StateValue::Array(vec![constructor_counter::<DefaultDB>()].into());
    let before = envelope(initial.clone());
    let context = QueryContext::new(ChargedState::new(initial), ContractAddress::default());
    let result = increment_counter(&context, 0, 1, None, &INITIAL_COST_MODEL).unwrap();
    let final_state = result.context.state.get_ref().clone();
    let after = envelope(final_state.clone());
    let StateValue::Array(fields) = &final_state else {
        panic!("expected field array")
    };
    assert_eq!(
        read_counter(fields.get(0).unwrap()).unwrap().to_string(),
        oracle["value"]
    );
    assert_eq!(before, oracle["before"]);
    assert_eq!(after, oracle["after"]);
}
