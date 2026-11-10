// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::Contract;
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!("transient-provenance-nested-source-ir.json")).unwrap()
}

#[test]
fn source_nested_producers_remain_visible_at_persistent_sinks() {
    let report = analyze(&source_contract(), Limits::default());
    for circuit in [
        "nested_hash",
        "nested_commit_value",
        "nested_commit_opening",
    ] {
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.circuit == circuit)
            .unwrap();
        assert_eq!(finding.sink, "persistent_hash");
        assert_eq!(finding.routes.len(), 2, "{circuit}: {:#?}", finding.routes);
        assert!(
            finding
                .routes
                .iter()
                .any(|route| route.producer.kind == ProducerKind::TransientHash)
        );
    }
    let commit_value = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "nested_commit_value")
        .unwrap();
    assert!(
        commit_value
            .routes
            .iter()
            .any(|route| route.producer.kind == ProducerKind::TransientCommit)
    );
    assert!(commit_value.routes.iter().any(|route| {
        route
            .dependencies
            .iter()
            .any(|step| step.operation == "transient_value")
    }));
    let commit_opening = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "nested_commit_opening")
        .unwrap();
    assert!(commit_opening.routes.iter().any(|route| {
        route
            .dependencies
            .iter()
            .any(|step| step.operation == "transient_opening")
    }));
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn source_map_lookup_key_is_traversed_without_claiming_result_origin() {
    let report = analyze(&source_contract(), Limits::default());
    let lookup = report
        .findings
        .iter()
        .filter(|finding| finding.circuit == "nested_lookup_key")
        .collect::<Vec<_>>();
    assert_eq!(lookup.len(), 1);
    assert_eq!(lookup[0].sink, "persistent_hash");
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| entry.circuit == "nested_lookup_key"
                && entry.reason == "map lookup result provenance unavailable")
    );
}

#[test]
fn nested_routes_stop_at_the_configured_limit() {
    let mut contract = source_contract();
    contract.circuits.retain(|circuit| circuit.name == "nested_hash");
    contract.stateful_circuits.clear();
    let report = analyze(&contract, Limits { max_routes: 1, ..Limits::default() });
    assert!(report.truncated);
    assert!(report.unavailable.iter().any(|entry| entry.reason == "producer route limit"));
    assert_eq!(report, analyze(&contract, Limits { max_routes: 1, ..Limits::default() }));
}
