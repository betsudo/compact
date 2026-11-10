// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr, StateReturn};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-membership-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_membership_conditions_keep_known_key_and_unknown_ledger_origins() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    assert_eq!(report, analyze(&contract, Limits::default()));
    assert!(report.unavailable.is_empty());
    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.circuit.as_str())
            .collect::<Vec<_>>(),
        ["set_gate", "map_gate", "helper_gate"]
    );
    for finding in &report.findings {
        assert_eq!(finding.sink, "cell_write");
        assert!(finding.routes.is_empty());
        assert_eq!(finding.control_routes.len(), 1);
        assert!(finding.has_unknown_control);
        assert!(
            finding.control_routes[0]
                .dependencies
                .iter()
                .any(|step| step.operation.ends_with("membership"))
        );
    }
    assert_eq!(
        report.findings[0].control_routes[0].producer.circuit,
        "set_gate"
    );
    assert_eq!(
        report.findings[1].control_routes[0].producer.circuit,
        "map_gate"
    );
    assert_eq!(
        report.findings[2].control_routes[0].producer.circuit,
        "has_transient_member"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "ordinary_gate")
    );
}

#[test]
fn typed_state_membership_returns_preserve_helper_control_routes() {
    for map in [false, true] {
        let mut contract = source_contract();
        let helper = contract
            .stateful_circuits
            .iter_mut()
            .find(|circuit| circuit.name == "has_transient_member")
            .unwrap();
        let value = Expr::TransientHash {
            value: Box::new(Expr::Parameter {
                name: "value".into(),
            }),
        };
        helper.return_value = if map {
            StateReturn::MapMember {
                field: "entries".into(),
                index: 2,
                key: value,
            }
        } else {
            StateReturn::SetMember {
                field: "members".into(),
                index: 1,
                value,
            }
        };
        let report = analyze(&contract, Limits::default());
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.circuit == "helper_gate")
            .unwrap();
        assert_eq!(finding.control_routes.len(), 1);
        assert!(finding.has_unknown_control);
        assert_eq!(
            finding.control_routes[0].producer.circuit,
            "has_transient_member"
        );
        assert!(!report.unavailable.iter().any(|entry| {
            entry.circuit == "helper_gate"
                && entry.reason == "non-expression stateful value result unavailable"
        }));
    }
}
