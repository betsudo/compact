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

//! Direct classifier boundaries, including shapes the renderer would reject.
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
fn set(v: &mut Value) -> &mut Value {
    v["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "set")
        .unwrap()
}
fn helper<'a>(v: &'a mut Value, collection: &str, name: &str) -> &'a mut Value {
    v[collection]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == name)
        .unwrap()
}
fn writes(v: &mut Value) -> &mut Value {
    &mut set(v)["actions"][0]["actions"][1]["action"]["action"]["actions"]
}

#[test]
fn genuine_set_admits_only_the_inspected_field_mutation_family() {
    assert!(admitted(original(), "set"));
    assert!(!admitted(original(), "clear"));
    assert!(!admitted(original(), "get"));
}

#[test]
fn signature_state_guard_private_witness_hash_and_ordered_write_boundaries_refuse() {
    type Edit = (&'static str, fn(&mut Value));
    let edits: &[Edit] = &[
        ("missing input", |v| set(v)["parameters"] = json!([])),
        ("extra input", |v| {
            set(v)["parameters"]
                .as_array_mut()
                .unwrap()
                .push(json!({"name":"x","ty":{"kind":"field"}}))
        }),
        ("non-Field input", |v| {
            set(v)["parameters"][0]["ty"] = json!({"kind":"bytes","length":32})
        }),
        ("non-Unit result", |v| {
            set(v)["result"] = json!({"kind":"field"})
        }),
        ("non-Unit return", |v| {
            set(v)["return_value"] = json!({"kind":"expression","value":{"kind":"unit"}})
        }),
        ("extra top action", |v| {
            set(v)["actions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"sequence","actions":[]}))
        }),
        ("missing state guard", |v| {
            set(v)["actions"][0]["actions"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        }),
        ("wrong guard state", |v| {
            set(v)["actions"][0]["actions"][0]["condition"]["arguments"][0]["value"]["variant"] =
                json!("set")
        }),
        ("guard constant", |v| {
            set(v)["actions"][0]["actions"][0]["condition"] = json!({"kind":"boolean","value":true})
        }),
        ("state helper writes", |v| {
            helper(v, "stateful_circuits", "in_state")["actions"] = json!([{"kind":"cell_write","field":"value","index":1,"value":{"kind":"field_literal","value":"0"}}])
        }),
        ("state helper constant", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"] =
                json!({"kind":"boolean","value":true})
        }),
        ("state helper wrong slot", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"]["left"]["index"] =
                json!(1)
        }),
        ("state helper recursive", |v| {
            helper(v, "stateful_circuits", "in_state")["return_value"]["value"] =
                json!({"kind":"call","name":"in_state","arguments":[]})
        }),
        ("state has third variant", |v| {
            v["ledger_fields"][2]["declaration"]["ty"]["variants"]
                .as_array_mut()
                .unwrap()
                .push(json!("third"))
        }),
        ("witness narrow declaration", |v| {
            v["witnesses"][0]["result"]["length"] = json!(31)
        }),
        ("witness call args", |v| {
            set(v)["actions"][0]["actions"][1]["bindings"][0]["value"]["arguments"] =
                json!([{"kind":"unit"}])
        }),
        ("witness declaration args", |v| {
            v["witnesses"][0]["parameters"] = json!([{"name":"x","ty":{"kind":"unit"}}])
        }),
        ("witness replaced by default", |v| {
            set(v)["actions"][0]["actions"][1]["bindings"][0]["value"] =
                json!({"kind":"default","ty":{"kind":"bytes","length":32}})
        }),
        ("second witness binding", |v| {
            let b = set(v)["actions"][0]["actions"][1]["bindings"][0].clone();
            set(v)["actions"][0]["actions"][1]["bindings"]
                .as_array_mut()
                .unwrap()
                .push(b);
        }),
        ("public input shadowed by witness", |v| {
            set(v)["parameters"][0]["name"] = json!("sk")
        }),
        ("public input shadowed by hash", |v| {
            set(v)["parameters"][0]["name"] = json!("apk")
        }),
        ("hash bypasses witness", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][1] =
                json!({"kind":"default","ty":{"kind":"bytes","length":32}})
        }),
        ("hash reads another witness", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][1] =
                json!({"kind":"witness_call","name":"private$secret_key","arguments":[]})
        }),
        ("hash prefix width", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"][0]["bytes"]
                .as_array_mut()
                .unwrap()
                .pop();
        }),
        ("hash result width", |v| {
            helper(v, "circuits", "public_key")["result"]["length"] = json!(31)
        }),
        ("hash extra tuple element", |v| {
            helper(v, "circuits", "public_key")["body"]["value"]["elements"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"unit"}))
        }),
        ("hash recursive", |v| {
            helper(v, "circuits", "public_key")["body"] =
                json!({"kind":"call","name":"public_key","arguments":[]})
        }),
        ("extra action", |v| {
            writes(v)
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"sequence","actions":[]}))
        }),
        ("missing write", |v| {
            writes(v).as_array_mut().unwrap().pop();
        }),
        ("write order", |v| {
            writes(v).as_array_mut().unwrap().swap(0, 1)
        }),
        ("authority leaks raw witness", |v| {
            writes(v)[0]["value"]["name"] = json!("sk")
        }),
        ("authority wrong slot", |v| writes(v)[0]["index"] = json!(1)),
        ("authority wrong width", |v| {
            v["ledger_fields"][0]["declaration"]["ty"]["length"] = json!(31)
        }),
        ("value discards public input", |v| {
            writes(v)[1]["value"] = json!({"kind":"field_literal","value":"0"})
        }),
        ("value uses raw witness", |v| {
            writes(v)[1]["value"]["name"] = json!("sk")
        }),
        ("value reads another Cell", |v| {
            writes(v)[1]["value"] = json!({"kind":"cell_read","field":"value","index":1})
        }),
        ("value wrong slot", |v| writes(v)[1]["index"] = json!(0)),
        ("wrong final state", |v| {
            writes(v)[2]["value"]["variant"] = json!("unset")
        }),
        ("state wrong slot", |v| writes(v)[2]["index"] = json!(1)),
    ];
    for (label, edit) in edits {
        let mut value = original();
        edit(&mut value);
        assert!(!admitted(value, "set"), "{label}");
    }
}

#[test]
fn public_value_identity_coercion_is_bounded_and_type_checked() {
    for count in [0, 1, 7, 8, 64] {
        let mut value = original();
        let arg = &mut writes(&mut value)[1]["value"];
        for _ in 0..count {
            *arg = json!({"kind":"coerce","ty":{"kind":"field"},"value":arg.clone()});
        }
        assert_eq!(admitted(value, "set"), count < 8, "{count}");
    }
    let mut value = original();
    writes(&mut value)[1]["value"] = json!({"kind":"coerce","ty":{"kind":"bytes","length":32},"value":{"kind":"parameter","name":"v"}});
    assert!(!admitted(value, "set"));
}
