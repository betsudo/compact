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

//! Operator-owned qualification, independent of all producer responses.
use super::{
    error::{Result, require},
    identity, json, publication,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub path: PathBuf,
    pub sha256: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unavailable {
    Refuse,
    Keygen,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Selection {
    #[default]
    Auto,
    Force,
}
fn tiny_max_k() -> u8 {
    8
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema: u8,
    pub forge: Executable,
    pub zkir: Executable,
    pub profile: String,
    pub ledger_revision: String,
    pub build_fingerprint: String,
    pub serialization: BTreeMap<String, String>,
    pub keygen_recipe: String,
    pub parameter_directory: PathBuf,
    pub parameters: BTreeMap<u8, String>,
    pub unavailable: Unavailable,
    #[serde(default)]
    pub selection: Selection,
    #[serde(default = "tiny_max_k")]
    pub tiny_max_k: u8,
}
impl Policy {
    #[cfg(test)]
    pub fn read(path: &std::path::Path) -> Result<Self> {
        Self::parse(&json::read(path, identity::MAX_JSON)?)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let value = json::parse(bytes)?;
        let p: Self = serde_json::from_value(value)?;
        require(
            p.schema == 1
                && p.profile == identity::PROFILE
                && p.ledger_revision == identity::REVISION,
            "policy must qualify Ledger8.1.3/ZKIR2.1.1",
        )?;
        require(
            valid_digest(&p.build_fingerprint)
                && valid_digest(&p.forge.sha256)
                && valid_digest(&p.zkir.sha256),
            "policy executable/build SHA256 required",
        )?;
        require(
            p.serialization == serialization()
                && p.keygen_recipe == "library-keygen/default-options/no-rng",
            "policy serialization/recipe unsupported",
        )?;
        require(
            p.forge.path.is_absolute()
                && p.zkir.path.is_absolute()
                && p.parameter_directory.is_absolute(),
            "policy tool/parameter paths must be absolute",
        )?;
        require(
            p.parameter_directory.is_dir() && !p.parameter_directory.is_symlink(),
            "policy parameter directory must be regular",
        )?;
        require(
            !p.parameters.is_empty() && p.parameters.values().all(|s| valid_digest(s)),
            "policy requires exact parameter digests",
        )?;
        Ok(p)
    }
    pub fn uses_forge(&self, k: u8) -> bool {
        matches!(self.selection, Selection::Force) || k > self.tiny_max_k
    }
    pub fn validate_tools(&self, forge_available: bool) -> Result<()> {
        validate_executable(&self.zkir)?;
        if forge_available {
            validate_executable(&self.forge)?;
        } else {
            require(
                !self.forge.path.try_exists()? && !self.forge.path.is_symlink(),
                "missing Forge changed during fallback",
            )?;
        }
        Ok(())
    }
    pub fn parameter(&self, k: u8) -> Result<String> {
        let expected = self.parameters.get(&k).ok_or_else(|| {
            super::error::CacheError::Refused(format!(
                "parameter k={k} has no independent qualification"
            ))
        })?;
        let path = self.parameter_directory.join(format!("bls_midnight_2p{k}"));
        require(
            publication::hash_file(&path, identity::MAX_ARTIFACT)? == *expected,
            "current parameter bytes differ from policy",
        )?;
        Ok(expected.clone())
    }
}
pub fn validate_executable(executable: &Executable) -> Result<()> {
    require(
        publication::hash_file(&executable.path, identity::MAX_ARTIFACT)? == executable.sha256,
        "actual executable digest differs from policy",
    )
}
pub fn valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn serialization() -> BTreeMap<String, String> {
    [
        ("binary_ir", "ir-source[v2]"),
        ("prover_key", "prover-key[v7](ir-source[v2])"),
        ("verifier_key", "verifier-key[v6]"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_unknown_policy_or_executable_fields() {
        assert!(
            serde_json::from_str::<Executable>(
                r#"{"path":"/bin/tool","sha256":"x","trust_me":true}"#
            )
            .is_err()
        );
        assert!(!valid_digest("SHA256"));
    }
}
