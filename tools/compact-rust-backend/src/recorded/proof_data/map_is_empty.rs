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

//! One root Map<Field, Field> emptiness read with no input or side effects.
//! The facade reuses the actual recorded call; classification never evaluates
//! a map query, helper, witness or expression graph.
use crate::ir::{Expr, LedgerField, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;

pub(super) fn supported(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    if circuit.internal
        || !circuit.actions.is_empty()
        || circuit.result != Type::Boolean
        || !circuit.parameters.is_empty()
    {
        return false;
    }
    let (field, index) = match &circuit.return_value {
        StateReturn::MapIsEmpty { field, index }
        | StateReturn::Expression {
            value: Expr::MapIsEmpty { field, index },
        } => (field, index),
        _ => return false,
    };
    super::root_field_map_slot(field, *index, fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn shape() -> Value {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../fixtures/upstream-672/export-map-field-field/compact-rust-ir.json"
        ))
        .unwrap();
        let read = value["stateful_circuits"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "is_empty")
            .unwrap()
            .clone();
        value["stateful_circuits"] = json!([read]);
        value
    }
    fn classified(value: Value) -> bool {
        let contract: crate::ir::Contract = serde_json::from_value(value).unwrap();
        let fields = contract
            .ledger_fields
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        supported(&contract.stateful_circuits[0], &fields)
    }

    #[test]
    fn direct_and_expression_reads_accept_only_equivalent_root_paths() {
        let mut value = shape();
        assert!(classified(value.clone()));
        value["ledger_fields"][0]["path"] = json!([]);
        assert!(classified(value.clone()));
        value["ledger_fields"][0]["path"] = json!([0]);
        let read = value["stateful_circuits"][0]["return_value"].take();
        value["stateful_circuits"][0]["return_value"] = json!({"kind":"expression","value":read});
        assert!(classified(value));
        let mut value = shape();
        value["ledger_fields"][0]["id"] = json!("renamed");
        value["ledger_fields"][0]["index"] = json!(15);
        value["ledger_fields"][0]["path"] = json!([15]);
        value["stateful_circuits"][0]["name"] = json!("vacant");
        value["stateful_circuits"][0]["return_value"]["field"] = json!("renamed");
        value["stateful_circuits"][0]["return_value"]["index"] = json!(15);
        assert!(classified(value));
    }

    #[test]
    fn classifier_itself_refuses_wrong_slots_types_parameters_and_actions() {
        for (pointer, replacement) in [
            ("/ledger_fields/0/id", json!("other")),
            ("/ledger_fields/0/index", json!(1)),
            ("/ledger_fields/0/path", json!([0, 1])),
            ("/ledger_fields/0/path", json!([1])),
            (
                "/ledger_fields/0/declaration/key",
                json!({"kind":"boolean"}),
            ),
            (
                "/ledger_fields/0/declaration/value",
                json!({"kind":"boolean"}),
            ),
            (
                "/ledger_fields/0/declaration",
                json!({"kind":"cell","ty":{"kind":"boolean"}}),
            ),
            ("/stateful_circuits/0/internal", json!(true)),
            ("/stateful_circuits/0/result", json!({"kind":"field"})),
            (
                "/stateful_circuits/0/parameters",
                json!([{"name":"key","ty":{"kind":"field"}}]),
            ),
            ("/stateful_circuits/0/return_value/field", json!("unknown")),
            ("/stateful_circuits/0/return_value/index", json!(1)),
            (
                "/stateful_circuits/0/actions",
                json!([{"kind":"map_reset","field":"c","index":0}]),
            ),
        ] {
            let mut value = shape();
            if pointer == "/ledger_fields/0/path" {
                value["ledger_fields"][0]["path"] = replacement;
            } else if pointer == "/stateful_circuits/0/internal" {
                value["stateful_circuits"][0]["internal"] = replacement;
            } else {
                *value.pointer_mut(pointer).unwrap() = replacement;
            }
            assert!(!classified(value), "{pointer}");
        }
        let mut value = shape();
        value["ledger_fields"][0]["index"] = json!(16);
        value["stateful_circuits"][0]["return_value"]["index"] = json!(16);
        assert!(!classified(value));
    }

    #[test]
    fn no_witness_call_coercion_branch_or_other_metadata_query_is_admitted() {
        for expression in [
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"coerce","ty":{"kind":"boolean"},"value":{"kind":"map_is_empty","field":"c","index":0}}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"map_is_empty","field":"c","index":0},"otherwise":{"kind":"boolean","value":false}}),
            json!({"kind":"map_size","field":"c","index":0}),
            json!({"kind":"set_is_empty","field":"c","index":0}),
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["return_value"] =
                json!({"kind":"expression","value":expression});
            assert!(!classified(value));
        }
    }
}
