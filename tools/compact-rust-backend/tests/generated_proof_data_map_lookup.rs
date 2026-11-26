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

//! Optional lookup admission on the intact eight-export original Map fixture.
//! These are generator tests; actual missing-cell execution/proofs are separate.
use compact_rust_backend::{ir::Contract, render, render_with_standalone_proof_data};
use quote::ToTokens;
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/export-map-field-field/compact-rust-ir.json"
    ))
    .unwrap()
}
fn lookup(value: &mut Value) -> &mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "lookup")
        .unwrap()
}
fn single() -> Value {
    let read = lookup(&mut original()).clone();
    let mut value = original();
    value["stateful_circuits"] = json!([read]);
    value
}
fn recorded_methods(source: &str, owner: &str) -> Vec<syn::ImplItemFn> {
    fn module(items: &[syn::Item], name: &str) -> Vec<syn::Item> {
        items
            .iter()
            .find_map(|i| match i {
                syn::Item::Mod(m) if m.ident == name => {
                    m.content.as_ref().map(|(_, items)| items.clone())
                }
                _ => None,
            })
            .unwrap()
    }
    let file = syn::parse_file(source).unwrap();
    let items = module(&module(&file.items, "ledger_contract"), "recorded");
    items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Impl(i) => {
                let syn::Type::Path(path) = i.self_ty.as_ref() else {
                    return None;
                };
                (path.path.segments.last().unwrap().ident == owner).then_some(i.items)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(method),
            _ => None,
        })
        .collect()
}
fn assert_capture_signature_and_order(method: &syn::ImplItemFn, name: &str, key: &str) {
    assert_eq!(method.sig.inputs.len(), 3);
    assert!(method.sig.generics.where_clause.is_none()); // no witnesses or Private: Clone
    let result = method.sig.output.to_token_stream().to_string();
    assert!(
        result.contains("RecordedProofData < Private , runtime :: Field >"),
        "{result}"
    );
    let body = method.block.to_token_stream().to_string();
    let input = format!("AlignedValue :: from ({key})");
    let call = format!("self . {name} (context , {key}) ?");
    assert_eq!(body.matches(&input).count(), 1);
    assert_eq!(body.matches(&call).count(), 1);
    assert_eq!(
        body.matches("RecordedProofData :: from_recorded").count(),
        1
    );
    assert!(body.find(&input).unwrap() < body.find(&call).unwrap());
    assert!(body.find(&call).unwrap() < body.find("RecordedProofData :: from_recorded").unwrap());
    assert!(body.contains(&format!("from_recorded (\"{name}\"")));
    assert!(body.contains("API_VERSION as usize"));
    assert!(!body.contains("lookup_map") && !body.contains("witness") && !body.contains("hash"));
    // The underlying `?` precedes construction. Missing lookup is propagated;
    // the facade has no fallback value or successful carrier on that path.
    let recorded = method
        .block
        .stmts
        .iter()
        .find_map(|stmt| match stmt {
            syn::Stmt::Local(local) if local.pat.to_token_stream().to_string() == "recorded" => {
                local.init.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert!(matches!(recorded.expr.as_ref(), syn::Expr::Try(_)));
}

#[test]
fn full_original_keeps_eight_apis_and_three_opt_in_map_read_families() {
    let value = original();
    assert_eq!(value["stateful_circuits"].as_array().unwrap().len(), 8);
    let contract: Contract = serde_json::from_value(value).unwrap();
    let plain = render(&contract).unwrap();
    let out = render_with_standalone_proof_data(&contract).unwrap();
    assert_eq!(out.capabilities.circuits.len(), 8);
    let methods = recorded_methods(&out.source, "Contract");
    for row in &out.capabilities.circuits {
        assert!(row.recorded && row.observed_call, "{}", row.name);
        assert_eq!(
            row.standalone_proof_data.is_some(),
            matches!(row.name.as_str(), "is_empty" | "member" | "lookup")
        );
        assert!(methods.iter().any(|m| m.sig.ident == row.name));
        assert!(
            methods
                .iter()
                .any(|m| m.sig.ident == format!("{}_call", row.name))
        );
    }
    assert!(
        recorded_methods(&plain, "Contract").iter().all(|m| !m
            .sig
            .ident
            .to_string()
            .ends_with("_proof_data"))
    );
    let lookup = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "lookup")
        .unwrap();
    assert_eq!(
        lookup.standalone_proof_data.as_ref().unwrap().api_version,
        1
    );
    assert_eq!(
        lookup.standalone_proof_data.as_ref().unwrap().method,
        "lookup_proof_data"
    );
    assert_capture_signature_and_order(
        methods
            .iter()
            .find(|m| m.sig.ident == "lookup_proof_data")
            .unwrap(),
        "lookup",
        "key",
    );
    assert!(
        !recorded_methods(&out.source, "BorrowedContract")
            .iter()
            .any(|m| m.sig.ident == "lookup_proof_data")
    );
}

#[test]
fn borrowed_and_renamed_expression_reads_capture_the_actual_declared_field_once() {
    let mut value = single();
    value["ledger_fields"][0]["id"] = json!("renamed");
    let c = lookup(&mut value);
    c["name"] = json!("read_item");
    c["parameters"][0]["name"] = json!("candidate");
    c["return_value"]["field"] = json!("renamed");
    c["return_value"]["key"]["name"] = json!("candidate");
    let read = c["return_value"].take();
    c["return_value"] = json!({"kind":"expression","value":read});
    value["witnesses"] = json!([{"name":"ask","parameters":[],"result":{"kind":"field"}}]);
    value["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"witnessed","parameters":[],"actions":[],"result":{"kind":"field"},
        "return_value":{"kind":"expression","value":{"kind":"witness_call","name":"ask","arguments":[]}}}));
    let out = render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
    let methods = recorded_methods(&out.source, "BorrowedContract");
    let method = methods
        .iter()
        .find(|m| m.sig.ident == "read_item_proof_data")
        .unwrap();
    assert_capture_signature_and_order(method, "read_item", "candidate");
}

