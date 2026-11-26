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

//! Generator admission only. The complete six-export contract stays intact.
use compact_rust_backend::{ir::Contract, render_with_standalone_proof_data};
use quote::ToTokens;
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/export-set-field/compact-rust-ir.json"
    ))
    .unwrap()
}
fn circuit<'a>(value: &'a mut Value, name: &str) -> &'a mut Value {
    value["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == name)
        .unwrap()
}
fn methods(source: &str, owner: &str) -> Vec<syn::ImplItemFn> {
    fn module<'a>(items: &'a [syn::Item], name: &str) -> &'a [syn::Item] {
        items
            .iter()
            .find_map(|item| match item {
                syn::Item::Mod(m) if m.ident == name => {
                    m.content.as_ref().map(|(_, items)| items.as_slice())
                }
                _ => None,
            })
            .unwrap()
    }
    let syntax = syn::parse_file(source).unwrap();
    module(module(&syntax.items, "ledger_contract"), "recorded")
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(i)
                if matches!(i.self_ty.as_ref(), syn::Type::Path(p)
                if p.path.segments.last().unwrap().ident == owner) =>
            {
                Some(&i.items)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(f) => Some(f.clone()),
            _ => None,
        })
        .collect()
}
enum ReadResult {
    Boolean,
    Uint64,
}
fn assert_read_capture(
    method: &syn::ImplItemFn,
    name: &str,
    parameter: Option<&str>,
    output: ReadResult,
) {
    assert_eq!(method.sig.ident, format!("{name}_proof_data"));
    assert_eq!(
        method.sig.inputs.len(),
        if parameter.is_some() { 3 } else { 2 }
    );
    assert!(method.sig.generics.where_clause.is_none());
    assert!(
        method
            .sig
            .generics
            .type_params()
            .all(|p| p.bounds.is_empty())
    );
    let syn::ReturnType::Type(_, result) = &method.sig.output else {
        panic!("Result output")
    };
    let syn::Type::Path(result) = result.as_ref() else {
        panic!("Result path")
    };
    let result = result.path.segments.last().unwrap();
    assert_eq!(result.ident, "Result");
    let syn::PathArguments::AngleBracketed(args) = &result.arguments else {
        panic!("Result arguments")
    };
    assert_eq!(args.args.len(), 2);
    let syn::GenericArgument::Type(syn::Type::Path(carrier)) = &args.args[0] else {
        panic!("typed carrier")
    };
    let carrier = carrier.path.segments.last().unwrap();
    assert_eq!(carrier.ident, "RecordedProofData");
    let syn::PathArguments::AngleBracketed(args) = &carrier.arguments else {
        panic!("carrier arguments")
    };
    assert_eq!(args.args.len(), 2);
    assert_eq!(args.args[0].to_token_stream().to_string(), "Private");
    let syn::GenericArgument::Type(syn::Type::Path(value)) = &args.args[1] else {
        panic!("typed result")
    };
    match output {
        ReadResult::Boolean => {
            assert_eq!(value.path.segments.len(), 1);
            assert_eq!(value.path.segments[0].ident, "bool");
        }
        ReadResult::Uint64 => {
            assert_eq!(value.path.segments.len(), 2);
            assert_eq!(value.path.segments[0].ident, "runtime");
            assert_eq!(value.path.segments[1].ident, "BoundedUint");
            let syn::PathArguments::AngleBracketed(args) = &value.path.segments[1].arguments else {
                panic!("unsigned maximum")
            };
            assert_eq!(args.args.len(), 1);
            let syn::GenericArgument::Const(syn::Expr::Lit(literal)) = &args.args[0] else {
                panic!("literal bound")
            };
            let syn::Lit::Int(maximum) = &literal.lit else {
                panic!("integer bound")
            };
            assert_eq!(
                maximum.base10_parse::<u128>().unwrap(),
                u128::from(u64::MAX)
            );
        }
    }
    if let Some(parameter) = parameter {
        let syn::FnArg::Typed(argument) = method.sig.inputs.last().unwrap() else {
            panic!("Field input")
        };
        assert_eq!(argument.pat.to_token_stream().to_string(), parameter);
        assert_eq!(
            argument.ty.to_token_stream().to_string(),
            "runtime :: Field"
        );
    }
    assert_eq!(method.block.stmts.len(), 4);
    let syn::Stmt::Item(syn::Item::Const(api)) = &method.block.stmts[0] else {
        panic!("API1 guard")
    };
    assert!(
        api.to_token_stream()
            .to_string()
            .contains("API_VERSION as usize")
    );
    let syn::Stmt::Local(input) = &method.block.stmts[1] else {
        panic!("typed input capture")
    };
    let syn::Expr::Call(aligned) = input.init.as_ref().unwrap().expr.as_ref() else {
        panic!("input conversion")
    };
    assert_eq!(
        aligned.func.to_token_stream().to_string(),
        "runtime :: fab :: AlignedValue :: from"
    );
    assert_eq!(aligned.args.len(), 1);
    if let Some(parameter) = parameter {
        assert_eq!(aligned.args[0].to_token_stream().to_string(), parameter);
    } else {
        assert!(matches!(&aligned.args[0], syn::Expr::Tuple(unit) if unit.elems.is_empty()));
    }
    let syn::Stmt::Local(recorded) = &method.block.stmts[2] else {
        panic!("actual recorded call")
    };
    let syn::Expr::Try(attempt) = recorded.init.as_ref().unwrap().expr.as_ref() else {
        panic!("propagate refusal before carrier")
    };
    let syn::Expr::MethodCall(call) = attempt.expr.as_ref() else {
        panic!("one actual method call")
    };
    assert_eq!(call.receiver.to_token_stream().to_string(), "self");
    assert_eq!(call.method, name);
    assert_eq!(call.args.len(), if parameter.is_some() { 2 } else { 1 });
    assert_eq!(call.args[0].to_token_stream().to_string(), "context");
    if let Some(parameter) = parameter {
        assert_eq!(call.args[1].to_token_stream().to_string(), parameter);
    }
    let syn::Stmt::Expr(syn::Expr::Call(carrier), None) = &method.block.stmts[3] else {
        panic!("returned carrier after actual call")
    };
    assert_eq!(
        carrier.func.to_token_stream().to_string(),
        "runtime :: proof_data :: RecordedProofData :: from_recorded"
    );
    assert_eq!(carrier.args.len(), 3);
    assert!(
        matches!(&carrier.args[0], syn::Expr::Lit(l) if matches!(&l.lit, syn::Lit::Str(s) if s.value() == name))
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
fn original_member_and_size_capture_exact_typed_inputs_and_outputs_without_bounds() {
    let contract: Contract = serde_json::from_value(original()).unwrap();
    assert_eq!(contract.stateful_circuits.len(), 6);
    let out = render_with_standalone_proof_data(&contract).unwrap();
    let generated = methods(&out.source, "Contract");
    for (name, parameter, ty) in [
        ("member", Some("value"), ReadResult::Boolean),
        ("size", None, ReadResult::Uint64),
    ] {
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == name)
            .unwrap();
        let api = row.standalone_proof_data.as_ref().unwrap();
        assert_eq!(api.api_version, 1);
        assert_eq!(api.method, format!("{name}_proof_data"));
        assert_read_capture(
            generated
                .iter()
                .find(|m| m.sig.ident == api.method)
                .unwrap(),
            name,
            parameter,
            ty,
        );
    }
    assert!(methods(&out.source, "BorrowedContract").is_empty());
}

#[test]
fn exact_expression_variants_keep_both_handles_free_of_unrelated_witness_bounds() {
    let mut value = original();
    for (name, renamed) in [("member", "contains"), ("size", "cardinality")] {
        let c = circuit(&mut value, name);
        c["name"] = json!(renamed);
        if name == "member" {
            c["parameters"][0]["name"] = json!("needle");
            c["return_value"]["value"]["name"] = json!("needle");
        }
        let read = c["return_value"].take();
        c["return_value"] = json!({"kind":"expression","value":read});
    }
    // A genuinely unrelated witness requires the generator's borrowed handle.
    value["witnesses"] = json!([{"name":"ask","parameters":[],"result":{"kind":"field"}}]);
    value["stateful_circuits"].as_array_mut().unwrap().push(json!({
        "name":"witnessed","parameters":[],"actions":[],"result":{"kind":"field"},
        "return_value":{"kind":"expression","value":{"kind":"witness_call","name":"ask","arguments":[]}}}));
    let out = render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
    for owner in ["Contract", "BorrowedContract"] {
        let generated = methods(&out.source, owner);
        assert_read_capture(
            generated
                .iter()
                .find(|m| m.sig.ident == "contains_proof_data")
                .unwrap(),
            "contains",
            Some("needle"),
            ReadResult::Boolean,
        );
        assert_read_capture(
            generated
                .iter()
                .find(|m| m.sig.ident == "cardinality_proof_data")
                .unwrap(),
            "cardinality",
            None,
            ReadResult::Uint64,
        );
    }
}

#[test]
fn broader_valid_recorded_queries_keep_ordinary_methods_without_optional_helpers() {
    for name in ["member", "size"] {
        for edit in [
            "boolean-element",
            "preceding-write",
            "branch",
            "extra-parameter",
        ] {
            let mut value = original();
            if edit == "boolean-element" {
                value["ledger_fields"].as_array_mut().unwrap().push(json!({"id":"flags","index":1,"path":[1],"declaration":{"kind":"set","ty":{"kind":"boolean"}}}));
            }
            let c = circuit(&mut value, name);
            match edit {
                "boolean-element" => {
                    c["return_value"]["field"] = json!("flags");
                    c["return_value"]["index"] = json!(1);
                    if name == "member" {
                        c["parameters"][0]["ty"] = json!({"kind":"boolean"});
                    }
                }
                "preceding-write" => {
                    c["actions"] = json!([{"kind":"set_insert","field":"c","index":0,"value":{"kind":"field_literal","value":"1"}}])
                }
                "branch" => {
                    let read = c["return_value"].take();
                    let fallback = if name == "member" {
                        json!({"kind":"boolean","value":false})
                    } else {
                        json!({"kind":"unsigned_literal","value":"0","max":"18446744073709551615"})
                    };
                    c["return_value"] = json!({"kind":"expression","value":{"kind":"if","condition":{"kind":"boolean","value":true},"then":read,"otherwise":fallback}});
                }
                "extra-parameter" => c["parameters"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"name":"unused","ty":{"kind":"field"}})),
                _ => unreachable!(),
            }
            let out =
                render_with_standalone_proof_data(&serde_json::from_value(value).unwrap()).unwrap();
            let row = out
                .capabilities
                .circuits
                .iter()
                .find(|c| c.name == name)
                .unwrap();
            assert!(row.recorded && row.observed_call, "{name}/{edit}");
            assert!(row.standalone_proof_data.is_none(), "{name}/{edit}");
            let generated = methods(&out.source, "Contract");
            assert!(generated.iter().any(|m| m.sig.ident == name));
            assert!(
                generated
                    .iter()
                    .any(|m| m.sig.ident == format!("{name}_call"))
            );
            assert!(
                !generated
                    .iter()
                    .any(|m| m.sig.ident == format!("{name}_proof_data"))
            );
        }
    }
}

#[test]
fn public_internal_and_sanitized_name_collisions_preserve_ordinary_recording() {
    for (name, source_name, collision, ordinary) in [
        ("member", "member", "member_proof_data", "member"),
        ("member", "mem$ber", "mem_ber_proof_data", "mem_ber"),
        ("size", "size", "size_proof_data", "size"),
        ("size", "si$ze", "si_ze_proof_data", "si_ze"),
    ] {
        for internal in [false, true] {
            let mut value = original();
            circuit(&mut value, name)["name"] = json!(source_name);
            value["stateful_circuits"].as_array_mut().unwrap().push(json!({"name":collision,"internal":internal,
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
                    .any(|m| m.sig.ident == ordinary)
            );
        }
    }
}
