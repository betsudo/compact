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

//! Separate opt-in rendering result; legacy public struct literals are unchanged.
use crate::{RenderedContract, RustCapabilityReport, RustCircuitCapability};
use serde::Serialize;
use serde_json::Value;
use std::ops::Deref;

#[derive(Debug, Clone, Serialize)]
pub struct StandaloneProofDataCapability {
    pub api_version: u32,
    pub method: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct StandaloneProofDataUnavailable {
    pub code: &'static str,
    pub method: String,
    pub detail: String,
}

pub(crate) struct ProofDataInfo {
    pub name: String,
    pub standalone_proof_data: Option<StandaloneProofDataCapability>,
    pub standalone_proof_data_unavailable: Option<StandaloneProofDataUnavailable>,
}
#[derive(Debug, Clone, Serialize)]
pub struct StandaloneProofDataCircuit {
    #[serde(flatten)]
    pub circuit: RustCircuitCapability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standalone_proof_data: Option<StandaloneProofDataCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standalone_proof_data_unavailable: Option<StandaloneProofDataUnavailable>,
}
impl Deref for StandaloneProofDataCircuit {
    type Target = RustCircuitCapability;
    fn deref(&self) -> &Self::Target {
        &self.circuit
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct StandaloneProofDataReport {
    pub schema_version: u32,
    pub circuits: Vec<StandaloneProofDataCircuit>,
}
impl StandaloneProofDataReport {
    pub(crate) fn new(base: &RustCapabilityReport, mut extras: Vec<ProofDataInfo>) -> Self {
        let circuits = base
            .circuits
            .iter()
            .map(|circuit| {
                let index = extras
                    .iter()
                    .position(|extra| extra.name == circuit.name)
                    .expect("one opt-in record per exported circuit");
                let extra = extras.remove(index);
                StandaloneProofDataCircuit {
                    circuit: circuit.clone(),
                    standalone_proof_data: extra.standalone_proof_data,
                    standalone_proof_data_unavailable: extra.standalone_proof_data_unavailable,
                }
            })
            .collect();
        assert!(extras.is_empty());
        Self {
            schema_version: base.schema_version,
            circuits,
        }
    }
    /// Apply only the existing frontend proof classification; helper eligibility stays independent.
    pub fn apply_contract_info(&mut self, info: &Value) -> Result<(), String> {
        let mut base = RustCapabilityReport {
            schema_version: self.schema_version,
            circuits: self.circuits.iter().map(|c| c.circuit.clone()).collect(),
        };
        base.apply_contract_info(info)?;
        self.schema_version = base.schema_version;
        for (extended, base) in self.circuits.iter_mut().zip(base.circuits) {
            extended.circuit = base;
        }
        Ok(())
    }
}
pub struct StandaloneProofDataRender {
    pub rendered: RenderedContract,
    pub capabilities: StandaloneProofDataReport,
}
impl Deref for StandaloneProofDataRender {
    type Target = RenderedContract;
    fn deref(&self) -> &Self::Target {
        &self.rendered
    }
}
