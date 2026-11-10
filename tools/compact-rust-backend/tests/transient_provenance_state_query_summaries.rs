// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr, StateReturn};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, Report, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-map-lookup-source-ir.json"
    ))
    .unwrap()
}

fn membership_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-membership-source-ir.json"
    ))
    .unwrap()
}

fn uncached(contract: &Contract, limits: Limits) -> Report {
    analyze(
        contract,
        Limits {
            max_summary_nodes: 0,
            ..limits
        },
    )
}

fn same_diagnostics(cached: &Report, uncached: &Report) {
    assert_eq!(cached.findings, uncached.findings);
    assert_eq!(cached.unavailable, uncached.unavailable);
    assert_eq!(cached.truncated, uncached.truncated);
}

#[test]
fn source_action_free_lookup_summary_retains_selection_origin() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    let selected = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "selected")
        .unwrap();
    assert_eq!(selected.producers.len(), 1);
    assert_eq!(selected.producers[0].kind, ProducerKind::TransientHash);
    assert_eq!(selected.producers[0].circuit, "selected");
    assert!(selected.has_nonformal_unknown);
    assert_eq!(selected.formal_dependencies.len(), 1);
    assert_eq!(selected.formal_dependencies[0].parameter, "value");
    assert!(
        report
            .helper_summaries
            .iter()
            .all(|summary| summary.circuit != "state_hit")
    );
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "helper_hit")
        .unwrap();
    assert_eq!(finding.routes[0].producer.circuit, "selected");
    assert!(finding.has_unknown_input);
    assert!(
        finding.routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "map_lookup_selection")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "ordinary")
    );
    same_diagnostics(&report, &uncached(&contract, Limits::default()));
    assert_eq!(report, analyze(&contract, Limits::default()));
}

#[test]
fn typed_query_return_and_actionful_fallback_keep_cached_uncached_parity() {
    let mut contract = source_contract();
    let selected = contract
        .stateful_circuits
        .iter_mut()
        .find(|circuit| circuit.name == "selected")
        .unwrap();
    selected.return_value = StateReturn::MapLookup {
        field: "table".into(),
        index: 0,
        key: Expr::TransientHash {
            value: Box::new(Expr::Parameter {
                name: "value".into(),
            }),
        },
    };
    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .helper_summaries
            .iter()
            .any(|summary| summary.circuit == "selected")
    );
    same_diagnostics(&report, &uncached(&contract, Limits::default()));

    let action = contract
        .stateful_circuits
        .iter()
        .find(|circuit| circuit.name == "state_hit")
        .unwrap()
        .actions[0]
        .clone();
    let selected = contract
        .stateful_circuits
        .iter_mut()
        .find(|circuit| circuit.name == "selected")
        .unwrap();
    selected.actions.push(action);
    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .helper_summaries
            .iter()
            .all(|summary| summary.circuit != "selected")
    );
    same_diagnostics(&report, &uncached(&contract, Limits::default()));
}

#[test]
fn source_membership_helper_and_typed_query_variants_keep_control_routes() {
    let source = membership_contract();
    for variant in ["source", "set", "map"] {
        let mut contract = source.clone();
        if variant != "source" {
            let helper = contract
                .stateful_circuits
                .iter_mut()
                .find(|circuit| circuit.name == "has_transient_member")
                .unwrap();
            let operand = Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            };
            helper.return_value = if variant == "set" {
                StateReturn::SetMember {
                    field: "members".into(),
                    index: 1,
                    value: operand,
                }
            } else {
                StateReturn::MapMember {
                    field: "entries".into(),
                    index: 2,
                    key: operand,
                }
            };
        }
        let report = analyze(&contract, Limits::default());
        assert!(
            report
                .helper_summaries
                .iter()
                .any(|summary| summary.circuit == "has_transient_member")
        );
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.circuit == "helper_gate")
            .unwrap();
        assert_eq!(finding.control_routes.len(), 1);
        assert_eq!(
            finding.control_routes[0].producer.circuit,
            "has_transient_member"
        );
        assert!(finding.has_unknown_control);
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.circuit == "ordinary_gate")
        );
        same_diagnostics(&report, &uncached(&contract, Limits::default()));
    }
}

#[test]
fn summary_budget_and_call_limits_keep_uncached_diagnostics() {
    let contract = source_contract();
    for limits in [
        Limits {
            max_nodes: 6,
            ..Limits::default()
        },
        Limits {
            max_call_depth: 0,
            ..Limits::default()
        },
        Limits {
            max_depth: 2,
            ..Limits::default()
        },
        Limits {
            max_routes: 1,
            ..Limits::default()
        },
        Limits {
            max_summary_nodes: 1,
            ..Limits::default()
        },
    ] {
        let cached = analyze(&contract, limits);
        same_diagnostics(&cached, &uncached(&contract, limits));
    }
}
