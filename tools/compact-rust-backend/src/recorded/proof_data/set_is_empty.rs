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

//! Optional capture of exactly one root Set<Field> emptiness query.
//! Classification does not evaluate or reconstruct the recorded execution.
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
        StateReturn::SetIsEmpty { field, index }
        | StateReturn::Expression {
            value: Expr::SetIsEmpty { field, index },
        } => (field, index),
        _ => return false,
    };
    super::root_field_set_slot(field, *index, fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    // Selecting one circuit is confined to these private classifier tests.
    // The generator and subsequent qualification use all six original exports.
    fn shape() -> Value {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../fixtures/upstream-672/export-set-field/compact-rust-ir.json"
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
    fn direct_expression_and_renamed_reads_accept_only_equivalent_root_slots() {
        for index in [0, 15] {
            for path in [vec![], vec![index]] {
                let mut value = shape();
                value["ledger_fields"][0]["id"] = json!("renamed");
                value["ledger_fields"][0]["index"] = json!(index);
                value["ledger_fields"][0]["path"] = json!(path);
                value["stateful_circuits"][0]["name"] = json!("vacant");
                value["stateful_circuits"][0]["return_value"]["field"] = json!("renamed");
                value["stateful_circuits"][0]["return_value"]["index"] = json!(index);
                assert!(classified(value.clone()));
                let read = value["stateful_circuits"][0]["return_value"].take();
                value["stateful_circuits"][0]["return_value"] =
                    json!({"kind":"expression","value":read});
                assert!(classified(value));
            }
        }
    }

    #[test]
    fn declaration_hash_key_must_match_the_declared_identity() {
        let contract: crate::ir::Contract = serde_json::from_value(shape()).unwrap();
        let mut declaration = contract.ledger_fields[0].clone();
        declaration.id = "other".into();
        let fields = HashMap::from([("c", &declaration)]);
        assert!(!supported(&contract.stateful_circuits[0], &fields));
        assert!(!supported(&contract.stateful_circuits[0], &HashMap::new()));
    }

    #[test]
    fn slot_type_arity_result_visibility_and_effect_boundaries_refuse() {
        for (pointer, replacement) in [
            ("/ledger_fields/0/id", json!("other")),
            ("/ledger_fields/0/index", json!(1)),
            ("/ledger_fields/0/path", json!([1])),
            ("/ledger_fields/0/path", json!([0, 1])),
            ("/ledger_fields/0/declaration/ty", json!({"kind":"boolean"})),
            (
                "/ledger_fields/0/declaration",
                json!({"kind":"map","key":{"kind":"field"},"value":{"kind":"field"}}),
            ),
            ("/stateful_circuits/0/result", json!({"kind":"field"})),
            (
                "/stateful_circuits/0/parameters",
                json!([{"name":"key","ty":{"kind":"field"}}]),
            ),
            ("/stateful_circuits/0/return_value/field", json!("missing")),
            ("/stateful_circuits/0/return_value/index", json!(1)),
            (
                "/stateful_circuits/0/actions",
                json!([{"kind":"set_reset","field":"c","index":0}]),
            ),
        ] {
            let mut value = shape();
            *value.pointer_mut(pointer).unwrap() = replacement;
            assert!(!classified(value), "{pointer}");
        }
        let mut value = shape();
        value["stateful_circuits"][0]["internal"] = json!(true);
        assert!(!classified(value));
        let mut value = shape();
        value["ledger_fields"][0]["index"] = json!(16);
        value["ledger_fields"][0]["path"] = json!([16]);
        value["stateful_circuits"][0]["return_value"]["index"] = json!(16);
        assert!(!classified(value));
    }

    #[test]
    fn witness_helper_cast_branch_binding_and_other_query_are_not_this_family() {
        for expression in [
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"coerce","ty":{"kind":"boolean"},"value":{"kind":"set_is_empty","field":"c","index":0}}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"set_is_empty","field":"c","index":0},"otherwise":{"kind":"boolean","value":false}}),
            json!({"kind":"let","bindings":[{"name":"local","ty":{"kind":"boolean"},"value":{"kind":"set_is_empty","field":"c","index":0}}],"body":{"kind":"parameter","name":"local"}}),
            json!({"kind":"map_is_empty","field":"c","index":0}),
            json!({"kind":"set_size","field":"c","index":0}),
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["return_value"] =
                json!({"kind":"expression","value":expression});
            assert!(!classified(value));
        }

        for returned in [
            json!({"kind":"map_is_empty","field":"c","index":0}),
            json!({"kind":"effectful","body":{"kind":"sequence","actions":[{"kind":"set_reset","field":"c","index":0}],"result":{"kind":"value","value":{"kind":"boolean","value":true}}}}),
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["return_value"] = returned;
            assert!(!classified(value));
        }
    }
}
