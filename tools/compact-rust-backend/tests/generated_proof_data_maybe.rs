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

//! Host-only checks from the genuine pinned-original Tiny frontend JSON.
//! Original fixtures are immutable; renamed/negative values below are supplemental.
use compact_rust_backend::{
    StandaloneProofDataRender, ir::Contract, render_with_standalone_proof_data,
};
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/tiny-get/compact-rust-ir.json"
    ))
    .unwrap()
}
fn render(value: Value) -> StandaloneProofDataRender {
    render_with_standalone_proof_data(&serde_json::from_value::<Contract>(value).unwrap()).unwrap()
}
fn circuit_mut<'a>(value: &'a mut Value, name: &str) -> &'a mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == name)
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
fn no_api(value: Value, name: &str, old_recorded_must_remain: bool) {
    let typed: Contract = serde_json::from_value(value).unwrap();
    match render_with_standalone_proof_data(&typed) {
        Ok(out) => {
            let capability = out
                .capabilities
                .circuits
                .iter()
                .find(|c| c.name == name)
                .unwrap();
            if old_recorded_must_remain {
                assert!(capability.recorded);
            }
            assert!(capability.standalone_proof_data.is_none());
            assert!(owned_method(&out.source, &format!("{name}_proof_data")).is_none());
        }
        Err(error) => assert!(
            !old_recorded_must_remain,
            "old API must still render: {error}"
        ),
    }
}
fn replace_strings(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(s) if s == old => *s = new.to_owned(),
        Value::Array(values) => values.iter_mut().for_each(|v| replace_strings(v, old, new)),
        Value::Object(values) => values
            .values_mut()
            .for_each(|v| replace_strings(v, old, new)),
        _ => {}
    }
}

