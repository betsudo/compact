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

//! Host checks of the intact pinned eight-export Map fixture. Supplemental
//! mutations below exercise optional facade boundaries, not new original tests.
use compact_rust_backend::{ir::Contract, render, render_with_standalone_proof_data};
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/export-map-field-field/compact-rust-ir.json"
    ))
    .unwrap()
}
fn selected(v: &mut Value) -> &mut Value {
    v["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "member")
        .unwrap()
}
fn single() -> Value {
    let mut v = original();
    let member = selected(&mut v).clone();
    v["stateful_circuits"] = json!([member]);
    v
}
fn with_borrowed_handle(mut v: Value) -> Value {
    // A separate witnessed circuit makes the renderer expose BorrowedContract.
    // member itself still performs no witness call or Private clone.
    v["witnesses"] = json!([{"name":"extra_witness","parameters":[],"result":{"kind":"field"}}]);
    v["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"witnessed","parameters":[],"actions":[],"result":{"kind":"field"},
        "return_value":{"kind":"expression","value":{"kind":"witness_call","name":"extra_witness","arguments":[]}}}));
    v
}
fn recorded_function(source: &str, name: &str) -> syn::ItemFn {
    let file = syn::parse_file(source).unwrap();
    for i in file.items {
        if let syn::Item::Mod(m) = i
            && m.ident == "ledger_contract"
        {
            for i in m.content.unwrap().1 {
                if let syn::Item::Mod(m) = i
                    && m.ident == "recorded"
                {
                    for i in m.content.unwrap().1 {
                        if let syn::Item::Fn(f) = i
                            && f.sig.ident == name
                        {
                            return f;
                        }
                    }
                }
            }
        }
    }
    panic!("actual recorded free function {name}")
}
fn method(source: &str, owner: &str, name: &str) -> Option<syn::ImplItemFn> {
    fn module(items: &[syn::Item], name: &str) -> Option<Vec<syn::Item>> {
        items.iter().find_map(|i| match i {
            syn::Item::Mod(m) if m.ident == name => Some(m.content.as_ref()?.1.clone()),
            _ => None,
        })
    }
    let file = syn::parse_file(source).unwrap();
    let items = module(&module(&file.items, "ledger_contract")?, "recorded")?;
    for i in items {
        let syn::Item::Impl(i) = i else { continue };
        let syn::Type::Path(p) = i.self_ty.as_ref() else {
            continue;
        };
        if p.path.segments.last().unwrap().ident != owner {
            continue;
        }
        for i in i.items {
            if let syn::ImplItem::Fn(m) = i
                && m.sig.ident == name
            {
                return Some(m);
            }
        }
    }
    None
}
fn refused(v: Value, old_api: bool) {
    let c: Contract = serde_json::from_value(v).unwrap();
    match render_with_standalone_proof_data(&c) {
        Ok(out) => {
            let row = out
                .capabilities
                .circuits
                .iter()
                .find(|c| c.name == "member")
                .unwrap();
            assert!(row.standalone_proof_data.is_none());
            if old_api {
                assert!(row.recorded);
            }
            for owner in ["Contract", "BorrowedContract"] {
                assert!(method(&out.source, owner, "member_proof_data").is_none());
            }
        }
        Err(e) => assert!(!old_api, "existing API must remain: {e}"),
    }
}

