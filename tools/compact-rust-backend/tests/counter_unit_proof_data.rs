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

//! Host-only admission/AST tests over genuine pinned Counter frontend IR.
//! No Compact assertion, generated Rust, SDK, model or proof executes here.
use compact_rust_backend::{
    StandaloneProofDataRender, ir::Contract, render_with_standalone_proof_data,
};
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/adt-counter/compact-rust-ir.json"
    ))
    .unwrap()
}

fn render(value: &Value) -> StandaloneProofDataRender {
    render_with_standalone_proof_data(&serde_json::from_value::<Contract>(value.clone()).unwrap())
        .unwrap()
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

fn admitted(value: &Value) -> StandaloneProofDataRender {
    let out = render(value);
    for row in value["stateful_circuits"].as_array().unwrap() {
        if row["internal"] == true {
            continue;
        }
        let name = row["name"].as_str().unwrap();
        let capability = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        assert!(capability.recorded && capability.observed_call);
        let proof = capability.standalone_proof_data.as_ref().unwrap();
        assert_eq!(proof.api_version, 1);
        let method = owned_method(&out.source, &proof.method).unwrap();
        assert_eq!(proof.method, method.sig.ident.to_string());
    }
    out
}

fn omitted(value: Value, circuit: &str, label: &str) {
    let contract: Contract = serde_json::from_value(value).unwrap();
    // Invalid typed/path inputs may be refused by existing native validation.
    // Private tests independently exercise the new classifier's own gates.
    if let Ok(out) = render_with_standalone_proof_data(&contract) {
        let c = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == circuit)
            .unwrap();
        assert!(c.standalone_proof_data.is_none(), "{label}");
    }
}

fn strings(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(s) if s.as_str() == from => *s = to.to_owned(),
        Value::Array(rows) => {
            for row in rows {
                strings(row, from, to);
            }
        }
        Value::Object(row) => {
            for child in row.values_mut() {
                strings(child, from, to);
            }
        }
        _ => (),
    }
}

fn assertion_count(value: &Value) -> usize {
    match value {
        Value::Object(row) => {
            (if row.get("kind") == Some(&json!("assert")) {
                1
            } else {
                0
            }) + row.values().map(assertion_count).sum::<usize>()
        }
        Value::Array(rows) => rows.iter().map(assertion_count).sum(),
        _ => 0,
    }
}

#[test]
fn genuine_original_two_unit_facades_capture_once_without_changing_assertions() {
    let value = original();
    assert_eq!(assertion_count(&value["stateful_circuits"][0]), 5);
    assert_eq!(assertion_count(&value["stateful_circuits"][1]), 2);
    let out = admitted(&value);
    for name in ["test_all", "test_big_increment"] {
        let method = owned_method(&out.source, &format!("{name}_proof_data")).unwrap();
        assert_eq!(method.sig.inputs.len(), 2); // self/context only, no public input.
        assert_eq!(method.block.stmts.len(), 4);
        let syn::Stmt::Item(syn::Item::Const(guard)) = &method.block.stmts[0] else {
            panic!("macro-free API guard")
        };
        assert!(quote::quote!(#guard).to_string().contains("API_VERSION"));
        let syn::Stmt::Local(input) = &method.block.stmts[1] else {
            panic!("input capture first")
        };
        let syn::Pat::Ident(input_name) = &input.pat else {
            panic!("input binding")
        };
        assert_eq!(input_name.ident, "input");
        let syn::Expr::Call(input_call) = input.init.as_ref().unwrap().expr.as_ref() else {
            panic!("typed Unit capture")
        };
        assert!(matches!(&input_call.args[0], syn::Expr::Tuple(unit) if unit.elems.is_empty()));
        let syn::Stmt::Local(recorded) = &method.block.stmts[2] else {
            panic!("one actual recorded call")
        };
        let syn::Pat::Ident(recorded_name) = &recorded.pat else {
            panic!("recorded binding")
        };
        assert_eq!(recorded_name.ident, "recorded");
        let syn::Expr::Try(call) = recorded.init.as_ref().unwrap().expr.as_ref() else {
            panic!("recorded error propagation")
        };
        let syn::Expr::MethodCall(call) = call.expr.as_ref() else {
            panic!("one real method call")
        };
        assert_eq!(call.method, name);
        assert_eq!(call.args.len(), 1);
        let syn::Stmt::Expr(syn::Expr::Call(convert), None) = &method.block.stmts[3] else {
            panic!("actual recorded conversion")
        };
        assert_eq!(convert.args.len(), 3);
        assert!(
            matches!(&convert.args[0], syn::Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(s) if s.value() == name))
        );
        assert!(matches!(&convert.args[1], syn::Expr::Path(p) if p.path.is_ident("input")));
        assert!(matches!(&convert.args[2], syn::Expr::Path(p) if p.path.is_ident("recorded")));
        // Result type structure ignores harmless prettyprinter trailing commas.
        let syn::ReturnType::Type(_, ty) = &method.sig.output else {
            panic!("Result type")
        };
        let syn::Type::Path(result) = ty.as_ref() else {
            panic!("Result path")
        };
        let segment = result.path.segments.last().unwrap();
        assert_eq!(segment.ident, "Result");
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            panic!("Result arguments")
        };
        assert_eq!(args.args.len(), 2);
    }
    for message in [
        "Initial value is 0",
        "Value is 0",
        "Value is 5",
        "Value is 1",
        "Value is as defined",
    ] {
        assert!(out.source.contains(message));
    }
}

