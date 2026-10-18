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

//! Opt-in artifact acquisition data, never a proof or test verdict.
use super::{
    error::{Result, require},
    identity::{Artifact, MAX_JSON},
};
use serde::Serialize;
use std::io::{self, Write};
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Acquisition {
    ForgeHit,
    ForgeMiss,
    AutoBypass,
    UnavailableKeygen,
}
#[derive(Serialize)]
pub struct Circuit {
    pub name: String,
    pub source_sha256: String,
    pub k: u8,
    pub rows: u64,
    pub parameter_sha256: String,
    pub acquisition_path: Acquisition,
    pub artifacts: Vec<Artifact>,
}
#[derive(Serialize)]
pub struct Report {
    schema: u8,
    pub circuits: Vec<Circuit>,
}
impl Report {
    pub fn new() -> Self {
        Self {
            schema: 1,
            circuits: Vec::new(),
        }
    }
}
struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) as u64 > MAX_JSON {
            return Err(io::Error::other(
                "artifact receipt exceeds finite JSON bound",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode(enabled: bool, report: &Report) -> Result<Option<Vec<u8>>> {
    if !enabled {
        return Ok(None);
    }
    let mut output = Bounded(Vec::new());
    serde_json::to_writer(&mut output, report)?;
    require(
        (output.0.len() as u64) < MAX_JSON,
        "artifact receipt has no room for final newline",
    )?;
    output.0.push(b'\n');
    Ok(Some(output.0))
}
pub fn emit(enabled: bool, report: &Report) -> Result<()> {
    if let Some(bytes) = encode(enabled, report)? {
        io::stdout().lock().write_all(&bytes)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_has_exact_schema_and_no_verdict_or_default_output() {
        let mut report = Report::new();
        report.circuits.push(Circuit {
            name: "fresh".into(),
            source_sha256: "a".repeat(64),
            k: 8,
            rows: 19,
            parameter_sha256: "b".repeat(64),
            acquisition_path: Acquisition::AutoBypass,
            artifacts: vec![Artifact {
                kind: "binary_ir".into(),
                digest: "c".repeat(64),
                size: 3,
            }],
        });
        assert!(encode(false, &report).unwrap().is_none());
        let value: serde_json::Value =
            serde_json::from_slice(&encode(true, &report).unwrap().unwrap()).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"schema":1,"circuits":[{"name":"fresh","source_sha256":"a".repeat(64),"k":8,"rows":19,"parameter_sha256":"b".repeat(64),"acquisition_path":"auto-bypass","artifacts":[{"kind":"binary_ir","digest":"c".repeat(64),"size":3}]}]})
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &encode(true, &Report::new()).unwrap().unwrap()
            )
            .unwrap(),
            serde_json::json!({"schema":1,"circuits":[]})
        );
    }
    #[test]
    fn acquisition_vocabulary_is_exact() {
        assert_eq!(
            serde_json::to_value([
                Acquisition::ForgeHit,
                Acquisition::ForgeMiss,
                Acquisition::AutoBypass,
                Acquisition::UnavailableKeygen
            ])
            .unwrap(),
            serde_json::json!([
                "forge-hit",
                "forge-miss",
                "auto-bypass",
                "unavailable-keygen"
            ])
        );
    }
    #[test]
    fn receipt_writer_refuses_over_bound_without_unbounded_buffer() {
        let mut output = Bounded(vec![0; MAX_JSON as usize]);
        assert!(output.write_all(b"x").is_err());
        assert_eq!(output.0.len(), MAX_JSON as usize);
    }
}
