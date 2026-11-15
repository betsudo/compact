// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Pinned upstream for/generic_bound, independently verified and funded locally.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as generic;
use midnight_compact_runtime::public_state::PublicStateSource;
use midnight_compact_runtime::transaction::ObservedCallError;
use midnight_transient_crypto::proofs::Proof;

const CIRCUIT: &str = "for_generic_bound";
const STEM: &str = "for-generic-bound";
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_FOR_GENERIC_BOUND_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_FOR_GENERIC_BOUND_GENERATED"),
    "/zkir/for_generic_bound.zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json")))? != GENERATED_IR
    {
        return Err("for/generic_bound complete Rust IR bytes differ".into());
    }
    if std::fs::read(root.join(format!("zkir/{CIRCUIT}.zkir")))? != GENERATED_ZKIR {
        return Err("for/generic_bound raw ZKIR bytes differ".into());
    }
    Ok(())
}

fn observed(deploy: &ContractDeploy<DefaultDB>) -> ObservedContractState {
    ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    )
}

fn reject_wrong_verifier_proof(root: &Path, proof: &Proof) -> Result<(), Box<dyn Error>> {
    let wrong = v3::read_verifier(root, "for_zero_iteration")?;
    let bytes = std::fs::read(root.join(format!("{STEM}.inputs")))?;
    let mut reader = bytes.as_slice();
    let inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty()
        || wrong
            .verify(&PARAMS_VERIFIER, proof, inputs.into_iter())
            .is_ok()
    {
        return Err("for/generic_bound proof accepted independent wrong verifier".into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local for/generic_bound proof refuses remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join(format!("{CIRCUIT}-proof-result.json"));
    if result_path.exists() {
        return Err("for/generic_bound proof result already exists".into());
    }
    let mut rng = StdRng::seed_from_u64(0x0721_0003);
    let initial = generic::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, CIRCUIT, 7)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = generic::for_generic_bound(observed.circuit_context(()))?;
    let recorded = generic::recorded::for_generic_bound(observed.circuit_context(()))?;
    if native.result.value() != 5
        || generic::PublicStateView::from(&native).total()?.value() != 5
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || native.gas_cost.read_time.into_picoseconds() != 765_000_000
        || native.gas_cost.compute_time.into_picoseconds() != 7_385_878_722
        || native.gas_cost.bytes_written != 72
        || native.gas_cost.bytes_deleted != 72
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
        || recorded.public.verify_ops().len() != 18
    {
        return Err("original for/generic_bound result, state, gas or transcript changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, "for_zero_iteration")?;
    let facade = generic::Contract::default();
    let call = || facade.recording().for_generic_bound_call(&observed, ());
    if !matches!(
        call()?.prepare(wrong, Fr::from(0_u64)),
        Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == CIRCUIT
    ) {
        return Err("wrong verifier accepted before for/generic_bound proof".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation
        || prepared.entry_point != EntryPointBuf(CIRCUIT.as_bytes().to_vec())
    {
        return Err("prepared for/generic_bound operation differs from source".into());
    }
    let standalone = v3::prove_standalone(root, CIRCUIT, STEM, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, CIRCUIT, STEM)?;
    reject_wrong_verifier_proof(root, &standalone)?;
    let transaction_proof_bytes =
        v3::apply(root, CIRCUIT, STEM, prepared, deploy, &state, &standalone)?;
    let proof = std::fs::read(root.join(format!("{STEM}.proof")))?;
    let transaction = std::fs::read(root.join(format!("{STEM}-transaction.proof")))?;
    let after = std::fs::read(root.join(format!("{STEM}-ledger-after.bin")))?;
    let replay = std::fs::read(root.join(format!("{STEM}-ledger-replay.bin")))?;
    if proof.is_empty()
        || transaction.is_empty()
        || proof == transaction
        || transaction.len() != transaction_proof_bytes
        || after != replay
    {
        return Err("for/generic_bound proof or strict replay artifact changed".into());
    }
    let result = serde_json::json!({
        "format":"compact-ledger9-for-generic-bound-proof/v1",
        "status":"passed",
        "selected_source_export_proved":CIRCUIT,
        "constructor_execution_proved":false,
        "all_exports_proved":false,
        "result":5,
        "ledger_total":5,
        "public_verify_ops":18,
        "private_transcript_outputs":0,
        "gas_read_picoseconds":765000000,
        "gas_compute_picoseconds":7385878722_u64,
        "gas_bytes_written":72,
        "gas_bytes_deleted":72,
        "k":7,
        "standalone_proof_bytes":proof.len(),
        "transaction_proof_bytes":transaction.len(),
        "transaction_proof_version":3,
        "standalone_verified_separately":true,
        "wrong_verifier_rejected_at_preparation":true,
        "wrong_verifier_proof_rejected":true,
        "corrupt_proof_rejected":true,
        "changed_binding_rejected":true,
        "positive_fee_dust_spends":1,
        "strict_application":"default",
        "replay_refusal":"IntentAlreadyExists",
        "replay_unchanged":true,
    });
    v3::write_new(result_path, &serde_json::to_vec_pretty(&result)?)?;
    println!("{result}");
    Ok(())
}
