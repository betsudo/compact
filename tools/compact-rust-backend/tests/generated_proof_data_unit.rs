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

//! Host-only Unit facade admission/AST checks. No runtime, SDK or proof execution.
//! The immutable fixture is genuine frontend output from pinned upstream672,
//! SHA256 bbe8e7c6d97e97c92c60363e2819497f7ed0586d0a09bf21c19c412a5172086a.
//! Renamed/direct-literal cases below are supplemental semantic mutations.
use compact_rust_backend::{
    StandaloneProofDataRender, ir::Contract, render_with_standalone_proof_data,
};
use serde_json::{Value, json};

type EditCase = (&'static str, fn(&mut Value));

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/gas-cost/compact-rust-ir.json"
    ))
    .unwrap()
}

fn circuit_name(value: &Value) -> String {
    value["stateful_circuits"][0]["name"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn render(value: Value) -> StandaloneProofDataRender {
    let contract: Contract = serde_json::from_value(value).unwrap();
    render_with_standalone_proof_data(&contract).unwrap()
}

fn owned_method(source: &str, name: &str) -> Option<syn::ImplItemFn> {
    let file = syn::parse_file(source).unwrap();
    for item in file.items {
        let syn::Item::Mod(outer) = item else {
            continue;
        };
        if outer.ident != "ledger_contract" {
            continue;
        }
        for item in outer.content.unwrap().1 {
            let syn::Item::Mod(module) = item else {
                continue;
            };
            if module.ident != "recorded" {
                continue;
            }
            for item in module.content.unwrap().1 {
                let syn::Item::Impl(implementation) = item else {
                    continue;
                };
                let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
                    continue;
                };
                if owner.path.segments.last().unwrap().ident != "Contract" {
                    continue;
                }
                for item in implementation.items {
                    if let syn::ImplItem::Fn(method) = item
                        && method.sig.ident == name
                    {
                        return Some(method);
                    }
                }
            }
        }
    }
    None
}

fn admitted(value: Value) -> (StandaloneProofDataRender, syn::ImplItemFn) {
    let name = circuit_name(&value);
    let out = render(value);
    let capability = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == name)
        .unwrap();
    assert!(capability.recorded && capability.observed_call);
    assert!(capability.recording_unavailable.is_none());
    assert!(capability.observed_call_unavailable.is_none());
    assert!(capability.standalone_proof_data_unavailable.is_none());
    let proof = capability.standalone_proof_data.as_ref().unwrap();
    assert_eq!(proof.api_version, 1);
    let method = owned_method(&out.source, &proof.method).unwrap();
    assert_eq!(proof.method, method.sig.ident.to_string());
    (out, method)
}

fn not_admitted(value: Value, label: &str) {
    let name = circuit_name(&value);
    let contract: Contract = serde_json::from_value(value).unwrap();
    // Malformed typed input may be rejected by existing native validation first.
    // An accepted old API must still never acquire the new optional facade.
    if let Ok(out) = render_with_standalone_proof_data(&contract) {
        if let Some(c) = out.capabilities.circuits.iter().find(|c| c.name == name) {
            assert!(c.standalone_proof_data.is_none(), "{label}");
        }
        assert!(
            owned_method(&out.source, &format!("{name}_proof_data")).is_none(),
            "unexpected new facade: {label}"
        );
    }
}

fn direct(value: &mut Value) {
    for action in value["stateful_circuits"][0]["actions"]
        .as_array_mut()
        .unwrap()
    {
        let literal = action["bindings"][0]["value"].clone();
        let mut inner = action["action"].clone();
        inner["value"] = literal;
        *action = inner;
    }
}

