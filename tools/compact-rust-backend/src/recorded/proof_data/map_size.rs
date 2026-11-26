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

//! One root Map<Field, Field> cardinality read with its checked Uint64 result.
//! Admission inspects a fixed shape; the facade captures the actual recording.
use crate::ir::{Expr, LedgerField, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;

pub(super) fn supported(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    if circuit.internal
        || !circuit.actions.is_empty()
        || !circuit.parameters.is_empty()
        || !matches!(&circuit.result, Type::Unsigned { max } if max == "18446744073709551615")
    {
        return false;
    }
    let (field, index) = match &circuit.return_value {
        StateReturn::MapSize { field, index }
        | StateReturn::Expression {
            value: Expr::MapSize { field, index },
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
            .find(|c| c["name"] == "size")
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
    fn direct_and_exact_expression_accept_renaming_and_both_root_encodings() {
        for index in [0, 15] {
            for path in [json!([]), json!([index])] {
                for expression in [false, true] {
                    let mut value = shape();
                    value["ledger_fields"][0]["id"] = json!("renamed_map");
                    value["ledger_fields"][0]["index"] = json!(index);
                    value["ledger_fields"][0]["path"] = path.clone();
                    let read = &mut value["stateful_circuits"][0];
                    read["name"] = json!("cardinality");
                    read["return_value"]["field"] = json!("renamed_map");
                    read["return_value"]["index"] = json!(index);
                    if expression {
                        let direct = read["return_value"].take();
                        read["return_value"] = json!({"kind":"expression","value":direct});
                    }
                    assert!(classified(value), "{index}/{path}/{expression}");
                }
            }
        }
    }

    #[test]
    fn shared_slot_guard_checks_declaration_identity_even_with_a_matching_hash_key() {
        let contract: crate::ir::Contract = serde_json::from_value(shape()).unwrap();
        let mut declaration = contract.ledger_fields[0].clone();
        declaration.id = "other".into();
        let fields = HashMap::from([("c", &declaration)]);
        assert!(!super::super::root_field_map_slot("c", 0, &fields));
        assert!(!supported(&contract.stateful_circuits[0], &fields));
    }

    #[test]
    fn classifier_itself_refuses_invalid_physical_slots_types_arity_and_effects() {
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
                json!({"kind":"cell","ty":{"kind":"field"}}),
            ),
            ("/stateful_circuits/0/internal", json!(true)),
            ("/stateful_circuits/0/result", json!({"kind":"field"})),
            ("/stateful_circuits/0/result", json!({"kind":"boolean"})),
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
            if pointer.ends_with("/path") || pointer.ends_with("/internal") {
                let parts: Vec<_> = pointer.rsplitn(2, '/').collect();
                value.pointer_mut(parts[1]).unwrap()[parts[0]] = replacement;
            } else {
                *value.pointer_mut(pointer).unwrap() = replacement;
            }
            assert!(!classified(value), "{pointer}");
        }
        let mut value = shape();
        value["ledger_fields"][0]["index"] = json!(16);
        value["ledger_fields"][0]["path"] = json!([16]);
        value["stateful_circuits"][0]["return_value"]["index"] = json!(16);
        assert!(!classified(value));
    }

    #[test]
    fn only_the_exact_uint64_result_maximum_is_admitted() {
        assert!(classified(shape()));
        for max in [
            "0",
            "1",
            "2",
            "18446744073709551614",
            "18446744073709551616",
            "36893488147419103231",
            "018446744073709551615",
            "-1",
            "bad",
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["result"]["max"] = json!(max);
            assert!(!classified(value), "{max}");
        }
    }

    #[test]
    fn witnesses_calls_casts_branches_and_other_queries_are_not_size_reads() {
        for expression in [
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"unsigned_cast","max":"18446744073709551615","value":{"kind":"map_size","field":"c","index":0}}),
            json!({"kind":"coerce","ty":{"kind":"unsigned","max":"18446744073709551615"},"value":{"kind":"map_size","field":"c","index":0}}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"map_size","field":"c","index":0},"otherwise":{"kind":"map_size","field":"c","index":0}}),
            json!({"kind":"map_is_empty","field":"c","index":0}),
            json!({"kind":"map_lookup","field":"c","index":0,"key":{"kind":"field_literal","value":"0"}}),
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["return_value"] =
                json!({"kind":"expression","value":expression});
            assert!(!classified(value));
        }
    }
}
