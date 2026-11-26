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

use super::{
    parameters::{Own, raw},
    statement::Prepared,
};
use anyhow::{Context, Result, ensure};
use compact_rust_proof_data::{
    codec,
    input::{Bundle, Control, Produced, ProducedCase},
};
use midnight_proofs::utils::SerdeFormat;
use midnight_transient_crypto::{
    curve::Fr,
    proofs::{Proof, ProverKey, VerifierKey, Zkir},
};
use midnight_zk_stdlib::MidnightVK;
use midnight_zkir::IrSource;
use rand::rngs::OsRng;
use std::{io::Cursor, path::Path};
fn vk(raw: &[u8], k: u8) -> Result<VerifierKey> {
    let key: VerifierKey = codec::decode(raw)?;
    key.init()?;
    let b = key.original_bytes();
    let mut c = Cursor::new(b.as_slice());
    let native = MidnightVK::read(&mut c, SerdeFormat::Processed)?;
    let mut again = Vec::new();
    native.write(&mut again, SerdeFormat::Processed)?;
    ensure!(
        c.position() == b.len() as u64
            && native.k() == u32::from(k)
            && again == b
            && codec::encode(&key)? == raw,
        "canonical initialized native/outer own-k VK"
    );
    Ok(key)
}
pub async fn produce(bundle: &Bundle, out: &Path, p: Prepared, own: Own) -> Result<Produced> {
    let (pk, key) = p.ir.keygen(&own).await?;
    ensure!(pk.init()?.k() == u32::from(p.k), "fresh PK own-k init");
    let pk_bytes = codec::encode(&pk)?;
    let key_bytes = codec::encode(&key)?;
    let pk: ProverKey<IrSource> = codec::decode(&pk_bytes)?;
    ensure!(pk.init()?.k() == u32::from(p.k), "canonical PK init");
    let original = vk(&key_bytes, p.k)?;
    let tagged_ir = codec::write_fresh(out, "original.tagged-ir", &p.tagged, 4 * 1024 * 1024)?;
    let own_params =
        codec::write_fresh(out, "params-own.raw", &raw(&own.params)?, 4 * 1024 * 1024)?;
    let prover_key = codec::write_fresh(out, "original.pk", &pk_bytes, 64 * 1024 * 1024)?;
    let verifier_key = codec::write_fresh(out, "original.vk", &key_bytes, 4 * 1024 * 1024)?;
    let mut cases = Vec::new();
    for c in &p.cases {
        let (proof, fields, skips) = p.ir.prove(OsRng, &own, pk.clone(), &c.pre).await?;
        ensure!(
            !proof.0.is_empty() && fields == c.fields && skips == c.skips,
            "whole checked SDK PI/skips equal independent projection"
        );
        let b = codec::encode(&proof)?;
        let _: Proof = codec::decode(&b)?;
        let pi = codec::encode(&fields)?;
        let _: Vec<Fr> = codec::decode(&pi)?;
        cases.push(ProducedCase {
            id: c.id.clone(),
            supplemental: c.supplemental,
            proof: codec::write_fresh(out, &format!("{}.proof", c.id), &b, 4 * 1024 * 1024)?,
            public_inputs: codec::write_fresh(out, &format!("{}.pis", c.id), &pi, 4 * 1024 * 1024)?,
            fields: fields.len(),
            skips,
        });
    }
    // ONE predetermined supplemental constraint mutation. No proof/verify/search.
    // Its actual same-k and canonical distinctness are gates, never predictions.
    let mut candidate: serde_json::Value = serde_json::from_slice(&p.raw)?;
    let ops = candidate["instructions"]
        .as_array_mut()
        .context("candidate roster")?;
    let first = ops
        .iter_mut()
        .find(|o| o["op"] == "impact")
        .context("first Impact")?;
    ensure!(
        first["guard"] == "0x01" && first["inputs"][0] == "0x30",
        "fixed candidate only"
    );
    first["inputs"][0] = serde_json::json!("0x31");
    let bytes = serde_json::to_vec(&candidate)?;
    let ir = IrSource::load(bytes.as_slice())?;
    let model = ir.model();
    ensure!(
        Zkir::k(&ir) == p.k && model.k() == p.k,
        "candidate NOT_ELIGIBLE: measured different k"
    );
    let (cpk, cvk) = ir.keygen(&own).await?;
    ensure!(cpk.init()?.k() == u32::from(p.k), "fresh control PK init");
    let a = codec::encode(&cpk)?;
    let reload: ProverKey<IrSource> = codec::decode(&a)?;
    ensure!(
        reload.init()?.k() == u32::from(p.k),
        "canonical control PK init"
    );
    let b = codec::encode(&cvk)?;
    let control = vk(&b, p.k)?;
    ensure!(
        !original.original_bytes().is_empty()
            && !control.original_bytes().is_empty()
            && key_bytes != b
            && original.original_bytes() != control.original_bytes(),
        "candidate NOT_ELIGIBLE: initialized VKs equal"
    );
    let control = Control {
        raw_ir: codec::write_fresh(out, "control.raw.json", &bytes, 4 * 1024 * 1024)?,
        tagged_ir: codec::write_fresh(
            out,
            "control.tagged-ir",
            &codec::encode(&ir)?,
            4 * 1024 * 1024,
        )?,
        prover_key: codec::write_fresh(out, "control.pk", &a, 64 * 1024 * 1024)?,
        verifier_key: codec::write_fresh(out, "control.vk", &b, 4 * 1024 * 1024)?,
        k: p.k,
        rows: model.rows(),
        initialized_distinct: true,
    };
    Ok(Produced {
        schema: "compact-rust-proof-data-produced/v1".into(),
        input_sha256: bundle.manifest_sha.clone(),
        profile: bundle.input.profile,
        sdk: bundle.input.sdk.clone(),
        k: p.k,
        rows: p.rows,
        tagged_ir,
        own_params,
        parent_sha256: bundle
            .input
            .parent
            .as_ref()
            .context("parent")?
            .sha256
            .clone(),
        prover_key,
        verifier_key,
        control,
        cases,
        independent_verification_performed: false,
    })
}
