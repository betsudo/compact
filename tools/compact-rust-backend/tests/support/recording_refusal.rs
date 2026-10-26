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

//! The exact former Fold refusal is now admitted; malformed lengths still fail.
use compact_rust_backend::ir::{Contract, Expr, Parameter, PureCircuit, Type};

const FOLD_STEP: &str = "recording_refusal_fold_step";
pub(crate) fn fold_control_input() -> Expr {
    Expr::VectorFoldCall {
        name: FOLD_STEP.into(),
        initial: Box::new(Expr::FieldLiteral { value: "0".into() }),
        source: Box::new(Expr::Tuple {
            elements: vec![Expr::FieldLiteral { value: "1".into() }],
        }),
        accumulator: Type::Field,
        element: Type::Field,
        length: 1,
    }
}
pub(crate) fn register_fold_control(contract: &mut Contract) {
    assert!(!contract.circuits.iter().any(|c| c.name == FOLD_STEP));
    contract.circuits.push(PureCircuit {
        source: None,
        name: FOLD_STEP.into(),
        internal: true,
        parameters: ["acc", "item"]
            .into_iter()
            .map(|name| Parameter {
                name: name.into(),
                ty: Type::Field,
            })
            .collect(),
        result: Type::Field,
        body: Expr::Add {
            left: Box::new(Expr::Parameter { name: "acc".into() }),
            right: Box::new(Expr::Parameter {
                name: "item".into(),
            }),
        },
    });
}

/// Qualify deliberate admission without weakening independent inactive-body checks.
/// Only the Fold length is changed for the negative, with its real callee kept.
pub(crate) fn assert_admitted_and_malformed_refused(
    contract: &Contract,
    entry: Option<&str>,
) -> compact_rust_backend::RenderedContract {
    let output = compact_rust_backend::render_with_capabilities(contract).unwrap();
    let capability = match entry {
        Some(name) => output
            .capabilities
            .circuits
            .iter()
            .find(|c| c.name == name)
            .unwrap(),
        None => &output.capabilities.circuits[0],
    };
    assert!(capability.recorded && capability.observed_call);
    assert!(capability.recording_unavailable.is_none());
    assert!(capability.observed_call_unavailable.is_none());
    fn invalidate(value: &mut serde_json::Value) -> usize {
        if value.get("kind").and_then(serde_json::Value::as_str) == Some("vector_fold_call")
            && value.get("name").and_then(serde_json::Value::as_str) == Some(FOLD_STEP)
        {
            assert_eq!(value["length"], 1);
            value["length"] = serde_json::json!(2);
            return 1;
        }
        match value {
            serde_json::Value::Object(fields) => fields.values_mut().map(invalidate).sum(),
            serde_json::Value::Array(values) => values.iter_mut().map(invalidate).sum(),
            _ => 0,
        }
    }
    let mut malformed = serde_json::to_value(contract).unwrap();
    assert_eq!(invalidate(&mut malformed), 1, "one retained Fold control");
    let malformed: Contract = serde_json::from_value(malformed).unwrap();
    // This is an intentionally native-invalid annotation, not a native-valid
    // unsupported operation or a missing-declaration stand-in.
    assert!(compact_rust_backend::render_with_capabilities(&malformed).is_err());
    output
}