#[test]
fn genuine_get_returns_actual_concrete_maybe_and_captures_before_one_call() {
    let mut value = original();
    let get = circuit_mut(&mut value, "get");
    assert_eq!(get["parameters"], json!([]));
    assert_eq!(get["actions"], json!([]));
    assert_eq!(
        get["result"]["fields"],
        json!([
            {"name":"is_some","ty":{"kind":"boolean"}},
            {"name":"value","ty":{"kind":"field"}}
        ])
    );
    let out = render(value);
    let capability = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "get")
        .unwrap();
    assert!(capability.recorded && capability.observed_call);
    let api = capability.standalone_proof_data.as_ref().unwrap();
    assert_eq!(api.api_version, 1);
    assert_eq!(api.method, "get_proof_data");
    assert!(capability.standalone_proof_data_unavailable.is_none());
    assert_eq!(
        out.capabilities
            .circuits
            .iter()
            .filter(|c| c.standalone_proof_data.is_some())
            .count(),
        2 // getter plus the separately admitted authenticated clear; set stays absent
    );
    let method = owned_method(&out.source, "get_proof_data").unwrap();
    assert_eq!(method.sig.inputs.len(), 2); // self + context; no public/private/witness argument
    let syn::ReturnType::Type(_, result) = &method.sig.output else {
        panic!("Result carrier")
    };
    let syn::Type::Path(result) = result.as_ref() else {
        panic!("Result path")
    };
    let syn::PathArguments::AngleBracketed(arguments) =
        &result.path.segments.last().unwrap().arguments
    else {
        panic!("Result arguments")
    };
    let syn::GenericArgument::Type(syn::Type::Path(data)) = arguments.args.first().unwrap() else {
        panic!("RecordedProofData")
    };
    assert_eq!(
        data.path.segments.last().unwrap().ident,
        "RecordedProofData"
    );
    let syn::PathArguments::AngleBracketed(arguments) =
        &data.path.segments.last().unwrap().arguments
    else {
        panic!("data arguments")
    };
    assert_eq!(arguments.args.len(), 2);
    let syn::GenericArgument::Type(output) = arguments.args.iter().nth(1).unwrap() else {
        panic!("typed actual result")
    };
    assert_eq!(
        quote::quote!(#output).to_string(),
        "crate :: types :: Maybe"
    );
    assert_eq!(method.block.stmts.len(), 4);
    let syn::Stmt::Item(syn::Item::Const(guard)) = &method.block.stmts[0] else {
        panic!("macro-free API guard")
    };
    let expected: syn::ItemConst =
        syn::parse_quote! { const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize]; };
    assert_eq!(
        quote::quote!(#guard).to_string(),
        quote::quote!(#expected).to_string()
    );
    let syn::Stmt::Local(input) = &method.block.stmts[1] else {
        panic!("Unit captured first")
    };
    let syn::Expr::Call(pack) = input.init.as_ref().unwrap().expr.as_ref() else {
        panic!("typed FAB capture")
    };
    assert!(matches!(pack.args.first().unwrap(), syn::Expr::Tuple(t) if t.elems.is_empty()));
    let syn::Stmt::Local(recorded) = &method.block.stmts[2] else {
        panic!("one recorded call")
    };
    let syn::Expr::Try(call) = recorded.init.as_ref().unwrap().expr.as_ref() else {
        panic!("execution errors propagate")
    };
    let syn::Expr::MethodCall(call) = call.expr.as_ref() else {
        panic!("actual recording method")
    };
    assert_eq!(call.method, "get");
    assert_eq!(call.args.len(), 1);
    let body = quote::quote!(#method).to_string();
    assert_eq!(body.matches("self . get (").count(), 1);
    let syn::Stmt::Expr(syn::Expr::Call(conversion), None) = &method.block.stmts[3] else {
        panic!("tail conversion returns the actual capture")
    };
    let syn::Expr::Path(function) = conversion.func.as_ref() else {
        panic!("associated from_recorded conversion")
    };
    assert_eq!(
        function
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>(),
        [
            "runtime",
            "proof_data",
            "RecordedProofData",
            "from_recorded"
        ]
    );
    assert_eq!(conversion.args.len(), 3);
    assert!(matches!(&conversion.args[0], syn::Expr::Lit(lit)
        if matches!(&lit.lit, syn::Lit::Str(name) if name.value() == "get")));
    assert!(matches!(&conversion.args[1], syn::Expr::Path(path) if path.path.is_ident("input")));
    assert!(matches!(&conversion.args[2], syn::Expr::Path(path) if path.path.is_ident("recorded")));
    assert!(!body.contains("Some") && !body.contains("expected") && !body.contains("witness"));
}

#[test]
fn semantic_export_helper_and_field_renames_do_not_depend_on_fixture_names() {
    let mut value = original();
    for (old, new) in [
        ("get", "retrieve"),
        ("in_state", "ready"),
        ("some", "present"),
        ("none", "absent"),
        ("state", "status"),
        ("value", "stored"),
    ] {
        // Renaming "value" also renames the Maybe field, which is its public
        // standard-library shape. Keep that field canonical below.
        replace_strings(&mut value, old, new);
    }
    fn restore_maybe(value: &mut Value) {
        match value {
            Value::Object(object) => {
                if object.get("kind") == Some(&json!("struct"))
                    && object.get("name") == Some(&json!("Maybe"))
                {
                    object.get_mut("fields").unwrap().as_array_mut().unwrap()[1]["name"] =
                        json!("value");
                }
                object.values_mut().for_each(restore_maybe);
            }
            Value::Array(values) => values.iter_mut().for_each(restore_maybe),
            _ => {}
        }
    }
    restore_maybe(&mut value);
    let out = render(value);
    let capability = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "retrieve")
        .unwrap();
    assert_eq!(
        capability.standalone_proof_data.as_ref().unwrap().method,
        "retrieve_proof_data"
    );
    let method = owned_method(&out.source, "retrieve_proof_data").unwrap();
    let body = quote::quote!(#method).to_string();
    assert!(body.contains("from_recorded (\"retrieve\""));
    assert_eq!(body.matches("self . retrieve (").count(), 1);
}

#[test]
fn transitive_stateful_write_or_witness_removes_only_optional_get_facade() {
    let mut writes = original();
    circuit_mut(&mut writes, "in_state")["actions"] = json!([
        {"kind":"cell_write","field":"value","index":1,"value":{"kind":"field_literal","value":"0"}}
    ]);
    no_api(writes, "get", true);
    let mut witnesses = original();
    witnesses["witnesses"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"probe","parameters":[],"result":{"kind":"boolean"}}));
    let helper = circuit_mut(&mut witnesses, "in_state");
    let then = helper["return_value"]["value"].clone();
    helper["return_value"]["value"] = json!({"kind":"if","condition":{"kind":"witness_call","name":"probe","arguments":[]},"then":then,"otherwise":{"kind":"boolean","value":false}});
    no_api(witnesses, "get", true);
}

#[test]
fn constant_struct_wrong_carrier_or_nonidentity_coercion_never_claims_getter() {
    let mut constant = original();
    circuit_mut(&mut constant, "get")["return_value"]["value"] =
        json!({"kind":"call","name":"none","arguments":[]});
    no_api(constant, "get", true); // a Maybe signature alone is not a ledger getter
    let mut unrelated = original();
    replace_strings(&mut unrelated, "Maybe", "UnrelatedPair");
    no_api(unrelated, "get", true);
    let mut coercion = original();
    circuit_mut(&mut coercion, "get")["return_value"]["value"]["then"]["arguments"][0]["ty"] =
        json!({"kind":"boolean"});
    no_api(coercion, "get", false); // existing native validation may reject first
}

