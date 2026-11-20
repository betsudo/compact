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

//! Representative pinned Bytes Fold and loop proofs with strict funded calls.
use super::{local_v3_application as v3, positional_transaction, *};
#[cfg(feature = "bytes-fold-boolean-proof")]
use crate::ledger_contract as fold;
#[cfg(feature = "bytes-loop-empty-proof")]
use crate::ledger_contract as empty_loop;
use midnight_compact_runtime::public_state::PublicStateSource;
use midnight_compact_runtime::transaction::ObservedCallError;
use midnight_transient_crypto::proofs::Proof;

#[cfg(feature = "bytes-fold-boolean-proof")]
const CIRCUIT: &str = "bytes_fold_boolean_reduction";
#[cfg(feature = "bytes-loop-empty-proof")]
const CIRCUIT: &str = "bytes_loop_empty_iteration";
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_GENERATED"),
    "/contract/compact-rust-ir.json"
));
#[cfg(feature = "bytes-fold-boolean-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_GENERATED"),
    "/zkir/bytes_fold_boolean_reduction.zkir"
));
#[cfg(feature = "bytes-loop-empty-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_GENERATED"),
    "/zkir/bytes_loop_empty_iteration.zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    let member = root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json"));
    if std::fs::read(member)? != GENERATED_IR {
        return Err(format!("{CIRCUIT} complete Rust IR bytes differ").into());
    }
    let member = root.join(format!("zkir/{CIRCUIT}.zkir"));
    if std::fs::read(member)? != GENERATED_ZKIR {
        return Err(format!("{CIRCUIT} raw ZKIR bytes differ").into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local next Bytes proof gate refuses a remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join(format!("{CIRCUIT}-proof-result.json"));
    if result_path.exists() {
        return Err("next Bytes proof result already exists".into());
    }
    #[cfg(feature = "bytes-fold-boolean-proof")]
    let call = prove_fold(root)?;
    #[cfg(feature = "bytes-loop-empty-proof")]
    let call = prove_empty_loop(root)?;
    let result = serde_json::json!({
        "format":"compact-ledger9-bytes-next-representative-proof/v1",
        "status":"passed",
        "constructor_execution_proved":false,
        "all_exports_proved":false,
        "selected_source_export_proved":CIRCUIT,
        "calls":[call],
    });
    v3::write_new(result_path, &serde_json::to_vec_pretty(&result)?)?;
    println!("{result}");
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

fn check_proof_artifacts(
    root: &Path,
    stem: &str,
    transaction_proof_bytes: usize,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let standalone = std::fs::read(root.join(format!("{stem}.proof")))?;
    let transaction = std::fs::read(root.join(format!("{stem}-transaction.proof")))?;
    if standalone.is_empty()
        || transaction.is_empty()
        || standalone == transaction
        || transaction.len() != transaction_proof_bytes
    {
        return Err("standalone and transaction-bound Bytes proofs are absent or identical".into());
    }
    let after = std::fs::read(root.join(format!("{stem}-ledger-after.bin")))?;
    let replay = std::fs::read(root.join(format!("{stem}-ledger-replay.bin")))?;
    if after != replay {
        return Err("Bytes strict replay changed ledger bytes".into());
    }
    Ok(serde_json::json!({
        "standalone_proof":format!("{stem}.proof"),
        "proof_bytes":standalone.len(),
        "transaction_proof":format!("{stem}-transaction.proof"),
        "transaction_proof_bytes":transaction.len(),
        "transaction_proof_version":3,
        "transaction_proof_relationship":"independently proved",
        "positive_fee_dust_spends":1,
        "strict_application":"default",
        "replay_refusal":"IntentAlreadyExists",
        "replay_unchanged":true,
        "standalone_verified_separately":true,
        "corrupt_proof_rejected":true,
        "changed_binding_rejected":true,
        "wrong_verifier_rejected_at_preparation":true,
        "wrong_verifier_proof_rejected":true,
    }))
}

fn reject_wrong_verifier_proof(
    root: &Path,
    other: &str,
    stem: &str,
    proof: &Proof,
) -> Result<(), Box<dyn Error>> {
    let wrong = v3::read_verifier(root, other)?;
    let bytes = std::fs::read(root.join(format!("{stem}.inputs")))?;
    let mut reader = bytes.as_slice();
    let inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty()
        || wrong
            .verify(&PARAMS_VERIFIER, proof, inputs.into_iter())
            .is_ok()
    {
        return Err("Bytes proof accepted wrong independent verifier".into());
    }
    Ok(())
}

#[cfg(feature = "bytes-fold-boolean-proof")]
fn prove_fold(root: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let name = "bytes_fold_boolean_reduction";
    let stem = "bytes-fold-boolean";
    let mut rng = StdRng::seed_from_u64(0x0721_0001);
    let initial = fold::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, name, 7)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = fold::bytes_fold_boolean_reduction(observed.circuit_context(()))?;
    let recorded = fold::recorded::bytes_fold_boolean_reduction(observed.circuit_context(()))?;
    if !native.result
        || !fold::PublicStateView::from(&native).fold_result()?
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || !native.private_transcript_outputs.is_empty()
        || recorded.public.verify_ops().is_empty()
    {
        return Err("original Bytes Fold result/state/recording changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, "bytes_loop_empty_iteration")?;
    let facade = fold::Contract::default();
    let call = || {
        facade
            .recording()
            .bytes_fold_boolean_reduction_call(&observed, ())
    };
    if !matches!(call()?.prepare(wrong, Fr::from(0_u64)),
                 Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == name)
    {
        return Err("wrong Bytes verifier accepted before proving".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation || prepared.entry_point != EntryPointBuf(name.as_bytes().to_vec()) {
        return Err("prepared Bytes Fold operation differs from installed source".into());
    }
    let standalone = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    reject_wrong_verifier_proof(root, "bytes_loop_empty_iteration", stem, &standalone)?;
    let transaction_proof_bytes =
        v3::apply(root, name, stem, prepared, deploy, &state, &standalone)?;
    let mut summary = check_proof_artifacts(root, stem, transaction_proof_bytes)?;
    summary["circuit"] = name.into();
    summary["k"] = 7.into();
    summary["result"] = true.into();
    Ok(summary)
}

#[cfg(feature = "bytes-loop-empty-proof")]
fn prove_empty_loop(root: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let name = "bytes_loop_empty_iteration";
    let stem = "bytes-loop-empty";
    let mut rng = StdRng::seed_from_u64(0x0721_0002);
    let initial = empty_loop::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, name, 6)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = empty_loop::bytes_loop_empty_iteration(observed.circuit_context(()))?;
    let recorded = empty_loop::recorded::bytes_loop_empty_iteration(observed.circuit_context(()))?;
    if native.result != Field::from(0_u64)
        || empty_loop::PublicStateView::from(&native).loop_result()? != Field::from(0_u64)
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || !native.private_transcript_outputs.is_empty()
    {
        return Err("original empty Bytes loop result/state/recording changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, "bytes_fold_boolean_reduction")?;
    let facade = empty_loop::Contract::default();
    let call = || {
        facade
            .recording()
            .bytes_loop_empty_iteration_call(&observed, ())
    };
    if !matches!(call()?.prepare(wrong, Fr::from(0_u64)),
                 Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == name)
    {
        return Err("wrong Bytes loop verifier accepted before proving".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation || prepared.entry_point != EntryPointBuf(name.as_bytes().to_vec()) {
        return Err("prepared Bytes loop operation differs from installed source".into());
    }
    let standalone = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    reject_wrong_verifier_proof(root, "bytes_fold_boolean_reduction", stem, &standalone)?;
    let transaction_proof_bytes =
        v3::apply(root, name, stem, prepared, deploy, &state, &standalone)?;
    let mut summary = check_proof_artifacts(root, stem, transaction_proof_bytes)?;
    summary["circuit"] = name.into();
    summary["k"] = 6.into();
    summary["result_field"] = 0.into();
    Ok(summary)
}
