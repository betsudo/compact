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
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-field-bytes-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_field_to_bytes_cast_preserves_transient_sink_routes() {
    let contract = source_contract();
    for name in ["cast_hit", "opening_hit"] {
        let circuit = contract
            .circuits
            .iter()
            .find(|circuit| circuit.name == name)
            .unwrap();
        let Expr::Let { bindings, .. } = &circuit.body else {
            panic!("expected source binding in {name}");
        };
        assert!(matches!(bindings[0].value, Expr::FieldToBytes32 { .. }));
    }
    let ordinary = contract
        .circuits
        .iter()
        .find(|circuit| circuit.name == "cast_plain")
        .unwrap();
    let Expr::PersistentHash { value } = &ordinary.body else {
        panic!("expected source persistent hash");
    };
    assert!(matches!(value.as_ref(), Expr::FieldToBytes32 { .. }));

    let report = analyze(&contract, Limits::default());
    assert_eq!(report, analyze(&contract, Limits::default()));
    assert!(report.unavailable.is_empty());
    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| {
                (
                    finding.circuit.as_str(),
                    finding.sink.as_str(),
                    finding.input.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        [
            ("cast_hit", "persistent_hash", "value"),
            ("opening_hit", "persistent_commit", "opening"),
        ]
    );
    for finding in &report.findings {
        assert_eq!(finding.routes.len(), 1);
        assert_eq!(finding.routes[0].producer.circuit, finding.circuit);
        assert!(
            finding.routes[0]
                .dependencies
                .iter()
                .any(|step| step.operation == "conversion")
        );
    }
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "cast_plain")
    );
}