#[test]
fn intact_eight_exports_keep_old_apis_and_only_map_reads_get_opt_in_helpers() {
    let v = original();
    assert_eq!(v["stateful_circuits"].as_array().unwrap().len(), 8);
    assert_eq!(
        selected(&mut v.clone())["return_value"],
        json!({
        "kind":"map_member","field":"c","index":0,
        "key":{"kind":"parameter","name":"key"}})
    );
    let c: Contract = serde_json::from_value(v).unwrap();
    let default = render(&c).unwrap();
    let out = render_with_standalone_proof_data(&c).unwrap();
    assert_eq!(out.capabilities.circuits.len(), 8);
    for row in &out.capabilities.circuits {
        assert!(row.recorded && row.observed_call, "{}", row.name);
        assert_eq!(
            row.standalone_proof_data.is_some(),
            matches!(row.name.as_str(), "is_empty" | "member" | "lookup" | "size")
        );
        {
            let owner = "Contract";
            assert!(method(&out.source, owner, &row.name).is_some());
            assert!(method(&out.source, owner, &format!("{}_call", row.name)).is_some());
            assert!(method(&default, owner, &format!("{}_proof_data", row.name)).is_none());
        }
    }
    let row = out
        .capabilities
        .circuits
        .iter()
        .find(|r| r.name == "member")
        .unwrap();
    assert_eq!(row.standalone_proof_data.as_ref().unwrap().api_version, 1);
    assert_eq!(
        row.standalone_proof_data.as_ref().unwrap().method,
        "member_proof_data"
    );
    assert!(method(&out.source, "BorrowedContract", "member_proof_data").is_none());
    let with_witness: Contract = serde_json::from_value(with_borrowed_handle(original())).unwrap();
    let borrowed = render_with_standalone_proof_data(&with_witness).unwrap();
    for (owner, source) in [
        ("Contract", out.source.as_str()),
        ("BorrowedContract", borrowed.source.as_str()),
    ] {
        let m = method(source, owner, "member_proof_data").unwrap();
        assert!(m.sig.generics.where_clause.is_none()); // no Private: Clone or witness bound
        assert_eq!(m.sig.inputs.len(), 3);
        let expected: syn::ImplItemFn = syn::parse_quote! {
            pub fn member_proof_data<Private>(
                &self,context:runtime::context::CircuitContext<Private>,key:runtime::Field,
            )->Result<runtime::proof_data::RecordedProofData<Private,bool>,runtime::proof_data::ProofDataError,> {
                const _: [();1]=[();runtime::proof_data::API_VERSION as usize];
                let input=runtime::fab::AlignedValue::from(key);
                let recorded=self.member(context,key)?;
                runtime::proof_data::RecordedProofData::from_recorded("member",input,recorded,)
            }
        };
        // Exact executable AST: one typed capture dominates exactly one real
        // recorded call and its actual Boolean result, with no transcript synthesis.
        let actual_sig = &m.sig;
        let expected_sig = &expected.sig;
        assert_eq!(
            quote::quote!(#actual_sig).to_string(),
            quote::quote!(#expected_sig).to_string()
        );
        let actual_body = &m.block;
        let expected_body = &expected.block;
        assert_eq!(
            quote::quote!(#actual_body).to_string(),
            quote::quote!(#expected_body).to_string()
        );
    }
}

#[test]
fn renamed_member_slot_and_key_are_semantic_and_expression_return_is_bounded() {
    let mut v = single();
    v["ledger_fields"][0]["id"] = json!("renamed_map");
    let m = selected(&mut v);
    m["name"] = json!("contains_key");
    m["parameters"][0]["name"] = json!("candidate");
    m["return_value"]["field"] = json!("renamed_map");
    m["return_value"]["key"]["name"] = json!("candidate");
    let read = m["return_value"].take();
    m["return_value"] = json!({"kind":"expression","value":read});
    let c: Contract = serde_json::from_value(v).unwrap();
    let out = render_with_standalone_proof_data(&c).unwrap();
    assert_eq!(
        out.capabilities.circuits[0]
            .standalone_proof_data
            .as_ref()
            .unwrap()
            .method,
        "contains_key_proof_data"
    );
    {
        let owner = "Contract";
        let m = method(&out.source, owner, "contains_key_proof_data").unwrap();
        let body = &m.block;
        let text = quote::quote!(#body).to_string();
        assert!(text.contains("AlignedValue :: from (candidate)"));
        assert_eq!(
            text.matches("self . contains_key (context , candidate)")
                .count(),
            1
        );
    }
}

#[test]
fn typed_nonfamily_domains_and_paths_preserve_old_recording_without_facade() {
    let mut v = single();
    v["ledger_fields"][0]["declaration"]["key"] = json!({"kind":"boolean"});
    selected(&mut v)["parameters"][0]["ty"] = json!({"kind":"boolean"});
    refused(v, true);
    let mut v = single();
    v["ledger_fields"][0]["declaration"]["value"] = json!({"kind":"boolean"});
    refused(v, true);
    let mut v = single();
    v["ledger_fields"][0]["path"] = json!([1, 0]);
    refused(v, false); // globally invalid shape already refuses before optional admission
    let mut v = single();
    selected(&mut v)["parameters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"unused","ty":{"kind":"field"}}));
    refused(v, true);
}

#[test]
fn writes_witnesses_loops_and_helper_keys_do_not_gain_facade() {
    let mut v = single();
    selected(&mut v)["actions"] = json!([{"kind":"map_reset","field":"c","index":0}]);
    refused(v, true);
    let mut v = single();
    v["witnesses"] = json!([{"name":"ask","parameters":[],"result":{"kind":"field"}}]);
    selected(&mut v)["return_value"]["key"] =
        json!({"kind":"witness_call","name":"ask","arguments":[]});
    refused(v, false); // existing Map-return operand lowering refuses this effectful key
    let mut v = single();
    v["circuits"] = json!([{"name":"identity","parameters":[{"name":"x","ty":{"kind":"field"}}],"result":{"kind":"field"},"body":{"kind":"parameter","name":"x"}}]);
    selected(&mut v)["return_value"]["key"] =
        json!({"kind":"call","name":"identity","arguments":[{"kind":"parameter","name":"key"}]});
    refused(v, true);
    let mut v = single();
    selected(&mut v)["return_value"]["key"] = json!({"kind":"field_literal","value":"1"});
    refused(v, true);
    let mut v = single();
    selected(&mut v)["return_value"]["key"] = json!({"kind":"vector_fold","accumulator":{"name":"acc","ty":{"kind":"field"}},"items":[{"name":"item","ty":{"kind":"field"}}],"initial":{"kind":"parameter","name":"key"},"sources":[{"kind":"vector","element":{"kind":"field"},"elements":[{"kind":"field_literal","value":"1"}]}],"body":{"kind":"value","value":{"kind":"parameter","name":"acc"}},"length":1});
    refused(v, false);
    let mut v = single();
    selected(&mut v)["return_value"]["index"] = json!(1);
    refused(v, false);
    let mut v = single();
    selected(&mut v)["return_value"]["field"] = json!("absent");
    refused(v, false);
}

#[test]
fn public_and_internal_helper_name_collisions_keep_original_member_call() {
    for internal in [false, true] {
        let mut v = single();
        let mut other = selected(&mut v).clone();
        other["name"] = json!("member_proof_data");
        other["internal"] = json!(internal);
        v["stateful_circuits"].as_array_mut().unwrap().push(other);
        let c: Contract = serde_json::from_value(v).unwrap();
        let out = render_with_standalone_proof_data(&c).unwrap();
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|r| r.name == "member")
            .unwrap();
        assert!(row.recorded && row.observed_call && row.standalone_proof_data.is_none());
        assert_eq!(
            row.standalone_proof_data_unavailable.as_ref().unwrap().code,
            "name_collision"
        );
        {
            let owner = "Contract";
            assert!(method(&out.source, owner, "member").is_some());
            assert!(method(&out.source, owner, "member_call").is_some());
        }
    }
}

#[test]
fn each_original_terminal_unit_map_operation_has_exactly_one_ordered_effect() {
    let v = original();
    for (name, kind, method_name) in [
        ("reset_to_default", "map_reset", "record_reset"),
        ("insert", "map_insert", "record_insert"),
        (
            "insert_default",
            "map_insert_default",
            "record_insert_default",
        ),
        ("remove", "map_remove", "record_remove"),
    ] {
        let c = v["stateful_circuits"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        assert_eq!(c["actions"], json!([]));
        assert_eq!(c["result"], json!({"kind":"unit"}));
        assert_eq!(c["return_value"]["kind"], "effectful");
        let body = &c["return_value"]["body"];
        assert_eq!(body["kind"], "sequence");
        assert_eq!(body["actions"].as_array().unwrap().len(), 1);
        assert_eq!(body["actions"][0]["kind"], kind);
        assert_eq!(body["actions"][0]["field"], "c");
        assert_eq!(body["actions"][0]["index"], 0);
        assert_eq!(
            body["result"],
            json!({"kind":"value","value":{"kind":"unit"}})
        );
        let typed: Contract = serde_json::from_value(v.clone()).unwrap();
        let out = render_with_standalone_proof_data(&typed).unwrap();
        let m = recorded_function(&out.source, name);
        let block = &m.block;
        let text = quote::quote!(#block).to_string();
        assert_eq!(
            text.matches(&format!(". {method_name} (")).count(),
            1,
            "{text}"
        );
    }
}
