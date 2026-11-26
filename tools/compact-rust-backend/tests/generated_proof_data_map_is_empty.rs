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

//! Generator contracts for the zero-argument root Map emptiness facade.
//! Actual generated-crate execution and SDK checks remain separate gates.
use compact_rust_backend::{ir::Contract, render, render_with_standalone_proof_data};
use quote::ToTokens;
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/export-map-field-field/compact-rust-ir.json"
    ))
    .unwrap()
}
fn selected(value: &mut Value) -> &mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "is_empty")
        .unwrap()
}
fn single() -> Value {
    let mut value = original();
    let read = selected(&mut value).clone();
    value["stateful_circuits"] = json!([read]);
    value
}
fn methods(source: &str, owner: &str) -> Vec<syn::ImplItemFn> {
    fn module(items: &[syn::Item], name: &str) -> Vec<syn::Item> {
        items
            .iter()
            .find_map(|item| match item {
                syn::Item::Mod(m) if m.ident == name => {
                    m.content.as_ref().map(|(_, items)| items.clone())
                }
                _ => None,
            })
            .unwrap()
    }
    let file = syn::parse_file(source).unwrap();
    module(&module(&file.items, "ledger_contract"), "recorded")
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
fn assert_empty_capture(method: &syn::ImplItemFn, circuit: &str) {
    assert_eq!(method.sig.ident, format!("{circuit}_proof_data"));
    assert_eq!(method.sig.inputs.len(), 2); // self/context, no synthetic input
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
    let locals = method
        .block
        .stmts
        .iter()
        .filter_map(|stmt| {
            let syn::Stmt::Local(local) = stmt else {
                return None;
            };
            let syn::Pat::Ident(binding) = &local.pat else {
                return None;
            };
            Some((
                binding.ident.to_string(),
                local.init.as_ref().unwrap().expr.as_ref(),
            ))
        })
        .collect::<Vec<_>>();
    let input = locals.iter().position(|(name, _)| name == "input").unwrap();
    let recorded = locals
        .iter()
        .position(|(name, _)| name == "recorded")
        .unwrap();
    assert!(input < recorded);
    let syn::Expr::Call(alignment) = locals[input].1 else {
        panic!("aligned Unit input")
    };
    assert_eq!(
        alignment.func.to_token_stream().to_string(),
        "runtime :: fab :: AlignedValue :: from"
    );
    assert_eq!(alignment.args.len(), 1);
    assert!(matches!(&alignment.args[0], syn::Expr::Tuple(unit) if unit.elems.is_empty()));
    // Failure from the actual recorded invocation propagates before a carrier exists.
    let syn::Expr::Try(invocation) = locals[recorded].1 else {
        panic!("propagated recorded result")
    };
    let syn::Expr::MethodCall(invocation) = invocation.expr.as_ref() else {
        panic!("actual recorded call")
    };
    assert_eq!(invocation.method, circuit);
    assert_eq!(invocation.receiver.to_token_stream().to_string(), "self");
    assert_eq!(invocation.args.len(), 1);
    assert_eq!(invocation.args[0].to_token_stream().to_string(), "context");
    let syn::Stmt::Expr(syn::Expr::Call(carrier), None) = method.block.stmts.last().unwrap() else {
        panic!("returned actual capture")
    };
    assert_eq!(
        carrier.func.to_token_stream().to_string(),
        "runtime :: proof_data :: RecordedProofData :: from_recorded"
    );
    assert_eq!(carrier.args.len(), 3);
    assert!(
        matches!(&carrier.args[0], syn::Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(name) if name.value() == circuit))
    );
    assert_eq!(carrier.args[1].to_token_stream().to_string(), "input");
    assert_eq!(carrier.args[2].to_token_stream().to_string(), "recorded");
    let body = method.block.to_token_stream().to_string();
    assert_eq!(body.matches(&format!("self . {circuit} (")).count(), 1);
    assert_eq!(
        body.matches("RecordedProofData :: from_recorded").count(),
        1
    );
    assert!(body.contains("API_VERSION as usize"));
}

