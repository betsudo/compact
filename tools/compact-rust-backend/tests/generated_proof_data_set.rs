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

//! Host facade checks using the exact pinned Tiny IR. This does not run a contract.
use compact_rust_backend::{ir::Contract, render, render_with_standalone_proof_data};
use serde_json::{Value, json};

fn original() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/upstream-672/tiny-get/compact-rust-ir.json"
    ))
    .unwrap()
}

fn method<'a>(items: &'a [syn::Item], handle: &str, name: &str) -> Option<&'a syn::ImplItemFn> {
    let child = |items: &'a [syn::Item], name: &str| {
        items.iter().find_map(|item| match item {
            syn::Item::Mod(module) if module.ident == name => {
                module.content.as_ref().map(|(_, items)| items.as_slice())
            }
            _ => None,
        })
    };
    // Native Contract<W> is a different owner from recorded::Contract. Never
    // satisfy a recording-facade assertion with a native or unrelated method.
    let items = child(child(items, "ledger_contract")?, "recorded")?;
    for item in items {
        let syn::Item::Impl(implementation) = item else {
            continue;
        };
        let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
            continue;
        };
        if owner.path.segments.last().unwrap().ident != handle {
            continue;
        }
        for item in &implementation.items {
            if let syn::ImplItem::Fn(method) = item
                && method.sig.ident == name
            {
                return Some(method);
            }
        }
    }
    None
}

fn rename(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(s) if s == old => *s = new.to_owned(),
        Value::Array(a) => a.iter_mut().for_each(|v| rename(v, old, new)),
        Value::Object(o) => o.values_mut().for_each(|v| rename(v, old, new)),
        _ => (),
    }
}

