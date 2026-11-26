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

//! Classifier refusals run independently of native renderer validation.
use super::*;
use crate::ir::Contract;
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/upstream-672/tiny-get/compact-rust-ir.json"
    ))
    .unwrap()
}

fn admitted(value: Value, name: &str) -> bool {
    let c: Contract = serde_json::from_value(value).unwrap();
    let fields = c.ledger_fields.iter().map(|f| (f.id.as_str(), f)).collect();
    let circuit = c.stateful_circuits.iter().find(|c| c.name == name).unwrap();
    supported(
        circuit,
        &c.stateful_circuits,
        &c.circuits,
        &fields,
        &c.witnesses,
    )
}

fn clear(value: &mut Value) -> &mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "clear")
        .unwrap()
}

fn helper<'a>(value: &'a mut Value, collection: &str, name: &str) -> &'a mut Value {
    value[collection]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == name)
        .unwrap()
}

#[test]
fn genuine_clear_only_admits_the_inspected_mutation_family() {
    assert!(admitted(original(), "clear"));
    assert!(!admitted(original(), "set"));
    assert!(!admitted(original(), "get")); // getter has a separate, unchanged classifier
}

#[test]
fn all_reachable_witness_hash_guard_write_and_signature_boundaries_are_checked() {
    type Edit = (&'static str, fn(&mut Value));
    let edits: &[Edit] = &[
        ("public parameter", |v| {
            clear(v)["parameters"] = json!([{"name":"x","ty":{"kind":"field"}}])
        }),
        ("non-Unit result", |v| {
            clear(v)["result"] = json!({"kind":"field"})
        }),
        ("non-Unit return", |v| {
            clear(v)["return_value"] = json!({"kind":"expression","value":{"kind":"unit"}})
        }),
        ("extra top action", |v| {
            clear(v)["actions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"sequence","actions":[]}))
        }),
        ("missing state guard", |v| {
            clear(v)["actions"][0]["actions"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        }),
        ("unset state guard", |v| {
            clear(v)["actions"][0]["actions"][0]["condition"]["arguments"][0]["value"]["variant"] =
                json!("unset")
        }),
        ("state helper extra write", |v| {
            helper(v, "stateful_circuits", "in_state")["actions"] = json!([{"kind":"cell_write","field":"value","index":1,"value":{"kind":"field_literal","value":"0"}}])
        }),
        ("state helper constant", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"] =
                json!({"kind":"boolean","value":true})
        }),
        ("state helper wrong field", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"]["left"]["field"] =
                json!("value")
        }),
        ("state helper wrong index", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"]["left"]["index"] =
                json!(1)
        }),
        ("state helper recursion", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"] =
                json!({"kind":"call","name":"in_state","arguments":[]})
        }),
        ("witness declaration width", |v| {
            v["witnesses"][0]["result"]["length"] = json!(31)
        }),
        ("witness arguments", |v| {
            clear(v)["actions"][0]["actions"][1]["bindings"][0]["value"]["arguments"] =
                json!([{"kind":"unit"}])
        }),
        ("witness declaration arguments", |v| {
            v["witnesses"][0]["parameters"] = json!([{"name":"x","ty":{"kind":"unit"}}])
        }),
        ("substitute private value", |v| {
            clear(v)["actions"][0]["actions"][1]["bindings"][0]["value"] =
                json!({"kind":"default","ty":{"kind":"bytes","length":32}})
        }),
        ("duplicate private binding", |v| {
            let b = clear(v)["actions"][0]["actions"][1]["bindings"][0].clone();
            clear(v)["actions"][0]["actions"][1]["bindings"]
                .as_array_mut()
                .unwrap()
                .push(b);
        }),
        ("hash bypasses witness", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][1] =
                json!({"kind":"default","ty":{"kind":"bytes","length":32}})
        }),
        ("hash changes type", |v| {
            helper(v, "circuits", "public_key")["result"]["length"] = json!(31)
        }),
        ("hash extra witness", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][1] =
                json!({"kind":"witness_call","name":"private$secret_key","arguments":[]})
        }),
        ("hash extra tuple element", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"unit"}))
        }),
        ("hash short prefix", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][0]["bytes"]
                .as_array_mut()
                .unwrap()
                .pop();
        }),
        ("hash helper recursion", |v| {
            helper(v, "circuits", "public_key")["body"] =
                json!({"kind":"call","name":"public_key","arguments":[]})
        }),
        ("missing authority guard", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        }),
        ("authority guard is constant", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][0]["condition"] =
                json!({"kind":"boolean","value":true})
        }),
        ("authority wrong slot", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][0]["condition"]["right"]
                ["index"] = json!(1)
        }),
        ("authority wrong type", |v| {
            v["ledger_fields"][0]["declaration"]["ty"]["length"] = json!(31)
        }),
        ("extra write", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"sequence","actions":[]}))
        }),
        ("nonzero value write", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][2]["value"]["value"] =
                json!("1")
        }),
        ("write different state", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][3]["value"]["variant"] =
                json!("set")
        }),
        ("wrong state write index", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][3]["index"] =
                json!(1)
        }),
        ("wrong zero authority", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"][1]["action"]["value"]
                ["name"] = json!("sk")
        }),
        ("write ordering", |v| {
            clear(v)["actions"][0]["actions"][1]["action"]["action"]["actions"]
                .as_array_mut()
                .unwrap()
                .swap(1, 2)
        }),
    ];
    for (label, edit) in edits {
        let mut value = original();
        edit(&mut value);
        assert!(!admitted(value, "clear"), "{label}");
    }
}

#[test]
fn identity_coercion_budget_refuses_oversized_or_nonidentity_wrappers() {
    for count in [0, 1, 7, 8, 64] {
        let mut value = original();
        let arg = &mut clear(&mut value)["actions"][0]["actions"][1]["action"]["bindings"][0]["value"]
            ["arguments"][0];
        for _ in 0..count {
            *arg = json!({"kind":"coerce","ty":{"kind":"bytes","length":32},"value":arg.clone()});
        }
        // Original already has one identity coercion.
        assert_eq!(admitted(value, "clear"), count < 7, "{count}");
    }
    let mut value = original();
    clear(&mut value)["actions"][0]["actions"][1]["action"]["bindings"][0]["value"]["arguments"]
        [0]["ty"]["length"] = json!(31);
    assert!(!admitted(value, "clear"));
}
