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

//! Complete original Set export generator checks; no contract is executed here.
use compact_rust_backend::{ir::Contract, render, render_with_standalone_proof_data};
use quote::ToTokens;
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/export-set-field/compact-rust-ir.json"
    ))
    .unwrap()
}
fn selected(value: &mut Value) -> &mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "is_empty" || c["name"] == "vacant")
        .unwrap()
}
fn methods(source: &str, owner: &str) -> Vec<syn::ImplItemFn> {
    fn child(items: &[syn::Item], name: &str) -> Vec<syn::Item> {
        items
            .iter()
            .find_map(|item| match item {
                syn::Item::Mod(module) if module.ident == name => {
                    module.content.as_ref().map(|(_, items)| items.clone())
                }
                _ => None,
            })
            .unwrap()
    }
    let syntax = syn::parse_file(source).unwrap();
    child(&child(&syntax.items, "ledger_contract"), "recorded").into_iter()
        .filter_map(|item| match item {
            syn::Item::Impl(i) if matches!(i.self_ty.as_ref(), syn::Type::Path(p) if p.path.segments.last().unwrap().ident == owner) => Some(i.items),
            _ => None,
        }).flatten().filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(method), _ => None,
        }).collect()
}
fn assert_capture(method: &syn::ImplItemFn, name: &str) {
    assert_eq!(method.sig.ident, format!("{name}_proof_data"));
    assert_eq!(method.sig.inputs.len(), 2);
    assert!(method.sig.generics.where_clause.is_none());
    assert!(
        method
            .sig
            .generics
            .type_params()
            .all(|p| p.bounds.is_empty())
    );
    assert!(
        method
            .sig
            .output
            .to_token_stream()
            .to_string()
            .contains("RecordedProofData < Private , bool >")
    );
    assert_eq!(method.block.stmts.len(), 4);
    let syn::Stmt::Item(syn::Item::Const(guard)) = &method.block.stmts[0] else {
        panic!("API guard")
    };
    assert!(
        guard
            .to_token_stream()
            .to_string()
            .contains("API_VERSION as usize")
    );
    let syn::Stmt::Local(input) = &method.block.stmts[1] else {
        panic!("typed Unit input")
    };
    let syn::Expr::Call(aligned) = input.init.as_ref().unwrap().expr.as_ref() else {
        panic!("Unit conversion")
    };
    assert_eq!(
        aligned.func.to_token_stream().to_string(),
        "runtime :: fab :: AlignedValue :: from"
    );
    assert_eq!(aligned.args.len(), 1);
    assert!(matches!(&aligned.args[0], syn::Expr::Tuple(unit) if unit.elems.is_empty()));
    let syn::Stmt::Local(recorded) = &method.block.stmts[2] else {
        panic!("actual invocation")
    };
    let syn::Expr::Try(call) = recorded.init.as_ref().unwrap().expr.as_ref() else {
        panic!("propagate error first")
    };
    let syn::Expr::MethodCall(call) = call.expr.as_ref() else {
        panic!("actual self call")
    };
    assert_eq!(call.method, name);
    assert_eq!(call.receiver.to_token_stream().to_string(), "self");
    assert_eq!(call.args.len(), 1);
    assert_eq!(call.args[0].to_token_stream().to_string(), "context");
    let syn::Stmt::Expr(syn::Expr::Call(carrier), None) = method.block.stmts.last().unwrap() else {
        panic!("returned carrier")
    };
    assert_eq!(
        carrier.func.to_token_stream().to_string(),
        "runtime :: proof_data :: RecordedProofData :: from_recorded"
    );
    assert_eq!(carrier.args.len(), 3);
    assert!(
        matches!(&carrier.args[0], syn::Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(s) if s.value() == name))
    );
    assert_eq!(carrier.args[1].to_token_stream().to_string(), "input");
    assert_eq!(carrier.args[2].to_token_stream().to_string(), "recorded");
    assert_eq!(
        method
            .block
            .to_token_stream()
            .to_string()
            .matches(&format!("self . {name} ("))
            .count(),
        1
    );
}

#[test]
fn intact_six_exports_gain_only_the_three_opt_in_set_read_helpers() {
    let contract: Contract = serde_json::from_value(original()).unwrap();
    assert_eq!(contract.stateful_circuits.len(), 6);
    let plain = render(&contract).unwrap();
    let out = render_with_standalone_proof_data(&contract).unwrap();
    let generated = methods(&out.source, "Contract");
    let before = methods(&plain, "Contract");
    assert_eq!(out.capabilities.circuits.len(), 6);
    for row in &out.capabilities.circuits {
        assert!(row.recorded && row.observed_call);
        assert_eq!(
            row.standalone_proof_data.is_some(),
            matches!(row.name.as_str(), "is_empty" | "member" | "size")
        );
        assert!(generated.iter().any(|m| m.sig.ident == row.name));
        assert!(
            generated
                .iter()
                .any(|m| m.sig.ident == format!("{}_call", row.name))
        );
    }
    assert!(
        before
            .iter()
            .all(|m| !m.sig.ident.to_string().ends_with("_proof_data"))
    );
    assert_eq!(
        generated
            .iter()
            .filter(|m| !m.sig.ident.to_string().ends_with("_proof_data"))
            .map(|m| m.to_token_stream().to_string())
            .collect::<Vec<_>>(),
        before
            .iter()
            .map(|m| m.to_token_stream().to_string())
            .collect::<Vec<_>>()
    );
    let api = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "is_empty")
        .unwrap()
        .standalone_proof_data
        .as_ref()
        .unwrap();
    assert_eq!(api.api_version, 1);
    assert_eq!(api.method, "is_empty_proof_data");
    assert_capture(
        generated
            .iter()
            .find(|m| m.sig.ident == api.method)
            .unwrap(),
        "is_empty",
    );
    // The actual original has no borrowed handle. Do not assume one exists.
    assert!(methods(&out.source, "BorrowedContract").is_empty());
    assert!(out.source.contains("RUST_RUNTIME_ABI == 54"));
}

