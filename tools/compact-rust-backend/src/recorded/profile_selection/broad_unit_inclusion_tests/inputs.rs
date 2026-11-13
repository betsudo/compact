// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Corpus identity and filesystem boundaries are separate from compiler outcomes.

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    path::{Component, Path},
};

pub(super) const MANIFEST_SHA256: &str =
    "36a663979756572cd6a9d10ea6bf122a7e2d1d78ed0504244bede61f8038d6b4";

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Case {
    pub id: String,
    pub file: String,
    pub root: String,
    pub rules: Vec<String>,
    pub sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    scope: String,
    cases: Vec<Case>,
}

pub(super) struct Input {
    pub case: Case,
    pub bytes: Vec<u8>,
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

fn validate_cases(cases: &[Case]) -> Result<(), String> {
    let mut ids = HashSet::new();
    let mut files = HashSet::new();
    for case in cases {
        if case.id.is_empty()
            || !case
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(format!("unsafe case identifier: {}", case.id));
        }
        if !relative_path(&case.file) || case.root.is_empty() || case.rules.is_empty() {
            return Err(format!("invalid case metadata: {}", case.id));
        }
        if !ids.insert(&case.id) || !files.insert(&case.file) {
            return Err(format!("duplicate case identity or input: {}", case.id));
        }
        if case.sha256.len() != 64
            || !case
                .sha256
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(format!("invalid input digest: {}", case.id));
        }
    }
    Ok(())
}

fn authenticated_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    if digest(bytes) != MANIFEST_SHA256 {
        return Err("original matrix manifest digest mismatch".into());
    }
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if manifest.cases.len() != 163 || manifest.scope.is_empty() {
        return Err("original matrix shape mismatch".into());
    }
    validate_cases(&manifest.cases)?;
    Ok(manifest)
}

