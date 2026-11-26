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

//! Separately authored verifier. Shared IO/codecs do not share statement logic.
mod material;
pub mod statement;
use crate::{
    codec,
    input::{self, Bundle, Produced},
};
use anyhow::{Context, Result, ensure};
use midnight_transient_crypto::{
    curve::Fr,
    proofs::{PARAMS_VERIFIER, Proof, VERIFIER_MAX_DEGREE, Zkir},
};
use midnight_zkir::IrSource;
use serde_json::{Value, json};
use std::path::Path;
const MIB: usize = 1024 * 1024;

/// A decode/admission failure never counts as an SDK refusal. Only this actual
/// SDK-call boundary records a returned accepted/rejected outcome.
pub fn verify_call<E, F: FnOnce() -> std::result::Result<(), E>>(
    trace: &mut Vec<Value>,
    id: &str,
    label: &str,
    accept: bool,
    f: F,
) -> Result<()> {
    let index = trace.len();
    trace.push(
        json!({"case":id,"label":label,"status":"STARTED","expectedAccept":accept,"accepted":null}),
    );
    let actual = f().is_ok();
    trace[index]["status"] = json!("RETURNED");
    trace[index]["accepted"] = json!(actual);
    ensure!(actual == accept, "unexpected SDK verification outcome");
    Ok(())
}
pub fn verify(
    manifest: &Path,
    produced_path: &Path,
    max_k: u8,
    allow_large: bool,
    trace: &mut Vec<Value>,
) -> Result<Value> {
    let bundle = Bundle::load(manifest)?;
    let produced_raw = codec::read_bytes(produced_path, MIB)?;
    let p: Produced = serde_json::from_slice(&produced_raw)?;
    let root = produced_path
        .parent()
        .context("produced parent")?
        .canonicalize()?;
    bundle.produced_roles(&root, &p)?;
    bundle.authenticate_gate()?;
    ensure!(
        p.schema == "compact-rust-proof-data-produced/v1"
            && p.input_sha256 == bundle.manifest_sha
            && p.profile == bundle.input.profile
            && p.sdk == bundle.input.sdk
            && !p.independent_verification_performed,
        "exact input/SDK/producer identity"
    );
    ensure!(
        p.cases.len() == bundle.input.cases.len(),
        "complete case roster"
    );
    let raw = bundle.read(&bundle.input.raw_ir, MIB)?;
    let original = IrSource::load(raw.as_slice())?;
    let model = original.model();
    let k = original.k();
    ensure!(
        model.k() == k && p.k == k && p.rows == model.rows(),
        "actual SDK model dimensions"
    );
    let (binding, opening) = bundle.caller()?;
    let mut statements = Vec::new();
    for (case, proof) in bundle.input.cases.iter().zip(&p.cases) {
        ensure!(
            case.id == proof.id && case.supplemental == proof.supplemental,
            "exact ordered cases"
        );
        let s = statement::derive(
            &raw,
            p.profile,
            case,
            &bundle.stream(case)?,
            binding,
            opening,
        )?;
        ensure!(
            proof.fields == s.fields.len() && proof.skips == s.skips,
            "public dimension metadata only, never returned PI bodies"
        );
        statements.push(s);
    }
    // Independent policy BEFORE any first params/key/proof reader; the producer
    // cannot promote a k13 circuit to the regular lane by changing metadata.
    input::policy(k, max_k, allow_large)?;
    let wrong_raw = codec::read_ref(&root, &p.control.raw_ir, MIB)?;
    let wrong_ir = IrSource::load(wrong_raw.as_slice())?;
    let wrong_model = wrong_ir.model();
    input::policy(wrong_ir.k(), max_k, allow_large)?;
    let mut fixed_control: Value = serde_json::from_slice(&raw)?;
    fixed_control["instructions"][0]["inputs"][0] = json!("0x31");
    ensure!(
        serde_json::from_slice::<Value>(&wrong_raw)? == fixed_control
            && wrong_ir != original
            && wrong_ir.k() == k
            && wrong_model.k() == k
            && p.control.k == k
            && p.control.rows == wrong_model.rows()
            && p.control.initialized_distinct,
        "single distinct same-k control candidate, no search"
    );
    let original_tagged: IrSource = codec::decode(&codec::read_ref(&root, &p.tagged_ir, MIB)?)?;
    let wrong_tagged: IrSource =
        codec::decode(&codec::read_ref(&root, &p.control.tagged_ir, MIB)?)?;
    ensure!(
        original_tagged == original
            && wrong_tagged == wrong_ir
            && p.control.raw_ir.path != p.control.tagged_ir.path,
        "separate RAW JSON and canonical tagged IR roles"
    );
    let own_bytes = codec::read_ref(&root, &p.own_params, 4 * MIB)?;
    let parent_ref = bundle
        .input
        .parent
        .as_ref()
        .context("same-SRS static parent14 required")?;
    let parent_bytes = bundle.read(parent_ref, 4 * MIB)?;
    ensure!(
        p.parent_sha256 == input::PARENT_SHA
            && codec::hash(&parent_bytes) == input::PARENT_SHA
            && VERIFIER_MAX_DEGREE == 14,
        "real current SDK default parent14 identity"
    );
    let parent = material::trusted_parent(&parent_bytes)?;
    let (own, own_verifier) = material::derived_own(&parent, u32::from(k), &own_bytes)?;
    material::same_family(&own, &parent, &own_bytes, &parent_bytes)?;
    let vk_bytes = codec::read_ref(&root, &p.verifier_key, MIB)?;
    let wrong_bytes = codec::read_ref(&root, &p.control.verifier_key, MIB)?;
    let key = material::verifier(&vk_bytes, u32::from(k))?;
    let wrong = material::verifier(&wrong_bytes, u32::from(k))?;
    ensure!(
        vk_bytes != wrong_bytes
            && !key.original_bytes().is_empty()
            && !wrong.original_bytes().is_empty()
            && key.original_bytes() != wrong.original_bytes(),
        "canonical initialized distinct outer/native same-k control key"
    );
    let mut cases = Vec::new();
    for (case, s) in p.cases.iter().zip(&statements) {
        let proof: Proof = codec::decode(&codec::read_ref(&root, &case.proof, MIB)?)?;
        ensure!(!proof.0.is_empty(), "nonempty canonical outer Proof");
        let pi = &s.fields;
        verify_call(trace, &case.id, "own-positive", true, || {
            key.verify(&own_verifier, &proof, pi.iter().copied())
        })?;
        verify_call(trace, &case.id, "default14-positive", true, || {
            key.verify(&PARAMS_VERIFIER, &proof, pi.iter().copied())
        })?;
        let mut changed = pi.clone();
        changed[0] = changed[0] + Fr::from(1u64);
        verify_call(trace, &case.id, "changed-binding", false, || {
            key.verify(&own_verifier, &proof, changed.into_iter())
        })?;
        let mut damaged = proof.0.clone();
        let middle = damaged.len() / 2;
        damaged[middle] ^= 1;
        let outer: Proof = codec::decode(&codec::encode(&Proof(damaged))?)?;
        verify_call(
            trace,
            &case.id,
            "outer-canonical-corrupt-proof",
            false,
            || key.verify(&own_verifier, &outer, pi.iter().copied()),
        )?;
        verify_call(
            trace,
            &case.id,
            "current-valid-distinct-same-k-vk",
            false,
            || wrong.verify(&own_verifier, &proof, pi.iter().copied()),
        )?;
        let flat = pi.iter().flat_map(|f| f.as_le_bytes()).collect::<Vec<_>>();
        cases.push(json!({"id":case.id,"supplemental":case.supplemental,"fields":pi.len(),"skips":s.skips,"statement_sha256":codec::hash(&flat),"sdk_calls":5}));
    }
    ensure!(
        trace.len() == 5 * p.cases.len()
            && trace
                .iter()
                .all(|x| x["status"] == "RETURNED" && x["accepted"] == x["expectedAccept"]),
        "complete actual SDK-call matrix"
    );
    Ok(
        json!({"schema":"compact-rust-proof-data-verification/v1","verified":true,"profile":p.profile,"input_sha256":bundle.manifest_sha,"produced_sha256":codec::hash(&produced_raw),"k":k,"rows_excluding_custom_gates_and_lookups":model.rows(),"default_k":14,"cases":cases,"calls":trace,"producer_pi_used":false,"private_preimage_read":false,"prover_key_read":false,"private_hash_evaluated":false,"before_state_regenerated":false,"shared_sdk_crypto":true,"corruption_scope":"canonical tagged OUTER Proof only; internal rejection layer unobserved","sdk1_linux_chain_qualified":false}),
    )
}
