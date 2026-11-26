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

use anyhow::{Context, Result, ensure};
use compact_rust_proof_data::{codec, verifier};
use serde_json::json;
use std::{os::unix::fs::MetadataExt, path::PathBuf};
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() >= 3,
        "INPUT_MANIFEST PRODUCED_JSON FRESH_REPORT_JSON [--max-k N] [--allow-large-k]"
    );
    let mut max_k = 8;
    let mut allow = false;
    let mut seen = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--max-k" => {
                ensure!(!seen, "duplicate max-k");
                max_k = args.get(i + 1).context("max-k value")?.parse::<u8>()?;
                seen = true;
                i += 2;
            }
            "--allow-large-k" => {
                ensure!(!allow, "duplicate large-k opt-in");
                allow = true;
                i += 1;
            }
            _ => anyhow::bail!("unknown verifier option"),
        }
    }
    let report = PathBuf::from(&args[2]);
    let parent = report.parent().context("report parent")?;
    let first = std::fs::symlink_metadata(parent)?;
    ensure!(
        first.is_dir()
            && !first.file_type().is_symlink()
            && first.uid() == unsafe { libc::geteuid() }
            && first.mode() & 0o077 == 0,
        "private owned report parent"
    );
    let name = report
        .file_name()
        .context("report leaf")?
        .to_str()
        .context("report UTF8")?;
    ensure!(!report.exists(), "fresh report only");
    let mut trace = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        verifier::verify(
            &PathBuf::from(&args[0]),
            &PathBuf::from(&args[1]),
            max_k,
            allow,
            &mut trace,
        )
    }));
    let after = std::fs::symlink_metadata(parent)?;
    ensure!(
        first.dev() == after.dev()
            && first.ino() == after.ino()
            && first.mode() == after.mode()
            && first.uid() == after.uid(),
        "original report parent retained"
    );
    match result {
        Ok(Ok(value)) => {
            codec::write_json(parent, name, &value)?;
            println!("INDEPENDENT_VERIFICATION_COMPLETE");
            Ok(())
        }
        _ => {
            let value = json!({"schema":"compact-rust-proof-data-verification-failure/v1","verified":false,"calls":trace,"error":"VERIFICATION_FAILED_PRIVATE_DETAILS_SUPPRESSED"});
            if codec::write_json(parent, name, &value).is_err() {
                eprintln!("SECONDARY_FAILURE_RECEIPT_ERROR");
            }
            anyhow::bail!("verification failed")
        }
    }
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("VERIFIER_PANIC_DETAILS_SUPPRESSED")));
    if run().is_err() {
        eprintln!("INDEPENDENT_VERIFICATION_FAILED");
        std::process::exit(1);
    }
}
