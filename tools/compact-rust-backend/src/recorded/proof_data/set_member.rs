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

//! One root Set<Field> membership query with its exact typed Field parameter.
use crate::ir::{Expr, LedgerField, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;

pub(super) fn supported(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    if circuit.internal
        || !circuit.actions.is_empty()
        || circuit.result != Type::Boolean
        || circuit.parameters.len() != 1
        || circuit.parameters[0].name.is_empty()
        || circuit.parameters[0].ty != Type::Field
    {
        return false;
    }
    let (field, index, value) = match &circuit.return_value {
        StateReturn::SetMember {
            field,
            index,
            value,
        } => (field, index, value),
        StateReturn::Expression {
            value:
                Expr::SetMember {
                    field,
                    index,
                    value,
                },
        } => (field, index, value.as_ref()),
        _ => return false,
    };
    super::root_field_set_slot(field, *index, fields)
        && matches!(value, Expr::Parameter { name } if *name == circuit.parameters[0].name)
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
            .find(|c| c["name"] == "member")
            .unwrap()
            .clone();
        v["stateful_circuits"] = json!([read]); // Private classifier only; generator tests retain six exports.
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
    fn direct_expression_and_renamed_parameter_accept_both_root_encodings() {
        for index in [0, 15] {
            for path in [vec![], vec![index]] {
                let mut v = shape();
                v["ledger_fields"][0]["id"] = json!("renamed");
                v["ledger_fields"][0]["index"] = json!(index);
                v["ledger_fields"][0]["path"] = json!(path);
                let c = &mut v["stateful_circuits"][0];
                c["name"] = json!("contains");
                c["parameters"][0]["name"] = json!("needle");
                c["return_value"]["field"] = json!("renamed");
                c["return_value"]["index"] = json!(index);
                c["return_value"]["value"]["name"] = json!("needle");
                assert!(classified(v.clone()));
                let read = v["stateful_circuits"][0]["return_value"].take();
                v["stateful_circuits"][0]["return_value"] =
                    json!({"kind":"expression","value":read});
                assert!(classified(v));
            }
        }
    }
    #[test]
    fn field_parameter_identity_and_boolean_arity_are_exact() {
        for (pointer, replacement) in [
            ("/stateful_circuits/0/parameters", json!([])),
            (
                "/stateful_circuits/0/parameters",
                json!([{"name":"value","ty":{"kind":"field"}},{"name":"extra","ty":{"kind":"field"}}]),
            ),
            ("/stateful_circuits/0/parameters/0/name", json!("")),
            (
                "/stateful_circuits/0/parameters/0/ty",
                json!({"kind":"boolean"}),
            ),
            ("/stateful_circuits/0/result", json!({"kind":"field"})),
            (
                "/stateful_circuits/0/return_value/value/name",
                json!("foreign"),
            ),
        ] {
            let mut v = shape();
            *v.pointer_mut(pointer).unwrap() = replacement;
            assert!(!classified(v), "{pointer}");
        }
    }
    #[test]
    fn physical_slot_type_visibility_and_effects_refuse() {
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
    }
    #[test]
    fn declared_hash_key_identity_is_required() {
        let contract: crate::ir::Contract = serde_json::from_value(shape()).unwrap();
        let mut field = contract.ledger_fields[0].clone();
        field.id = "other".into();
        assert!(!supported(
            &contract.stateful_circuits[0],
            &HashMap::from([("c", &field)])
        ));
        assert!(!supported(&contract.stateful_circuits[0], &HashMap::new()));
    }
    #[test]
    fn values_are_not_reconstructed_from_literals_calls_casts_or_branches() {
        for value in [
            json!({"kind":"field_literal","value":"0"}),
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"coerce","ty":{"kind":"field"},"value":{"kind":"parameter","name":"value"}}),
            json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"parameter","name":"value"},"otherwise":{"kind":"field_literal","value":"1"}}),
        ] {
            let mut v = shape();
            v["stateful_circuits"][0]["return_value"]["value"] = value;
            assert!(!classified(v));
        }
        let mut v = shape();
        v["stateful_circuits"][0]["return_value"] = json!({"kind":"map_member","field":"c","index":0,"key":{"kind":"parameter","name":"value"}});
        assert!(!classified(v));
    }
}
