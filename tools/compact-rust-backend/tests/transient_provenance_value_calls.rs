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

use compact_rust_backend::ir::{Contract, Expr, ReturnPlan, StateReturn, Type};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-value-calls-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_value_helper_and_standard_library_conversion_preserve_provenance() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.circuit.as_str())
            .collect::<Vec<_>>(),
        ["forwarded", "standard_nonce", "direct"]
    );
    assert_eq!(report.findings[2].routes[0].producer.circuit, "make");
    assert!(
        report.findings[2].routes[0]
            .dependencies
            .iter()
            .any(|d| d.operation == "call_return:make")
    );
    assert_eq!(report.findings[1].routes[0].producer.circuit, "evolveNonce");
    assert!(
        report.findings[1].routes[0]
            .dependencies
            .iter()
            .any(|d| d.operation == "conversion")
    );
    assert!(!report.findings.iter().any(|f| f.circuit == "ordinary"));
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn unsupported_stateful_value_return_is_unavailable() {
    let mut contract = source_contract();
    contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "make")
        .unwrap()
        .return_value = StateReturn::Effectful {
        body: ReturnPlan::Value { value: Expr::Unit },
    };
    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .unavailable
            .iter()
            .any(|u| u.reason == "effectful return with separate actions")
    );
    assert!(!report.findings.iter().any(|f| f.circuit == "direct"));
}

#[test]
fn non_expression_return_still_reports_callee_action_sink() {
    let mut contract = source_contract();
    contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "make")
        .unwrap()
        .return_value = StateReturn::CellRead {
        field: "stored".into(),
        index: 0,
    };
    let direct = contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "direct")
        .unwrap();
    direct.result = Type::Field;
    direct.return_value = StateReturn::Expression {
        value: Expr::Call {
            name: "make".into(),
            arguments: vec![Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }],
        },
    };
    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.circuit == "direct" && f.sink_circuit == "make" && f.sink == "cell_write")
    );
    assert!(
        report.unavailable.iter().any(|u| u.circuit == "direct"
            && u.reason == "non-expression stateful value result unavailable")
    );
}

#[test]
fn invalid_value_helper_does_not_analyze_unreachable_argument() {
    let mut contract = source_contract();
    contract.circuits.clear();
    contract.stateful_circuits.retain(|c| c.name == "direct");
    contract.stateful_circuits[0].return_value = StateReturn::Expression {
        value: Expr::Call {
            name: "missing".into(),
            arguments: vec![Expr::PersistentHash {
                value: Box::new(Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                }),
            }],
        },
    };
    let report = analyze(&contract, Limits::default());
    assert!(report.findings.is_empty());
    assert_eq!(report.unavailable[0].reason, "unknown pure helper");
}
