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

//! Exact-source wide-zero multiplication, selected proofs and strict application.
use super::local_v3_application::{self as v3, write_new};
use super::*;
use compact_rust_wide_zero_multiplication_source_fixture::{
    ledger_contract as c, ledger_slots as slots,
};
use midnight_compact_runtime::{self as r, recording::RecordingFrame};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;

const IR: &str =
    include_str!("../../../tests-rust-backend/wide-zero-multiplication-source/original-ir.json");
const PROGRAMS: [&str; 4] = [
    include_str!(
        "../../../tests-rust-backend/wide-zero-multiplication-source/checked-programs/program-0.json"
    ),
    include_str!(
        "../../../tests-rust-backend/wide-zero-multiplication-source/checked-programs/program-1.json"
    ),
    include_str!(
        "../../../tests-rust-backend/wide-zero-multiplication-source/checked-programs/program-2.json"
    ),
    include_str!(
        "../../../tests-rust-backend/wide-zero-multiplication-source/checked-programs/program-3.json"
    ),
];
type Wide = r::WideUint<4722366482869645213695, { u128::MAX }>;

fn validate_source(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        IR,
    )?;
    let folder = root.join("contract/checked-programs");
    let mut names = std::fs::read_dir(&folder)?
        .map(|p| p.map(|p| p.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let expected = (0..4)
        .map(|i| std::ffi::OsString::from(format!("program-{i}.json")))
        .collect::<Vec<_>>();
    if names != expected {
        return Err("complete wide-zero program roster differs".into());
    }
    for (i, expected) in PROGRAMS.iter().enumerate() {
        let actual: serde_json::Value =
            serde_json::from_slice(&std::fs::read(folder.join(format!("program-{i}.json")))?)?;
        let expected: serde_json::Value = serde_json::from_str(expected)?;
        if actual != expected {
            return Err("complete wide-zero checked program differs".into());
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("wide-zero requires local proving without MIDNIGHT_PROOF_SERVER".into());
    }
    // Independent Rust admission of the entire source and all programs BEFORE
    // touching state, verifier keys, funding or any proof operation.
    validate_source(root)?;
    if root.join("wide-zero-proof-result.json").exists() {
        return Err("proof result already exists".into());
    }
    let mut rows = Vec::new();
    for (index, name) in ["multiplyRight", "multiplyLeft"].into_iter().enumerate() {
        let mut bytes = vec![255; 25];
        if index == 0 {
            bytes.fill(0);
            bytes[16] = 1;
        }
        let x = Wide::from_le_bytes(&bytes)?;
        let mut rng = StdRng::seed_from_u64(0x0700_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(42_u64))?;
        let (ir, verifier, operation) = v3::operation(root, name, 7)?;
        assert_eq!(ir.model().rows(), 102);
        let entry = EntryPointBuf(name.as_bytes().to_vec());
        let contract = ContractState::new(
            initial.ledger_state.get_ref().clone(),
            HashMap::new().insert(entry, operation),
            ContractMaintenanceAuthority::default(),
        );
        let deploy = ContractDeploy::new(&mut rng, contract);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let (native, recorded, functional_native, functional) = if index == 0 {
            (
                c::multiplyRight(observed.circuit_context(42), x)?,
                c::recorded::multiplyRight(observed.circuit_context(42), x)?,
                c::functional::multiplyRight::Program::new(x)
                    .execute_native(observed.circuit_context(42))?,
                c::functional::multiplyRight::Program::new(x)
                    .execute_recorded(observed.circuit_context(42))?,
            )
        } else {
            (
                c::multiplyLeft(observed.circuit_context(42), x)?,
                c::recorded::multiplyLeft(observed.circuit_context(42), x)?,
                c::functional::multiplyLeft::Program::new(x)
                    .execute_native(observed.circuit_context(42))?,
                c::functional::multiplyLeft::Program::new(x)
                    .execute_recorded(observed.circuit_context(42))?,
            )
        };
        let manual = slots::calls
            .record_increment(RecordingFrame::new(observed.circuit_context(42)), 1)?
            .finish(r::BoundedUint::<255>::new(0)?);
        for actual in [
            &native,
            &recorded.execution,
            &functional_native,
            &functional.execution,
            &manual.execution,
        ] {
            if actual.result.value() != 0
                || actual.context.private_state != 42
                || actual.context.query.state != native.context.query.state
                || actual.context.query.effects != native.context.query.effects
                || actual.gas_cost != native.gas_cost
                || !actual.private_transcript_outputs.is_empty()
            {
                return Err("wide-zero native/recorded/functional/manual execution differs".into());
            }
        }
        for actual in [&recorded, &functional] {
            if actual.public.verify_ops() != manual.public.verify_ops()
                || actual.public.verify_ops().is_empty()
            {
                return Err("wide-zero manual recording differs".into());
            }
            let replay = actual.public.initial().query(
                actual.public.verify_ops(),
                None,
                &actual.execution.context.cost_model,
            )?;
            if replay.context.state != native.context.query.state
                || replay.context.effects != native.context.query.effects
            {
                return Err("wide-zero public trace replay differs".into());
            }
        }
        if slots::calls.inspect(native.context.query.state.get_ref())? != 1 {
            return Err("Counter differs".into());
        }
        let expected = native.context.query.state.get_ref().clone();
        let randomness: Fr = rand::Rng::r#gen(&mut rand::rngs::OsRng);
        let prepared = r::transaction::RecordedCall::new(&observed, functional, name, x)
            .prepare(verifier.clone(), randomness)?;
        let ordinary = r::transaction::RecordedCall::new(&observed, recorded, name, x)
            .prepare(verifier.clone(), randomness)?;
        if format!("{prepared:?}") != format!("{ordinary:?}") {
            return Err("wide-zero functional and ordinary preparation differ".into());
        }
        if prepared.input != AlignedValue::from(x)
            || prepared.output != AlignedValue::from(r::BoundedUint::<255>::new(0)?)
        {
            return Err("wide-zero exact public input/result atoms or alignment differs".into());
        }
        let proof = prove(root, name, index, &prepared, &verifier, &ir, x)?;
        let transaction_proof_bytes = v3::apply(
            root,
            name,
            &format!("wide-zero-{index}"),
            prepared,
            deploy,
            &expected,
            &proof,
        )?;
        rows.push(serde_json::json!({"name":name,"proof_bytes":proof.0.len(),"transaction_proof_bytes":transaction_proof_bytes,
            "transaction_proof_version":3,"transaction_proof_distinct":true,"positive_fee_dust_spends":1,"result":0,"changed_binding_rejected":true,
            "changed_input_same_geometry":true,"changed_result_same_geometry":true,"changed_input_rejected":true,"changed_result_rejected":true,
            "changed_proof_bytes_rejected":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-wide-zero-proof/v2","status":"passed","calls":rows,
        "constructor_execution_proved":false,"scope":"two fresh functional-derived ordinary call proofs and strict-funded applications; no pure/failing/witnessed call proof claim"});
    write_new(
        root.join("wide-zero-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
pub(super) fn verify(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_source(root)?;
    for (index, name) in ["multiplyRight", "multiplyLeft"].into_iter().enumerate() {
        let (_, verifier, _) = v3::operation(root, name, 7)?;
        v3::verify_standalone(root, &format!("wide-zero-{index}"), &verifier)?;
        println!("{name}: standalone v3 proof verified and corruption refused");
    }
    Ok(())
}

fn commitment(call: &ContractCallPrototype<DefaultDB>) -> Fr {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    transient_commit(&fields[..], call.communication_commitment_rand)
}

fn prove(
    root: &Path,
    name: &'static str,
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    ir: &IrSourceV3,
    original_input: Wide,
) -> Result<Proof, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected wide-zero proof version".into()),
        };
    let pk: ProverKey<IrSourceV3> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
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
            "wide-zero proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("wide-zero prover returned an empty proof".into());
    }
    let mut changed_proof = proof.clone();
    changed_proof.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &changed_proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("wide-zero verifier accepted corrupted proof bytes".into());
    }
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
        return Err("wide-zero proof accepted changed public binding".into());
    }
    let mut changed_input_call = call.clone();
    let mut changed_bytes = original_input.as_le_bytes().to_vec();
    changed_bytes[0] ^= 1;
    changed_input_call.input = AlignedValue::from(Wide::from_le_bytes(&changed_bytes)?);
    if changed_input_call.input == call.input
        || changed_input_call.input.alignment != call.input.alignment
        || changed_input_call.input.value.0.len() != call.input.value.0.len()
    {
        return Err("changed wide input did not preserve source geometry".into());
    }
    let changed_input_commitment = self::commitment(&changed_input_call);
    if changed_input_commitment == commitment {
        return Err("changed input did not change commitment".into());
    }
    let mut changed_input = inputs.clone();
    changed_input[1] = changed_input_commitment;
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_input.into_iter())
        .is_ok()
    {
        return Err("wide-zero proof accepted changed input commitment".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(r::BoundedUint::<255>::new(1)?);
    if changed_call.output == call.output
        || changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed wide-zero result did not preserve source geometry".into());
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
        return Err("wide-zero proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed wide-zero proof version".into()),
    };
    if changed_preimage.check(ir).is_ok() {
        return Err("wide-zero IR accepted changed result".into());
    }
    write_new(root.join(format!("wide-zero-{index}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(root.join(format!("wide-zero-{index}.preimage")), &bytes)?;
    let mut bytes = Vec::new();
    tagged_serialize(&inputs, &mut bytes)?;
    write_new(root.join(format!("wide-zero-{index}.inputs")), &bytes)?;
    println!(
        "wide-zero {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof)
}
