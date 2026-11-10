// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr, StateAction};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-action-calls-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_action_helpers_preserve_call_site_and_lexical_sink_evidence() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.circuit.as_str())
            .collect::<Vec<_>>(),
        ["make_and_save", "direct_call", "nested_call",]
    );
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.sink == "cell_write" && f.sink_circuit == "save")
    );
    assert!(report.findings.iter().all(|f| {
        f.routes[0]
            .dependencies
            .iter()
            .any(|d| d.operation == "call_argument:value")
    }));
    assert_eq!(
        report.findings[0].routes[0].producer.circuit,
        "make_and_save"
    );
    assert_eq!(report.findings[1].routes[0].producer.circuit, "direct_call");
    assert_eq!(
        report.findings[2].routes[0].producer.circuit,
        "make_and_save"
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn recursive_and_budgeted_action_helpers_are_unavailable() {
    let mut contract = source_contract();
    let helper = contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "save")
        .unwrap();
    helper.actions = vec![StateAction::CircuitCall {
        name: "save".into(),
        arguments: vec![compact_rust_backend::ir::Expr::Parameter {
            name: "value".into(),
        }],
    }];
    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .unavailable
            .iter()
            .any(|u| u.reason == "recursive stateful helper")
    );
    assert!(report.findings.is_empty());

    let tight = analyze(
        &source_contract(),
        Limits {
            max_call_depth: 0,
            ..Limits::default()
        },
    );
    assert!(
        tight
            .unavailable
            .iter()
            .any(|u| u.reason == "stateful helper call depth limit")
    );
    assert!(tight.truncated);
}

#[test]
fn invalid_action_call_does_not_analyze_unreachable_argument() {
    let mut contract = source_contract();
    contract.stateful_circuits.retain(|circuit| circuit.name == "direct_call");
    contract.stateful_circuits[0].actions = vec![StateAction::CircuitCall {
        name: "missing".into(),
        arguments: vec![Expr::PersistentHash {
            value: Box::new(Expr::TransientHash {
                value: Box::new(Expr::Parameter { name: "value".into() }),
            }),
        }],
    }];
    let report = analyze(&contract, Limits::default());
    assert!(report.findings.is_empty());
    assert_eq!(report.unavailable[0].reason, "unknown stateful helper");
}