#[test]
fn renaming_and_consistent_root_slot_permutation_are_semantic() {
    let mut value = original();
    for (from, to) in [
        ("test_all", "inspect$counter"),
        ("test_big_increment", "advance"),
        ("c", "left"),
        ("c2", "right"),
        ("tmp_18", "amount"),
        ("t_15", "product"),
    ] {
        strings(&mut value, from, to);
    }
    fn swap(value: &mut Value) {
        match value {
            Value::Object(row) => {
                if let Some(index) = row.get_mut("index") {
                    *index = json!(1 - index.as_u64().unwrap());
                }
                if let Some(path) = row.get_mut("path") {
                    path[0] = json!(1 - path[0].as_u64().unwrap());
                }
                for v in row.values_mut() {
                    swap(v);
                }
            }
            Value::Array(rows) => {
                for row in rows {
                    swap(row);
                }
            }
            _ => (),
        }
    }
    swap(&mut value);
    for circuit in value["stateful_circuits"].as_array_mut().unwrap() {
        for action in circuit["actions"].as_array_mut().unwrap() {
            if action["kind"] == "assert" {
                let left = action["condition"]["left"].clone();
                action["condition"]["left"] = action["condition"]["right"].clone();
                action["condition"]["right"] = left;
            }
        }
    }
    let out = admitted(&value);
    let cap = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "inspect$counter")
        .unwrap();
    assert_eq!(
        cap.standalone_proof_data.as_ref().unwrap().method,
        "inspect_counter_proof_data"
    );
}

#[test]
fn safe_alternative_literals_and_ordered_shadowing_are_admitted() {
    let mut value = original();
    strings(&mut value, "16", "2"); // Arithmetic remains checked, different safe values.
    let action = &mut value["stateful_circuits"][0]["actions"][3];
    action["bindings"] = json!([
        {"name":"saved","ty":{"kind":"unsigned","max":"65535"},"value":{"kind":"unsigned_literal","value":"5","max":"65535"}},
        {"name":"saved","ty":{"kind":"unsigned","max":"65535"},"value":{"kind":"parameter","name":"saved"}},
        {"name":"tmp_18","ty":{"kind":"unsigned","max":"65535"},"value":{"kind":"parameter","name":"saved"}}
    ]);
    admitted(&value);
}

