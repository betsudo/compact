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

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

fn bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn verify(manifest: &str, active_capture: &Value) {
    let package = Path::new(manifest);
    let oracle = package.join("oracle");
    let root = package.join("../..");
    let binding: Value =
        serde_json::from_slice(&bytes(&oracle.join("ledger9-binding.json"))).unwrap();
    assert_eq!(
        binding["format"],
        "compact-ledger9-reducer-capture-binding/v1"
    );
    assert_eq!(binding["compact_runtime"], "0.17.0");
    assert_eq!(binding["onchain_runtime"], "4.0.1");
    let old_bytes = bytes(&oracle.join("cases.json"));
    assert_eq!(digest(&old_bytes), binding["ledger8_capture_sha256"]);
    let historical_binding = bytes(&oracle.join("reviewed-rust-binding.json"));
    assert_eq!(
        digest(&historical_binding),
        binding["ledger8_binding_sha256"]
    );
    let historical_binding: Value = serde_json::from_slice(&historical_binding).unwrap();
    assert_eq!(
        historical_binding["format"],
        "compact-reviewed-rust-binding/v1"
    );
    let historical_captures = historical_binding["captures"].as_array().unwrap();
    let bound_capture = historical_captures
        .iter()
        .filter(|row| row["file"] == "cases.json")
        .collect::<Vec<_>>();
    assert_eq!(bound_capture.len(), 1);
    assert_eq!(
        bound_capture[0]["sha256"],
        binding["ledger8_capture_sha256"]
    );
    let historical: Value = serde_json::from_slice(&old_bytes).unwrap();
    let current_bytes = bytes(&oracle.join("ledger9-cases.json"));
    assert_eq!(digest(&current_bytes), binding["ledger9_capture_sha256"]);
    let current: Value = serde_json::from_slice(&current_bytes).unwrap();
    assert_eq!(&current, active_capture);
    assert_eq!(current["format"], historical["format"]);
    if let Some(version) = current.get("runtimeVersion") {
        assert_eq!(version, &binding["compact_runtime"]);
    }
    assert_eq!(
        current["runtimeJavaScriptSha256"],
        binding["runtime_builtins_sha256"]
    );
    assert_eq!(
        current["generatedJavaScriptSha256"],
        binding["generated_javascript_sha256"]
    );
    let old_provenance = historical["provenance"].as_array().unwrap();
    let provenance = current["provenance"].as_array().unwrap();
    assert_eq!(old_provenance.len(), provenance.len());
    let mut paths = BTreeSet::new();
    for (old, row) in old_provenance.iter().zip(provenance) {
        let relative = row["path"].as_str().unwrap();
        assert_eq!(old["path"], relative);
        assert!(paths.insert(relative));
        let path = Path::new(relative);
        assert!(
            path.components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
        );
        assert_eq!(
            digest(&bytes(&root.join(path))),
            row["sha256"],
            "{relative}"
        );
    }
    let old_cases = historical["cases"].as_array().unwrap();
    let cases = current["cases"].as_array().unwrap();
    assert_eq!(old_cases.len(), cases.len());
    for (old, new) in old_cases.iter().zip(cases) {
        let mut old = old.as_object().unwrap().clone();
        let mut new = new.as_object().unwrap().clone();
        for field in ["before", "after"] {
            assert_eq!(old.remove(field).is_some(), new.remove(field).is_some());
        }
        assert_eq!(old, new, "reviewed reducer case semantics changed");
    }
}
