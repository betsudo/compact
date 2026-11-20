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

//! Fresh source-bound Field Fold proofs and funded default-strict applications.
use super::local_v3_application::{self as v3, write_new};
use super::*;
use midnight_compact_runtime::{self as r, Field, FixedVector};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;
mod cases;
mod empty_ordered;
const NAME: &str = "run";

fn validate_source(case: &str, root: &Path) -> Result<(), Box<dyn Error>> {
    let ir = match case {
        "single" => {
            include_str!("../../../tests-rust-backend/direct-fold-single-source/original-ir.json")
        }
        "empty-ordered" => include_str!(
            "../../../tests-rust-backend/direct-fold-empty-ordered-source/original-ir.json"
        ),
        "captured" => {
            include_str!("../../../tests-rust-backend/direct-fold-captured-source/original-ir.json")
        }
        "counter-capture" => include_str!(
            "../../../tests-rust-backend/direct-fold-counter-capture-source/original-ir.json"
        ),
        "counter-local-capture" => include_str!(
            "../../../tests-rust-backend/direct-fold-counter-local-capture-source/original-ir.json"
        ),
        "multiple-inputs" => include_str!(
            "../../../tests-rust-backend/direct-fold-multiple-inputs-source/original-ir.json"
        ),
        "effectful-reducer" => include_str!(
            "../../../tests-rust-backend/direct-fold-effectful-reducer-source/original-ir.json"
        ),
        "witness-reducer" => include_str!(
            "../../../tests-rust-backend/direct-fold-witness-reducer-source/original-ir.json"
        ),
        "inline-effectful" => include_str!(
            "../../../tests-rust-backend/direct-fold-inline-effectful-source/original-ir.json"
        ),
        "inline-branch" => include_str!(
            "../../../tests-rust-backend/direct-fold-inline-branch-source/original-ir.json"
        ),
        _ => return Err("unknown direct Fold proof case".into()),
    };
    // Pair the complete original IR before any key/state/proof action.
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        ir,
    )
}

pub(super) fn run(case: &str, root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("direct Fold requires local proving without MIDNIGHT_PROOF_SERVER".into());
    }
    validate_source(case, root)?;
    if root.join("fold-proof-result.json").exists() {
        return Err("Fold proof result already exists".into());
    }
    let calls = match case {
        "single" => vec![cases::single(root, 0)?],
        "empty-ordered" => vec![empty_ordered::run(root, 0)?],
        "captured" => vec![cases::captured(root, 0)?],
        "counter-capture" => vec![cases::counter_capture(root, 0)?],
        "counter-local-capture" => vec![cases::counter_local_capture(root, 0)?],
        "multiple-inputs" => vec![cases::multiple(root, 0)?],
        "effectful-reducer" => vec![cases::stateful(root, 0)?],
        "witness-reducer" => vec![cases::witnessed(root, 0)?],
        "inline-effectful" => vec![cases::inline(root, 0)?],
        "inline-branch" => vec![
            cases::branch(root, 0, true)?,
            cases::branch(root, 1, false)?,
        ],
        _ => unreachable!("case validated before key/state actions"),
    };
    let result = serde_json::json!({"format":"compact-direct-fold-proof/v2", "status":"passed", "case":case,
        "scope":"selected Field-returning source-bound functional-derived call proofs and funded default-strict local transactions",
        "constructor_execution_proved":false, "unselected_sources_proved":false, "calls":calls});
    write_new(
        root.join("fold-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
pub(super) fn verify(case: &str, root: &Path) -> Result<(), Box<dyn Error>> {
    validate_source(case, root)?;
    let verifier = v3::read_verifier(root, NAME)?;
    for index in 0..if case == "inline-branch" { 2 } else { 1 } {
        v3::verify_standalone(root, &format!("fold-{index}"), &verifier)?;
        println!("{case}/{index}: standalone v3 proof verified and corruption refused");
    }
    Ok(())
}

fn deployment(
    root: &Path,
    data: StateValue<DefaultDB>,
    rng: &mut StdRng,
) -> Result<(ContractDeploy<DefaultDB>, IrSourceV3, VerifierKey), Box<dyn Error>> {
    let ir = IrSourceV3::load(File::open(root.join("zkir/run.zkir"))?)?;
    let k = ir.k();
    if !(1..=8).contains(&k) {
        return Err("direct Fold exceeds the automatic tiny parameter cap".into());
    }
    let (ir, verifier, operation) = v3::operation(root, NAME, k)?;
    let entry = EntryPointBuf(NAME.as_bytes().to_vec());
    let contract = ContractState::new(
        data,
        HashMap::new().insert(entry, operation),
        ContractMaintenanceAuthority::default(),
    );
    Ok((ContractDeploy::new(rng, contract), ir, verifier))
}

fn commitment(call: &ContractCallPrototype<DefaultDB>) -> Fr {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    transient_commit(&fields[..], call.communication_commitment_rand)
}

fn prove(
    root: &Path,
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    ir: &IrSourceV3,
) -> Result<Proof, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected fold proof version".into()),
        };
    let pk: ProverKey<IrSourceV3> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/run.prover"),
    )?))?;
    let checked = preimage.check(ir)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, skips) = futures_executor::block_on(ir.prove(
        ChaCha20Rng::from_rng(rand::rngs::OsRng)?,
        &params,
        pk,
        &preimage,
    ))?;
    if skips != checked
        || inputs.first() != Some(&preimage.binding_input)
        || inputs.get(1) != Some(&commitment)
    {
        return Err(
            "fold proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    let mut changed_binding = inputs.clone();
    changed_binding[0] = if inputs[0] == Fr::from(0_u64) {
        Fr::from(1_u64)
    } else {
        Fr::from(0_u64)
    };
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_binding.into_iter())
        .is_ok()
    {
        return Err("fold proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(Field::from(42_u64));
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
        || changed_call
            .output
            .value
            .0
            .iter()
            .zip(&call.output.value.0)
            .any(|(a, b)| a.0.len() != b.0.len())
    {
        return Err("changed Fold result geometry differs".into());
    }
    let changed_commitment = self::commitment(&changed_call);
    if changed_commitment == commitment {
        return Err("changed result did not change commitment".into());
    }
    let mut changed_result = inputs.clone();
    changed_result[1] = changed_commitment;
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_result.into_iter())
        .is_ok()
    {
        return Err("fold proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed fold proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("fold IR accepted changed result".into());
    }
    write_new(root.join(format!("fold-{index}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(root.join(format!("fold-{index}.preimage")), &bytes)?;
    let mut bytes = Vec::new();
    tagged_serialize(&inputs, &mut bytes)?;
    write_new(root.join(format!("fold-{index}.inputs")), &bytes)?;
    println!(
        "fold {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof)
}