#[test]
fn recursive_or_unresolved_transitive_calls_are_not_admitted() {
    let mut recursive = original();
    circuit_mut(&mut recursive, "in_state")["return_value"]["value"] =
        json!({"kind":"call","name":"in_state","arguments":[{"kind":"parameter","name":"s"}]});
    no_api(recursive, "get", false);
    let mut missing = original();
    circuit_mut(&mut missing, "get")["return_value"]["value"]["otherwise"]["name"] =
        json!("absent_unknown_helper");
    no_api(missing, "get", false);
}

#[test]
fn public_or_private_helper_name_collision_keeps_old_get_interfaces() {
    for internal in [false, true] {
        let mut value = original();
        let mut collider = circuit_mut(&mut value, "get").clone();
        collider["name"] = json!("get_proof_data");
        collider["internal"] = json!(internal);
        value["stateful_circuits"]
            .as_array_mut()
            .unwrap()
            .push(collider);
        let out = render(value);
        let capability = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "get")
            .unwrap();
        assert!(capability.recorded && capability.observed_call);
        assert!(capability.standalone_proof_data.is_none());
        assert_eq!(
            capability
                .standalone_proof_data_unavailable
                .as_ref()
                .unwrap()
                .code,
            "name_collision"
        );
        assert!(owned_method(&out.source, "get").is_some());
        assert!(owned_method(&out.source, "get_call").is_some());
    }
}

#[test]
fn concrete_compiler_maybe_instantiation_name_is_preserved_in_output_type() {
    let mut value = original();
    replace_strings(&mut value, "Maybe", "MaybeCompact1");
    let out = render(value);
    let method = owned_method(&out.source, "get_proof_data").unwrap();
    let output = &method.sig.output;
    assert!(
        quote::quote!(#output)
            .to_string()
            .contains("crate :: types :: MaybeCompact1")
    );
}

#[test]
fn equivalent_inlined_read_graph_is_supported_but_unknown_pure_body_is_not() {
    let mut value = original();
    let actual = circuit_mut(&mut value, "get")["return_value"]["value"].clone();
    let maybe = circuit_mut(&mut value, "get")["result"].clone();
    let field_read = actual["then"]["arguments"][0].clone();
    let condition = actual["condition"].clone();
    circuit_mut(&mut value, "get")["return_value"]["value"] = json!({
        "kind":"if","condition":condition,
        "then":{"kind":"struct_literal","ty":maybe,"fields":[{"kind":"boolean","value":true},field_read]},
        "otherwise":{"kind":"struct_literal","ty":maybe,"fields":[{"kind":"boolean","value":false},{"kind":"field_literal","value":"0"}]}
    });
    let out = render(value);
    assert!(owned_method(&out.source, "get_proof_data").is_some());
    let mut unknown = original();
    unknown["circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "none")
        .unwrap()["body"]["fields"][1] = json!({"kind":"add","left":{"kind":"field_literal","value":"0"},"right":{"kind":"field_literal","value":"0"}});
    no_api(unknown, "get", true); // ordinary pure arithmetic is valid but outside this closed family
}

#[test]
fn repeated_acyclic_pure_dag_retains_existing_renderer_resource_refusal() {
    let mut value = original();
    value["circuits"].as_array_mut().unwrap().push(json!({
        "name":"fan_base","internal":true,"parameters":[{"name":"x","ty":{"kind":"field"}}],
        "result":{"kind":"field"},"body":{"kind":"parameter","name":"x"}
    }));
    let mut previous = "fan_base".to_owned();
    for level in 0..13 {
        let name = format!("fan_{level}");
        let call =
            json!({"kind":"call","name":previous,"arguments":[{"kind":"parameter","name":"x"}]});
        value["circuits"].as_array_mut().unwrap().push(json!({
            "name":name,"internal":true,"parameters":[{"name":"x","ty":{"kind":"field"}}],
            "result":{"kind":"field"},
            "body":{"kind":"if","condition":{"kind":"boolean","value":true},"then":call,"otherwise":call}
        }));
        previous = name;
    }
    let get = circuit_mut(&mut value, "get");
    let read = get["return_value"]["value"]["then"]["arguments"][0].clone();
    get["return_value"]["value"]["then"]["arguments"][0] =
        json!({"kind":"call","name":previous,"arguments":[read]});
    // Renderer admission precedes optional helper classification. This fixture
    // reaches the existing render-path limit, not the new classifier budget.
    let typed: Contract = serde_json::from_value(value).unwrap();
    let error = render_with_standalone_proof_data(&typed)
        .err()
        .expect("existing renderer limit");
    assert!(matches!(
        error,
        compact_rust_backend::RenderError::ResourceLimit {
            resource: "legacy_recorded_render_path_units",
            limit: 1664,
            observed: 1745
        }
    ));
}