#[test]
fn intact_original_has_eight_recordings_and_exactly_three_opt_in_read_families() {
    let value = original();
    assert_eq!(value["stateful_circuits"].as_array().unwrap().len(), 8);
    let contract: Contract = serde_json::from_value(value).unwrap();
    let plain = render(&contract).unwrap();
    let out = render_with_standalone_proof_data(&contract).unwrap();
    assert_eq!(out.capabilities.circuits.len(), 8);
    let generated = methods(&out.source, "Contract");
    for row in &out.capabilities.circuits {
        assert!(row.recorded && row.observed_call, "{}", row.name);
        assert_eq!(
            row.standalone_proof_data.is_some(),
            matches!(row.name.as_str(), "is_empty" | "member" | "lookup")
        );
        assert!(generated.iter().any(|m| m.sig.ident == row.name));
        assert!(
            generated
                .iter()
                .any(|m| m.sig.ident == format!("{}_call", row.name))
        );
    }
    let before = methods(&plain, "Contract");
    assert!(
        before
            .iter()
            .all(|m| !m.sig.ident.to_string().ends_with("_proof_data"))
    );
    let normal = generated
        .iter()
        .filter(|m| !m.sig.ident.to_string().ends_with("_proof_data"))
        .map(|m| m.to_token_stream().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        normal,
        before
            .iter()
            .map(|m| m.to_token_stream().to_string())
            .collect::<Vec<_>>()
    );
    let row = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "is_empty")
        .unwrap();
    let capability = row.standalone_proof_data.as_ref().unwrap();
    assert_eq!(capability.api_version, 1);
    assert_eq!(capability.method, "is_empty_proof_data");
    assert_empty_capture(
        generated
            .iter()
            .find(|m| m.sig.ident == capability.method)
            .unwrap(),
        "is_empty",
    );
    assert!(
        !methods(&out.source, "BorrowedContract")
            .iter()
            .any(|m| m.sig.ident == "is_empty_proof_data")
    );
}

#[test]
fn renamed_expression_on_borrowed_handle_has_no_spurious_witness_or_clone_bound() {
    let mut value = single();
    value["ledger_fields"][0]["id"] = json!("renamed_map");
    value["ledger_fields"][0]["path"] = json!([0]);
    let circuit = selected(&mut value);
    circuit["name"] = json!("vacant");
    circuit["return_value"]["field"] = json!("renamed_map");
    let read = circuit["return_value"].take();
    circuit["return_value"] = json!({"kind":"expression","value":read});
    // A different circuit needs a witness; the admitted emptiness read does not.
    value["witnesses"] = json!([{"name":"ask","parameters":[],"result":{"kind":"field"}}]);
    value["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"witnessed","parameters":[],"actions":[],"result":{"kind":"field"},
        "return_value":{"kind":"expression","value":{"kind":"witness_call","name":"ask","arguments":[]}}}));
    let out = render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
    for owner in ["Contract", "BorrowedContract"] {
        let generated = methods(&out.source, owner);
        assert_empty_capture(
            generated
                .iter()
                .find(|m| m.sig.ident == "vacant_proof_data")
                .unwrap(),
            "vacant",
        );
    }
}

#[test]
fn broader_valid_recordings_do_not_silently_gain_the_optional_family() {
    for edit in [
        "boolean-value-map",
        "preceding-write",
        "composed-expression",
    ] {
        let mut value = single();
        match edit {
            "boolean-value-map" => {
                value["ledger_fields"][0]["declaration"]["value"] = json!({"kind":"boolean"})
            }
            "preceding-write" => {
                selected(&mut value)["actions"] =
                    json!([{"kind":"map_reset","field":"c","index":0}])
            }
            "composed-expression" => {
                selected(&mut value)["return_value"] = json!({"kind":"expression","value":{
                "kind":"if","condition":{"kind":"boolean","value":true},
                "then":{"kind":"map_is_empty","field":"c","index":0},
                "otherwise":{"kind":"boolean","value":false}}})
            }
            _ => unreachable!(),
        }
        let out =
            render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
        let row = &out.capabilities.circuits[0];
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
fn public_and_internal_helper_name_collisions_keep_ordinary_recording_available() {
    for internal in [false, true] {
        let mut value = single();
        value["stateful_circuits"].as_array_mut().unwrap().push(json!({
            "name":"is_empty_proof_data","internal":internal,"parameters":[],"actions":[],
            "result":{"kind":"boolean"},"return_value":{"kind":"expression","value":{"kind":"boolean","value":true}}}));
        let out =
            render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "is_empty")
            .unwrap();
        assert!(row.recorded && row.observed_call && row.standalone_proof_data.is_none());
        assert_eq!(
            row.standalone_proof_data_unavailable.as_ref().unwrap().code,
            "name_collision"
        );
        let generated = methods(&out.source, "Contract");
        assert!(generated.iter().any(|m| m.sig.ident == "is_empty"));
        assert!(generated.iter().any(|m| m.sig.ident == "is_empty_call"));
    }
}