pub(super) fn load(path: &Path) -> Result<Vec<Input>, String> {
    let manifest = authenticated_manifest(&std::fs::read(path).map_err(|e| e.to_string())?)?;
    let base = path
        .parent()
        .ok_or("matrix has no parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut canonical_inputs = HashSet::new();
    let mut inputs = Vec::new();
    for case in manifest.cases {
        let path = base
            .join(&case.file)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(&base) || !canonical_inputs.insert(path.clone()) {
            return Err(format!("escaped or duplicate canonical input: {}", case.id));
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        if digest(&bytes) != case.sha256 {
            return Err(format!("input digest mismatch: {}", case.id));
        }
        inputs.push(Input { case, bytes });
    }
    Ok(inputs)
}

const RETIREMENT_SHA256: &str = "eda751f75344d765430226e2651201e947c1d1d584818c43fe024424df65df0c";

fn retirement_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != RETIREMENT_SHA256 {
        return Err("unreviewed legacy retirement manifest".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// #597 added only optional resource metadata to this retained oracle owner.
// Authenticate its immutable checkpoint before the original retirement projection.
const RESOURCE_METADATA_SHA256: &str =
    "a4d3559923d732989a8245bb1984f131479feba07554a2e5006dec16b89f8145";
const RESOURCE_METADATA: &[u8] =
    include_bytes!("../../../../recording_admission_inventory.resource-diagnostics-597.json");

fn resource_metadata_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != RESOURCE_METADATA_SHA256 {
        return Err("unreviewed resource metadata checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// #611 replaces obsolete public refusal controls in one test-only oracle owner.
// Pin this separate checkpoint to avoid a self-referential inventory manifest.
const COMMIT_INPUTS_SHA256: &str =
    "4feeb870cede6548b17cc870c6d67f545274f9c98c73923e691acca665c45caf";
const COMMIT_INPUTS: &[u8] = include_bytes!("legacy-commit-inputs-611.json");

fn commit_inputs_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != COMMIT_INPUTS_SHA256 {
        return Err("unreviewed commitment test checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// #614 preserves the complete early audited-local renderer as a test-only oracle.
// Reverse only its two authenticated cfg annotations before older projections.
const AUDITED_LOCAL_SHA256: &str =
    "3e433671c8b8c9086350ccbbe2f9898385a258a6482b5c97ac378f28014c9bcf";
const AUDITED_LOCAL: &[u8] = include_bytes!("legacy-audited-local-614.json");

fn audited_local_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != AUDITED_LOCAL_SHA256 {
        return Err("unreviewed audited-local checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// A new closed Expr variant needs one explicit refusal in the old oracle.
// Authenticate and remove exactly that arm before the unchanged #614 stage.
const MAP_SIZE_ENUM_SHA256: &str =
    "940108ed736f9e14e505cd4a60bd1ce2956ca0d93f768c86bda17d330c3be775";
const MAP_SIZE_ENUM: &[u8] = include_bytes!("map-size-expression-enum-610.json");

fn map_size_enum_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != MAP_SIZE_ENUM_SHA256 {
        return Err("unreviewed Map size enum compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// #661 adds only explicit new-tag denials to retired exhaustive matchers.
// Reconstruct exact predecessor bytes before the unchanged older projections.
const FOLD_ENUM_SHA256: &str = "fcb487313efc00ce6931ba0e57c41daf63f56f6060a01de916ff1a5a558e8b15";
const FOLD_ENUM: &[u8] = include_bytes!("fold-expression-enum-661.json");
fn fold_enum_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != FOLD_ENUM_SHA256 {
        return Err("unreviewed Fold enum compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// SHA-512 adds one operand to historical traversals and one old-oracle refusal.
const SHA512_ENUM: &[u8] = include_bytes!("sha512-expression-enum.json");
fn sha512_enum_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "665d8a3ed631131bc5569292e644d3d3ce8dde1c6703229df45070990de31558" {
        return Err("unreviewed SHA-512 enum compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}
fn before_sha512_enum(path: &str, current: &str) -> Result<String, String> {
    let before_secp256k1 = before_secp256k1_cast(path, current)?;
    legacy_source(path, &before_secp256k1, &sha512_enum_manifest(SHA512_ENUM)?)
}

const SECP256K1_CAST: &[u8] = include_bytes!("secp256k1-bytes-cast-expression.json");
fn secp256k1_cast_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "2c0c3f6b849fe7db1f8a141ff2db1c41e2eac36a52a7e70f957b0c1745f227a8" {
        return Err("unreviewed secp256k1 cast compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}
// Fixed byte conversions add two closed variants to retained exhaustive matchers.
const FIXED_BYTES_ENUM: &[u8] = include_bytes!("fixed-bytes-expression.json");
fn fixed_bytes_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "a2da256f5d860e8359ced0c98c69bd0fd12a830466ec795c93db4b9e80f3f1d2" {
        return Err("unreviewed fixed bytes compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}
const VECTOR_TO_BYTES: &[u8] = include_bytes!("vector-to-bytes-expression.json");
fn vector_to_bytes_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "7cf84de7b79af54d0fe946b1d18fb1d25ca9e7311433cb056c029b14eba10117" {
        return Err("unreviewed Vector-to-Bytes compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

const JUBJUB_CAST: &[u8] = include_bytes!("jubjub-scalar-cast-expression.json");

fn jubjub_cast_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "efd83beeb5ea8735b50220ac676f889c29b85e37b4c06dd1d8c4c1c81a94bd7d" {
        return Err("unreviewed Jubjub scalar cast compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

const NESTED_MAP: &[u8] = include_bytes!("nested-map-expression.json");

fn nested_map_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "16626bbcfeecb3ec6fc44039d7b3c71fe425fe4a6b4383f78c590d9cb79f1c07" {
        return Err("unreviewed nested Map compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// The event variant extends the historical walkers with an explicit operand or refusal.
const SHIELDED_SPEND_EVENT: &[u8] = include_bytes!("shielded-spend-event-expression.json");

fn shielded_spend_event_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "76f84b0985527396e52f606d44e35171f82c67aff119b1e59f6a61be7653ea34" {
        return Err("unreviewed ShieldedSpend event compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

const BYTES_ZERO_PAD: &[u8] = include_bytes!("bytes-zero-pad-expression.json");

fn bytes_zero_pad_manifest() -> Result<Value, String> {
    if digest(BYTES_ZERO_PAD) != "83e0e815ae24786000f0a1d6f1dd16d6a831356700a9533d7003cbe91b7df965" {
        return Err("unreviewed Bytes zero-padding compatibility checkpoint".into());
    }
    serde_json::from_slice(BYTES_ZERO_PAD).map_err(|error| error.to_string())
}

const UINT_BYTES: &[u8] = include_bytes!("uint-bytes-expression.json");

fn uint_bytes_manifest() -> Result<Value, String> {
    if digest(UINT_BYTES) != "78ec997f27d23f45ce4d65217c92e8da11843bf29da96a3c16a45df844a14216" {
        return Err("unreviewed Uint byte conversion compatibility checkpoint".into());
    }
    serde_json::from_slice(UINT_BYTES).map_err(|error| error.to_string())
}

const FIELD_BYTES: &[u8] = include_bytes!("field-bytes-expression.json");

fn field_bytes_manifest() -> Result<Value, String> {
    if digest(FIELD_BYTES) != "747714f4a07b6f430b8f7f58a4172c836a821b9cbfe89dfe6733671a0831d6a8" {
        return Err("unreviewed Field byte conversion compatibility checkpoint".into());
    }
    serde_json::from_slice(FIELD_BYTES).map_err(|error| error.to_string())
}

const BYTES_FROM_CHUNKS: &[u8] = include_bytes!("bytes-from-chunks-expression.json");

fn bytes_from_chunks_manifest() -> Result<Value, String> {
    if digest(BYTES_FROM_CHUNKS) != "3de7907495cf58af98752fe9fca5a5818f73238537784bbd44daff6e8bb09c40" {
        return Err("unreviewed Bytes chunk compatibility checkpoint".into());
    }
    serde_json::from_slice(BYTES_FROM_CHUNKS).map_err(|error| error.to_string())
}

const ENUM_CAST: &[u8] = include_bytes!("enum-cast-expression.json");

fn enum_cast_manifest() -> Result<Value, String> {
    if digest(ENUM_CAST) != "bf12fb434e7532ecc2db9dc669442e3c02f468f80ffccee5879fefffe46f058d" {
        return Err("unreviewed enum cast compatibility checkpoint".into());
    }
    serde_json::from_slice(ENUM_CAST).map_err(|error| error.to_string())
}

const WIDE_UINT_BYTES: &[u8] = include_bytes!("wide-uint-bytes-expression.json");

fn wide_uint_bytes_manifest() -> Result<Value, String> {
    if digest(WIDE_UINT_BYTES) != "5f0bcd041ee2631eb97c9e0333fb469aac311b6d41944cc735983e1b911b8766" {
        return Err("unreviewed wide Uint byte compatibility checkpoint".into());
    }
    serde_json::from_slice(WIDE_UINT_BYTES).map_err(|error| error.to_string())
}

fn before_nested_map(path: &str, current: &str) -> Result<String, String> {
    let current = legacy_source(path, current, &wide_uint_bytes_manifest()?)?;
    let current = legacy_source(path, &current, &enum_cast_manifest()?)?;
    let current = legacy_source(path, &current, &bytes_from_chunks_manifest()?)?;
    let current = legacy_source(path, &current, &field_bytes_manifest()?)?;
    let current = legacy_source(path, &current, &uint_bytes_manifest()?)?;
    let current = legacy_source(path, &current, &bytes_zero_pad_manifest()?)?;
    let current = legacy_source(path, &current, &shielded_spend_event_manifest(SHIELDED_SPEND_EVENT)?)?;
    legacy_source(path, &current, &nested_map_manifest(NESTED_MAP)?)
}

fn before_jubjub_cast(path: &str, current: &str) -> Result<String, String> {
    let current = before_nested_map(path, current)?;
    legacy_source(path, &current, &jubjub_cast_manifest(JUBJUB_CAST)?)
}

fn before_vector_to_bytes(path: &str, current: &str) -> Result<String, String> {
    let current = before_jubjub_cast(path, current)?;
    legacy_source(path, &current, &vector_to_bytes_manifest(VECTOR_TO_BYTES)?)
}

fn before_secp256k1_cast(path: &str, current: &str) -> Result<String, String> {
    let before_vector = before_vector_to_bytes(path, current)?;
    let before_bytes = legacy_source(
        path,
        &before_vector,
        &fixed_bytes_manifest(FIXED_BYTES_ENUM)?,
    )?;
    let before_accessors = legacy_source(path, &before_bytes, &secp256k1_accessors_manifest()?)?;
    let before_unary = legacy_source(
        path,
        &before_accessors,
        &secp256k1_unary_manifest(SECP256K1_UNARY)?,
    )?;
    legacy_source(
        path,
        &before_unary,
        &secp256k1_cast_manifest(SECP256K1_CAST)?,
    )
}

const SECP256K1_ACCESSORS: &[u8] = include_bytes!("secp256k1-point-accessor-expression.json");
fn secp256k1_accessors_manifest() -> Result<Value, String> {
    if digest(SECP256K1_ACCESSORS)
        != "61f0176a053d08fd3ca08f12127998b3e4fc99113ed7d62fd59bff4a8028ece8"
    {
        return Err("unreviewed secp256k1 accessor compatibility checkpoint".into());
    }
    serde_json::from_slice(SECP256K1_ACCESSORS).map_err(|error| error.to_string())
}

const SECP256K1_UNARY: &[u8] = include_bytes!("secp256k1-unary-expression.json");
fn secp256k1_unary_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "fb7beda87e26b8eddf818f4b8a047e28307f24c52dcf910e55a534f05f9a96ad" {
        return Err("unreviewed secp256k1 unary compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// Keccak adds an encoding field to three historical traversal patterns.
const KECCAK_ENCODING: &[u8] = include_bytes!("keccak-encoding-schema20.json");
fn keccak_encoding_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "c00e3084f3be4d4a6cb1216d6f21fb4314726945ec96bd9e399de0a3d1440ca4" {
        return Err("unreviewed Keccak encoding compatibility checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}
fn before_keccak_encoding(path: &str, current: &str) -> Result<String, String> {
    legacy_source(path, current, &keccak_encoding_manifest(KECCAK_ENCODING)?)
}
fn before_fold_enum(path: &str, current: &str) -> Result<String, String> {
    let before_sha512 = before_sha512_enum(path, current)?;
    let before_keccak = before_keccak_encoding(path, &before_sha512)?;
    legacy_source(path, &before_keccak, &fold_enum_manifest(FOLD_ENUM)?)
}

// #617 extracts the two live shared definitions from the frozen Plan oracle.
// Authenticate their new owner and restore the exact signed predecessor first.
const START_FUNDING_SHA256: &str =
    "adfbe698a69f9a31e0bc36bdfc0260099cdfd6daf43d222d97a49811042ebfed";
const START_FUNDING: &[u8] = include_bytes!("legacy-start-funding-617.json");

fn start_funding_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != START_FUNDING_SHA256 {
        return Err("unreviewed StartFunding extraction checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

// #618 makes the old result-shape diagnostic classifier test-only. Authenticate
// its exact source and changed producer assertions before earlier authorities.
const GENERIC_RETIREMENT: &[u8] = include_bytes!("legacy-generic-retirement-618.json");
fn generic_retirement_manifest(bytes: &[u8]) -> Result<Value, String> {
    if digest(bytes) != "b2093ad98a8d8a3e4d087f52e05c15d9bb2780ccc329ea284cdaff63ac40fcd4" {
        return Err("unreviewed generic retirement checkpoint".into());
    }
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}

fn shared_source(manifest: &Value, path: &str, bytes: &[u8]) -> Result<(), String> {
    let expected = manifest["shared_sources"]
        .get(path)
        .and_then(Value::as_str)
        .ok_or("unreviewed shared oracle source")?;
    let current = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let checkpoint = generic_retirement_manifest(GENERIC_RETIREMENT)?;
    let current = before_nested_map(path, current)?;
    let previous = legacy_source(path, &current, &checkpoint)?;
    if digest(previous.as_bytes()) != expected {
        return Err(format!("shared oracle source drift: {path}"));
    }
    Ok(())
}

fn projected_legacy_source(
    path: &str,
    current: &str,
    retirement: &Value,
    resource_metadata: &Value,
) -> Result<String, String> {
    let commit_inputs = commit_inputs_manifest(COMMIT_INPUTS)?;
    let audited_local = audited_local_manifest(AUDITED_LOCAL)?;
    let start_funding = start_funding_manifest(START_FUNDING)?;
    let generic = generic_retirement_manifest(GENERIC_RETIREMENT)?;
    let map_size_enum = map_size_enum_manifest(MAP_SIZE_ENUM)?;
    let before_fold = before_fold_enum(path, current)?;
    let before_map_size = legacy_source(path, &before_fold, &map_size_enum)?;
    let before_generic = legacy_source(path, &before_map_size, &generic)?;
    let before_start_funding = legacy_source(path, &before_generic, &start_funding)?;
    let before_audited_local = legacy_source(path, &before_start_funding, &audited_local)?;
    let before_commit = legacy_source(path, &before_audited_local, &commit_inputs)?;
    let before_metadata = legacy_source(path, &before_commit, resource_metadata)?;
    legacy_source(path, &before_metadata, retirement)
}

fn legacy_source(path: &str, current: &str, retirement: &Value) -> Result<String, String> {
    let Some(owner) = retirement["owners"].get(path) else {
        return Ok(current.to_owned());
    };
    if digest(current.as_bytes()) != owner["current_sha256"] {
        return Err(format!("retired oracle owner drift: {path}"));
    }
    let mut previous = current.to_owned();
    for edit in owner["predecessor_reconstruction"]
        .as_array()
        .ok_or("missing retirement edits")?
    {
        let current = edit["current"].as_str().ok_or("missing current edit")?;
        let old = edit["previous"].as_str().ok_or("missing previous edit")?;
        if current.is_empty() || previous.matches(current).count() != 1 {
            return Err(format!("nonunique retirement edit: {path}"));
        }
        previous = previous.replacen(current, old, 1);
    }
    if digest(previous.as_bytes()) != owner["previous_sha256"] {
        return Err(format!("signed oracle reconstruction differs: {path}"));
    }
    Ok(previous)
}

pub(super) fn legacy_authority() -> Value {
    let mut authority: Value = serde_json::from_str(include_str!("legacy-authority.json")).unwrap();
    let retirement = retirement_manifest(include_bytes!("legacy-retirement.json")).unwrap();
    let resource_metadata = resource_metadata_manifest(RESOURCE_METADATA).unwrap();
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let start_funding = start_funding_manifest(START_FUNDING).unwrap();
    for path in start_funding["shared_sources"].as_object().unwrap().keys() {
        assert!(relative_path(path));
        shared_source(
            &start_funding,
            path,
            &std::fs::read(repository.join(path)).unwrap(),
        )
        .unwrap();
    }
    for row in authority["files"].as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        assert!(relative_path(path));
        let current = std::fs::read_to_string(repository.join(path)).unwrap();
        let previous =
            projected_legacy_source(path, &current, &retirement, &resource_metadata).unwrap();
        assert_eq!(
            digest(previous.as_bytes()),
            row["sha256"],
            "old source drift: {path}"
        );
    }
    authority["retirement_projection"] = retirement;
    authority["resource_metadata_projection"] = resource_metadata;
    authority["commit_input_test_projection"] = commit_inputs_manifest(COMMIT_INPUTS).unwrap();
    authority["audited_local_projection"] = audited_local_manifest(AUDITED_LOCAL).unwrap();
    authority["vector_to_bytes_projection"] = vector_to_bytes_manifest(VECTOR_TO_BYTES).unwrap();
    authority["fixed_bytes_projection"] = fixed_bytes_manifest(FIXED_BYTES_ENUM).unwrap();
    authority["secp256k1_cast_projection"] = secp256k1_cast_manifest(SECP256K1_CAST).unwrap();
    authority["secp256k1_accessors_projection"] = secp256k1_accessors_manifest().unwrap();
    authority["secp256k1_unary_projection"] = secp256k1_unary_manifest(SECP256K1_UNARY).unwrap();
    authority["sha512_enum_projection"] = sha512_enum_manifest(SHA512_ENUM).unwrap();
    authority["keccak_encoding_projection"] = keccak_encoding_manifest(KECCAK_ENCODING).unwrap();
    authority["fold_enum_projection"] = fold_enum_manifest(FOLD_ENUM).unwrap();
    authority["map_size_enum_projection"] = map_size_enum_manifest(MAP_SIZE_ENUM).unwrap();
    authority["start_funding_projection"] = start_funding_manifest(START_FUNDING).unwrap();
    authority
}

#[test]
fn map_size_enum_projection_preserves_exact_old_oracle_and_rejects_drift() {
    let projection = map_size_enum_manifest(MAP_SIZE_ENUM).unwrap();
    assert!(map_size_enum_manifest(br#"{"owners":{}}"#).is_err());
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        AUDITED_LOCAL_SHA256
    );
    let old = audited_local_manifest(AUDITED_LOCAL).unwrap();
    let path = "tools/compact-rust-backend/src/recorded/audited_local.rs";
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let current = std::fs::read_to_string(root.join(path)).unwrap();
    let current = before_fold_enum(path, &current).unwrap();
    let previous = legacy_source(path, &current, &projection).unwrap();
    assert_eq!(
        digest(previous.as_bytes()),
        old["owners"][path]["current_sha256"]
    );
    assert_eq!(
        current.replacen("        | Expr::MapSize { .. }\n", "", 1),
        previous,
        "only the new enum refusal is adapted; old bodies remain exact"
    );
    assert!(legacy_source(path, &(current.clone() + "\n// drift\n"), &projection).is_err());
    assert!(legacy_source(path, &previous, &projection).is_err());
    let mut forged = projection;
    forged["owners"][path]["predecessor_reconstruction"][0]["previous"] =
        Value::String("unreviewed acceptance".into());
    assert!(map_size_enum_manifest(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn retired_oracle_projection_rejects_drift_and_forged_authority() {
    let bytes = include_bytes!("legacy-retirement.json");
    let retirement = retirement_manifest(bytes).unwrap();
    let resource_metadata = resource_metadata_manifest(RESOURCE_METADATA).unwrap();
    assert!(resource_metadata_manifest(br#"{"owners":{}}"#).is_err());
    assert!(retirement_manifest(br#"{"owners":{}}"#).is_err());
    let commit_inputs = commit_inputs_manifest(COMMIT_INPUTS).unwrap();
    assert!(commit_inputs_manifest(br#"{"owners":{}}"#).is_err());
    assert!(generic_retirement_manifest(br#"{"owners":{}}"#).is_err());
    generic_retirement_manifest(GENERIC_RETIREMENT).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let start_funding = start_funding_manifest(START_FUNDING).unwrap();
    assert!(start_funding_manifest(br#"{"owners":{}}"#).is_err());
    let mut forged = start_funding.clone();
    forged["scope"] = Value::String("unreviewed".into());
    assert!(start_funding_manifest(&serde_json::to_vec(&forged).unwrap()).is_err());
    assert_eq!(start_funding["owners"].as_object().unwrap().len(), 1);
    assert_eq!(
        start_funding["shared_sources"].as_object().unwrap().len(),
        1
    );
    for path in start_funding["shared_sources"].as_object().unwrap().keys() {
        let bytes = std::fs::read(root.join(path)).unwrap();
        shared_source(&start_funding, path, &bytes).unwrap();
        let mut changed = bytes.clone();
        changed.extend_from_slice(b"\n// drift\n");
        assert!(shared_source(&start_funding, path, &changed).is_err());
        assert!(shared_source(&start_funding, "unreviewed.rs", &bytes).is_err());
    }

    for (path, row) in start_funding["owners"].as_object().unwrap() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let current = before_nested_map(path, &current).unwrap();
        let previous = legacy_source(path, &current, &start_funding).unwrap();
        assert_eq!(digest(previous.as_bytes()), row["previous_sha256"]);
        assert!(legacy_source(path, &(current.clone() + "\n// drift\n"), &start_funding).is_err());
        for edit in row["predecessor_reconstruction"].as_array().unwrap() {
            let partial = current.replacen(
                edit["current"].as_str().unwrap(),
                edit["previous"].as_str().unwrap(),
                1,
            );
            assert_ne!(partial, current);
            assert!(
                projected_legacy_source(path, &partial, &retirement, &resource_metadata).is_err()
            );
        }
    }
    let audited_local = audited_local_manifest(AUDITED_LOCAL).unwrap();
    let map_size_enum = map_size_enum_manifest(MAP_SIZE_ENUM).unwrap();
    assert!(audited_local_manifest(br#"{"owners":{}}"#).is_err());
    for (path, row) in audited_local["owners"].as_object().unwrap() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let before_fold = before_fold_enum(path, &current).unwrap();
        let before_map_size = legacy_source(path, &before_fold, &map_size_enum).unwrap();
        assert_eq!(
            digest(
                legacy_source(path, &before_map_size, &audited_local)
                    .unwrap()
                    .as_bytes()
            ),
            row["previous_sha256"]
        );
        assert!(
            projected_legacy_source(
                path,
                &(current.clone() + "\n// drift\n"),
                &retirement,
                &resource_metadata,
            )
            .is_err()
        );
        let edits = row["predecessor_reconstruction"].as_array().unwrap();
        assert_eq!(edits.len(), 2);
        for edit in edits {
            let partial = current.replacen(
                edit["current"].as_str().unwrap(),
                edit["previous"].as_str().unwrap(),
                1,
            );
            assert_ne!(partial, current);
            assert!(
                projected_legacy_source(path, &partial, &retirement, &resource_metadata).is_err()
            );
        }
    }
    for (path, row) in commit_inputs["owners"].as_object().unwrap() {
        let live = std::fs::read_to_string(root.join(path)).unwrap();
        let generic = generic_retirement_manifest(GENERIC_RETIREMENT).unwrap();
        let live = before_nested_map(path, &live).unwrap();
        let current = legacy_source(path, &live, &generic).unwrap();
        assert!(legacy_source(path, &(live + "\n// drift\n"), &generic).is_err());
        assert_eq!(
            digest(
                legacy_source(path, &current, &commit_inputs)
                    .unwrap()
                    .as_bytes()
            ),
            row["previous_sha256"]
        );
        assert!(legacy_source(path, &(current.clone() + "\n// drift\n"), &commit_inputs).is_err());
        let edit = &row["predecessor_reconstruction"][0];
        let partial = current.replacen(
            edit["current"].as_str().unwrap(),
            edit["previous"].as_str().unwrap(),
            1,
        );
        assert_ne!(partial, current);
        assert!(projected_legacy_source(path, &partial, &retirement, &resource_metadata).is_err());
    }
    for (path, row) in retirement["owners"].as_object().unwrap() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(
            digest(
                projected_legacy_source(path, &current, &retirement, &resource_metadata)
                    .unwrap()
                    .as_bytes()
            ),
            row["previous_sha256"]
        );
        assert!(
            projected_legacy_source(
                path,
                &(current.clone() + "\n// drift\n"),
                &retirement,
                &resource_metadata,
            )
            .is_err()
        );
        let edit = &row["predecessor_reconstruction"][0];
        let generic = generic_retirement_manifest(GENERIC_RETIREMENT).unwrap();
        let before_fold = before_fold_enum(path, &current).unwrap();
        let before_map_size = legacy_source(path, &before_fold, &map_size_enum).unwrap();
        let before_generic = legacy_source(path, &before_map_size, &generic).unwrap();
        let before_start = legacy_source(path, &before_generic, &start_funding).unwrap();
        let before_audited = legacy_source(path, &before_start, &audited_local).unwrap();
        let before_commit = legacy_source(path, &before_audited, &commit_inputs).unwrap();
        let before_metadata = legacy_source(path, &before_commit, &resource_metadata).unwrap();
        let partial = before_metadata.replacen(
            edit["current"].as_str().unwrap(),
            edit["previous"].as_str().unwrap(),
            1,
        );
        assert!(projected_legacy_source(path, &partial, &retirement, &resource_metadata).is_err());
    }
    // Every original source digest remains mandatory after all exact stages.
    let authority = legacy_authority();
    assert_eq!(authority["files"].as_array().unwrap().len(), 49);
    let path = "tools/compact-rust-backend/src/recorded/typed_plan/composition.rs";
    let current = std::fs::read_to_string(root.join(path)).unwrap();
    let edit = &resource_metadata["owners"][path]["predecessor_reconstruction"][0];
    let partial = current.replacen(
        edit["current"].as_str().unwrap(),
        edit["previous"].as_str().unwrap(),
        1,
    );
    assert_ne!(partial, current);
    assert!(projected_legacy_source(path, &partial, &retirement, &resource_metadata).is_err());
}

#[test]
fn corpus_identity_rejects_duplicate_and_escaped_inputs() {
    let original = Case {
        id: "one".into(),
        file: "matrix-inputs/one.json".into(),
        root: "root".into(),
        rules: vec!["LocalUnit".into()],
        sha256: "a".repeat(64),
    };
    assert!(validate_cases(std::slice::from_ref(&original)).is_ok());
    assert!(validate_cases(&[original.clone(), original.clone()]).is_err());
    for path in [
        "../one.json",
        "/one.json",
        "matrix-inputs/../../one.json",
        "",
    ] {
        let mut changed = original.clone();
        changed.file = path.into();
        assert!(validate_cases(&[changed]).is_err(), "{path}");
    }
    let mut duplicate_file = original.clone();
    duplicate_file.id = "two".into();
    assert!(validate_cases(&[original.clone(), duplicate_file]).is_err());
    let mut malformed = original;
    malformed.sha256 = "0".repeat(63);
    assert!(validate_cases(&[malformed]).is_err());
    assert!(authenticated_manifest(br#"{"scope":"forged","cases":[]}"#).is_err());
}

#[test]
fn fold_enum_projection_preserves_every_predecessor_and_rejects_partial_or_forged_drift() {
    let projection = fold_enum_manifest(FOLD_ENUM).unwrap();
    assert!(fold_enum_manifest(br#"{"owners":{}}"#).is_err());
    assert_eq!(projection["owners"].as_object().unwrap().len(), 4);
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        MAP_SIZE_ENUM_SHA256
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (path, row) in projection["owners"].as_object().unwrap() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let current = before_sha512_enum(path, &current).unwrap();
        let current = before_keccak_encoding(path, &current).unwrap();
        let previous = legacy_source(path, &current, &projection).unwrap();
        assert_eq!(digest(previous.as_bytes()), row["previous_sha256"]);
        let edits = row["predecessor_reconstruction"].as_array().unwrap();
        assert_eq!(edits.len(), 1);
        assert!(
            edits[0]["current"]
                .as_str()
                .unwrap()
                .contains("Expr::VectorFold")
        );
        assert_eq!(edits[0]["previous"], "");
        assert!(legacy_source(path, &(current.clone() + "\n// drift\n"), &projection).is_err());
        assert!(legacy_source(path, &previous, &projection).is_err());
    }
    let mut forged = projection;
    let path = "tools/compact-rust-backend/src/recorded/audited_local.rs";
    forged["owners"][path]["predecessor_reconstruction"][0]["previous"] =
        Value::String("unreviewed acceptance".into());
    assert!(fold_enum_manifest(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn keccak_encoding_projection_preserves_exact_fold_oracles() {
    let projection = keccak_encoding_manifest(KECCAK_ENCODING).unwrap();
    let fold = fold_enum_manifest(FOLD_ENUM).unwrap();
    assert!(keccak_encoding_manifest(br#"{"owners":{}}"#).is_err());
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        FOLD_ENUM_SHA256
    );
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 3);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in owners.keys() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let current = before_sha512_enum(path, &current).unwrap();
        let predecessor = before_keccak_encoding(path, &current).unwrap();
        assert_eq!(
            digest(predecessor.as_bytes()),
            fold["owners"][path]["current_sha256"]
        );
        assert_eq!(
            current.replacen(
                "Expr::Keccak256 { value, .. }",
                "Expr::Keccak256 { value }",
                1
            ),
            predecessor
        );
        assert!(before_keccak_encoding(path, &predecessor).is_err());
        assert!(before_keccak_encoding(path, &(current + "\n// drift\n")).is_err());
    }
}

#[test]
fn sha512_enum_projection_preserves_exact_predecessors() {
    let projection = sha512_enum_manifest(SHA512_ENUM).unwrap();
    let keccak = keccak_encoding_manifest(KECCAK_ENCODING).unwrap();
    let fold = fold_enum_manifest(FOLD_ENUM).unwrap();
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(KECCAK_ENCODING)
    );
    assert!(sha512_enum_manifest(br#"{"owners":{}}"#).is_err());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 4);
    for (path, owner) in owners {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let current = before_secp256k1_cast(path, &current).unwrap();
        let predecessor = legacy_source(path, &current, &projection).unwrap();
        let pinned = keccak["owners"].get(path).unwrap_or(&fold["owners"][path]);
        assert_eq!(digest(predecessor.as_bytes()), pinned["current_sha256"]);
        let edits = owner["predecessor_reconstruction"].as_array().unwrap();
        assert_eq!(edits.len(), 1);
        let edit = edits[0]["current"].as_str().unwrap();
        assert!(edit.contains("Expr::Sha512"));
        assert_eq!(edits[0]["previous"], "");
        assert_eq!(current.replacen(edit, "", 1), predecessor);
        assert!(legacy_source(path, &predecessor, &projection).is_err());
        assert!(legacy_source(path, &(current + "\n// drift\n"), &projection).is_err());
    }
}

#[test]
fn secp256k1_cast_projection_authenticates_exact_sha512_predecessors() {
    let projection = secp256k1_cast_manifest(SECP256K1_CAST).unwrap();
    let sha512 = sha512_enum_manifest(SHA512_ENUM).unwrap();
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(SHA512_ENUM)
    );
    assert!(secp256k1_cast_manifest(br#"{"owners":{}}"#).is_err());
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 4);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in owners.keys() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let predecessor = before_secp256k1_cast(path, &current).unwrap();
        assert_eq!(
            digest(predecessor.as_bytes()),
            sha512["owners"][path]["current_sha256"]
        );
        assert!(before_secp256k1_cast(path, &predecessor).is_err());
        assert!(before_secp256k1_cast(path, &(current + "\n// drift\n")).is_err());
    }
}

#[test]
fn fixed_bytes_projection_preserves_all_exact_predecessors() {
    let projection = fixed_bytes_manifest(FIXED_BYTES_ENUM).unwrap();
    let accessors = secp256k1_accessors_manifest().unwrap();
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(SECP256K1_ACCESSORS)
    );
    assert!(fixed_bytes_manifest(br#"{"owners":{}}"#).is_err());
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 4);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in owners.keys() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let current = before_vector_to_bytes(path, &current).unwrap();
        let previous = legacy_source(path, &current, &projection).unwrap();
        assert_eq!(
            digest(previous.as_bytes()),
            accessors["owners"][path]["current_sha256"]
        );
        assert!(legacy_source(path, &previous, &projection).is_err());
        assert!(legacy_source(path, &(current + "\n// unrelated drift\n"), &projection).is_err());
    }
}

#[test]
fn vector_to_bytes_projection_preserves_exact_fixed_bytes_predecessors() {
    let projection = vector_to_bytes_manifest(VECTOR_TO_BYTES).unwrap();
    let fixed_bytes = fixed_bytes_manifest(FIXED_BYTES_ENUM).unwrap();
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(FIXED_BYTES_ENUM)
    );
    assert!(vector_to_bytes_manifest(br#"{"owners":{}}"#).is_err());
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 4);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in owners.keys() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let previous = before_vector_to_bytes(path, &current).unwrap();
        assert_eq!(
            digest(previous.as_bytes()),
            fixed_bytes["owners"][path]["current_sha256"]
        );
        assert!(before_vector_to_bytes(path, &previous).is_err());
        assert!(before_vector_to_bytes(path, &(current + "\n// unrelated drift\n")).is_err());
    }
}

#[test]
fn jubjub_cast_projection_preserves_exact_vector_to_bytes_predecessors() {
    let projection = jubjub_cast_manifest(JUBJUB_CAST).unwrap();
    let vector = vector_to_bytes_manifest(VECTOR_TO_BYTES).unwrap();
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(VECTOR_TO_BYTES)
    );
    assert!(jubjub_cast_manifest(br#"{"owners":{}}"#).is_err());
    let owners = projection["owners"].as_object().unwrap();
    assert_eq!(owners.len(), 4);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in owners.keys() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let previous = before_jubjub_cast(path, &current).unwrap();
        assert_eq!(
            digest(previous.as_bytes()),
            vector["owners"][path]["current_sha256"]
        );
        let current = before_nested_map(path, &current).unwrap();
        assert!(before_jubjub_cast(path, &previous).is_err());
        assert!(before_jubjub_cast(path, &(current + "\n// unrelated drift\n")).is_err());
    }
}

#[test]
fn nested_map_projection_reconstructs_exact_historical_owners() {
    let projection = nested_map_manifest(NESTED_MAP).unwrap();
    assert!(nested_map_manifest(br#"{"owners":{}}"#).is_err());
    assert_eq!(
        projection["predecessor_manifest"]["sha256"],
        digest(JUBJUB_CAST)
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (path, owner) in projection["owners"].as_object().unwrap() {
        let current = std::fs::read_to_string(root.join(path)).unwrap();
        let previous = before_nested_map(path, &current).unwrap();
        assert_eq!(digest(previous.as_bytes()), owner["previous_sha256"]);
        assert!(before_nested_map(path, &previous).is_err());
        assert!(before_nested_map(path, &(current + "\n// unrelated drift\n")).is_err());
    }
}
