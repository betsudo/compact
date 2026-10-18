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

//! Exact Forge recipe-v1 identities; no trust in opaque response identifiers.
use super::{
    error::{Result, require},
    json,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const PROFILE: &str = "ledger8-8.1.3/zkir-2.1.1/transient-2.1.1/base-1.0.2/serialize-1.1.1/proofs-0.7.1/circuits-6.1.0/stdlib-1.1.0/curves-0.2.0/tagged-v1/forge-recipe-v1";
pub const REVISION: &str = "b5d3e965a18e4cac17fac4f38446fc55c524439a";
pub const FORMAT: &str = "midnight-zkir-v2-json";
pub const MAX_JSON: u64 = 16 * 1024 * 1024;
pub const MAX_ARTIFACT: u64 = 8 * 1024 * 1024 * 1024;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn id(domain: &str, value: &impl Serialize) -> Result<String> {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend(serde_json::to_vec(value)?);
    Ok(digest(&bytes))
}
pub fn name(source: &[u8]) -> String {
    format!("compact-{}", &digest(source)[..24])
}
pub fn bundle(name: &str, source_sha: &str) -> Result<String> {
    id("midnight-zkir-forge/bundle/v1", &(name, FORMAT, source_sha))
}
pub fn derivation(source_sha: &str, build: &str, k: u8, parameter_sha: &str) -> Result<String> {
    id(
        "midnight-zkir-forge/derive/v1",
        &(
            source_sha,
            PROFILE,
            REVISION,
            build,
            k,
            parameter_sha,
            "library-keygen/default-options/no-rng",
        ),
    )
}
#[derive(Clone, Debug, Serialize)]
pub struct Artifact {
    // Field order is part of Forge's existing recipe serialization.
    pub kind: String,
    pub digest: String,
    pub size: u64,
}
impl Artifact {
    pub fn from_record(value: &Value) -> Result<Self> {
        Self::new(
            json::text(value, "kind")?,
            json::text(value, "digest")?,
            json::number(value, "size")?,
        )
    }
    fn new(kind: &str, digest: &str, size: u64) -> Result<Self> {
        require(
            matches!(kind, "source" | "binary_ir" | "prover" | "verifier"),
            "unknown artifact kind",
        )?;
        require(
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "invalid artifact digest",
        )?;
        require(size > 0 && size <= MAX_ARTIFACT, "invalid artifact length")?;
        Ok(Self {
            kind: kind.into(),
            digest: digest.into(),
            size,
        })
    }
    pub fn extension(&self) -> &str {
        match self.kind.as_str() {
            "source" => "zkir",
            "binary_ir" => "bzkir",
            "prover" => "prover",
            "verifier" => "verifier",
            _ => unreachable!(),
        }
    }
}
pub fn artifact_set(artifacts: &[Artifact]) -> Result<String> {
    require(
        artifacts
            .iter()
            .map(|a| a.kind.as_str())
            .eq(["binary_ir", "prover", "verifier"]),
        "Forge artifact order/kinds differ from recipe-v1",
    )?;
    id("midnight-zkir-forge/artifacts/v1", &artifacts)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recipe_binds_current_bytes_parameters_and_build() {
        let baseline = derivation("a", "test-build", 6, "p").unwrap();
        assert_ne!(baseline, derivation("b", "test-build", 6, "p").unwrap());
        assert_ne!(baseline, derivation("a", "test-build", 7, "p").unwrap());
        assert_ne!(baseline, derivation("a", "test-build", 6, "q").unwrap());
        assert_ne!(baseline, derivation("a", "unknown", 6, "p").unwrap());
    }
    #[test]
    fn artifact_order_is_not_silently_normalized() {
        let a = |kind: &str| Artifact {
            kind: kind.into(),
            digest: "a".repeat(64),
            size: 1,
        };
        assert!(artifact_set(&[a("verifier"), a("binary_ir"), a("prover")]).is_err());
    }
}