#[test]
fn existing_nonfamily_lookup_recording_stays_available_without_optional_helper() {
    for edit in ["boolean-value", "literal-key", "preceding-write"] {
        let mut value = single();
        match edit {
            "boolean-value" => {
                value["ledger_fields"][0]["declaration"]["value"] = json!({"kind":"boolean"});
                lookup(&mut value)["result"] = json!({"kind":"boolean"});
            }
            "literal-key" => {
                lookup(&mut value)["return_value"]["key"] =
                    json!({"kind":"field_literal","value":"1"})
            }
            "preceding-write" => {
                lookup(&mut value)["actions"] = json!([{"kind":"map_reset","field":"c","index":0}])
            }
            _ => unreachable!(),
        }
        let out =
            render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
        let row = &out.capabilities.circuits[0];
        assert!(row.recorded && row.observed_call, "{edit}");
        assert!(row.standalone_proof_data.is_none(), "{edit}");
        assert!(
            !recorded_methods(&out.source, "Contract")
                .iter()
                .any(|m| m.sig.ident == "lookup_proof_data")
        );
    }
}

#[test]
fn public_and_internal_name_collisions_preserve_the_original_lookup_call() {
    for internal in [false, true] {
        let mut value = single();
        value["stateful_circuits"].as_array_mut().unwrap().push(json!({
            "name":"lookup_proof_data","internal":internal,"parameters":[],"actions":[],
            "result":{"kind":"field"},"return_value":{"kind":"expression","value":{"kind":"field_literal","value":"0"}}}));
        let out =
            render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "lookup")
            .unwrap();
        assert!(row.recorded && row.observed_call && row.standalone_proof_data.is_none());
        assert_eq!(
            row.standalone_proof_data_unavailable.as_ref().unwrap().code,
            "name_collision"
        );
        let methods = recorded_methods(&out.source, "Contract");
        assert!(methods.iter().any(|m| m.sig.ident == "lookup"));
        assert!(methods.iter().any(|m| m.sig.ident == "lookup_call"));
    }
}
