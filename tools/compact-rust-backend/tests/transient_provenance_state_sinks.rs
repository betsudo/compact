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

use compact_rust_backend::ir::{Contract, Expr, ReturnPlan, StateReturn};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-state-sinks-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_ledger_sinks_preserve_routes_and_inputs() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    let cases = report
        .findings
        .iter()
        .map(|finding| {
            (
                finding.circuit.as_str(),
                finding.sink.as_str(),
                finding.input.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cases,
        [
            ("cell_hit", "cell_write", "value"),
            ("set_hit", "set_insert", "value"),
            ("map_key_hit", "map_insert", "key"),
            ("map_value_hit", "map_insert", "value"),
            ("branch_hit", "cell_write", "value"),
            ("return_plan_hit", "cell_write", "value"),
            ("return_plan_hit", "persistent_hash", "value"),
            ("control_hit", "cell_write", "value"),
            ("control_hit", "cell_write", "value"),
        ]
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "cell_miss")
    );
    assert_eq!(report.findings[4].conditions, ["actions[0]:then"]);
    assert!(
        report.findings[6].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "binding:digest")
    );
    assert!(
        report.findings[7..]
            .iter()
            .all(|finding| finding.routes.is_empty() && !finding.control_routes.is_empty())
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.source.is_some())
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn effectful_return_plan_is_traversed() {
    let mut contract = source_contract();
    let circuit = contract
        .stateful_circuits
        .iter_mut()
        .find(|circuit| circuit.name == "return_plan_hit")
        .unwrap();
    circuit.actions.clear();
    circuit.return_value = StateReturn::Effectful {
        body: ReturnPlan::Let {
            bindings: vec![compact_rust_backend::ir::LocalBinding {
                name: "digest".into(),
                ty: compact_rust_backend::ir::Type::Field,
                value: Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                },
            }],
            result: Box::new(ReturnPlan::Sequence {
                actions: vec![compact_rust_backend::ir::StateAction::CellWrite {
                    field: "stored".into(),
                    index: 0,
                    value: Expr::Parameter {
                        name: "digest".into(),
                    },
                }],
                result: Box::new(ReturnPlan::Value { value: Expr::Unit }),
            }),
        },
    };
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.circuit == "return_plan_hit"
                && finding.sink_path.contains("return_value.body"))
    );
}

#[test]
fn state_actions_obey_budget() {
    let mut contract = source_contract();
    contract
        .stateful_circuits
        .retain(|circuit| circuit.name == "cell_hit");
    let report = analyze(
        &contract,
        Limits {
            max_nodes: 2,
            ..Limits::default()
        },
    );
    assert!(report.truncated);
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| entry.reason.ends_with("limit"))
    );
}

#[test]
fn malformed_effectful_return_does_not_report_unreachable_actions() {
    let mut contract = source_contract();
    contract
        .stateful_circuits
        .retain(|circuit| circuit.name == "cell_hit");
    contract.stateful_circuits[0].return_value = StateReturn::Effectful {
        body: ReturnPlan::Value { value: Expr::Unit },
    };
    let report = analyze(&contract, Limits::default());
    assert!(report.findings.is_empty());
    assert_eq!(
        report.unavailable[0].reason,
        "effectful return with separate actions"
    );
}