#[test]
fn original_frontend_unit_capture_precedes_exactly_one_context_only_call() {
    let value = original();
    assert_eq!(value["schema_version"], 20);
    let c = &value["stateful_circuits"][0];
    assert_eq!(c["parameters"], json!([]));
    assert_eq!(c["result"], json!({"kind":"unit"}));
    assert_eq!(c["return_value"], json!({"kind":"unit"}));
    assert_eq!(c["actions"].as_array().unwrap().len(), 2);
    assert_eq!(c["actions"][0]["kind"], "let");
    assert_eq!(c["actions"][0]["action"]["kind"], "set_remove");
    assert_eq!(c["actions"][1]["action"]["kind"], "set_insert");
    let (_, method) = admitted(value);
    assert_eq!(method.sig.inputs.len(), 2); // &self and context, no public arguments.
    let output = &method.sig.output;
    let expected: syn::ReturnType = syn::parse_quote! {
        -> Result<runtime::proof_data::RecordedProofData<Private, ()>, runtime::proof_data::ProofDataError,>
    };
    assert_eq!(
        quote::quote!(#output).to_string(),
        quote::quote!(#expected).to_string()
    );
    assert_eq!(method.block.stmts.len(), 4);
    let syn::Stmt::Item(syn::Item::Const(guard)) = &method.block.stmts[0] else {
        panic!("macro-free API handshake")
    };
    let expected: syn::ItemConst = syn::parse_quote! {
        const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize];
    };
    assert_eq!(
        quote::quote!(#guard).to_string(),
        quote::quote!(#expected).to_string()
    );
    let syn::Stmt::Local(input) = &method.block.stmts[1] else {
        panic!("input first")
    };
    let syn::Expr::Call(pack) = input.init.as_ref().unwrap().expr.as_ref() else {
        panic!("official typed Unit packer")
    };
    let callee = &pack.func;
    assert_eq!(
        quote::quote!(#callee).to_string(),
        "runtime :: fab :: AlignedValue :: from"
    );
    assert_eq!(pack.args.len(), 1);
    assert!(matches!(pack.args.first().unwrap(), syn::Expr::Tuple(t) if t.elems.is_empty()));
    let syn::Stmt::Local(recorded) = &method.block.stmts[2] else {
        panic!("execution second")
    };
    let syn::Expr::Try(attempt) = recorded.init.as_ref().unwrap().expr.as_ref() else {
        panic!("execution error must propagate before conversion")
    };
    let syn::Expr::MethodCall(call) = attempt.expr.as_ref() else {
        panic!("recorded call")
    };
    assert_eq!(call.method, "testGasCost");
    assert_eq!(call.args.len(), 1);
    assert!(matches!(call.args.first().unwrap(), syn::Expr::Path(p) if p.path.is_ident("context")));
    struct Calls(Vec<String>);
    impl syn::visit_mut::VisitMut for Calls {
        fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
            self.0.push(call.method.to_string());
            syn::visit_mut::visit_expr_method_call_mut(self, call);
        }
    }
    let mut calls = Calls(Vec::new());
    syn::visit_mut::VisitMut::visit_block_mut(&mut calls, &mut method.block.clone());
    assert_eq!(calls.0, vec!["testGasCost"]);
    let syn::Stmt::Expr(syn::Expr::Call(convert), None) = &method.block.stmts[3] else {
        panic!("convert actual recorded execution")
    };
    let callee = &convert.func;
    assert_eq!(
        quote::quote!(#callee).to_string(),
        "runtime :: proof_data :: RecordedProofData :: from_recorded"
    );
    assert_eq!(convert.args.len(), 3);
    assert!(
        matches!(&convert.args[0], syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) if s.value() == "testGasCost")
    );
    for (index, name) in [(1, "input"), (2, "recorded")] {
        assert!(matches!(&convert.args[index], syn::Expr::Path(p) if p.path.is_ident(name)));
    }
}

#[test]
fn renamed_fields_bindings_slots_and_unequal_literals_are_not_fixture_dispatch() {
    let mut value = original();
    value["ledger_fields"][0]["id"] = json!("source_pool");
    value["ledger_fields"][0]["index"] = json!(1);
    value["ledger_fields"][0]["path"] = json!([1]);
    value["ledger_fields"][1]["id"] = json!("destination_pool");
    value["ledger_fields"][1]["index"] = json!(0);
    value["ledger_fields"][1]["path"] = json!([0]);
    value["constructor"]["steps"][0]["bindings"][0]["name"] = json!("initial_value");
    value["constructor"]["steps"][0]["bindings"][0]["value"]["value"] = json!("23");
    value["constructor"]["steps"][0]["step"]["field"] = json!("source_pool");
    value["constructor"]["steps"][0]["step"]["index"] = json!(1);
    value["constructor"]["steps"][0]["step"]["value"]["name"] = json!("initial_value");
    let c = &mut value["stateful_circuits"][0];
    c["name"] = json!("carry_field_payload");
    for (i, binding, field, index, literal) in [
        (0, "removed_value", "source_pool", 1, "23"),
        (1, "inserted_value", "destination_pool", 0, "47"),
    ] {
        c["actions"][i]["bindings"][0]["name"] = json!(binding);
        c["actions"][i]["bindings"][0]["value"]["value"] = json!(literal);
        c["actions"][i]["action"]["field"] = json!(field);
        c["actions"][i]["action"]["index"] = json!(index);
        c["actions"][i]["action"]["value"]["name"] = json!(binding);
    }
    let (_, method) = admitted(value);
    assert_eq!(method.sig.ident, "carry_field_payload_proof_data");
}

#[test]
fn direct_literals_and_same_declared_set_are_structurally_supported() {
    let mut value = original();
    direct(&mut value);
    admitted(value);
    let mut value = original();
    value["stateful_circuits"][0]["actions"][1]["action"]["field"] = json!("setA");
    value["stateful_circuits"][0]["actions"][1]["action"]["index"] = json!(0);
    admitted(value);
    let mut value = original();
    for action in value["stateful_circuits"][0]["actions"]
        .as_array_mut()
        .unwrap()
    {
        action["bindings"][0]["name"] = json!("local_value");
        action["action"]["value"]["name"] = json!("local_value");
    }
    admitted(value); // Separate closed scopes may reuse a local spelling.
}

#[test]
fn unrelated_signatures_and_action_sequences_do_not_acquire_the_unit_facade() {
    let edits: [EditCase; 7] = [
        ("internal", |v| {
            v["stateful_circuits"][0]["internal"] = json!(true)
        }),
        ("parameter", |v| {
            v["stateful_circuits"][0]["parameters"] =
                json!([{"name":"unused","ty":{"kind":"field"}}])
        }),
        ("Boolean result", |v| {
            v["stateful_circuits"][0]["result"] = json!({"kind":"boolean"});
            v["stateful_circuits"][0]["return_value"] =
                json!({"kind":"expression","value":{"kind":"boolean","value":true}});
        }),
        ("noncanonical Unit return", |v| {
            v["stateful_circuits"][0]["return_value"] =
                json!({"kind":"expression","value":{"kind":"unit"}})
        }),
        ("one action", |v| {
            v["stateful_circuits"][0]["actions"]
                .as_array_mut()
                .unwrap()
                .pop();
        }),
        ("reverse order", |v| {
            v["stateful_circuits"][0]["actions"]
                .as_array_mut()
                .unwrap()
                .reverse()
        }),
        ("extra effect", |v| {
            v["stateful_circuits"][0]["actions"].as_array_mut().unwrap().push(json!({"kind":"set_remove","field":"setB","index":1,"value":{"kind":"field_literal","value":"10"}}))
        }),
    ];
    for (label, edit) in edits {
        let mut value = original();
        edit(&mut value);
        not_admitted(value, label);
    }
}

#[test]
fn let_bindings_must_be_one_closed_typed_literal_consumed_in_their_own_scope() {
    let edits: [EditCase; 8] = [
        ("wrong binding type", |v| {
            v["stateful_circuits"][0]["actions"][0]["bindings"][0]["ty"] = json!({"kind":"boolean"})
        }),
        ("wrong local name", |v| {
            v["stateful_circuits"][0]["actions"][0]["action"]["value"]["name"] = json!("missing")
        }),
        ("other action scope", |v| {
            v["stateful_circuits"][0]["actions"][1]["action"]["value"]["name"] = json!("tmp_1")
        }),
        ("self reference", |v| {
            v["stateful_circuits"][0]["actions"][0]["bindings"][0]["value"] =
                json!({"kind":"parameter","name":"tmp_1"})
        }),
        ("extra binding", |v| {
            v["stateful_circuits"][0]["actions"][0]["bindings"].as_array_mut().unwrap().push(json!({"name":"unused","ty":{"kind":"field"},"value":{"kind":"field_literal","value":"1"}}))
        }),
        ("guessed coercion", |v| {
            v["stateful_circuits"][0]["actions"][0]["bindings"][0]["value"] = json!({"kind":"coerce","ty":{"kind":"field"},"value":{"kind":"field_literal","value":"10"}})
        }),
        ("nested let", |v| {
            let old = v["stateful_circuits"][0]["actions"][0].clone();
            v["stateful_circuits"][0]["actions"][0] = json!({"kind":"let","bindings":[{"name":"outer","ty":{"kind":"field"},"value":{"kind":"field_literal","value":"1"}}],"action":old});
        }),
        ("witness value", |v| {
            v["witnesses"] =
                json!([{"name":"secret_value","parameters":[],"result":{"kind":"field"}}]);
            v["stateful_circuits"][0]["actions"][0]["bindings"][0]["value"] =
                json!({"kind":"call","name":"secret_value","arguments":[]});
        }),
    ];
    for (label, edit) in edits {
        let mut value = original();
        edit(&mut value);
        not_admitted(value, label);
    }
}

#[test]
fn wrong_declared_slots_types_and_unsupported_actions_are_fail_closed() {
    let edits: [EditCase; 5] = [
        ("wrong action index", |v| {
            v["stateful_circuits"][0]["actions"][0]["action"]["index"] = json!(1)
        }),
        ("unknown field", |v| {
            v["stateful_circuits"][0]["actions"][0]["action"]["field"] = json!("missing")
        }),
        ("non-Set declaration", |v| {
            v["ledger_fields"][0]["declaration"]["kind"] = json!("cell")
        }),
        ("overlapping physical slots", |v| {
            v["ledger_fields"][1]["path"] = json!([0])
        }),
        ("Set reset", |v| {
            v["stateful_circuits"][0]["actions"][1]["action"] =
                json!({"kind":"set_reset","field":"setB","index":1})
        }),
    ];
    for (label, edit) in edits {
        let mut value = original();
        edit(&mut value);
        not_admitted(value, label);
    }
}

#[test]
fn unit_helper_collisions_omit_only_the_optional_api() {
    for internal in [false, true] {
        let mut value = original();
        let mut collision = value["stateful_circuits"][0].clone();
        collision["name"] = json!("testGasCost_proof_data");
        collision["internal"] = json!(internal);
        value["stateful_circuits"]
            .as_array_mut()
            .unwrap()
            .push(collision);
        let out = render(value);
        let c = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "testGasCost")
            .unwrap();
        assert!(c.recorded && c.observed_call);
        assert!(c.recording_unavailable.is_none() && c.observed_call_unavailable.is_none());
        assert!(c.standalone_proof_data.is_none());
        assert_eq!(
            c.standalone_proof_data_unavailable.as_ref().unwrap().code,
            "name_collision"
        );
        assert!(owned_method(&out.source, "testGasCost").is_some());
        assert!(owned_method(&out.source, "testGasCost_call").is_some());
    }
}

#[test]
fn unit_metadata_keeps_the_existing_schema_join_and_macro_free_handshake() {
    let (out, method) = admitted(original());
    assert_eq!(out.capabilities.schema_version, 2);
    let mut report = out.capabilities.clone();
    let before = serde_json::to_value(&report).unwrap();
    report
        .apply_contract_info(&json!({"circuits":[{"name":"testGasCost","proof":true}]}))
        .unwrap();
    assert_eq!(report.schema_version, 3);
    let after = serde_json::to_value(&report).unwrap();
    for field in ["recorded", "observed_call", "standalone_proof_data"] {
        assert_eq!(before["circuits"][0][field], after["circuits"][0][field]);
    }
    struct Macros(usize);
    impl syn::visit_mut::VisitMut for Macros {
        fn visit_macro_mut(&mut self, item: &mut syn::Macro) {
            self.0 += 1;
            syn::visit_mut::visit_macro_mut(self, item);
        }
    }
    let mut macros = Macros(0);
    syn::visit_mut::VisitMut::visit_block_mut(&mut macros, &mut method.block.clone());
    assert_eq!(
        macros.0, 0,
        "new facade must not make helper reachability opaque"
    );
}
