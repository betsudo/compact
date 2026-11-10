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

use compact_rust_backend::ir::{Contract, Expr, ReturnPlan, StateReturn};
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-effectful-returns-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_effectful_value_calls_preserve_producer_forwarding_and_control() {
    let report = analyze(&source_contract(), Limits::default());
    assert!(report.unavailable.is_empty(), "{:#?}", report.unavailable);
    let produced = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "produced" && finding.sink == "persistent_hash")
        .unwrap();
    assert_eq!(produced.routes[0].producer.circuit, "choose");
    assert!(
        produced.routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "call_return:choose")
    );
    let forwarded = report
        .findings
        .iter()
        .filter(|finding| finding.circuit == "forwarded")
        .collect::<Vec<_>>();
    assert_eq!(
        forwarded
            .iter()
            .filter(|finding| finding.sink == "cell_write")
            .count(),
        2
    );
    assert_eq!(
        forwarded
            .iter()
            .filter(|finding| finding.sink == "persistent_hash")
            .count(),
        1
    );
    assert!(forwarded.iter().all(|finding| {
        finding
            .routes
            .iter()
            .any(|route| route.producer.circuit == "forwarded")
    }));
    let controlled = report
        .findings
        .iter()
        .filter(|finding| finding.circuit == "control")
        .collect::<Vec<_>>();
    assert_eq!(
        controlled
            .iter()
            .filter(|finding| finding.sink == "cell_write" && !finding.control_routes.is_empty())
            .count(),
        2
    );
    assert!(
        controlled
            .iter()
            .any(|finding| finding.sink == "persistent_hash"
                && finding.routes.iter().any(|route| route
                    .dependencies
                    .iter()
                    .any(|step| step.operation == "conditional_control")))
    );
    let local = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "local_binding" && finding.sink == "persistent_hash")
        .unwrap();
    assert_eq!(local.routes[0].producer.circuit, "local");
    assert!(
        local.routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "binding:digest")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "ordinary")
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn identical_literal_effectful_arms_do_not_inherit_transient_control() {
    let mut contract = source_contract();
    let helper = contract
        .stateful_circuits
        .iter_mut()
        .find(|circuit| circuit.name == "controlled")
        .unwrap();
    helper.return_value = StateReturn::Effectful {
        body: ReturnPlan::Conditional {
            condition: Expr::Equal {
                left: Box::new(Expr::TransientHash {
                    value: Box::new(Expr::Parameter {
                        name: "value".into(),
                    }),
                }),
                right: Box::new(Expr::FieldLiteral { value: "0".into() }),
            },
            then: Box::new(ReturnPlan::Value {
                value: Expr::FieldLiteral { value: "1".into() },
            }),
            otherwise: Box::new(ReturnPlan::Value {
                value: Expr::FieldLiteral { value: "1".into() },
            }),
        },
    };
    let report = analyze(&contract, Limits::default());
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "control")
    );
    assert!(report.unavailable.is_empty());
}
