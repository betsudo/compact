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
use compact_rust_backend::transient_provenance::{Limits, analyze};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-pure-actions-source-ir.json"
    ))
    .unwrap()
}

#[test]
fn source_pure_action_helpers_preserve_caller_and_helper_producers() {
    let report = analyze(&source_contract(), Limits::default());
    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.circuit.as_str())
            .collect::<Vec<_>>(),
        ["make_and_check", "forwarded", "produced",]
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.sink == "persistent_hash" && finding.sink_circuit == "check")
    );
    assert_eq!(
        report.findings[0].routes[0].producer.circuit,
        "make_and_check"
    );
    assert_eq!(report.findings[1].routes[0].producer.circuit, "forwarded");
    assert_eq!(
        report.findings[2].routes[0].producer.circuit,
        "make_and_check"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.circuit == "ordinary")
    );
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| entry.reason == "assertion execution control unavailable")
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn invalid_pure_action_call_does_not_visit_argument_sinks() {
    let mut contract = source_contract();
    contract.circuits.clear();
    contract
        .stateful_circuits
        .retain(|circuit| circuit.name == "forwarded");
    contract.stateful_circuits[0].actions = vec![StateAction::PureCall {
        name: "missing".into(),
        arguments: vec![Expr::PersistentHash {
            value: Box::new(Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }),
        }],
    }];
    let report = analyze(&contract, Limits::default());
    assert!(report.findings.is_empty());
    assert_eq!(report.unavailable[0].reason, "unknown pure action helper");
}

#[test]
fn pure_action_calls_report_arity_and_depth_limits() {
    let mut contract = source_contract();
    let forwarded = contract
        .stateful_circuits
        .iter_mut()
        .find(|circuit| circuit.name == "forwarded")
        .unwrap();
    let StateAction::PureCall { arguments, .. } = &mut forwarded.actions[0] else {
        panic!("expected source pure action call");
    };
    arguments.clear();
    let wrong_arity = analyze(&contract, Limits::default());
    assert!(
        wrong_arity
            .unavailable
            .iter()
            .any(|entry| entry.circuit == "forwarded"
                && entry.reason == "pure action helper arity mismatch")
    );
    assert!(
        !wrong_arity
            .findings
            .iter()
            .any(|finding| finding.circuit == "forwarded")
    );

    let tight = analyze(
        &source_contract(),
        Limits {
            max_call_depth: 0,
            ..Limits::default()
        },
    );
    assert!(tight.truncated);
    assert!(
        tight
            .unavailable
            .iter()
            .any(|entry| entry.reason == "pure action helper call depth limit")
    );
}

#[test]
fn recursive_pure_action_helper_is_unavailable() {
    let mut contract = source_contract();
    let check = contract
        .circuits
        .iter_mut()
        .find(|circuit| circuit.name == "check")
        .unwrap();
    check.body = Expr::Call {
        name: "check".into(),
        arguments: vec![Expr::Parameter {
            name: "value".into(),
        }],
    };

    let report = analyze(&contract, Limits::default());
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| { entry.circuit == "check" && entry.reason == "recursive pure helper" })
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.sink_circuit == "check")
    );
}
