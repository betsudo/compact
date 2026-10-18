// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Recompute current binary IR/model and authenticate real parameter bytes.
use super::{
    error::{CacheError, Result, require},
    identity,
    policy::Policy,
    process, publication,
};
use std::{fs, path::Path, process::Command};
pub struct Model {
    pub k: u8,
    pub parameter_sha: String,
    pub binary_sha: String,
}
pub fn fresh(policy: &Policy, source: &[u8], stage: &Path) -> Result<Model> {
    let version = process::output(Command::new(&policy.zkir.path).arg("--version"))?;
    require(
        matches!(
            String::from_utf8_lossy(&version).trim(),
            "zkir 2.1.1" | "midnight-zkir 2.1.1"
        ),
        "actual ZKIR must be version2.1.1",
    )?;
    let path = stage.join("current.zkir");
    fs::write(&path, source)?;
    let output = process::model_output(
        Command::new(&policy.zkir.path)
            .arg("mock-compile")
            .arg(&path)
            .env("MIDNIGHT_PP", &policy.parameter_directory),
    )?;
    let k = parse_k(&String::from_utf8_lossy(&output))?;
    Ok(Model {
        k,
        parameter_sha: policy.parameter(k)?,
        binary_sha: publication::hash_file(&path.with_extension("bzkir"), identity::MAX_ARTIFACT)?,
    })
}
fn parse_k(text: &str) -> Result<u8> {
    let mut matches = text.match_indices("(k=");
    let (offset, _) = matches
        .next()
        .ok_or_else(|| CacheError::Refused("fresh ZKIR model has no parameter size".into()))?;
    require(matches.next().is_none(), "ambiguous fresh ZKIR model")?;
    let (number, rows) = text[offset + 3..]
        .split_once(", rows=")
        .ok_or_else(|| CacheError::Refused("invalid fresh ZKIR model".into()))?;
    require(
        rows.split_once(')')
            .is_some_and(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())),
        "invalid fresh ZKIR rows",
    )?;
    number
        .parse()
        .map_err(|_| CacheError::Refused("invalid fresh ZKIR parameter size".into()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_unambiguous() {
        assert_eq!(parse_k("(k=6, rows=20)").unwrap(), 6);
        for s in [
            "k=6",
            "(k=6, rows=x)",
            "(k=256, rows=1)",
            "(k=6, rows=1)(k=7, rows=2)",
        ] {
            assert!(parse_k(s).is_err());
        }
    }
}