#[test]
fn unrelated_witness_adds_a_borrowed_handle_without_changing_query_bounds() {
    let mut value = original();
    value["ledger_fields"][0]["id"] = json!("renamed");
    for c in value["stateful_circuits"].as_array_mut().unwrap() {
        // All six circuits remain; rename slot uses throughout their existing IR.
        fn rename(v: &mut Value) {
            match v {
                Value::Object(o) => {
                    if o.get("field") == Some(&json!("c")) {
                        o.insert("field".into(), json!("renamed"));
                    }
                    o.values_mut().for_each(rename);
                }
                Value::Array(a) => a.iter_mut().for_each(rename),
                _ => (),
            }
        }
        rename(c);
    }
    let read = selected(&mut value)["return_value"].take();
    selected(&mut value)["name"] = json!("vacant");
    selected(&mut value)["return_value"] = json!({"kind":"expression","value":read});
    value["witnesses"] = json!([{"name":"ask","parameters":[],"result":{"kind":"field"}}]);
    value["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"witnessed","parameters":[],"actions":[],"result":{"kind":"field"},
        "return_value":{"kind":"expression","value":{"kind":"witness_call","name":"ask","arguments":[]}}}));
    let contract = serde_json::from_value(value).unwrap();
    let out = render_with_standalone_proof_data(&contract).unwrap();
    for owner in ["Contract", "BorrowedContract"] {
        let methods = methods(&out.source, owner);
        assert_capture(
            methods
                .iter()
                .find(|m| m.sig.ident == "vacant_proof_data")
                .unwrap(),
            "vacant",
        );
    }
}

#[test]
fn broader_valid_recordings_keep_ordinary_apis_but_not_the_new_family() {
    for edit in ["boolean-element", "preceding-write", "parameter", "branch"] {
        let mut value = original();
        match edit {
            "boolean-element" => {
                // Other exports contain Field mutations: keep the full original
                // declaration and test a separate valid Boolean Set read instead.
                value["ledger_fields"].as_array_mut().unwrap().push(json!({"id":"flags","index":1,"path":[1],"declaration":{"kind":"set","ty":{"kind":"boolean"}}}));
                selected(&mut value)["return_value"] =
                    json!({"kind":"set_is_empty","field":"flags","index":1});
            }
            "preceding-write" => {
                selected(&mut value)["actions"] = json!([{"kind":"set_insert","field":"c","index":0,"value":{"kind":"field_literal","value":"1"}}])
            }
            "parameter" => {
                selected(&mut value)["parameters"] =
                    json!([{"name":"unused","ty":{"kind":"field"}}])
            }
            "branch" => {
                selected(&mut value)["return_value"] = json!({"kind":"expression","value":{"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"set_is_empty","field":"c","index":0},"otherwise":{"kind":"boolean","value":false}}})
            }
            _ => unreachable!(),
        }
        let out =
            render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "is_empty")
            .unwrap();
        assert!(row.recorded && row.observed_call, "{edit}");
        assert!(row.standalone_proof_data.is_none(), "{edit}");
        assert!(
            !methods(&out.source, "Contract")
                .iter()
                .any(|m| m.sig.ident == "is_empty_proof_data")
        );
    }
}

#[test]
fn helper_name_collision_keeps_the_existing_recorded_method_available() {
    for (source_name, collision_name) in [
        ("is_empty", "is_empty_proof_data"),
        ("is$empty", "is_empty_proof_data"),
    ] {
        for internal in [false, true] {
            let mut value = original();
            selected(&mut value)["name"] = json!(source_name);
            value["stateful_circuits"].as_array_mut().unwrap().push(json!({"name":collision_name,"internal":internal,
            "parameters":[],"actions":[],"result":{"kind":"boolean"},"return_value":{"kind":"expression","value":{"kind":"boolean","value":true}}}));
            let out =
                render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
            let row = out
                .capabilities
                .circuits
                .iter()
                .find(|c| c.name == source_name)
                .unwrap();
            assert!(row.recorded && row.observed_call && row.standalone_proof_data.is_none());
            assert_eq!(
                row.standalone_proof_data_unavailable.as_ref().unwrap().code,
                "name_collision"
            );
            assert!(
                methods(&out.source, "Contract")
                    .iter()
                    .any(|m| m.sig.ident == "is_empty")
            );
        }
    }
}
