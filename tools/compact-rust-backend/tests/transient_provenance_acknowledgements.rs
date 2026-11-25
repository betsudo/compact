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
use compact_rust_backend::transient_provenance::{
    Acknowledgement, DependencyRole, Finding, Limits, advise, analyze,
};

fn source_contract() -> Contract {
    serde_json::from_str(include_str!(
        "transient-provenance-value-calls-source-ir.json"
    ))
    .unwrap()
}

fn acknowledgement(finding: &Finding) -> Acknowledgement {
    Acknowledgement {
        circuit: finding.circuit.clone(),
        sink: finding.sink.clone(),
        sink_circuit: finding.sink_circuit.clone(),
        sink_path: finding.sink_path.clone(),
        input: finding.input.clone(),
        role: DependencyRole::Data,
        route: finding.routes[0].clone(),
        reason: "protocol nonce is intentionally upgraded".into(),
    }
}

#[test]
fn exact_standard_library_acknowledgement_preserves_evidence() {
    let report = analyze(&source_contract(), Limits::default());
    let nonce = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "standard_nonce")
        .unwrap();
    assert_eq!(nonce.routes[0].producer.circuit, "evolveNonce");
    assert!(
        nonce.routes[0]
            .dependencies
            .iter()
            .any(|step| step.operation == "conversion")
    );
    let policy = advise(&report, &[acknowledgement(nonce)], 4);
    assert!(policy.unavailable.is_empty());
    assert!(policy.unused_acknowledgements.is_empty());
    assert_eq!(
        policy
            .findings
            .iter()
            .map(|finding| finding.warning)
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    assert_eq!(
        policy
            .findings
            .iter()
            .map(|finding| &finding.finding)
            .collect::<Vec<_>>(),
        report.findings.iter().collect::<Vec<_>>()
    );
    assert_eq!(report, analyze(&source_contract(), Limits::default()));
}

#[test]
fn stale_blank_and_over_budget_acknowledgements_never_hide_warning() {
    let report = analyze(&source_contract(), Limits::default());
    let nonce = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "standard_nonce")
        .unwrap();
    let mut stale = acknowledgement(nonce);
    stale.input = "opening".into();
    let mut blank = acknowledgement(nonce);
    blank.reason = "  ".into();
    let mut wrong_sink = acknowledgement(nonce);
    wrong_sink.sink = "persistent_commit".into();
    let policy = advise(&report, &[stale, blank, wrong_sink], 4);
    assert!(policy.findings.iter().all(|finding| finding.warning));
    assert_eq!(policy.unused_acknowledgements.len(), 3);

    let over_budget = advise(&report, &[acknowledgement(nonce)], 0);
    assert!(over_budget.truncated);
    assert_eq!(
        over_budget.unavailable.last().unwrap().reason,
        "acknowledgement limit"
    );
    assert!(over_budget.findings.iter().all(|finding| finding.warning));
}

#[test]
fn every_data_and_control_route_requires_acknowledgement() {
    let mut report = analyze(&source_contract(), Limits::default());
    let extra = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "forwarded")
        .unwrap()
        .routes[0]
        .clone();
    let mut nonce = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "standard_nonce")
        .unwrap()
        .clone();
    nonce.routes.push(extra.clone());
    nonce.control_routes.push(extra.clone());
    report.findings = vec![nonce.clone()];
    let one = acknowledgement(&nonce);
    let partial = advise(&report, std::slice::from_ref(&one), 4);
    assert_eq!(partial.findings[0].acknowledged_data_routes, [true, false]);
    assert_eq!(partial.findings[0].acknowledged_control_routes, [false]);
    assert!(partial.findings[0].warning);
    assert_eq!(partial.findings[0].finding, nonce);

    let mut two = one.clone();
    two.route = extra.clone();
    let data_only = advise(&report, &[one.clone(), two.clone()], 4);
    assert_eq!(data_only.findings[0].acknowledged_data_routes, [true, true]);
    assert_eq!(data_only.findings[0].acknowledged_control_routes, [false]);
    assert!(data_only.findings[0].warning);

    let mut control = two.clone();
    control.role = DependencyRole::Control;
    let complete = advise(&report, &[one, two, control], 4);
    assert_eq!(complete.findings[0].acknowledged_data_routes, [true, true]);
    assert_eq!(complete.findings[0].acknowledged_control_routes, [true]);
    assert!(!complete.findings[0].warning);
}
