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

use compact_rust_backend::ir::Contract;
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, advise, analyze};

fn contract(source_ir: &str) -> Contract {
    serde_json::from_str(source_ir).unwrap()
}

#[test]
fn source_leaf_helpers_materialize_formal_and_producer_summaries() {
    let contract = contract(include_str!("transient-provenance-helpers-source-ir.json"));
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    let identity = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "identity")
        .unwrap();
    assert!(identity.producers.is_empty());
    assert_eq!(identity.formal_dependencies.len(), 1);
    assert_eq!(identity.formal_dependencies[0].parameter, "value");
    assert!(!identity.has_nonformal_unknown);
    assert_eq!(
        advise(&report, &[], 0).helper_summaries,
        report.helper_summaries
    );

    let derive = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "derive")
        .unwrap();
    assert_eq!(derive.producers.len(), 1);
    assert_eq!(derive.producers[0].kind, ProducerKind::TransientHash);
    assert_eq!(derive.formal_dependencies[0].parameter, "value");

    let combine = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "combine")
        .unwrap();
    assert_eq!(
        combine
            .formal_dependencies
            .iter()
            .map(|formal| formal.parameter.as_str())
            .collect::<Vec<_>>(),
        ["digest", "extra"]
    );
    assert!(
        !report
            .helper_summaries
            .iter()
            .any(|summary| summary.circuit == "persist")
    );
}

#[test]
fn source_findings_match_uncached_call_traversal() {
    for source_ir in [
        include_str!("transient-provenance-helper-summaries-source-ir.json"),
        include_str!("transient-provenance-helpers-source-ir.json"),
        include_str!("transient-provenance-aggregates-source-ir.json"),
        include_str!("transient-provenance-state-sinks-source-ir.json"),
        include_str!("transient-provenance-state-collections-source-ir.json"),
        include_str!("transient-provenance-value-calls-source-ir.json"),
        include_str!("transient-provenance-nested-source-ir.json"),
    ] {
        let contract = contract(source_ir);
        let cached = analyze(&contract, Limits::default());
        let uncached = analyze(
            &contract,
            Limits {
                max_summary_nodes: 0,
                ..Limits::default()
            },
        );
        assert_eq!(cached.findings, uncached.findings);
        assert_eq!(cached.unavailable, uncached.unavailable);
        assert_eq!(cached.truncated, uncached.truncated);
    }
}

#[test]
fn source_product_conversion_and_branch_summaries_keep_exact_sink_evidence() {
    let contract = contract(include_str!(
        "transient-provenance-helper-summaries-source-ir.json"
    ));
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.circuit.as_str())
            .collect::<Vec<_>>(),
        [
            "projected_hit",
            "relayed_hit",
            "conversion_hit",
            "branch_hit",
            "produced_hit",
            "mixed_order_hit",
        ]
    );
    for name in ["projected_plain", "relayed_plain"] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.circuit == name)
        );
    }
    let pair = report
        .helper_summaries
        .iter()
        .find(|summary| summary.circuit == "pair")
        .unwrap();
    assert_eq!(
        pair.formal_dependencies
            .iter()
            .map(|formal| formal.parameter.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    for name in ["relay", "convert", "choose", "derive", "mix"] {
        assert!(
            report
                .helper_summaries
                .iter()
                .any(|summary| summary.circuit == name)
        );
    }
    assert!(
        report.findings[2].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "conversion")
    );
    assert_eq!(
        report.findings[5]
            .routes
            .iter()
            .map(|route| route.producer.circuit.as_str())
            .collect::<Vec<_>>(),
        ["mixed_order_hit", "mix"]
    );
    let uncached = analyze(
        &contract,
        Limits {
            max_summary_nodes: 0,
            ..Limits::default()
        },
    );
    assert_eq!(report.findings, uncached.findings);
    assert_eq!(report.unavailable, uncached.unavailable);
}

#[test]
fn summary_budget_falls_back_to_call_traversal() {
    let contract = contract(include_str!("transient-provenance-helpers-source-ir.json"));
    let limited = analyze(
        &contract,
        Limits {
            max_summary_nodes: 1,
            ..Limits::default()
        },
    );
    let uncached = analyze(
        &contract,
        Limits {
            max_summary_nodes: 0,
            ..Limits::default()
        },
    );
    assert_eq!(limited.findings, uncached.findings);
    assert_eq!(limited.unavailable, uncached.unavailable);
    assert_eq!(limited.truncated, uncached.truncated);
    assert!(limited.helper_summaries.len() <= 1);
}

#[test]
fn cached_and_uncached_diagnostics_agree_under_analysis_limits() {
    let contract = contract(include_str!(
        "transient-provenance-helper-summaries-source-ir.json"
    ));
    for limits in [
        Limits {
            max_nodes: 8,
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
            max_depth: 5,
            ..Limits::default()
        },
        Limits {
            max_component_nodes: 2,
            ..Limits::default()
        },
        Limits {
            max_parameters: 1,
            ..Limits::default()
        },
    ] {
        let cached = analyze(&contract, limits);
        let uncached = analyze(
            &contract,
            Limits {
                max_summary_nodes: 0,
                ..limits
            },
        );
        assert_eq!(cached.findings, uncached.findings, "{limits:#?}");
        assert_eq!(cached.unavailable, uncached.unavailable, "{limits:#?}");
        assert_eq!(cached.truncated, uncached.truncated, "{limits:#?}");
    }
}
