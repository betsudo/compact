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

//! Capture exactly one root Set<Field> cardinality read; never evaluate a query.
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
        StateReturn::SetSize { field, index }
        | StateReturn::Expression {
            value: Expr::SetSize { field, index },
        } => (field, index),
        _ => return false,
    };
    super::root_field_set_slot(field, *index, fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn shape() -> Value {
        let mut v: Value = serde_json::from_str(include_str!(
            "../../../fixtures/upstream-672/export-set-field/compact-rust-ir.json"
        ))
        .unwrap();
        let read = v["stateful_circuits"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "size")
            .unwrap()
            .clone();
        v["stateful_circuits"] = json!([read]); // Only the private classifier isolates a circuit.
        v
    }
    fn classified(v: Value) -> bool {
        let contract: crate::ir::Contract = serde_json::from_value(v).unwrap();
        let fields = contract
            .ledger_fields
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();
        supported(&contract.stateful_circuits[0], &fields)
    }
    #[test]
    fn direct_and_exact_expression_accept_renaming_and_root_boundaries() {
        for index in [0, 15] {
            for path in [vec![], vec![index]] {
                let mut v = shape();
                v["ledger_fields"][0]["id"] = json!("renamed");
                v["ledger_fields"][0]["index"] = json!(index);
                v["ledger_fields"][0]["path"] = json!(path);
                v["stateful_circuits"][0]["name"] = json!("cardinality");
                v["stateful_circuits"][0]["return_value"]["field"] = json!("renamed");
                v["stateful_circuits"][0]["return_value"]["index"] = json!(index);
                assert!(classified(v.clone()));
                let read = v["stateful_circuits"][0]["return_value"].take();
                v["stateful_circuits"][0]["return_value"] =
                    json!({"kind":"expression","value":read});
                assert!(classified(v));
            }
        }
    }
    #[test]
    fn only_the_exact_uint64_result_signature_is_admitted() {
        for result in [json!({"kind":"boolean"}), json!({"kind":"field"})] {
            let mut v = shape();
            v["stateful_circuits"][0]["result"] = result;
            assert!(!classified(v));
        }
        for max in [
            "0",
            "1",
            "18446744073709551614",
            "18446744073709551616",
            "018446744073709551615",
        ] {
            let mut v = shape();
            v["stateful_circuits"][0]["result"]["max"] = json!(max);
            assert!(!classified(v), "{max}");
        }
        let mut v = shape();
        v["stateful_circuits"][0]["parameters"] = json!([{"name":"unused","ty":{"kind":"field"}}]);
        assert!(!classified(v));
    }
    #[test]
    fn physical_identity_type_visibility_and_effects_refuse() {
        for (pointer, replacement) in [
            ("/ledger_fields/0/id", json!("other")),
            ("/ledger_fields/0/index", json!(1)),
            ("/ledger_fields/0/path", json!([1])),
            ("/ledger_fields/0/path", json!([0, 1])),
            (
                "/ledger_fields/0/declaration",
                json!({"kind":"map","key":{"kind":"field"},"value":{"kind":"field"}}),
            ),
            ("/ledger_fields/0/declaration/ty", json!({"kind":"boolean"})),
            ("/stateful_circuits/0/return_value/field", json!("missing")),
            (
                "/stateful_circuits/0/actions",
                json!([{"kind":"set_reset","field":"c","index":0}]),
            ),
        ] {
            let mut v = shape();
            *v.pointer_mut(pointer).unwrap() = replacement;
            assert!(!classified(v), "{pointer}");
        }
        let mut v = shape();
        v["stateful_circuits"][0]["internal"] = json!(true);
        assert!(!classified(v));
        let mut v = shape();
        v["ledger_fields"][0]["index"] = json!(16);
        v["ledger_fields"][0]["path"] = json!([16]);
        v["stateful_circuits"][0]["return_value"]["index"] = json!(16);
        assert!(!classified(v));
        let contract: crate::ir::Contract = serde_json::from_value(shape()).unwrap();
        let mut declaration = contract.ledger_fields[0].clone();
        declaration.id = "other".into();
        assert!(!supported(
            &contract.stateful_circuits[0],
            &HashMap::from([("c", &declaration)])
        ));
    }
    #[test]
    fn broader_returns_are_not_cardinality_captures() {
        for expr in [
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"unsigned_cast","max":"18446744073709551615","value":{"kind":"set_size","field":"c","index":0}}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"set_size","field":"c","index":0},"otherwise":{"kind":"set_size","field":"c","index":0}}),
            json!({"kind":"map_size","field":"c","index":0}),
        ] {
            let mut v = shape();
            v["stateful_circuits"][0]["return_value"] = json!({"kind":"expression","value":expr});
            assert!(!classified(v));
        }
    }
}
