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

//! Supplemental typed-IR provenance controls; no fixture, clock or SDK execution.
use compact_rust_backend::ir::{Contract, Expr, StateReturn, Type};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, Report, analyze};

const U64_MAX: &str = "18446744073709551615";

fn clock(less: bool, bound: Expr) -> Expr {
    if less {
        Expr::KernelBlockTimeLessThan {
            bound: Box::new(bound),
        }
    } else {
        Expr::KernelBlockTimeGreaterThan {
            bound: Box::new(bound),
        }
    }
}

fn transient_bound() -> Expr {
    Expr::UnsignedCast {
        max: U64_MAX.into(),
        value: Box::new(Expr::TransientHash {
            value: Box::new(Expr::Parameter {
                name: "value".into(),
            }),
        }),
    }
}

fn literal_bound() -> Expr {
    Expr::UnsignedLiteral {
        value: "5".into(),
        max: U64_MAX.into(),
    }
}

fn data_contract(less: bool, bound: Expr) -> Contract {
    let mut contract: Contract = serde_json::from_str(include_str!(
        "transient-provenance-map-lookup-source-ir.json"
    ))
    .unwrap();
    contract
        .stateful_circuits
        .retain(|c| ["selected", "helper_hit"].contains(&c.name.as_str()));
    let helper = contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "selected")
        .unwrap();
    helper.result = Type::Boolean;
    helper.return_value = StateReturn::Expression {
        value: clock(less, bound),
    };
    contract
}

fn control_contract(less: bool) -> Contract {
    let mut contract: Contract = serde_json::from_str(include_str!(
        "transient-provenance-membership-source-ir.json"
    ))
    .unwrap();
    contract
        .stateful_circuits
        .retain(|c| ["has_transient_member", "helper_gate"].contains(&c.name.as_str()));
    let helper = contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "has_transient_member")
        .unwrap();
    helper.return_value = StateReturn::Expression {
        value: clock(less, transient_bound()),
    };
    contract
}

fn same_diagnostics(contract: &Contract, limits: Limits) -> Report {
    let cached = analyze(contract, limits);
    let uncached = analyze(
        contract,
        Limits {
            max_summary_nodes: 0,
            ..limits
        },
    );
    assert_eq!(cached.findings, uncached.findings);
    assert_eq!(cached.unavailable, uncached.unavailable);
    assert_eq!(cached.truncated, uncached.truncated);
    cached
}

#[test]
fn typed_time_bound_keeps_known_data_control_and_formal_dependencies() {
    for less in [true, false] {
        let operation = if less {
            "block_time_less_than"
        } else {
            "block_time_greater_than"
        };
        let contract = data_contract(less, transient_bound());
        let report = same_diagnostics(&contract, Limits::default());
        assert!(report.unavailable.is_empty());
        let summary = report
            .helper_summaries
            .iter()
            .find(|s| s.circuit == "selected")
            .unwrap();
        assert!(summary.has_nonformal_unknown, "the clock remains external");
        assert_eq!(summary.producers.len(), 1);
        assert_eq!(summary.formal_dependencies.len(), 1);
        assert_eq!(summary.formal_dependencies[0].parameter, "value");
        let finding = report
            .findings
            .iter()
            .find(|f| f.circuit == "helper_hit")
            .unwrap();
        assert_eq!(finding.sink, "persistent_hash");
        assert!(finding.has_unknown_input);
        assert_eq!(finding.routes.len(), 1);
        assert_eq!(finding.routes[0].producer.kind, ProducerKind::TransientHash);
        assert!(
            finding.routes[0]
                .dependencies
                .iter()
                .any(|d| d.operation == operation)
        );
        assert!(
            finding
                .parameter_dependencies
                .iter()
                .any(|d| d.name == "value")
        );

        let control = same_diagnostics(&control_contract(less), Limits::default());
        assert!(control.unavailable.is_empty());
        let finding = control
            .findings
            .iter()
            .find(|f| f.circuit == "helper_gate")
            .unwrap();
        assert!(finding.routes.is_empty());
        assert_eq!(finding.control_routes.len(), 1);
        assert!(finding.has_unknown_control);
        assert_eq!(
            finding.control_routes[0].producer.kind,
            ProducerKind::TransientHash
        );
        assert!(
            finding.control_routes[0]
                .dependencies
                .iter()
                .any(|d| d.operation == operation)
        );
        assert!(
            finding
                .control_parameter_dependencies
                .iter()
                .any(|d| d.name == "value")
        );
    }
}

#[test]
fn typed_time_visits_nested_persistent_sink_in_bound() {
    for less in [true, false] {
        let bound = Expr::Sequence {
            steps: vec![Expr::PersistentHash {
                value: Box::new(Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                }),
            }],
            value: Box::new(literal_bound()),
        };
        let report = same_diagnostics(&data_contract(less, bound), Limits::default());
        assert!(report.unavailable.is_empty());
        let nested = report
            .findings
            .iter()
            .filter(|f| f.sink_path.contains(".bound.steps[0]"))
            .collect::<Vec<_>>();
        assert!(
            !nested.is_empty(),
            "bound traversal must retain the nested sink"
        );
        for finding in nested {
            assert_eq!(finding.sink, "persistent_hash");
            assert_eq!(finding.routes.len(), 1);
            assert_eq!(finding.routes[0].producer.kind, ProducerKind::TransientHash);
        }
    }
}

#[test]
fn typed_time_literal_bound_stays_unknown_without_invented_producer() {
    for less in [true, false] {
        let report = same_diagnostics(&data_contract(less, literal_bound()), Limits::default());
        assert!(report.findings.is_empty());
        assert!(report.unavailable.is_empty());
        let summary = report
            .helper_summaries
            .iter()
            .find(|s| s.circuit == "selected")
            .unwrap();
        assert!(summary.has_nonformal_unknown);
        assert!(summary.producers.is_empty());
        assert!(summary.formal_dependencies.is_empty());
    }
}

#[test]
fn typed_time_summary_and_traversal_bounds_keep_uncached_diagnostics() {
    for less in [true, false] {
        let contract = data_contract(less, transient_bound());
        for limits in [
            Limits {
                max_nodes: 6,
                ..Limits::default()
            },
            Limits {
                max_depth: 2,
                ..Limits::default()
            },
            Limits {
                max_summary_nodes: 1,
                ..Limits::default()
            },
        ] {
            same_diagnostics(&contract, limits);
        }
    }
}