#[test]
fn witnessed_set_captures_one_typed_field_then_calls_the_borrowed_recorded_method_once() {
    let contract: Contract = serde_json::from_value(original()).unwrap();
    let out = render_with_standalone_proof_data(&contract).unwrap();
    let row = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "set")
        .unwrap();
    assert!(row.recorded && row.observed_call);
    let api = row.standalone_proof_data.as_ref().unwrap();
    assert_eq!(api.api_version, 1);
    assert_eq!(api.method, "set_proof_data");
    let syntax = syn::parse_file(&out.source).unwrap();
    assert!(method(&syntax.items, "Contract", "set_proof_data").is_none());
    let actual = method(&syntax.items, "BorrowedContract", "set_proof_data").unwrap();
    let expected_bound: syn::WhereClause = syn::parse_quote!(where W: super::TryWitnesses<Private>);
    let bound = actual.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(bound.predicates.len(), 1);
    let predicate = bound.predicates.first().unwrap();
    let expected_predicate = expected_bound.predicates.first().unwrap();
    assert_eq!(
        quote::quote!(#predicate).to_string(),
        quote::quote!(#expected_predicate).to_string()
    );
    assert_eq!(actual.sig.inputs.len(), 3); // self, context, one public Field
    let syn::FnArg::Typed(arg) = actual.sig.inputs.last().unwrap() else {
        panic!("public Field");
    };
    assert!(matches!(arg.pat.as_ref(),syn::Pat::Ident(p) if p.ident=="v"));
    let ty = &arg.ty;
    assert!(quote::quote!(#ty).to_string().ends_with("Field"));
    let output = &actual.sig.output;
    assert!(
        quote::quote!(#output)
            .to_string()
            .contains("RecordedProofData < Private , () >")
    );
    assert_eq!(actual.block.stmts.len(), 4); // no witness/hash reconstruction or second call
    let syn::Stmt::Item(syn::Item::Const(guard)) = &actual.block.stmts[0] else {
        panic!("API1 guard");
    };
    let expected: syn::ItemConst = syn::parse_quote!(
        const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize];
    );
    assert_eq!(
        quote::quote!(#guard).to_string(),
        quote::quote!(#expected).to_string()
    );
    let syn::Stmt::Local(input) = &actual.block.stmts[1] else {
        panic!("capture before invocation");
    };
    let syn::Expr::Call(pack) = input.init.as_ref().unwrap().expr.as_ref() else {
        panic!("typed input capture");
    };
    let syn::Expr::Path(path) = pack.func.as_ref() else {
        panic!("AlignedValue conversion");
    };
    assert_eq!(
        path.path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>(),
        ["runtime", "fab", "AlignedValue", "from"]
    );
    assert_eq!(pack.args.len(), 1);
    assert!(matches!(&pack.args[0],syn::Expr::Path(p) if p.path.is_ident("v")));
    let syn::Stmt::Local(recorded) = &actual.block.stmts[2] else {
        panic!("actual call");
    };
    let syn::Expr::Try(call) = recorded.init.as_ref().unwrap().expr.as_ref() else {
        panic!("propagate actual error");
    };
    let syn::Expr::MethodCall(call) = call.expr.as_ref() else {
        panic!("recorded set method");
    };
    assert_eq!(call.method, "set");
    assert!(matches!(call.receiver.as_ref(),syn::Expr::Path(p) if p.path.is_ident("self")));
    assert_eq!(call.args.len(), 2);
    for (arg, name) in call.args.iter().zip(["context", "v"]) {
        assert!(matches!(arg,syn::Expr::Path(p) if p.path.is_ident(name)));
    }
    let syn::Stmt::Expr(syn::Expr::Call(convert), None) = &actual.block.stmts[3] else {
        panic!("return actual carrier");
    };
    let syn::Expr::Path(path) = convert.func.as_ref() else {
        panic!("carrier");
    };
    assert_eq!(
        path.path
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
    assert_eq!(convert.args.len(), 3);
    assert!(
        matches!(&convert.args[0],syn::Expr::Lit(lit) if matches!(&lit.lit,syn::Lit::Str(s) if s.value()=="set"))
    );
    for (arg, name) in convert.args.iter().skip(1).zip(["input", "recorded"]) {
        assert!(matches!(arg,syn::Expr::Path(p) if p.path.is_ident(name)));
    }
    assert!(method(&syntax.items, "BorrowedContract", "set").is_some());
    assert!(method(&syntax.items, "BorrowedContract", "set_call").is_some());
    assert!(method(&syntax.items, "Contract", "set").is_none());
    for handle in ["Contract", "BorrowedContract"] {
        assert!(method(&syntax.items, handle, "get_proof_data").is_some());
        let default = syn::parse_file(&render(&contract).unwrap()).unwrap();
        assert!(method(&default.items, handle, "set_proof_data").is_none());
    }
    assert!(method(&syntax.items, "BorrowedContract", "clear_proof_data").is_some());
}

#[test]
fn semantic_renames_admit_without_a_set_name_allowlist() {
    let mut value = original();
    for (old, new) in [
        ("set", "store"),
        ("v", "new_value"),
        ("sk", "secret_bytes"),
        ("apk", "owner_key"),
        ("authority", "owner"),
        ("state", "phase"),
        ("in_state", "phase_is"),
        ("private$secret_key", "secret"),
        ("public_key", "derive_key"),
    ] {
        rename(&mut value, old, new);
    }
    let c: Contract = serde_json::from_value(value).unwrap();
    let out = render_with_standalone_proof_data(&c).unwrap();
    let row = out
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "store")
        .unwrap();
    assert_eq!(
        row.standalone_proof_data.as_ref().unwrap().method,
        "store_proof_data"
    );
}

#[test]
fn public_or_internal_name_collision_preserves_native_recorded_apis() {
    for internal in [false, true] {
        let mut value = original();
        let mut collider = value["stateful_circuits"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "set")
            .unwrap()
            .clone();
        collider["name"] = json!("set_proof_data");
        collider["internal"] = json!(internal);
        value["stateful_circuits"]
            .as_array_mut()
            .unwrap()
            .push(collider);
        let c: Contract = serde_json::from_value(value).unwrap();
        let out = render_with_standalone_proof_data(&c).unwrap();
        let row = out
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == "set")
            .unwrap();
        assert!(row.recorded && row.observed_call && row.standalone_proof_data.is_none());
        assert_eq!(
            row.standalone_proof_data_unavailable.as_ref().unwrap().code,
            "name_collision"
        );
    }
}
