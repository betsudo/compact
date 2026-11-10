// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_rust_backend::ir::Contract;
use compact_rust_backend::transient_provenance::{
    Acknowledgement, DependencyRole, Limits, advise, analyze,
};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

fn source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/transient-provenance-map-lookup-source-ir.json")
}

fn source_contract() -> Contract {
    serde_json::from_slice(&std::fs::read(source_path()).unwrap()).unwrap()
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_compact-transient-provenance"))
}

struct TemporaryJson(PathBuf);

impl TemporaryJson {
    fn new(bytes: &[u8]) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "compact-transient-provenance-{}-{stamp}-{}.json",
            std::process::id(),
            NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
}

impl Drop for TemporaryJson {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn source_ir_cli_emits_same_bounded_advisory_without_mutating_compilation() {
    let expected = advise(&analyze(&source_contract(), Limits::default()), &[], 128);
    let output = binary().arg(source_path()).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(expected.findings.iter().any(|finding| finding.warning));
    assert!(
        !expected
            .findings
            .iter()
            .any(|finding| finding.finding.circuit == "ordinary")
    );
}

#[test]
fn exact_acknowledgement_changes_warning_only_and_preserves_evidence() {
    let contract = source_contract();
    let report = analyze(&contract, Limits::default());
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.circuit == "helper_hit")
        .unwrap();
    let acknowledgement = Acknowledgement {
        circuit: finding.circuit.clone(),
        sink: finding.sink.clone(),
        sink_circuit: finding.sink_circuit.clone(),
        sink_path: finding.sink_path.clone(),
        input: finding.input.clone(),
        role: DependencyRole::Data,
        route: finding.routes[0].clone(),
        reason: "intentional compatibility path".into(),
    };
    let file = TemporaryJson::new(&serde_json::to_vec(&vec![acknowledgement.clone()]).unwrap());
    let output = binary()
        .arg(source_path())
        .arg("--acknowledgements")
        .arg(&file.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = advise(&report, &[acknowledgement], 128);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(
        !expected
            .findings
            .iter()
            .find(|finding| finding.finding.circuit == "helper_hit")
            .unwrap()
            .warning
    );
    assert!(expected.findings.iter().any(|finding| finding.warning));
    assert!(expected.unused_acknowledgements.is_empty());
}

#[test]
fn malformed_acknowledgement_is_an_error_without_partial_report() {
    let file = TemporaryJson::new(br#"[{"unexpected":true}]"#);
    let output = binary()
        .arg(source_path())
        .arg("--acknowledgements")
        .arg(&file.0)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown field"));
}

#[test]
fn wrong_schema_is_rejected_without_a_partial_report() {
    let mut contract = source_contract();
    contract.schema_version += 1;
    let file = TemporaryJson::new(&serde_json::to_vec(&contract).unwrap());
    let output = binary().arg(&file.0).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unsupported typed IR schema"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn oversized_name_is_rejected_before_analysis() {
    let mut contract = source_contract();
    contract.stateful_circuits[0].name = "x".repeat(513);
    let file = TemporaryJson::new(&serde_json::to_vec(&contract).unwrap());
    let output = binary().arg(&file.0).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("typed IR string exceeds"));
}

#[test]
fn oversized_json_node_count_is_rejected_before_typed_deserialization() {
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source_path()).unwrap()).unwrap();
    value["extra_nodes"] = serde_json::Value::Array(vec![serde_json::Value::Null; 16_384]);
    let file = TemporaryJson::new(&serde_json::to_vec(&value).unwrap());
    let output = binary().arg(&file.0).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("JSON nodes"));
}

#[test]
fn duplicate_typed_ir_field_is_rejected_without_a_partial_report() {
    let source = std::fs::read(source_path()).unwrap();
    let mut duplicate = br#"{"schema_version":20,"#.to_vec();
    duplicate.extend_from_slice(&source[1..]);
    let file = TemporaryJson::new(&duplicate);
    let output = binary().arg(&file.0).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate field"));
}

#[test]
fn acknowledgement_count_limit_is_explicit_and_applies_none() {
    let report = analyze(&source_contract(), Limits::default());
    let finding = &report.findings[0];
    let acknowledgement = Acknowledgement {
        circuit: finding.circuit.clone(),
        sink: finding.sink.clone(),
        sink_circuit: finding.sink_circuit.clone(),
        sink_path: finding.sink_path.clone(),
        input: finding.input.clone(),
        role: DependencyRole::Data,
        route: finding.routes[0].clone(),
        reason: "intentional".into(),
    };
    let file = TemporaryJson::new(&serde_json::to_vec(&vec![acknowledgement; 129]).unwrap());
    let output = binary()
        .arg(source_path())
        .arg("--acknowledgements")
        .arg(&file.0)
        .output()
        .unwrap();
    assert!(output.status.success());
    let advisory: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(advisory["truncated"], true);
    assert!(
        advisory["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["warning"] == true)
    );
    assert!(
        advisory["unavailable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["reason"] == "acknowledgement limit")
    );
}
