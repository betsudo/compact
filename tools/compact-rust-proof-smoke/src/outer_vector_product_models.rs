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

//! Observe every intact outer-Vector export before selecting any proof cohort.
use super::*;

pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    let declarations: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../compact-rust-backend/tests/outer_vector_product_semantics/sources.json"
    ))?;
    let mut rows = Vec::new();
    for spec in declarations.as_array().ok_or("invalid source roster")? {
        let role = spec["role"].as_str().ok_or("missing source role")?;
        let entries = spec["entries"]
            .as_array()
            .ok_or("missing complete exports")?;
        let capabilities: serde_json::Value = serde_json::from_slice(&std::fs::read(
            root.join(role).join("contract/rust-capabilities.json"),
        )?)?;
        let caps = capabilities["circuits"]
            .as_array()
            .ok_or("missing capabilities")?;
        if capabilities["schema_version"] != 3
            || caps.len() != entries.len()
            || entries.iter().any(|entry| {
                caps.iter()
                    .filter(|c| {
                        c["name"] == *entry
                            && c["proof_required"] == true
                            && c["recorded"] == true
                            && c["observed_call"] == true
                    })
                    .count()
                    != 1
            })
        {
            return Err("model applicability differs from emitted capabilities".into());
        }
        let mut actual = std::fs::read_dir(root.join(role).join("zkir"))?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        actual.sort();
        let mut expected = entries
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(|name| std::ffi::OsString::from(format!("{name}.zkir")))
                    .ok_or("invalid export name")
            })
            .collect::<Result<Vec<_>, _>>()?;
        expected.sort();
        if actual != expected {
            return Err("complete model membership differs".into());
        }
        for entry in entries {
            let name = entry.as_str().ok_or("invalid export name")?;
            let ir = IrSource::load(BufReader::new(File::open(
                root.join(role).join(format!("zkir/{name}.zkir")),
            )?))?;
            rows.push(
                serde_json::json!({"source":role,"circuit":name,"k":ir.model().k(),
                "proof_required":true,"selected_for_proof":false}),
            );
        }
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-outer-vector-product-model-inventory/v1",
        "circuits":rows,"proof_execution":false,"key_generation":false,"selection_made":false,
        "scope":"complete intact model observation; no proof or integration acceptance"})
    );
    Ok(())
}
