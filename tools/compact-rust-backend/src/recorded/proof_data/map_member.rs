// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! One root Map<Field, Field> membership read with its typed public Field key.
//! Classification inspects a fixed shape; it never walks helper bodies or
//! evaluates/reconstructs a key, ledger operation, witness or transcript.
use crate::ir::{Expr, LedgerField, LedgerFieldKind, StateReturn, StatefulCircuit, Type};
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
    let (field, index, key) = match &circuit.return_value {
        StateReturn::MapMember { field, index, key } => (field, index, key),
        StateReturn::Expression {
            value: Expr::MapMember { field, index, key },
        } => (field, index, key.as_ref()),
        _ => return false,
    };
    let Some(declaration) = fields.get(field.as_str()) else {
        return false;
    };
    // The renderer validates all paths globally. This optional family admits
    // only the root Map slot, including its two equivalent path encodings.
    declaration.id == *field
        && declaration.index == *index
        && *index < 16
        && declaration.path.len() <= 1
        && declaration.physical_path() == vec![*index]
        && declaration.declaration
            == (LedgerFieldKind::Map {
                key: Type::Field,
                value: Type::Field,
            })
        && matches!(key, Expr::Parameter { name } if *name == circuit.parameters[0].name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn original_shape() -> Value {
        json!({"schema_version":20,"ledger_fields":[{"id":"c","index":0,
            "declaration":{"kind":"map","key":{"kind":"field"},"value":{"kind":"field"}}}],
            "circuits":[],"stateful_circuits":[{"name":"member",
            "parameters":[{"name":"key","ty":{"kind":"field"}}],
            "actions":[],"result":{"kind":"boolean"},"return_value":{
                "kind":"map_member","field":"c","index":0,
                "key":{"kind":"parameter","name":"key"}}}]})
    }
    fn classified(value: &Value) -> bool {
        let c: crate::ir::Contract = serde_json::from_value(value.clone()).unwrap();
        let fields = c.ledger_fields.iter().map(|f| (f.id.as_str(), f)).collect();
        supported(&c.stateful_circuits[0], &fields)
    }

    #[test]
    fn fixed_member_shape_accepts_equivalent_root_paths_and_expression_return() {
        let mut v = original_shape();
        assert!(classified(&v));
        v["ledger_fields"][0]["path"] = json!([0]);
        assert!(classified(&v));
        let r = v["stateful_circuits"][0]["return_value"].take();
        v["stateful_circuits"][0]["return_value"] = json!({"kind":"expression","value":r});
        assert!(classified(&v));
    }

    #[test]
    fn optional_classifier_refuses_malformed_slots_paths_keys_arity_and_actions_itself() {
        // Exercise this classifier directly: earlier renderer refusals cannot
        // masquerade as coverage of its structural boundary.
        let cases: &[(&str, Value)] = &[
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
            ("/stateful_circuits/0/result", json!({"kind":"field"})),
            ("/stateful_circuits/0/parameters", json!([])),
            (
                "/stateful_circuits/0/parameters",
                json!([
                {"name":"key","ty":{"kind":"field"}},
                {"name":"extra","ty":{"kind":"field"}}]),
            ),
            (
                "/stateful_circuits/0/parameters/0/ty",
                json!({"kind":"boolean"}),
            ),
            ("/stateful_circuits/0/return_value/field", json!("unknown")),
            ("/stateful_circuits/0/return_value/index", json!(1)),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"parameter","name":"unknown"}),
            ),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"field_literal","value":"1"}),
            ),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"call","name":"key_helper","arguments":[]}),
            ),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"witness_call","name":"ask","arguments":[]}),
            ),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"vector_fold","accumulator":{"name":"acc","ty":{"kind":"field"}},"items":[{"name":"item","ty":{"kind":"field"}}],"initial":{"kind":"parameter","name":"key"},"sources":[{"kind":"vector","element":{"kind":"field"},"elements":[]}],"body":{"kind":"value","value":{"kind":"parameter","name":"acc"}},"length":0}),
            ),
            (
                "/stateful_circuits/0/return_value/key",
                json!({"kind":"coerce","ty":{"kind":"field"},"value":{"kind":"parameter","name":"key"}}),
            ),
            (
                "/stateful_circuits/0/actions",
                json!([{"kind":"map_reset","field":"c","index":0}]),
            ),
        ];
        for (pointer, replacement) in cases {
            let mut v = original_shape();
            if *pointer == "/stateful_circuits/0/internal" || *pointer == "/ledger_fields/0/path" {
                if *pointer == "/stateful_circuits/0/internal" {
                    v["stateful_circuits"][0]["internal"] = replacement.clone();
                } else {
                    v["ledger_fields"][0]["path"] = replacement.clone();
                }
            } else {
                *v.pointer_mut(pointer).unwrap() = replacement.clone();
            }
            assert!(!classified(&v), "{pointer}: {replacement}");
        }
        let mut v = original_shape();
        v["ledger_fields"][0]["index"] = json!(16);
        v["stateful_circuits"][0]["return_value"]["index"] = json!(16);
        assert!(!classified(&v));
    }
}
