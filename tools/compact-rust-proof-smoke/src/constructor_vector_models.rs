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

//! Measure every intact constructor fixture model without keys or proofs.
use super::*;

pub(super) fn inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    observe(
        root,
        include_bytes!("../../compact-rust-backend/tests/constructor_vector_operands/sources.json"),
        14,
        42,
        "compact-constructor-vector-model-inventory/v2",
        true,
    )
}

pub(super) fn equality_inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    observe(
        root,
        include_bytes!("../../compact-rust-backend/tests/constructor_equality/sources.json"),
        8,
        24,
        "compact-constructor-equality-model-inventory/v1",
        false,
    )
}

pub(super) fn equality_boundaries_inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    observe(
        root,
        include_bytes!(
            "../../compact-rust-backend/tests/constructor_equality_boundaries/sources.json"
        ),
        8,
        24,
        "compact-constructor-equality-boundaries-model-inventory/v1",
        false,
    )
}

pub(super) fn tuple_equality_inventory(root: &Path) -> Result<(), Box<dyn Error>> {
    observe(
        root,
        include_bytes!(
            "../../compact-rust-backend/tests/constructor_equality_tuple_guards/sources.json"
        ),
        16,
        48,
        "compact-constructor-tuple-equality-model-inventory/v1",
        false,
    )
}

fn observe(
    root: &Path,
    roster: &[u8],
    source_count: usize,
    model_count: usize,
    format: &str,
    ledger9: bool,
) -> Result<(), Box<dyn Error>> {
    let declarations: serde_json::Value = serde_json::from_slice(roster)?;
    let declarations = declarations
        .as_array()
        .ok_or("invalid constructor source roster")?;
    if declarations.len() != source_count {
        return Err("complete constructor source roster differs".into());
    }
    let mut expected_roles = declarations
        .iter()
        .map(|s| {
            s["role"]
                .as_str()
                .map(std::ffi::OsString::from)
                .ok_or("invalid role")
        })
        .collect::<Result<Vec<_>, _>>()?;
    // StagedOutput retains one regular lock sidecar for each published SDK.
    // Account for those exact names without opening or changing their contents.
    let expected_locks = expected_roles
        .iter()
        .map(|role| std::ffi::OsString::from(format!(".{}.compactc.lock", role.to_string_lossy())))
        .collect::<Vec<_>>();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let kind = entry.file_type()?;
        if (expected_roles.contains(&name) && !kind.is_dir())
            || (expected_locks.contains(&name) && !kind.is_file())
        {
            return Err("constructor SDK directory or lock sidecar kind differs".into());
        }
    }
    expected_roles.extend(expected_locks);
    expected_roles.sort();
    let mut actual_roles = std::fs::read_dir(root)?
        .map(|e| e.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    actual_roles.sort();
    if expected_roles != actual_roles {
        return Err("complete constructor SDK membership differs".into());
    }
    let mut rows = Vec::new();
    for spec in declarations {
        let role = spec["role"]
            .as_str()
            .ok_or("missing constructor source role")?;
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
            return Err(
                "constructor model applicability differs from complete capabilities".into(),
            );
        }
        let directory = root.join(role).join("zkir");
        let mut actual = std::fs::read_dir(&directory)?
            .map(|e| e.map(|e| e.file_name()))
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
            return Err("complete constructor model membership differs".into());
        }
        for entry in entries {
            let name = entry.as_str().ok_or("invalid constructor export name")?;
            let path = directory.join(format!("{name}.zkir"));
            let bytes = std::fs::read(&path)?;
            let input = BufReader::new(std::io::Cursor::new(&bytes));
            let (k, v3_rows) = if ledger9 {
                let ir = midnight_zkir_v3::IrSource::load(input)?;
                let model = ir.model();
                (model.k(), Some(model.rows()))
            } else {
                let ir = IrSource::load(input)?;
                (ir.model().k(), None)
            };
            if std::fs::read(&path)? != bytes {
                return Err("constructor model bytes changed during observation".into());
            }
            let mut row = serde_json::json!({"source":role,"circuit":name,"k":k,"zkir_bytes":bytes.len(),
                "proof_required":true,"within_tiny_automatic_cap":k<=8,"selected_for_proof":false});
            if let Some(count) = v3_rows {
                row["rows"] = serde_json::json!(count);
            }
            rows.push(row);
        }
    }
    if rows.len() != model_count {
        return Err("complete constructor model count differs".into());
    }
    println!(
        "{}",
        serde_json::json!({"format":format,
        "source_roster_bytes":roster.len(),"circuits":rows,
        "key_generation":false,"proof_execution":false,"selection_made":false,
        "scope":"complete intact model k observation only; cap classification is not proof eligibility, constructor proof or integration acceptance"})
    );
    Ok(())
}
