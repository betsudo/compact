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

//! Optional compiler-facing text transport for the typed-IR advisory report.

use compact_rust_backend::ir::{Contract, SourceLocation};
use compact_rust_backend::transient_provenance::{
    Acknowledgement, AdvisoryReport, DependencyRole, Limits, ProducerKind, Route, advise, analyze,
};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub(crate) const MAX_ACKNOWLEDGEMENTS: usize = 128;

pub(crate) fn read_acknowledgements(
    path: Option<&Path>,
) -> Result<Vec<Acknowledgement>, Box<dyn Error>> {
    match path {
        Some(path) => Ok(serde_json::from_slice(&crate::input::read(File::open(
            path,
        )?)?)?),
        None => Ok(Vec::new()),
    }
}

pub(crate) fn analyze_advisory(
    contract: &Contract,
    acknowledgements: &[Acknowledgement],
) -> AdvisoryReport {
    advise(
        &analyze(contract, Limits::default()),
        acknowledgements,
        MAX_ACKNOWLEDGEMENTS,
    )
}

fn clipped(value: &str) -> String {
    const MAX_CHARS: usize = 160;
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn source(source: Option<&SourceLocation>) -> String {
    source.map_or_else(
        || "unknown source".into(),
        |source| {
            format!(
                "{:?}:{}:{}",
                clipped(&source.file),
                source.line,
                source.column
            )
        },
    )
}

fn route_line(
    compiler: &str,
    finding: &compact_rust_backend::transient_provenance::Finding,
    route: &Route,
    role: DependencyRole,
    acknowledged: bool,
    output: &mut impl Write,
) -> io::Result<()> {
    let severity = if acknowledged {
        "note[transient-persistence-acknowledged]"
    } else {
        "warning[transient-persistence]"
    };
    let kind = match route.producer.kind {
        ProducerKind::TransientHash => "transientHash",
        ProducerKind::TransientCommit => "transientCommit",
    };
    let role = match role {
        DependencyRole::Data => "data",
        DependencyRole::Control => "possible control",
    };
    let flow = route
        .dependencies
        .iter()
        .map(|step| format!("{}@{}", clipped(&step.operation), clipped(&step.path)))
        .collect::<Vec<_>>()
        .join(" -> ");
    writeln!(
        output,
        "{compiler}: {severity}: {}: circuit {:?} sink {:?}/{:?} in {:?} ({role}) depends on {kind} in {:?}; producer={:?}; sink={:?}; flow={:?}; unknown_input={}; unknown_control={}",
        source(finding.sink_source.as_ref().or(finding.source.as_ref())),
        clipped(&finding.circuit),
        clipped(&finding.sink),
        clipped(&finding.input),
        clipped(&finding.sink_circuit),
        clipped(&route.producer.circuit),
        clipped(&route.producer.path),
        clipped(&finding.sink_path),
        flow,
        finding.has_unknown_input,
        finding.has_unknown_control,
    )
}

pub(crate) fn write_advisory(
    compiler: &str,
    report: &AdvisoryReport,
    output: &mut impl Write,
) -> io::Result<()> {
    for advisory in &report.findings {
        for (route, acknowledged) in advisory
            .finding
            .routes
            .iter()
            .zip(&advisory.acknowledged_data_routes)
        {
            route_line(
                compiler,
                &advisory.finding,
                route,
                DependencyRole::Data,
                *acknowledged,
                output,
            )?;
        }
        for (route, acknowledged) in advisory
            .finding
            .control_routes
            .iter()
            .zip(&advisory.acknowledged_control_routes)
        {
            route_line(
                compiler,
                &advisory.finding,
                route,
                DependencyRole::Control,
                *acknowledged,
                output,
            )?;
        }
    }
    for unavailable in &report.unavailable {
        writeln!(
            output,
            "{compiler}: note[transient-analysis-unavailable]: circuit {:?} at {:?}: {}",
            clipped(&unavailable.circuit),
            clipped(&unavailable.path),
            clipped(&unavailable.reason),
        )?;
    }
    for acknowledgement in &report.unused_acknowledgements {
        writeln!(
            output,
            "{compiler}: note[transient-acknowledgement-unused]: circuit {:?} sink {:?} at {:?}",
            clipped(&acknowledgement.circuit),
            clipped(&acknowledgement.sink),
            clipped(&acknowledgement.sink_path),
        )?;
    }
    writeln!(
        output,
        "{compiler}: note[transient-provenance]: {} finding(s), {} unavailable, truncated={}; no finding certifies stability",
        report.findings.len(),
        report.unavailable.len(),
        report.truncated,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_contract() -> Contract {
        serde_json::from_str(include_str!(
            "../../tests/transient-provenance-source-ir.json"
        ))
        .unwrap()
    }

    #[test]
    fn source_filename_cannot_inject_diagnostic_lines() {
        let position = SourceLocation {
            file: "contract.compact\ncompactc: forged warning".into(),
            line: 9,
            column: 1,
        };
        let formatted = source(Some(&position));
        assert_eq!(formatted.lines().count(), 1);
        assert!(formatted.contains("\\ncompactc: forged warning"));
    }

    #[test]
    fn source_backed_warnings_and_controls_are_deterministic() {
        let contract = source_contract();
        let report = analyze_advisory(&contract, &[]);
        let mut first = Vec::new();
        write_advisory("compactc", &report, &mut first).unwrap();
        let mut second = Vec::new();
        write_advisory("compactc", &report, &mut second).unwrap();
        assert_eq!(first, second);
        let text = String::from_utf8(first).unwrap();
        assert_eq!(text.matches("warning[transient-persistence]").count(), 5);
        assert!(text.contains("circuit \"converted\""));
        assert!(text.contains("conversion@"));
        assert!(!text.contains("circuit \"ordinary\""));
        assert!(!text.contains("circuit \"ordinary_witness\""));
        assert!(text.contains("no finding certifies stability"));
    }

    #[test]
    fn exact_acknowledgement_changes_severity_and_preserves_evidence() {
        let contract = source_contract();
        let raw = analyze(&contract, Limits::default());
        let finding = &raw.findings[0];
        let acknowledgement = Acknowledgement {
            circuit: finding.circuit.clone(),
            sink: finding.sink.clone(),
            sink_circuit: finding.sink_circuit.clone(),
            sink_path: finding.sink_path.clone(),
            input: finding.input.clone(),
            role: DependencyRole::Data,
            route: finding.routes[0].clone(),
            reason: "reviewed transient use".into(),
        };
        let report = analyze_advisory(&contract, &[acknowledgement]);
        assert_eq!(report.findings[0].finding, *finding);
        let mut output = Vec::new();
        write_advisory("compactc", &report, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert_eq!(
            text.matches("note[transient-persistence-acknowledged]")
                .count(),
            1
        );
        assert_eq!(text.matches("warning[transient-persistence]").count(), 4);
    }
}
