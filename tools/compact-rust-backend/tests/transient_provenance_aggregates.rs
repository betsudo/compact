// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-aggregates-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_products_projections_and_branches_keep_precise_evidence() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.circuit.as_str())
            .collect::<Vec<_>>(),
        [
            "struct_hit",
            "tuple_hit",
            "vector_hit",
            "whole_product",
            "branch_data",
            "branch_struct",
            "branch_control",
            "branch_sink",
        ]
    );
    assert!(!report.findings.iter().any(|f| {
        [
            "struct_miss",
            "tuple_miss",
            "vector_miss",
            "branch_struct_miss",
        ]
        .contains(&f.circuit.as_str())
    }));
    assert!(
        report.findings[0].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "projection:first")
    );
    assert!(
        report.findings[1].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "tuple_projection")
    );
    assert!(report.findings[4].has_unknown_input);
    assert!(report.findings[5].has_unknown_input);
    assert!(
        report.findings[6].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "comparison")
    );
    assert!(
        report.findings[6].routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "conditional_control")
    );
    assert_eq!(report.findings[7].conditions, ["body:then"]);
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn wide_product_stops_at_node_budget() {
    let mut contract = source_contract();
    contract.circuits.truncate(1);
    contract.circuits[0].body = Expr::PersistentHash {
        value: Box::new(Expr::Tuple {
            elements: (0..256)
                .map(|n| Expr::FieldLiteral {
                    value: n.to_string(),
                })
                .collect(),
        }),
    };
    let report = analyze(
        &contract,
        Limits {
            max_nodes: 3,
            ..Limits::default()
        },
    );
    assert!(report.findings.is_empty());
    assert!(report.truncated);
    assert_eq!(report.unavailable.len(), 1);
    assert_eq!(report.unavailable[0].reason, "expression node limit");
}

#[test]
fn identical_literal_arms_do_not_inherit_control_origin() {
    let mut contract = source_contract();
    contract.circuits.truncate(1);
    contract.circuits[0].body = Expr::PersistentHash {
        value: Box::new(Expr::If {
            condition: Box::new(Expr::Equal {
                left: Box::new(Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                }),
                right: Box::new(Expr::FieldLiteral { value: "0".into() }),
            }),
            then: Box::new(Expr::FieldLiteral { value: "1".into() }),
            otherwise: Box::new(Expr::FieldLiteral { value: "1".into() }),
        }),
    };
    let report = analyze(&contract, Limits::default());
    assert!(report.findings.is_empty());
    assert!(report.unavailable.is_empty());
}
