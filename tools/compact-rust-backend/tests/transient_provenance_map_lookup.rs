// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::{Contract, Expr, StateReturn};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-map-lookup-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_lookup_selection_reaches_state_and_persistent_sinks() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    assert_eq!(report, analyze(&contract, Limits::default()));
    assert!(report.unavailable.is_empty());
    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| (finding.circuit.as_str(), finding.sink.as_str()))
            .collect::<Vec<_>>(),
        [
            ("state_hit", "cell_write"),
            ("hash_hit", "persistent_hash"),
            ("helper_hit", "persistent_hash"),
        ]
    );
    for finding in &report.findings {
        assert_eq!(finding.routes.len(), 1);
        assert!(finding.has_unknown_input);
        assert!(
            finding.routes[0]
                .dependencies
                .iter()
                .any(|step| step.operation == "map_lookup_selection")
        );
    }
    assert_eq!(report.findings[0].routes[0].producer.circuit, "state_hit");
    assert_eq!(report.findings[1].routes[0].producer.circuit, "hash_hit");
    assert_eq!(report.findings[2].routes[0].producer.circuit, "selected");
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "ordinary")
    );
}

#[test]
fn typed_map_lookup_return_keeps_helper_selection_origin() {
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
    let helper = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "helper_hit")
        .unwrap();
    assert_eq!(helper.routes.len(), 1);
    assert!(helper.has_unknown_input);
    assert_eq!(helper.routes[0].producer.circuit, "selected");
    assert!(
        helper.routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "map_lookup_selection")
    );
    assert!(!report.unavailable.iter().any(|entry| {
        entry.circuit == "helper_hit"
            && entry.reason == "non-expression stateful value result unavailable"
    }));
}
