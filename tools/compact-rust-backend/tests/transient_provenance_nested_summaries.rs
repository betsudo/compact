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

use compact_rust_backend::ir::{Contract, Expr};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, Report, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-nested-summaries-source-ir.json"
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
fn source_nested_pure_summaries_keep_producers_and_formals() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    for name in ["relay", "two_hops", "wrap", "wrap_twice", "mix"] {
        assert!(
            report
                .helper_summaries
                .iter()
                .any(|summary| summary.circuit == name)
        );
    }
    for name in ["persist", "persist_wrapper", "sinkful_hit"] {
        assert!(
            !report
                .helper_summaries
                .iter()
                .any(|summary| summary.circuit == name)
        );
    }
    let twice = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "wrap_twice")
        .unwrap();
    assert_eq!(twice.producers.len(), 1);
    assert_eq!(twice.producers[0].kind, ProducerKind::TransientHash);
    assert_eq!(twice.producers[0].circuit, "derive");
    assert_eq!(twice.formal_dependencies.len(), 1);
    assert_eq!(twice.formal_dependencies[0].parameter, "value");

    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.circuit.as_str())
            .collect::<Vec<_>>(),
        ["forwarded_hit", "produced_hit", "mixed_hit", "sinkful_hit"]
    );
    assert_eq!(
        report.findings[0].routes[0].producer.circuit,
        "forwarded_hit"
    );
    assert_eq!(report.findings[1].routes[0].producer.circuit, "derive");
    assert!(
        report.findings[1].routes[0]
            .producer
            .path
            .contains("callee:wrap_twice.body.callee:wrap.body.callee:derive.body")
    );
    assert!(report.findings[2].has_unknown_input);
    assert_eq!(
        report.findings[0]
            .parameter_dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>(),
        ["value"]
    );
    assert_eq!(
        report.findings[2]
            .parameter_dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>(),
        ["ordinary", "value"]
    );
    assert_eq!(report.findings[3].sink_circuit, "persist");
    same_diagnostics(&report, &uncached(&contract, Limits::default()));
    assert_eq!(report, analyze(&contract, Limits::default()));
}

#[test]
fn nested_summary_call_depth_and_node_budgets_match_body_traversal() {
    let contract = source_contract();
    for limits in [
        Limits {
            max_call_depth: 1,
            ..Limits::default()
        },
        Limits {
            max_call_depth: 2,
            ..Limits::default()
        },
        Limits {
            max_nodes: 8,
            ..Limits::default()
        },
        Limits {
            max_depth: 5,
            ..Limits::default()
        },
        Limits {
            max_steps: 2,
            ..Limits::default()
        },
        Limits {
            max_routes: 1,
            ..Limits::default()
        },
        Limits {
            max_component_nodes: 2,
            ..Limits::default()
        },
        Limits {
            max_summary_nodes: 1,
            ..Limits::default()
        },
    ] {
        let report = analyze(&contract, limits);
        same_diagnostics(&report, &uncached(&contract, limits));
    }
}

#[test]
fn recursive_nested_helpers_stay_unavailable() {
    let mut contract = source_contract();
    let identity = contract
        .circuits
        .iter_mut()
        .find(|circuit| circuit.name == "identity")
        .unwrap();
    identity.body = Expr::Call {
        name: "relay".into(),
        arguments: vec![Expr::Parameter {
            name: "value".into(),
        }],
    };
    let cached = analyze(&contract, Limits::default());
    assert!(
        cached
            .unavailable
            .iter()
            .any(|item| item.reason == "recursive pure helper")
    );
    same_diagnostics(&cached, &uncached(&contract, Limits::default()));
}

#[test]
fn synthetic_stateful_value_call_prevents_pure_wrapper_caching() {
    let mut contract = source_contract();
    let value_calls: Contract = serde_json::from_str(include_str!(
        "transient-provenance-value-calls-source-ir.json"
    ))
    .unwrap();
    let mut make = value_calls
        .stateful_circuits
        .into_iter()
        .find(|circuit| circuit.name == "make")
        .unwrap();
    // Keep the stateful value-call shape, while removing its write so the
    // cache exclusion is tested independently of sink detection.
    make.actions.clear();
    contract.stateful_circuits.push(make);
    let identity = contract
        .circuits
        .iter_mut()
        .find(|circuit| circuit.name == "identity")
        .unwrap();
    identity.body = Expr::Call {
        name: "make".into(),
        arguments: vec![Expr::Parameter {
            name: "value".into(),
        }],
    };
    let cached = analyze(&contract, Limits::default());
    for name in ["identity", "relay", "two_hops", "mix"] {
        assert!(
            !cached
                .helper_summaries
                .iter()
                .any(|summary| summary.circuit == name)
        );
    }
    same_diagnostics(&cached, &uncached(&contract, Limits::default()));
}
