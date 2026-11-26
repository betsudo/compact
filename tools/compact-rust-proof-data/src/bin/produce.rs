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
use compact_rust_proof_data::{
    codec,
    input::{Bundle, policy},
};
use std::{fs, path::Path};
#[path = "../producer/parameters.rs"]
mod parameters;
#[path = "../producer/proof.rs"]
mod proof;
#[path = "../producer/statement.rs"]
mod statement;
async fn run() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    ensure!(
        a.len() >= 4 && ["check", "prove"].contains(&a[1].as_str()),
        "produce check|prove INPUT_MANIFEST FRESH_OUT [--max-k N] [--allow-large-k]"
    );
    let mut max = 8;
    let mut allow = false;
    let mut i = 4;
    while i < a.len() {
        match a[i].as_str() {
            "--max-k" => {
                i += 1;
                max = a.get(i).context("max-k value")?.parse()?;
            }
            "--allow-large-k" => allow = true,
            _ => anyhow::bail!("unknown argument"),
        };
        i += 1;
    }
    let bundle = Bundle::load(Path::new(&a[2]))?;
    bundle.authenticate_gate()?;
    let out = codec::fresh_dir(Path::new(&a[3]))?;
    let body = async {
        let prepared = statement::prepare(&bundle)?;
        codec::write_json(&out, "model.json", &statement::report(&prepared))?;
        policy(prepared.k, max, allow)?;
        if a[1] == "prove" {
            let own = parameters::derive(&bundle, prepared.k)?;
            let result = proof::produce(&bundle, &out, prepared, own).await?;
            codec::write_json(&out, "produced.json", &result)?;
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(first) = body {
        let _ = codec::write_json(
            &out,
            "failure.json",
            &serde_json::json!({"failed":true,"error":format!("{first:#}")}),
        );
        return Err(first);
    }
    ensure!(fs::metadata(&out)?.is_dir(), "output retained");
    println!("PROOF_DATA_PRODUCER_COMPLETE");
    Ok(())
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("PRIVATE_SDK_PANIC")));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        futures_executor::block_on(run())
    }));
    if !matches!(result, Ok(Ok(()))) {
        eprintln!("PROOF_DATA_PRODUCER_FAILED_PRIVATE_OUTPUT_RETAINED");
        std::process::exit(1);
    }
}
