// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::Contract;
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-assertions-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_state_assertions_report_nested_sink_and_control_limit() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    assert_eq!(report, analyze(&contract, Limits::default()));

    assert_eq!(report.findings.len(), 1);
    let finding = &report.findings[0];
    assert_eq!(finding.circuit, "nested_sink");
    assert_eq!(finding.sink, "persistent_hash");
    assert_eq!(finding.sink_circuit, "nested_sink");
    assert_eq!(finding.routes[0].producer.circuit, "nested_sink");
    assert_eq!(
        report
            .unavailable
            .iter()
            .map(|entry| (entry.circuit.as_str(), entry.reason.as_str()))
            .collect::<Vec<_>>(),
        [
            ("transient_gate", "assertion execution control unavailable"),
            ("nested_sink", "assertion execution control unavailable"),
        ]
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.circuit != "ordinary_gate")
    );
    assert!(
        report
            .unavailable
            .iter()
            .all(|entry| entry.circuit != "ordinary_gate")
    );
}

#[test]
fn unavailable_assertion_control_respects_record_limit() {
    let report = analyze(
        &source_contract(),
        Limits {
            max_unavailable: 1,
            ..Limits::default()
        },
    );
    assert!(report.truncated);
    assert_eq!(report.unavailable.len(), 1);
    assert_eq!(report.unavailable[0].circuit, "transient_gate");
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].circuit, "nested_sink");
}
