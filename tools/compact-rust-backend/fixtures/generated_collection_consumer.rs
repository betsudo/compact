// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use compact_contract_ledger9_map_transition_v018 as map;
use compact_contract_ledger9_set_transition_v018 as set;
use compact_contract_list_field as list;
use midnight_compact_runtime::context::CircuitResult;
use midnight_compact_runtime::ledger::{ContractAddress, DefaultDB, StateValue};
use midnight_compact_runtime::recording::RecordedCircuitResult;
use midnight_compact_runtime::{Field, context::ConstructorContext};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf,
};
use midnight_serialize::tagged_serialize;
use midnight_storage::storage::HashMap;

fn state_bytes(state: StateValue<DefaultDB>, names: &[&str]) -> String {
    let mut operations: HashMap<EntryPointBuf, ContractOperation, DefaultDB> = HashMap::new();
    for name in names {
        operations = operations.insert(
            EntryPointBuf(name.as_bytes().to_vec()),
            ContractOperation::new(None, None),
        );
    }
    let contract = ContractState::new(state, operations, ContractMaintenanceAuthority::default());
    let mut bytes = Vec::new();
    tagged_serialize(&contract, &mut bytes).unwrap();
    hex::encode(bytes)
}

fn reference(kind: &str) -> serde_json::Value {
    let root = std::env::var("COMPACT_REPO_ROOT").expect("parity runner supplies repository root");
    let path = format!("{root}/runtime-rs/tests/fixtures/ledger9-{kind}-transition.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn expect_state(
    actual: StateValue<DefaultDB>,
    names: &[&str],
    oracle: &serde_json::Value,
    step: Option<usize>,
    kind: &str,
) {
    let expected = match step {
        Some(i) => &oracle["steps"][i]["stateHex"],
        None => &oracle["afterInit"]["stateHex"],
    };
    assert_eq!(
        state_bytes(actual, names),
        expected.as_str().unwrap(),
        "{kind} step {step:?}"
    );
}

fn replay(out: RecordedCircuitResult<(), ()>) -> CircuitResult<(), ()> {
    let checked = out
        .public
        .initial()
        .query(
            out.public.verify_ops(),
            None,
            &out.execution.context.cost_model,
        )
        .expect("independent public replay succeeds");
    assert_eq!(checked.context.state, out.execution.context.query.state);
    assert_eq!(checked.context.effects, out.execution.context.query.effects);
    assert_eq!(checked.gas_cost, out.execution.gas_cost);
    out.execution
}

macro_rules! mutation {
    ($recorded:expr, $module:ident, $name:ident, $context:expr $(, $arg:expr)* $(,)?) => {
        if $recorded {
            replay($module::ledger_contract::recorded::$name($context $(, $arg)*).unwrap())
        } else {
            $module::ledger_contract::$name($context $(, $arg)*).unwrap()
        }
    };
}

fn set_sequence(recorded: bool) {
    let names = [
        "add",
        "contains",
        "add_field",
        "contains_field",
        "remove",
        "seen_size",
        "seen_is_empty",
        "reset_fields",
    ];
    let oracle = reference("set");
    let initial = set::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();
    expect_state(
        initial.ledger_state.get_ref().clone(),
        &names,
        &oracle,
        None,
        "set",
    );
    let context = initial.into_circuit_context(ContractAddress::default());
    let call = mutation!(recorded, set, add, context, true);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(0),
        "set",
    );
    let call = mutation!(recorded, set, remove, call.context, true);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(1),
        "set",
    );
    let call = mutation!(recorded, set, add_field, call.context, Field::from(42_u64));
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(2),
        "set",
    );
    let call = mutation!(recorded, set, reset_fields, call.context);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(3),
        "set",
    );
}

fn map_sequence(recorded: bool) {
    let names = [
        "put",
        "put_default",
        "has",
        "get",
        "remove_key",
        "table_size",
        "table_is_empty",
        "reset_table",
    ];
    let oracle = reference("map");
    let initial = map::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();
    expect_state(
        initial.ledger_state.get_ref().clone(),
        &names,
        &oracle,
        None,
        "map",
    );
    let context = initial.into_circuit_context(ContractAddress::default());
    let call = mutation!(recorded, map, put, context, true, Field::from(42_u64));
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(0),
        "map",
    );
    let call = mutation!(recorded, map, remove_key, call.context, true);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(1),
        "map",
    );
    let call = mutation!(recorded, map, put_default, call.context, true);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(2),
        "map",
    );
    let call = mutation!(recorded, map, reset_table, call.context);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(3),
        "map",
    );
}

fn list_sequence(recorded: bool) {
    let names = [
        "item_count",
        "items_empty",
        "first_item",
        "prepend",
        "drop_first",
        "clear_items",
    ];
    let oracle = reference("list");
    let initial = list::ledger_contract::initial_state(ConstructorContext::new(())).unwrap();
    expect_state(
        initial.ledger_state.get_ref().clone(),
        &names,
        &oracle,
        None,
        "list",
    );
    let context = initial.into_circuit_context(ContractAddress::default());
    let call = mutation!(recorded, list, prepend, context, Field::from(42_u64));
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(0),
        "list",
    );
    let call = mutation!(recorded, list, prepend, call.context, Field::from(7_u64));
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(1),
        "list",
    );
    let call = mutation!(recorded, list, drop_first, call.context);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(2),
        "list",
    );
    let call = mutation!(recorded, list, clear_items, call.context);
    expect_state(
        call.context.query.state.get_ref().clone(),
        &names,
        &oracle,
        Some(3),
        "list",
    );
}

fn main() {
    for recorded in [false, true] {
        set_sequence(recorded);
        map_sequence(recorded);
        list_sequence(recorded);
    }
    println!(
        "generated Set/Map/List ordinary and recorded/replayed transitions match retained TypeScript oracles"
    );
}
