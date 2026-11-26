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

//! One root Map<Field, Field> lookup with its declared public Field key.
//! The actual recorded call owns the value and any missing-cell refusal;
//! admission neither reads the map nor reconstructs the read transcript.
use crate::ir::{Expr, LedgerField, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;

pub(super) fn supported(circuit: &StatefulCircuit, fields: &HashMap<&str, &LedgerField>) -> bool {
    if circuit.internal
        || !circuit.actions.is_empty()
        || circuit.result != Type::Field
        || circuit.parameters.len() != 1
        || circuit.parameters[0].name.is_empty()
        || circuit.parameters[0].ty != Type::Field
    {
        return false;
    }
    let (field, index, key) = match &circuit.return_value {
        StateReturn::MapLookup { field, index, key } => (field, index, key),
        StateReturn::Expression {
            value: Expr::MapLookup { field, index, key },
        } => (field, index, key.as_ref()),
        _ => return false,
    };
    super::root_field_map_slot(field, *index, fields)
        && matches!(key, Expr::Parameter { name } if *name == circuit.parameters[0].name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn shape() -> Value {
        json!({"schema_version":20,"ledger_fields":[{"id":"c","index":0,
            "declaration":{"kind":"map","key":{"kind":"field"},"value":{"kind":"field"}}}],
            "circuits":[],"stateful_circuits":[{"name":"lookup",
                "parameters":[{"name":"key","ty":{"kind":"field"}}],
                "actions":[],"result":{"kind":"field"},"return_value":{
                    "kind":"map_lookup","field":"c","index":0,
                    "key":{"kind":"parameter","name":"key"}}}]})
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
    fn direct_field_lookup_accepts_only_equivalent_root_encodings() {
        let mut value = shape();
        assert!(classified(value.clone()));
        value["ledger_fields"][0]["path"] = json!([0]);
        assert!(classified(value.clone()));
        let read = value["stateful_circuits"][0]["return_value"].take();
        value["stateful_circuits"][0]["return_value"] = json!({"kind":"expression","value":read});
        assert!(classified(value));
    }

    #[test]
    fn classification_is_semantic_and_checks_its_own_slot_type_and_return_boundary() {
        let mut value = shape();
        value["ledger_fields"][0]["id"] = json!("renamed");
        value["ledger_fields"][0]["index"] = json!(15);
        let c = &mut value["stateful_circuits"][0];
        c["name"] = json!("read_item");
        c["parameters"][0]["name"] = json!("candidate");
        c["return_value"]["field"] = json!("renamed");
        c["return_value"]["index"] = json!(15);
        c["return_value"]["key"]["name"] = json!("candidate");
        assert!(classified(value));

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
            ("/stateful_circuits/0/internal", json!(true)),
            ("/stateful_circuits/0/result", json!({"kind":"boolean"})),
            ("/stateful_circuits/0/parameters", json!([])),
            ("/stateful_circuits/0/parameters/0/name", json!("")),
            (
                "/stateful_circuits/0/parameters/0/ty",
                json!({"kind":"boolean"}),
            ),
            ("/stateful_circuits/0/return_value/field", json!("unknown")),
            ("/stateful_circuits/0/return_value/index", json!(1)),
            (
                "/stateful_circuits/0/return_value/kind",
                json!("map_member"),
            ),
            (
                "/stateful_circuits/0/actions",
                json!([{"kind":"map_reset","field":"c","index":0}]),
            ),
        ] {
            let mut value = shape();
            // These optional members are absent in the original shape.
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
        let mut value = shape();
        value["stateful_circuits"][0]["parameters"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"extra","ty":{"kind":"field"}}));
        assert!(!classified(value));
    }

    #[test]
    fn keys_never_admit_literals_calls_witnesses_coercions_or_folds() {
        for key in [
            json!({"kind":"parameter","name":"other"}),
            json!({"kind":"field_literal","value":"1"}),
            json!({"kind":"call","name":"helper","arguments":[]}),
            json!({"kind":"witness_call","name":"ask","arguments":[]}),
            json!({"kind":"coerce","ty":{"kind":"field"},"value":{"kind":"parameter","name":"key"}}),
            json!({"kind":"vector_fold","accumulator":{"name":"acc","ty":{"kind":"field"}},"items":[{"name":"item","ty":{"kind":"field"}}],"initial":{"kind":"parameter","name":"key"},"sources":[{"kind":"vector","element":{"kind":"field"},"elements":[]}],"body":{"kind":"value","value":{"kind":"parameter","name":"acc"}},"length":0}),
        ] {
            let mut value = shape();
            value["stateful_circuits"][0]["return_value"]["key"] = key;
            assert!(!classified(value));
        }
    }
}
