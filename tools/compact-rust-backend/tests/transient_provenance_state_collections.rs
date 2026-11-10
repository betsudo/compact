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

use compact_rust_backend::ir::{Contract, Expr, StateAction};
use compact_rust_backend::transient_provenance::{Limits, ProducerKind, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-state-collections-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_list_and_merkle_writes_keep_transient_origins() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert!(!report.truncated);
    let cases = report
        .findings
        .iter()
        .filter(|finding| !finding.routes.is_empty())
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
            ("list_hit", "list_push_front", "value"),
            ("tree_hit", "merkle_insert", "value"),
            ("tree_index_hit", "merkle_insert_index", "value"),
            ("tree_hash_hit", "merkle_insert_hash", "hash"),
            ("tree_hash_index_hit", "merkle_insert_hash_index", "hash"),
            ("history_hit", "historic_merkle_insert", "value"),
            ("history_index_hit", "historic_merkle_insert_index", "value"),
            ("history_hash_hit", "historic_merkle_insert_hash", "hash"),
            (
                "history_hash_index_hit",
                "historic_merkle_insert_hash_index",
                "hash",
            ),
        ]
    );
    for finding in report
        .findings
        .iter()
        .filter(|finding| !finding.routes.is_empty())
    {
        assert_eq!(finding.routes.len(), 1);
        assert_eq!(finding.routes[0].producer.kind, ProducerKind::TransientHash);
        assert_eq!(finding.routes[0].producer.circuit, finding.circuit);
        assert_eq!(finding.parameter_dependencies.len(), 1);
        assert_eq!(finding.parameter_dependencies[0].name, "value");
        assert!(finding.source.is_some());
        assert!(finding.sink_source.is_some());
        if finding.input == "hash" {
            assert!(
                finding.routes[0]
                    .dependencies
                    .iter()
                    .any(|step| step.operation == "conversion"),
                "{finding:#?}"
            );
        }
    }
    assert_eq!(report, analyze(&contract, Limits::default()));
}

#[test]
fn source_list_and_merkle_branch_control_is_separate_from_data() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    let controls = report
        .findings
        .iter()
        .filter(|finding| finding.circuit.ends_with("control_hit"))
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    assert_eq!(controls[0].sink, "list_push_front");
    assert_eq!(controls[1].sink, "merkle_insert");
    for finding in controls {
        assert_eq!(finding.input, "value");
        assert!(finding.routes.is_empty());
        assert_eq!(finding.control_routes.len(), 1);
        assert_eq!(
            finding.control_routes[0].producer.kind,
            ProducerKind::TransientHash
        );
        assert_eq!(finding.parameter_dependencies[0].name, "ordinary");
        assert_eq!(finding.control_parameter_dependencies[0].name, "value");
        assert!(finding.has_unknown_input);
        assert!(finding.has_unknown_control);
    }
}

#[test]
fn source_plain_and_default_index_writes_remain_valid_unknown_inputs() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    for name in [
        "list_plain",
        "tree_plain",
        "tree_default_position",
        "history_default_position",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.circuit == name)
        );
        assert!(!report.unavailable.iter().any(|entry| entry.circuit == name));
    }
    for name in ["tree_default_position", "history_default_position"] {
        let circuit = contract
            .stateful_circuits
            .iter()
            .find(|circuit| circuit.name == name)
            .unwrap();
        assert!(matches!(
            circuit.actions[0],
            StateAction::MerkleInsertIndexDefault { .. }
                | StateAction::HistoricMerkleInsertIndexDefault { .. }
        ));
    }
}

#[test]
fn synthetic_ir_indexed_merkle_actions_report_distinct_position_origin() {
    // Current Compact source compilation rejects Field-to-Uint downcasts. This
    // schema-20 shape isolates the indexed action handlers without claiming a
    // source-reachable transient position.
    let contract = source_contract();
    let mut circuits = Vec::new();
    for (historic, field, index, name) in [
        (false, "tree", 1, "tree_hash_index_hit"),
        (true, "history", 2, "history_hash_index_hit"),
    ] {
        let source = contract
            .stateful_circuits
            .iter()
            .find(|circuit| circuit.name == name)
            .unwrap();
        for default_value in [false, true] {
            let mut circuit = source.clone();
            circuit.name = format!(
                "{field}_position_{}",
                if default_value { "default" } else { "hash" }
            );
            let position = Expr::UnsignedCast {
                max: "18446744073709551615".into(),
                value: Box::new(Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                }),
            };
            circuit.actions = vec![match (historic, default_value) {
                (false, false) => StateAction::MerkleInsertHashIndex {
                    field: field.into(),
                    index,
                    hash: Expr::BytesLiteral { bytes: vec![0; 32] },
                    position,
                },
                (true, false) => StateAction::HistoricMerkleInsertHashIndex {
                    field: field.into(),
                    index,
                    hash: Expr::BytesLiteral { bytes: vec![0; 32] },
                    position,
                },
                (false, true) => StateAction::MerkleInsertIndexDefault {
                    field: field.into(),
                    index,
                    position,
                },
                (true, true) => StateAction::HistoricMerkleInsertIndexDefault {
                    field: field.into(),
                    index,
                    position,
                },
            }];
            circuits.push(circuit);
        }
    }
    let mut contract = contract;
    contract.circuits.clear();
    contract.stateful_circuits = circuits;
    let report = analyze(&contract, Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(report.findings.len(), 4);
    for finding in &report.findings {
        assert_eq!(finding.input, "position");
        assert_eq!(finding.routes.len(), 1);
        assert_eq!(finding.routes[0].producer.kind, ProducerKind::TransientHash);
        assert_eq!(finding.parameter_dependencies[0].name, "value");
        assert!(
            finding.routes[0]
                .dependencies
                .iter()
                .any(|step| step.operation == "conversion")
        );
    }
    assert_eq!(report, analyze(&contract, Limits::default()));
}
