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

//! Standalone, opt-in typed-IR advisory. It does not invoke the frontend or renderer.

use compact_rust_backend::ir::{Contract, SCHEMA_VERSION};
use compact_rust_backend::transient_provenance::{Acknowledgement, Limits, advise, analyze};
use std::fs::File;
use std::io::{Read, Write};
use std::process::ExitCode;

const MAX_IR_BYTES: u64 = 2 * 1024 * 1024;
const MAX_JSON_NODES: usize = 16_384;
const MAX_STRING_BYTES: usize = 512;
const MAX_TOTAL_STRING_BYTES: usize = 512 * 1024;
const MAX_ACK_BYTES: u64 = 1024 * 1024;
const MAX_ACKNOWLEDGEMENTS: usize = 128;
const USAGE: &str =
    "usage: compact-transient-provenance <typed-ir.json> [--acknowledgements <exact-routes.json>]";

fn read_bounded(path: &str, maximum: u64) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|error| format!("{path}: {error}"))?;
    let mut data = Vec::new();
    file.take(maximum + 1)
        .read_to_end(&mut data)
        .map_err(|error| format!("{path}: {error}"))?;
    if data.len() as u64 > maximum {
        return Err(format!("{path}: input exceeds {maximum} bytes"));
    }
    Ok(data)
}

#[derive(Default)]
struct JsonBudget {
    nodes: usize,
    string_bytes: usize,
}

fn check_string(value: &str, budget: &mut JsonBudget) -> Result<(), String> {
    if value.len() > MAX_STRING_BYTES {
        return Err(format!("typed IR string exceeds {MAX_STRING_BYTES} bytes"));
    }
    budget.string_bytes = budget.string_bytes.saturating_add(value.len());
    if budget.string_bytes > MAX_TOTAL_STRING_BYTES {
        return Err(format!(
            "typed IR string total exceeds {MAX_TOTAL_STRING_BYTES} bytes"
        ));
    }
    Ok(())
}

fn check_json(value: &serde_json::Value, budget: &mut JsonBudget) -> Result<(), String> {
    budget.nodes = budget.nodes.saturating_add(1);
    if budget.nodes > MAX_JSON_NODES {
        return Err(format!("typed IR exceeds {MAX_JSON_NODES} JSON nodes"));
    }
    match value {
        serde_json::Value::String(value) => check_string(value, budget)?,
        serde_json::Value::Array(values) => {
            for value in values {
                check_json(value, budget)?;
            }
        }
        serde_json::Value::Object(values) => {
            for (key, value) in values {
                check_string(key, budget)?;
                check_json(value, budget)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let Some(ir_path) = args.next() else {
        return Err(USAGE.into());
    };
    if ir_path == "--help" && args.len() == 0 {
        println!("{USAGE}");
        return Ok(());
    }
    let ir_path = ir_path
        .into_string()
        .map_err(|_| "typed IR path is not Unicode".to_string())?;
    if ir_path.starts_with('-') {
        return Err(USAGE.into());
    }
    let acknowledgement_path = match args.next() {
        None => None,
        Some(flag) if flag == "--acknowledgements" => Some(
            args.next()
                .ok_or_else(|| USAGE.to_string())?
                .into_string()
                .map_err(|_| "acknowledgement path is not Unicode".to_string())?,
        ),
        Some(_) => return Err(USAGE.into()),
    };
    if args.next().is_some() {
        return Err(USAGE.into());
    }

    let ir = read_bounded(&ir_path, MAX_IR_BYTES)?;
    let value: serde_json::Value =
        serde_json::from_slice(&ir).map_err(|error| format!("{ir_path}: {error}"))?;
    check_json(&value, &mut JsonBudget::default())
        .map_err(|error| format!("{ir_path}: {error}"))?;
    let contract: Contract =
        serde_json::from_slice(&ir).map_err(|error| format!("{ir_path}: {error}"))?;
    if contract.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "{ir_path}: unsupported typed IR schema {}; expected {SCHEMA_VERSION}",
            contract.schema_version
        ));
    }
    let acknowledgements: Vec<Acknowledgement> = match acknowledgement_path {
        Some(path) => {
            let data = read_bounded(&path, MAX_ACK_BYTES)?;
            serde_json::from_slice(&data).map_err(|error| format!("{path}: {error}"))?
        }
        None => Vec::new(),
    };
    let report = analyze(&contract, Limits::default());
    let advisory = advise(&report, &acknowledgements, MAX_ACKNOWLEDGEMENTS);
    let mut output = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &advisory)
        .map_err(|error| format!("write advisory: {error}"))?;
    writeln!(&mut output).map_err(|error| format!("write advisory: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
