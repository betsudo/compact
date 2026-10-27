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
// The direct VectorFoldCall child is a native-valid recording refusal; fixed Field
// commitments themselves are supported. Do not replace this with malformed types.
use super::*;
#[test]
fn map_slots_operands_and_let_types_remain_native_authorities() {
    for mode in [
        "kind",
        "key_type",
        "value_type",
        "index",
        "lookup_key",
        "binding_type",
        "escaped_local",
        "layout",
        "identifier",
    ] {
        let mut v = fixture();
        match mode {
            "kind" => v["ledger_fields"][0]["declaration"] = json!({"kind":"counter"}),
            "key_type" => v["ledger_fields"][0]["declaration"]["key"] = json!({"kind":"boolean"}),
            "value_type" => {
                v["ledger_fields"][0]["declaration"]["value"] = json!({"kind":"boolean"})
            }
            "index" => named(&mut v, "ensure")["actions"][0]["condition"]["index"] = json!(99),
            "lookup_key" => {
                named(&mut v, "ensure")["return_value"]["value"]["bindings"][0]["value"]["key"]["name"] =
                    json!("missing")
            }
            "binding_type" => {
                named(&mut v, "ensure")["return_value"]["value"]["bindings"][0]["ty"] =
                    json!({"kind":"boolean"})
            }
            "escaped_local" => {
                named(&mut v, "ensure")["return_value"]["value"]["body"]["name"] = json!("escaped")
            }
            "layout" => v["ledger_fields"][0]["path"] = json!([2, 7]),
            "identifier" => named(&mut v, "ensure")["name"] = json!("bad root"),
            _ => unreachable!(),
        }
        let c = contract(v);
        let (result, rows) = trace::capture_json(|| crate::render_with_capabilities(&c));
        assert!(result.is_err(), "{mode}");
        assert!(rows.as_array().unwrap().is_empty(), "native first {mode}");
    }
}
#[test]
fn neighboring_interfaces_keep_boundaries_while_fold_values_are_admitted() {
    for types in [
        vec![Type::OpaqueBytes],
        vec![Type::OpaqueString, Type::OpaqueString],
    ] {
        let mut c = contract(fixture());
        let r = c
            .stateful_circuits
            .iter_mut()
            .find(|r| r.name == "ensure")
            .unwrap();
        r.parameters = types
            .into_iter()
            .enumerate()
            .map(|(i, ty)| Parameter {
                name: format!("p{i}"),
                ty,
            })
            .collect();
        r.actions.clear();
        r.return_value = StateReturn::Expression {
            value: Expr::FieldLiteral { value: "42".into() },
        };
        let cx = Context::new(&c);
        assert!(cx.terminal(cx.circuits["ensure"]).unwrap().is_none());
        crate::render_with_capabilities(&c).unwrap();
    }
    let mut c = contract(fixture());
    crate::recorded::test_support::register_unsupported_fold(&mut c);
    let r = c
        .stateful_circuits
        .iter_mut()
        .find(|r| r.name == "ensure")
        .unwrap();
    r.actions.clear();
    r.return_value = StateReturn::Expression {
        value: Expr::TransientHash {
            value: Box::new(Expr::PersistentCommit {
                value: Box::new(crate::recorded::test_support::unsupported_fixed_input()),
                opening: Box::new(Expr::BytesLiteral { bytes: vec![0; 32] }),
            }),
        },
    };
    let cx = Context::new(&c);
    cx.checked(cx.circuits["ensure"]).unwrap();
    assert!(cx.terminal(cx.circuits["ensure"]).unwrap().is_some());
    let output = crate::render_with_capabilities(&c).unwrap();
    assert!(
        output
            .capabilities
            .circuits
            .iter()
            .any(|r| r.name == "ensure" && r.recorded && r.observed_call)
    );
}
#[test]
fn recursive_or_invalid_unselected_calls_fail_before_any_recording_route() {
    for recursive in [false, true] {
        let mut v = fixture();
        let r = named(&mut v, "ensure");
        r["actions"] = json!([]);
        r["return_value"] = if recursive {
            json!({"kind":"expression","value":{"kind":"call","name":"ensure","arguments":[{"kind":"parameter","name":"key"}]}})
        } else {
            json!({"kind":"expression","value":{"kind":"if","condition":{"kind":"boolean","value":false},"then":{"kind":"boolean","value":true},"otherwise":{"kind":"field_literal","value":"42"}}})
        };
        let c = contract(v);
        let (result, rows) = trace::capture_json(|| crate::render_with_capabilities(&c));
        assert!(result.is_err());
        assert!(rows.as_array().unwrap().is_empty());
    }
}
