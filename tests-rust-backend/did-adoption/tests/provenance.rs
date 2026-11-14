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

//! The retained Ledger 8 review and the current Ledger 9 captures each have
//! their own byte identities. The reviewed case matrix still governs coverage.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};
fn data(name: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("oracle")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn hash_rows(rows: &Value, reviewed_ledger9_sources: bool) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for row in rows.as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        assert!(!Path::new(path).is_absolute());
        assert!(!path.split('/').any(|p| p == ".."));
        // The reviewed Runtime 0.17.0 capture also pins this test's original
        // source. Keep that historical byte identity when the live test adds
        // checks for later Ledger 9 runtime changes.
        let frozen = if reviewed_ledger9_sources {
            match path {
                "runtime/src/built-ins.ts" => Some((
                    "tools/compact-rust-backend/reviewed-ledger9-reducer-source-05912594/built-ins.ts",
                    "89f0f6a65c7e62f85bbd88044e872d9d3555f6ebeb45dba460e1601c025f2a4b",
                )),
                "runtime/src/compact-types.ts" => Some((
                    "tools/compact-rust-backend/reviewed-ledger9-reducer-source-05912594/compact-types.ts",
                    "07328c4c793ef7c81171d9835c8535543959bfef1c1f67c730ed8fe65e7576f4",
                )),
                "tests-rust-backend/did-adoption/tests/provenance.rs" => Some((
                    "tests-rust-backend/did-adoption/oracle/ledger9/reviewed-provenance-05912594.rs",
                    "b864ec12133eff1391a010296ab7d7c122b9a66e9f0be2fc746b8f7ed6000884",
                )),
                _ => None,
            }
        } else {
            None
        };
        let source = if let Some((reviewed_path, expected_sha)) = frozen {
            assert_eq!(
                row["sha256"], expected_sha,
                "reviewed source identity: {path}"
            );
            root.join(reviewed_path)
        } else {
            root.join(path)
        };
        assert_eq!(
            hex::encode(Sha256::digest(fs::read(source).unwrap())),
            row["sha256"],
            "stale evidence: {path}"
        );
    }
}
#[test]
fn retained_ledger8_evidence_keeps_its_original_identity() {
    let binding = data("ledger9/binding.json");
    assert_eq!(
        binding["ledger8_head"],
        "09c75e2891283fed65f2db385925a5a4b9409af6"
    );
    hash_rows(&binding["ledger8_artifacts"], false);
}

#[test]
fn ledger9_capture_source_and_test_identities() {
    let binding = data("ledger9/binding.json");
    assert_eq!(binding["format"], "compact-did-ledger9-capture-binding/v1");
    assert_eq!(binding["compact_runtime"], "0.17.0");
    assert_eq!(binding["onchain_runtime"], "4.0.1");
    hash_rows(&binding["ledger9_artifacts"], true);
    for file in [
        "lifecycle.json",
        "pure.json",
        "point-lifecycle.json",
        "alias-lifecycle.json",
        "service-lifecycle.json",
        "schnorr-method-lifecycle.json",
        "jwk-method-lifecycle.json",
    ] {
        let capture = data(&format!("ledger9/{file}"));
        hash_rows(&capture["provenance"], true);
        assert_eq!(
            capture["generatedJavaScriptSha256"],
            binding["did_javascript_sha256"]
        );
        assert_eq!(
            capture["runtimeJavaScriptSha256"],
            binding["runtime_builtins_sha256"]
        );
    }
    let composition: Value = serde_json::from_str(include_str!(
        "../../../runtime-rs/tests/fixtures/ledger9-unit-composition.json"
    ))
    .unwrap();
    hash_rows(&composition["provenance"], true);
    assert_eq!(
        composition["sources"],
        serde_json::json!([
            {"sha256": binding["crypto_javascript_sha256"]},
            {"sha256": binding["did_javascript_sha256"]},
        ])
    );
    assert_eq!(
        composition["runtimeSha256"],
        binding["runtime_builtins_sha256"]
    );
}
#[test]
fn exact_reviewed_native_cohort_and_outcomes() {
    let capture = data("ledger9/lifecycle.json");
    let matrix = data("reviewed-matrix.json");
    let scenarios = capture["scenarios"].as_array().unwrap();
    assert_eq!(scenarios.len(), 11);
    assert_eq!(
        scenarios
            .iter()
            .filter(|s| s.get("constructorError").is_some())
            .count(),
        4
    );
    let rows: Vec<_> = scenarios
        .iter()
        .flat_map(|s| {
            s["steps"].as_array().unwrap().iter().map(move |r| {
                (
                    format!(
                        "{}/{}",
                        s["id"].as_str().unwrap(),
                        r["id"].as_str().unwrap()
                    ),
                    r,
                )
            })
        })
        .collect();
    assert_eq!(rows.len(), 122);
    let ids: BTreeSet<_> = rows.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids.len(), rows.len());
    assert_eq!(
        matrix["lifecycle_cases"].as_array().unwrap().len(),
        rows.len()
    );
    for (id, row) in &rows {
        let reviewed = matrix["lifecycle_cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["case"] == *id)
            .unwrap();
        assert_eq!(reviewed["export"], row["name"]);
        assert_eq!(reviewed["exact_error"], row["error"]);
        assert_eq!(
            reviewed["outcome"],
            if row.get("error").is_some() {
                "error"
            } else {
                "success"
            }
        );
    }
    let exports: BTreeSet<_> = rows
        .iter()
        .map(|(_, r)| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        exports,
        BTreeSet::from([
            "rotateControllerKey",
            "recoverControllerKey",
            "setAlsoKnownAs",
            "setVerificationMethod",
            "removeVerificationMethod",
            "setSchnorrJubjubVerificationMethod",
            "removeSchnorrJubjubVerificationMethod",
            "verifySchnorrJubjubDigestSignature",
            "setVerificationMethodRelation",
            "setService",
            "removeService",
            "deactivate"
        ])
    );
    for name in exports {
        assert!(
            rows.iter()
                .any(|(_, r)| r["name"] == name && r.get("error").is_none()),
            "missing native success {name}"
        );
    }
}
#[test]
fn exact_twelve_direct_pure_exports_and_unique_cases() {
    let capture = data("ledger9/pure.json");
    let matrix = data("reviewed-matrix.json");
    let rows = capture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 102);
    let exports: BTreeSet<_> = rows.iter().map(|r| r["name"].as_str().unwrap()).collect();
    assert_eq!(exports.len(), 12);
    let ids: BTreeSet<_> = rows
        .iter()
        .map(|r| (r["name"].as_str().unwrap(), r["id"].as_str().unwrap()))
        .collect();
    assert_eq!(ids.len(), 102);
    assert_eq!(matrix["pure_cases"].as_array().unwrap().len(), 102);
    for r in rows {
        assert!(
            matrix["pure_cases"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["export"] == r["name"] && m["case"] == r["id"])
        );
    }
}
