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

use anyhow::{Result, ensure};
use compact_rust_proof_data::{
    codec,
    input::{Case, Stream, hex, scalar},
};
use generated::runtime;
use midnight_transient_crypto::{
    curve::Fr, fab::ValueReprAlignedValue, proofs::ProofPreimage, repr::FieldRepr,
};
use std::path::{Path, PathBuf};
pub fn args() -> Result<(String, PathBuf, Fr, Fr)> {
    let a: Vec<_> = std::env::args().collect();
    ensure!(
        a.len() == 5,
        "capture OP FRESH_OUT BINDING_LE RANDOMNESS_LE"
    );
    let b = scalar(&a[3])?;
    let r = scalar(&a[4])?;
    ensure!(
        b != Fr::from(0u64) && r != Fr::from(0u64) && b != r,
        "explicit caller"
    );
    Ok((a[1].clone(), codec::fresh_dir(Path::new(&a[2]))?, b, r))
}
pub fn emit<P, O: Clone + Into<runtime::fab::AlignedValue>>(
    out: &Path,
    id: &str,
    supplemental: bool,
    data: &runtime::proof_data::RecordedProofData<P, O>,
    binding: Fr,
    random: Fr,
) -> Result<Case> {
    let pre = data.to_preimage(binding, random);
    let bytes = data.serialize_preimage(binding, random)?;
    let decoded: ProofPreimage = codec::decode(&bytes)?;
    ensure!(
        codec::encode(&pre)? == codec::encode(&decoded)? && decoded.binding_input == binding,
        "actual canonical recorded preimage"
    );
    let preimage = codec::write_fresh(out, &format!("{id}.preimage"), &bytes, 4 * 1024 * 1024)?;
    let public = Stream {
        schema: "compact-rust-proof-data-public/v1".into(),
        inputs: pre.public_transcript_inputs.iter().map(hex).collect(),
        outputs: pre.public_transcript_outputs.iter().map(hex).collect(),
    };
    let public_stream = codec::write_json(out, &format!("{id}.public.json"), &public)?;
    Ok(Case {
        id: id.into(),
        supplemental,
        input: pre.inputs.iter().map(hex).collect(),
        output: ValueReprAlignedValue(data.output().clone())
            .field_vec()
            .iter()
            .map(hex)
            .collect(),
        preimage,
        public_stream,
    })
}
pub fn state(state: &runtime::ledger::StateValue<runtime::ledger::DefaultDB>) -> Result<Vec<u8>> {
    codec::encode(state)
}
pub fn gas(g: &runtime::context::RunningCost) -> serde_json::Value {
    serde_json::json!({"read":g.read_time.into_picoseconds().to_string(),"compute":g.compute_time.into_picoseconds().to_string(),"written":g.bytes_written.to_string(),"deleted":g.bytes_deleted.to_string()})
}
pub fn finish(out: &Path, cases: &[Case]) -> Result<()> {
    codec::write_json(out, "cases.json", &cases)?;
    println!("GENERATED_CAPTURE_COMPLETE");
    Ok(())
}
