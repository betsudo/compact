// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Host-only generated API checks. No runtime/SDK execution or fixture repair.
use compact_rust_backend::{ir::Contract, render_with_standalone_proof_data};
use serde_json::{Value, json};

fn contract() -> Value {
    let mut circuits = Vec::new();
    for (name, kind) in [
        ("arbitrary_lt", "kernel_block_time_less_than"),
        ("arbitrary_gt", "kernel_block_time_greater_than"),
    ] {
        circuits.push(json!({
            "name":name,"internal":true,
            "parameters":[{"name":"time","ty":{"kind":"unsigned","max":"18446744073709551615"}}],
            "actions":[],"result":{"kind":"boolean"},
            "return_value":{"kind":"expression","value":{"kind":kind,"bound":{"kind":"parameter","name":"time"}}}
        }));
    }
    for (name, callee, negated) in [
        ("less", "arbitrary_lt", false),
        ("greater_equal", "arbitrary_lt", true),
        ("greater", "arbitrary_gt", false),
        ("less_equal", "arbitrary_gt", true),
    ] {
        let call = json!({"kind":"call","name":callee,"arguments":[{"kind":"coerce","ty":{"kind":"unsigned","max":"18446744073709551615"},"value":{"kind":"parameter","name":"context"}}]});
        let value = if negated {
            json!({"kind":"if","condition":call,"then":{"kind":"boolean","value":false},"otherwise":{"kind":"boolean","value":true}})
        } else {
            call
        };
        circuits.push(json!({"name":name,"parameters":[{"name":"context","ty":{"kind":"unsigned","max":"18446744073709551615"}}],"actions":[],"result":{"kind":"boolean"},"return_value":{"kind":"expression","value":value}}));
    }
    json!({"schema_version":20,"ledger_fields":[],"witnesses":[],"circuits":[],"stateful_circuits":circuits})
}
fn render(value: Value) -> compact_rust_backend::StandaloneProofDataRender {
    let contract: Contract = serde_json::from_value(value).unwrap();
    render_with_standalone_proof_data(&contract).unwrap()
}
#[test]
fn four_semantic_exports_capture_before_one_execution_with_fixed_names() {
    let rendered = render(contract());
    let file = syn::parse_file(&rendered.source).unwrap();
    let mut helpers = Vec::new();
    for item in &file.items {
        if let syn::Item::Mod(m) = item
            && m.ident == "ledger_contract"
        {
            let (_, items) = m.content.as_ref().unwrap();
            for item in items {
                if let syn::Item::Mod(m) = item
                    && m.ident == "recorded"
                {
                    for item in &m.content.as_ref().unwrap().1 {
                        if let syn::Item::Impl(i) = item {
                            for item in &i.items {
                                if let syn::ImplItem::Fn(f) = item
                                    && f.sig.ident.to_string().ends_with("_proof_data")
                                {
                                    helpers.push(f.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(helpers.len(), 4);
    for f in helpers {
        assert!(f.attrs.iter().all(|a| !a.path().is_ident("cfg")));
        assert_eq!(f.block.stmts.len(), 4);
        let syn::Stmt::Local(input) = &f.block.stmts[1] else {
            panic!("input capture must dominate execution")
        };
        let syn::Pat::Ident(input_name) = &input.pat else {
            panic!("captured input")
        };
        assert_eq!(input_name.ident, "input");
        let syn::Stmt::Local(recorded) = &f.block.stmts[2] else {
            panic!("one recorded call")
        };
        let syn::Expr::Try(attempt) = recorded.init.as_ref().unwrap().expr.as_ref() else {
            panic!("fallible execution")
        };
        let syn::Expr::MethodCall(call) = attempt.expr.as_ref() else {
            panic!("actual recording method")
        };
        assert_eq!(
            call.method.to_string(),
            f.sig.ident.to_string().trim_end_matches("_proof_data")
        );
        struct Calls(Vec<String>);
        impl syn::visit_mut::VisitMut for Calls {
            fn visit_expr_method_call_mut(&mut self, node: &mut syn::ExprMethodCall) {
                self.0.push(node.method.to_string());
                syn::visit_mut::visit_expr_method_call_mut(self, node);
            }
        }
        let mut calls = Calls(Vec::new());
        syn::visit_mut::VisitMut::visit_block_mut(&mut calls, &mut f.block.clone());
        assert_eq!(calls.0, vec![call.method.to_string()]);
        let body = quote::quote!(#f).to_string();
        let input = body.find("let input =").unwrap();
        let execute = body.find("let recorded = self").unwrap();
        assert!(input < execute);
        assert_eq!(body.matches("let recorded = self").count(), 1);
        let syn::Stmt::Item(syn::Item::Const(version_guard)) = &f.block.stmts[0] else {
            panic!("compile-time API version guard")
        };
        let expected: syn::ItemConst = syn::parse_quote! {
            const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize];
        };
        assert_eq!(
            quote::quote!(#version_guard).to_string(),
            quote::quote!(#expected).to_string()
        );
        let name = f
            .sig
            .ident
            .to_string()
            .trim_end_matches("_proof_data")
            .to_owned();
        assert!(body.contains(&format!("from_recorded (\"{name}\"")));
        assert!(!body.contains("fixture") && !body.contains("from (0"));
    }
    assert_eq!(rendered.capabilities.circuits.len(), 4);
    for c in rendered.capabilities.circuits {
        let api = c.standalone_proof_data.as_ref().unwrap();
        assert_eq!(api.api_version, 1);
        assert_eq!(api.method, format!("{}_proof_data", c.name));
        assert!(c.recorded && c.observed_call);
        assert!(c.proof_required.is_none());
    }
}
#[test]
fn optional_api_preserves_unused_recorded_helper_pruning() {
    // The semantic four-export time family inlines both internal comparison
    // bodies. An opaque macro in any new facade would conservatively retain
    // those otherwise-unused helpers; the optional API must not cause that.
    let out = render(contract());
    assert!(
        out.capabilities
            .circuits
            .iter()
            .all(|c| c.standalone_proof_data.is_some())
    );
    struct PrivateBodies(Vec<String>);
    impl syn::visit_mut::VisitMut for PrivateBodies {
        fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
            let name = function.sig.ident.to_string();
            if name.starts_with("__compact_recorded_body_") {
                self.0.push(name);
            }
            syn::visit_mut::visit_item_fn_mut(self, function);
        }
    }
    let mut private_bodies = PrivateBodies(Vec::new());
    let mut file = syn::parse_file(&out.source).unwrap();
    syn::visit_mut::VisitMut::visit_file_mut(&mut private_bodies, &mut file);
    assert!(
        private_bodies.0.is_empty(),
        "unexpected retained helpers: {:?}",
        private_bodies.0
    );
}
#[test]
fn unrelated_boolean_and_non_u64_families_do_not_claim_the_api() {
    let mut value = contract();
    value["stateful_circuits"][2]["return_value"] =
        json!({"kind":"expression","value":{"kind":"boolean","value":true}});
    let out = render(value);
    let less = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "less")
        .unwrap();
    assert!(less.standalone_proof_data.is_none());
    assert!(less.recorded);
    let mut value = contract();
    value["stateful_circuits"][2]["parameters"][0]["ty"]["max"] = json!("255");
    // A malformed typed call is allowed to refuse earlier; it cannot emit the new capability.
    let typed: Contract = serde_json::from_value(value).unwrap();
    if let Ok(out) = render_with_standalone_proof_data(&typed) {
        assert!(
            out.capabilities
                .circuits
                .iter()
                .find(|c| c.name == "less")
                .unwrap()
                .standalone_proof_data
                .is_none()
        );
    }
}
fn assert_collision_preserves_old_apis(value: Value, root: &str) {
    let out = render(value);
    let c = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == root)
        .unwrap();
    assert!(c.recorded && c.observed_call);
    assert!(c.recording_unavailable.is_none() && c.observed_call_unavailable.is_none());
    assert!(c.standalone_proof_data.is_none());
    assert_eq!(
        c.standalone_proof_data_unavailable.as_ref().unwrap().code,
        "name_collision"
    );
    assert!(out.source.contains("RecordedCircuitResult"));
    assert!(out.source.contains("pub fn less"));
}
#[test]
fn helper_collision_omits_only_the_new_optional_api() {
    for internal in [false, true] {
        let mut value = contract();
        let mut collision = value["stateful_circuits"][2].clone();
        collision["name"] = json!("less_proof_data");
        collision["internal"] = json!(internal);
        value["stateful_circuits"]
            .as_array_mut()
            .unwrap()
            .push(collision);
        assert_collision_preserves_old_apis(value, "less");
    }
}
#[test]
fn boolean_helper_preserves_time_reachability_in_its_argument() {
    let mut value = contract();
    let call = value["stateful_circuits"][2]["return_value"]["value"].clone();
    value["stateful_circuits"][2]["return_value"]["value"] =
        json!({"kind":"call","name":"negate","arguments":[call]});
    value["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"negate","internal":true,"parameters":[{"name":"x","ty":{"kind":"boolean"}}],
        "actions":[],"result":{"kind":"boolean"},
        "return_value":{"kind":"expression","value":{"kind":"if","condition":{"kind":"parameter","name":"x"},"then":{"kind":"boolean","value":false},"otherwise":{"kind":"boolean","value":true}}}
    }));
    let out = render(value);
    assert!(
        out.capabilities
            .circuits
            .iter()
            .find(|c| c.name == "less")
            .unwrap()
            .standalone_proof_data
            .is_some()
    );
}
#[test]
fn recursive_or_action_bearing_helpers_never_claim_the_standalone_api() {
    for recursive in [false, true] {
        let mut value = contract();
        if recursive {
            value["stateful_circuits"][0]["return_value"]["value"] = json!({"kind":"call","name":"arbitrary_lt","arguments":[{"kind":"parameter","name":"time"}]});
        } else {
            value["stateful_circuits"][0]["actions"] =
                json!([{"kind":"expression","value":{"kind":"boolean","value":true}}]);
        }
        let typed: Contract = serde_json::from_value(value).unwrap();
        if let Ok(out) = render_with_standalone_proof_data(&typed) {
            assert!(
                out.capabilities
                    .circuits
                    .iter()
                    .filter(|c| c.name == "less" || c.name == "greater_equal")
                    .all(|c| c.standalone_proof_data.is_none())
            );
        }
    }
}

#[test]
fn actual_emitted_identifier_and_raw_key_name_have_distinct_owners() {
    let mut value = contract();
    value["stateful_circuits"][2]["name"] = json!("price$check");
    let out = render(value.clone());
    let c = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "price$check")
        .unwrap();
    assert_eq!(
        c.standalone_proof_data.as_ref().unwrap().method,
        "price_check_proof_data"
    );
    assert!(out.source.contains("pub fn price_check_proof_data"));
    assert!(out.source.contains(r#""price$check""#));
    let mut collision = value["stateful_circuits"][2].clone();
    collision["name"] = json!("price_check_proof_data");
    collision["internal"] = json!(true);
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .push(collision);
    let out = render(value);
    let c = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "price$check")
        .unwrap();
    assert!(c.recorded && c.observed_call && c.standalone_proof_data.is_none());
    assert_eq!(
        c.standalone_proof_data_unavailable.as_ref().unwrap().method,
        "price_check_proof_data"
    );
}
#[test]
fn additive_api_metadata_does_not_change_the_schema2_to3_proof_join() {
    let out = render(contract());
    assert_eq!(out.capabilities.schema_version, 2);
    for proof in [false, true] {
        let mut report = out.capabilities.clone();
        let before = serde_json::to_value(&report).unwrap();
        let info = json!({"circuits":report.circuits.iter().map(|c|json!({"name":c.name,"proof":proof})).collect::<Vec<_>>()});
        report.apply_contract_info(&info).unwrap();
        assert_eq!(report.schema_version, 3);
        let after = serde_json::to_value(&report).unwrap();
        for (index, c) in report.circuits.iter().enumerate() {
            assert_eq!(c.proof_required, Some(proof));
            assert_eq!(
                c.recording_status,
                Some(if proof {
                    compact_rust_backend::RecordingStatus::Available
                } else {
                    compact_rust_backend::RecordingStatus::NotApplicable
                })
            );
            for field in ["recorded", "observed_call", "standalone_proof_data"] {
                assert_eq!(
                    before["circuits"][index][field],
                    after["circuits"][index][field]
                );
            }
        }
    }
}
