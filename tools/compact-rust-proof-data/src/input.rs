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

use crate::codec::{self, FileRef};
use anyhow::{Context, Result, ensure};
use midnight_transient_crypto::curve::Fr;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub const LEDGER: &str = "6fbc2e3900845a858f70eebf8a0deab1d4d3c0d6";
pub const ZK: &str = "bca8db4d9e2c1ecaa5d1db5ba7a201935b6c6c8a";
pub const ZKIR: &str = "3377ad9e48741f60e0aa17010b6f23e34223d906";
pub const PARENT_SHA: &str = "fc253016885ec830e97808c9ec920bb5cab5c21af590380a6cb5eb0538e2b244";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    MapMember,
    MapLookup,
    MapIsEmpty,
    SetIsEmpty,
    SetMember,
    SetSize,
    UnitUint64,
    TinyClear,
    TinySet,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Sdk {
    pub ledger: String,
    pub zk: String,
    pub zkir: String,
    pub gate: FileRef,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generation {
    pub compiler_version: String,
    pub source_revision: String,
    pub files: Vec<FileRef>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub supplemental: bool,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub preimage: FileRef,
    pub public_stream: FileRef,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub schema: String,
    pub profile: Profile,
    pub sdk: Sdk,
    pub raw_ir: FileRef,
    pub parent: Option<FileRef>,
    pub binding_le: String,
    pub randomness_le: String,
    pub generation: Generation,
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    pub schema: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}
pub struct Bundle {
    pub input: Input,
    pub root: PathBuf,
    pub manifest_sha: String,
}
pub fn scalar(text: &str) -> Result<Fr> {
    ensure!(
        text.len() == 64
            && text
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "canonical lowercase LE32 scalar"
    );
    let raw = (0..32)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let f = Fr::from_le_bytes(&raw).context("noncanonical SDK field")?;
    ensure!(
        f.as_le_bytes() == raw.as_slice(),
        "canonical field roundtrip"
    );
    Ok(f)
}
pub fn hex(f: &Fr) -> String {
    f.as_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
pub fn policy(k: u8, max_k: u8, allow_large: bool) -> Result<()> {
    ensure!(
        (1..=13).contains(&max_k) && k > 0 && k <= max_k,
        "measured k exceeds explicit budget"
    );
    ensure!(
        max_k <= 8 || allow_large,
        "max-k above8 requires explicit allow-large-k"
    );
    ensure!(
        k <= 8 || allow_large,
        "large measured circuit requires explicit opt-in"
    );
    Ok(())
}
/// Pure role preflight: no file stat/open, including private/material roles.
/// Roots are canonicalized by their manifest readers; leaf components are strict.
pub fn disjoint_roles(rows: &[(&Path, &FileRef)]) -> Result<()> {
    let mut paths = BTreeSet::new();
    for (root, row) in rows {
        let rel = Path::new(&row.path);
        ensure!(
            !row.path.is_empty()
                && !rel.is_absolute()
                && rel
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "contained role path"
        );
        ensure!(
            row.bytes <= 64 * 1024 * 1024
                && row.sha256.len() == 64
                && row
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "canonical bounded role descriptor"
        );
        ensure!(
            paths.insert(root.join(rel)),
            "public/private/material role alias"
        );
    }
    Ok(())
}
impl Bundle {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = codec::read_bytes(path, 1024 * 1024)?;
        let input: Input = serde_json::from_slice(&raw)?;
        let root = path.parent().context("manifest parent")?.canonicalize()?;
        ensure!(
            input.schema == "compact-rust-proof-data-input/v1"
                && input.sdk.ledger == LEDGER
                && input.sdk.zk == ZK
                && input.sdk.zkir == ZKIR,
            "current closed input/SDK identity"
        );
        let binding = scalar(&input.binding_le)?;
        let random = scalar(&input.randomness_le)?;
        ensure!(
            binding != Fr::from(0u64) && random != Fr::from(0u64) && binding != random,
            "explicit nonzero distinct caller fields"
        );
        ensure!(
            !input.cases.is_empty() && input.cases.len() <= 8,
            "finite case roster"
        );
        let mut ids = BTreeSet::new();
        for case in &input.cases {
            ensure!(
                !case.id.is_empty()
                    && case.id.len() <= 64
                    && case
                        .id
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
                    && ids.insert(&case.id),
                "unique safe case id"
            );
            for f in case.input.iter().chain(&case.output) {
                scalar(f)?;
            }
            match input.profile {
                Profile::MapMember | Profile::SetMember => {
                    ensure!(
                        case.input.len() == 1 && case.output.len() == 1,
                        "Field Boolean signature"
                    );
                    let b = scalar(&case.output[0])?;
                    ensure!(b == Fr::from(0u64) || b == Fr::from(1u64), "Boolean output");
                }
                Profile::MapLookup => ensure!(
                    case.input.len() == 1 && case.output.len() == 1,
                    "Field Field lookup signature"
                ),
                Profile::MapIsEmpty | Profile::SetIsEmpty => {
                    ensure!(
                        case.input.is_empty() && case.output.len() == 1,
                        "Unit Boolean is_empty signature"
                    );
                    let bit = scalar(&case.output[0])?;
                    ensure!(
                        bit == Fr::from(0u64) || bit == Fr::from(1u64),
                        "Boolean output"
                    );
                }
                Profile::UnitUint64 | Profile::SetSize => {
                    ensure!(
                        case.input.is_empty() && case.output.len() == 1,
                        "Unit Uint64 size signature"
                    );
                    let value = scalar(&case.output[0])?;
                    let bytes = value.as_le_bytes();
                    ensure!(
                        bytes.len() == 32 && bytes[8..].iter().all(|b| *b == 0),
                        "Uint64 output range"
                    );
                }
                Profile::TinyClear => ensure!(
                    case.input.is_empty() && case.output.is_empty(),
                    "Unit clear IO"
                ),
                Profile::TinySet => ensure!(
                    case.input.len() == 1 && case.output.is_empty(),
                    "Field Unit set IO"
                ),
            }
        }
        let this = Self {
            input,
            root,
            manifest_sha: codec::hash(&raw),
        };
        this.input_roles()?;
        // Generation payloads are bound by the capture/build runner receipt. The
        // verifier does not traverse arbitrary generation files or private bodies.
        Ok(this)
    }
    fn roles(&self) -> Vec<(&Path, &FileRef)> {
        let mut rows = vec![
            (self.root.as_path(), &self.input.sdk.gate),
            (self.root.as_path(), &self.input.raw_ir),
        ];
        if let Some(parent) = &self.input.parent {
            rows.push((self.root.as_path(), parent));
        }
        for c in &self.input.cases {
            rows.extend([
                (self.root.as_path(), &c.preimage),
                (self.root.as_path(), &c.public_stream),
            ]);
        }
        rows.extend(
            self.input
                .generation
                .files
                .iter()
                .map(|r| (self.root.as_path(), r)),
        );
        rows
    }
    fn input_roles(&self) -> Result<()> {
        disjoint_roles(&self.roles())
    }
    pub fn produced_roles(&self, root: &Path, p: &Produced) -> Result<()> {
        let mut rows = self.roles();
        for r in [
            &p.tagged_ir,
            &p.own_params,
            &p.prover_key,
            &p.verifier_key,
            &p.control.raw_ir,
            &p.control.tagged_ir,
            &p.control.prover_key,
            &p.control.verifier_key,
        ] {
            rows.push((root, r));
        }
        for c in &p.cases {
            rows.extend([(root, &c.proof), (root, &c.public_inputs)]);
        }
        disjoint_roles(&rows)
    }
    pub fn authenticate_gate(&self) -> Result<()> {
        let gate: serde_json::Value =
            serde_json::from_slice(&self.read(&self.input.sdk.gate, 8 * 1024 * 1024)?)?;
        ensure!(
            gate["schema"] == "compact-rust-proof-data-sdk-gate/v1"
                && gate["passed"] == true
                && gate["kind"] == "sdk"
                && gate["ledger"] == LEDGER
                && gate["zk"] == ZK
                && gate["zkir"] == ZKIR,
            "actual current SDK-only gate required"
        );
        Ok(())
    }
    pub fn read(&self, row: &FileRef, cap: usize) -> Result<Vec<u8>> {
        codec::read_ref(&self.root, row, cap)
    }
    pub fn stream(&self, case: &Case) -> Result<Stream> {
        let s: Stream = serde_json::from_slice(&self.read(&case.public_stream, 1024 * 1024)?)?;
        ensure!(
            s.schema == "compact-rust-proof-data-public/v1",
            "public stream schema"
        );
        Ok(s)
    }
    pub fn caller(&self) -> Result<(Fr, Fr)> {
        Ok((
            scalar(&self.input.binding_le)?,
            scalar(&self.input.randomness_le)?,
        ))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedCase {
    pub id: String,
    pub supplemental: bool,
    pub proof: FileRef,
    pub public_inputs: FileRef,
    pub fields: usize,
    pub skips: Vec<Option<usize>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub raw_ir: FileRef,
    pub tagged_ir: FileRef,
    pub prover_key: FileRef,
    pub verifier_key: FileRef,
    pub k: u8,
    pub rows: usize,
    pub initialized_distinct: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Produced {
    pub schema: String,
    pub input_sha256: String,
    pub profile: Profile,
    pub sdk: Sdk,
    pub k: u8,
    pub rows: usize,
    pub tagged_ir: FileRef,
    pub own_params: FileRef,
    pub parent_sha256: String,
    pub prover_key: FileRef,
    pub verifier_key: FileRef,
    pub control: Control,
    pub cases: Vec<ProducedCase>,
    pub independent_verification_performed: bool,
}
