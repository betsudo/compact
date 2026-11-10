// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr, Parameter, Type};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!("transient-provenance-helpers-source-ir.json")).unwrap()
}

#[test]
fn source_helpers_produce_forward_and_consume_known_origins() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.circuit.as_str())
            .collect::<Vec<_>>(),
        [
            "helper_producer",
            "helper_forwarder",
            "helper_sink",
            "helper_mixed"
        ]
    );
    for finding in &report.findings {
        assert_eq!(finding.routes.len(), 1);
        assert_eq!(finding.routes[0].producer.kind, ProducerKind::TransientHash);
        assert!(
            finding.routes[0]
                .dependencies
                .iter()
                .any(|step| step.operation.starts_with("call_argument:")
                    || step.operation.starts_with("call_return:"))
        );
    }
    assert!(
        report.findings[0].routes[0]
            .producer
            .path
            .contains("callee:derive.body")
    );
    assert_eq!(report.findings[0].routes[0].producer.circuit, "derive");
    assert_eq!(
        report.findings[0].routes[0]
            .producer
            .source
            .as_ref()
            .unwrap()
            .line,
        11
    );
    assert!(
        report.findings[1].routes[0]
            .dependencies
            .iter()
            .filter(|step| step.operation.starts_with("call_return:identity"))
            .count()
            >= 2
    );
    assert!(report.findings[2].sink_path.contains("callee:persist.body"));
    assert_eq!(report.findings[2].source.as_ref().unwrap().line, 32);
    assert_eq!(report.findings[2].sink_circuit, "persist");
    assert_eq!(report.findings[2].sink_source.as_ref().unwrap().line, 15);
    assert!(report.findings[3].has_unknown_input);
    assert!(
        report.findings[3].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "arithmetic")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| ["ordinary_forwarded", "transient_only"].contains(&f.circuit.as_str()))
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn missing_recursive_and_over_budget_helpers_are_unavailable() {
    let mut contract = source_contract();
    let index = contract
        .circuits
        .iter()
        .position(|c| c.name == "helper_producer")
        .unwrap();
    contract.circuits[index].body = Expr::PersistentHash {
        value: Box::new(Expr::Call {
            name: "missing".into(),
            arguments: vec![Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }],
        }),
    };
    let missing = analyze(&contract, Limits::default());
    assert!(
        !missing
            .findings
            .iter()
            .any(|f| f.circuit == "helper_producer")
    );
    assert!(
        missing
            .unavailable
            .iter()
            .any(|u| u.circuit == "helper_producer" && u.reason == "unknown pure helper")
    );

    let mut contract = source_contract();
    let index = contract
        .circuits
        .iter()
        .position(|c| c.name == "identity")
        .unwrap();
    contract.circuits[index].body = Expr::Call {
        name: "identity".into(),
        arguments: vec![Expr::Parameter {
            name: "value".into(),
        }],
    };
    let recursive = analyze(&contract, Limits::default());
    assert!(
        recursive
            .unavailable
            .iter()
            .any(|u| u.reason == "recursive pure helper")
    );
    assert!(
        !recursive
            .findings
            .iter()
            .any(|f| f.circuit == "helper_forwarder")
    );

    let depth_limited = analyze(
        &source_contract(),
        Limits {
            max_call_depth: 0,
            ..Limits::default()
        },
    );
    assert!(depth_limited.truncated);
    assert!(
        depth_limited
            .unavailable
            .iter()
            .any(|u| u.reason == "pure helper call depth limit")
    );
    assert!(
        !depth_limited
            .findings
            .iter()
            .any(|f| f.circuit == "helper_producer")
    );

    let mut duplicate = source_contract();
    let identity = duplicate
        .circuits
        .iter()
        .find(|c| c.name == "identity")
        .unwrap()
        .clone();
    duplicate.circuits.push(identity);
    let ambiguous = analyze(&duplicate, Limits::default());
    assert!(
        ambiguous
            .unavailable
            .iter()
            .any(|u| u.circuit == "helper_forwarder" && u.reason == "ambiguous pure helper")
    );
    assert!(
        !ambiguous
            .findings
            .iter()
            .any(|f| f.circuit == "helper_forwarder")
    );

    let mut duplicate_formal = source_contract();
    let identity = duplicate_formal
        .circuits
        .iter_mut()
        .find(|c| c.name == "identity")
        .unwrap();
    identity.parameters.push(Parameter {
        name: "value".into(),
        ty: Type::Field,
    });
    let forwarder = duplicate_formal
        .circuits
        .iter_mut()
        .find(|c| c.name == "helper_forwarder")
        .unwrap();
    forwarder.body = Expr::PersistentHash {
        value: Box::new(Expr::Call {
            name: "identity".into(),
            arguments: vec![
                Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                },
                Expr::Parameter {
                    name: "value".into(),
                },
            ],
        }),
    };
    let duplicate_formal_report = analyze(&duplicate_formal, Limits::default());
    assert!(
        duplicate_formal_report
            .unavailable
            .iter()
            .any(|u| u.circuit == "helper_forwarder"
                && u.reason == "duplicate pure helper parameter")
    );
    assert!(
        !duplicate_formal_report
            .findings
            .iter()
            .any(|f| f.circuit == "helper_forwarder")
    );
}
