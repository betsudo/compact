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

use compact_rust_backend::ir::{Contract, Expr, LocalBinding, Type};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!("transient-provenance-source-ir.json")).unwrap()
}

#[test]
fn source_compiled_positive_and_negative_controls() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.circuit.as_str())
            .collect::<Vec<_>>(),
        ["direct", "committed", "mixed", "converted", "opening"]
    );
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.sink.as_str())
            .collect::<Vec<_>>(),
        [
            "persistent_hash",
            "persistent_commit",
            "persistent_hash",
            "persistent_hash",
            "persistent_commit"
        ]
    );
    assert_eq!(
        report.findings[0].routes[0].producer.kind,
        ProducerKind::TransientHash
    );
    assert!(report.findings[0].has_unknown_input);
    assert_eq!(
        report.findings[1].routes[0].producer.kind,
        ProducerKind::TransientCommit
    );
    assert_eq!(report.findings[1].input, "value");
    assert!(
        report.findings[2].has_unknown_input,
        "ordinary parameter must remain unknown"
    );
    assert!(
        report.findings[2].routes[0]
            .dependencies
            .iter()
            .any(|d| d.operation == "arithmetic")
    );
    assert!(
        report.findings[3].routes[0]
            .dependencies
            .iter()
            .any(|d| d.operation == "conversion")
    );
    assert_eq!(report.findings[3].source.as_ref().unwrap().line, 35);
    assert_eq!(report.findings[4].input, "opening");
    assert_eq!(
        report.findings[4].routes[0].producer.kind,
        ProducerKind::TransientHash
    );
    // Both ordinary Field and witness-origin Field are valid persistent inputs.
    // The no-finding result for either does not certify cross-upgrade stability.
    assert!(
        !report
            .findings
            .iter()
            .any(|f| ["ordinary", "ordinary_witness", "transient_only"]
                .contains(&f.circuit.as_str()))
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn unsupported_flow_and_limits_are_explicit_and_deterministic() {
    let mut contract = source_contract();
    contract.circuits[0].body = Expr::PersistentHash {
        value: Box::new(Expr::Call {
            name: "unknown_helper".into(),
            arguments: vec![Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }],
        }),
    };
    let report = analyze(&contract, Limits::default());
    assert!(!report.findings.iter().any(|f| f.circuit == "direct"));
    assert!(
        report
            .unavailable
            .iter()
            .any(|u| u.circuit == "direct" && u.reason == "unknown pure helper")
    );

    contract.circuits[0].body = Expr::PersistentHash {
        value: Box::new(Expr::WitnessCall {
            name: "supplied".into(),
            arguments: vec![Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }],
        }),
    };
    let witness_boundary = analyze(&contract, Limits::default());
    assert!(
        !witness_boundary
            .findings
            .iter()
            .any(|f| f.circuit == "direct")
    );
    assert!(
        witness_boundary.unavailable.iter().any(
            |u| u.circuit == "direct" && u.reason == "witness result provenance is unavailable"
        )
    );

    let tight = Limits {
        max_nodes: 2,
        max_depth: 2,
        max_routes: 1,
        max_steps: 1,
        max_findings: 1,
        max_unavailable: 3,
        ..Limits::default()
    };
    let first = analyze(&source_contract(), tight);
    assert_eq!(first, analyze(&source_contract(), tight));
    assert!(!first.unavailable.is_empty());
    assert!(first.truncated);
    assert!(first.unavailable.len() <= 3);
    assert!(first.findings.len() <= 1);
    let zero = analyze(
        &source_contract(),
        Limits {
            max_nodes: 0,
            max_unavailable: 0,
            ..Limits::default()
        },
    );
    assert_eq!(zero.unavailable.len(), 1);
    assert!(zero.truncated);
}

#[test]
fn wide_binding_budget_stops_after_the_bounded_prefix() {
    let mut contract = source_contract();
    contract.circuits.truncate(1);
    contract.stateful_circuits.clear();
    contract.circuits[0].body = Expr::Let {
        bindings: (0..256)
            .map(|n| LocalBinding {
                name: format!("v{n}"),
                ty: Type::Field,
                value: Expr::FieldLiteral {
                    value: n.to_string(),
                },
            })
            .collect(),
        body: Box::new(Expr::PersistentHash {
            value: Box::new(Expr::Parameter {
                name: "v255".into(),
            }),
        }),
    };
    let report = analyze(
        &contract,
        Limits {
            max_nodes: 2,
            ..Limits::default()
        },
    );
    assert!(report.findings.is_empty());
    assert_eq!(report.unavailable.len(), 1);
    assert_eq!(report.unavailable[0].reason, "expression node limit");
    assert!(report.truncated);
}

#[test]
fn optional_analysis_does_not_mutate_ir_or_rendering() {
    let contract = source_contract();
    let before = serde_json::to_vec(&contract).unwrap();
    let rendered = compact_rust_backend::render_with_capabilities(&contract).unwrap();
    let _report = analyze(&contract, Limits::default());
    assert_eq!(before, serde_json::to_vec(&contract).unwrap());
    let after = compact_rust_backend::render_with_capabilities(&contract).unwrap();
    assert_eq!(rendered.source, after.source);
    assert_eq!(
        serde_json::to_value(rendered.capabilities).unwrap(),
        serde_json::to_value(after.capabilities).unwrap()
    );
}