#[test]
fn unsupported_types_slots_actions_calls_and_conditionals_omit_optional_helpers() {
    for (label, action) in [
        (
            "call",
            json!({"kind":"circuit_call","name":"test_big_increment","arguments":[]}),
        ),
        (
            "pure helper",
            json!({"kind":"pure_call","name":"unknown","arguments":[]}),
        ),
        (
            "mixed write",
            json!({"kind":"set_reset","field":"c","index":0}),
        ),
        (
            "branch",
            json!({"kind":"if","condition":{"kind":"boolean","value":false},"then":{"kind":"counter_reset","field":"c","index":0},"otherwise":{"kind":"counter_reset","field":"c","index":0}}),
        ),
    ] {
        let mut value = original();
        value["stateful_circuits"][0]["actions"][1] = action;
        omitted(value, "test_all", label);
    }
    for label in ["wrong kind", "index", "aliased path"] {
        let mut value = original();
        match label {
            "wrong kind" => {
                value["ledger_fields"][0]["declaration"] =
                    json!({"kind":"set","ty":{"kind":"field"}})
            }
            "index" => value["stateful_circuits"][0]["actions"][1]["index"] = json!(1),
            "aliased path" => value["ledger_fields"][1]["path"] = json!([0]),
            _ => unreachable!(),
        }
        omitted(value, "test_all", label);
    }
}

#[test]
fn constant_scope_and_arithmetic_mutations_never_gain_helpers() {
    for label in [
        "free",
        "wrong amount type",
        "literal range",
        "underflow",
        "overflow",
        "dead witness",
    ] {
        let mut value = original();
        let binding = &mut value["stateful_circuits"][1]["actions"][1]["bindings"][0];
        match label {
            "free" => binding["value"] = json!({"kind":"parameter","name":"outside"}),
            "wrong amount type" => binding["ty"] = json!({"kind":"field"}),
            "literal range" => {
                binding["value"] = json!({"kind":"unsigned_literal","value":"65536","max":"65535"})
            }
            "underflow" => {
                binding["value"] = json!({"kind":"unsigned_subtract","max":"65535","left":{"kind":"unsigned_literal","value":"0","max":"65535"},"right":{"kind":"unsigned_literal","value":"1","max":"65535"}})
            }
            "overflow" => {
                binding["value"] = json!({"kind":"unsigned_multiply","max":"65535","left":{"kind":"unsigned_literal","value":"65535","max":"65535"},"right":{"kind":"unsigned_literal","value":"2","max":"65535"}})
            }
            "dead witness" => {
                binding["value"] = json!({"kind":"if","condition":{"kind":"boolean","value":true},"then":{"kind":"unsigned_literal","value":"5","max":"65535"},"otherwise":{"kind":"call","name":"witness","arguments":[]}})
            }
            _ => unreachable!(),
        }
        omitted(value, "test_big_increment", label);
    }
}

#[test]
fn raw_internal_and_sanitized_collisions_preserve_old_recorded_methods() {
    for internal in [false, true] {
        for sanitized in [false, true] {
            let mut value = original();
            if sanitized {
                strings(&mut value, "test_all", "sum$all");
            }
            let old_name = if sanitized { "sum_all" } else { "test_all" };
            let external = if sanitized { "sum$all" } else { "test_all" };
            let before = admitted(&value);
            let before = owned_method(&before.source, old_name).unwrap();
            value["stateful_circuits"].as_array_mut().unwrap().push(json!({
                "name":format!("{old_name}_proof_data"),"internal":internal,
                "parameters":[],"result":{"kind":"unit"},"return_value":{"kind":"unit"},"actions":[]
            }));
            let out = render(&value);
            let cap = out
                .capabilities
                .circuits
                .iter()
                .find(|c| c.name == external)
                .unwrap();
            assert!(cap.recorded && cap.observed_call && cap.standalone_proof_data.is_none());
            assert_eq!(
                cap.standalone_proof_data_unavailable.as_ref().unwrap().code,
                "name_collision"
            );
            let after = owned_method(&out.source, old_name).unwrap();
            assert_eq!(
                quote::quote!(#before).to_string(),
                quote::quote!(#after).to_string()
            );
        }
    }
}

#[test]
fn counter_helpers_keep_macro_free_pruning_and_optional_schema_join() {
    let out = admitted(&original());
    assert!(!out.source.contains("fn __compact_recorded_body_"));
    // Optional helpers do not classify compiler proof readiness on their own.
    assert_eq!(out.capabilities.schema_version, 2);
    assert!(
        out.capabilities
            .circuits
            .iter()
            .all(|c| c.proof_required.is_none())
    );
}
