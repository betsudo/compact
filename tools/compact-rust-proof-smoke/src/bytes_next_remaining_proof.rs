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

//! Four pinned Bytes Fold/loop proofs with funded strict application.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime::public_state::PublicStateSource;
use midnight_compact_runtime::transaction::ObservedCallError;
use midnight_transient_crypto::proofs::Proof;

#[cfg(feature = "bytes-fold-conditional-proof")]
const CIRCUIT: &str = "bytes_fold_conditional_accumulation";
#[cfg(feature = "bytes-fold-summation-proof")]
const CIRCUIT: &str = "bytes_fold_summation";
#[cfg(feature = "bytes-loop-spread-proof")]
const CIRCUIT: &str = "bytes_loop_spread_iteration";
#[cfg(feature = "bytes-loop-iteration-proof")]
const CIRCUIT: &str = "bytes_loop_iteration";
#[cfg(feature = "bytes-manual-update-proof")]
const CIRCUIT: &str = "bytes_manual_update_semantics";
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/contract/compact-rust-ir.json"
));
#[cfg(feature = "bytes-fold-conditional-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/zkir/bytes_fold_conditional_accumulation.zkir"
));
#[cfg(feature = "bytes-fold-summation-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/zkir/bytes_fold_summation.zkir"
));
#[cfg(feature = "bytes-loop-spread-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/zkir/bytes_loop_spread_iteration.zkir"
));
#[cfg(feature = "bytes-loop-iteration-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/zkir/bytes_loop_iteration.zkir"
));
#[cfg(feature = "bytes-manual-update-proof")]
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_NEXT_REMAINING_GENERATED"),
    "/zkir/bytes_manual_update_semantics.zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json")))? != GENERATED_IR
    {
        return Err(format!("{CIRCUIT} complete Rust IR bytes differ").into());
    }
    if std::fs::read(root.join(format!("zkir/{CIRCUIT}.zkir")))? != GENERATED_ZKIR {
        return Err(format!("{CIRCUIT} raw ZKIR bytes differ").into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local remaining Bytes proof gate refuses a remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join(format!("{CIRCUIT}-proof-result.json"));
    if result_path.exists() {
        return Err("remaining Bytes proof result already exists".into());
    }
    #[cfg(feature = "bytes-fold-conditional-proof")]
    let call = prove_conditional(root)?;
    #[cfg(feature = "bytes-fold-summation-proof")]
    let call = prove_summation(root)?;
    #[cfg(feature = "bytes-loop-spread-proof")]
    let call = prove_spread(root)?;
    #[cfg(feature = "bytes-loop-iteration-proof")]
    let call = prove_iteration(root)?;
    #[cfg(feature = "bytes-manual-update-proof")]
    let call = prove_manual_update(root)?;
    let result = serde_json::json!({
        "format":"compact-ledger9-bytes-next-remaining-proof/v1",
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
        return Err("standalone and transaction-bound Bytes proofs absent or identical".into());
    }
    if std::fs::read(root.join(format!("{stem}-ledger-after.bin")))?
        != std::fs::read(root.join(format!("{stem}-ledger-replay.bin")))?
    {
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

macro_rules! field_proof {
    ($feature:literal, $function:ident, $call:ident, $facade_call:ident, $view:ident,
     $name:literal, $stem:literal, $k:literal, $expected:literal,
     $other:literal, $seed:literal) => {
        #[cfg(feature = $feature)]
        fn $function(root: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
            let mut rng = StdRng::seed_from_u64($seed);
            let initial = contract::initial_state(ConstructorContext::new(()))?;
            let (_, verifier, operation) = v3::operation(root, $name, $k)?;
            let deploy = v3::deploy(
                initial.ledger_state.get_ref().clone(),
                $name,
                operation.clone(),
                &mut rng,
            );
            let observed = observed(&deploy);
            let native = contract::$call(observed.circuit_context(()))?;
            let recorded = contract::recorded::$call(observed.circuit_context(()))?;
            if native.result != Field::from($expected as u128)
                || contract::PublicStateView::from(&native).$view()?
                    != Field::from($expected as u128)
                || native.result != recorded.execution.result
                || native.public_state() != recorded.execution.public_state()
                || native.gas_cost != recorded.execution.gas_cost
                || !native.private_transcript_outputs.is_empty()
            {
                return Err("original Bytes Fold/loop result, state or recording changed".into());
            }
            let state = native.context.query.state.get_ref().clone();
            let wrong = v3::read_verifier(root, $other)?;
            let facade = contract::Contract::default();
            let call = || facade.recording().$facade_call(&observed, ());
            if !matches!(call()?.prepare(wrong, Fr::from(0_u64)),
                         Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == $name)
            {
                return Err("wrong Bytes verifier accepted before proving".into());
            }
            let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
            if prepared.op != operation
                || prepared.entry_point != EntryPointBuf($name.as_bytes().to_vec())
            {
                return Err("prepared Bytes operation differs from installed source".into());
            }
            let standalone = v3::prove_standalone(root, $name, $stem, &prepared, &verifier, None)?;
            positional_transaction::verify_separately(root, $name, $stem)?;
            reject_wrong_verifier_proof(root, $other, $stem, &standalone)?;
            let proof_bytes = v3::apply(root, $name, $stem, prepared, deploy, &state, &standalone)?;
            let mut summary = check_proof_artifacts(root, $stem, proof_bytes)?;
            summary["circuit"] = $name.into();
            summary["k"] = $k.into();
            summary["result_field"] = $expected.into();
            Ok(summary)
        }
    };
}

field_proof!(
    "bytes-fold-conditional-proof",
    prove_conditional,
    bytes_fold_conditional_accumulation,
    bytes_fold_conditional_accumulation_call,
    fold_result,
    "bytes_fold_conditional_accumulation",
    "bytes-fold-conditional",
    7,
    15,
    "bytes_fold_summation",
    0x0721_1001
);
field_proof!(
    "bytes-loop-spread-proof",
    prove_spread,
    bytes_loop_spread_iteration,
    bytes_loop_spread_iteration_call,
    loop_result,
    "bytes_loop_spread_iteration",
    "bytes-loop-spread",
    6,
    0,
    "bytes_loop_iteration",
    0x0721_1003
);
field_proof!(
    "bytes-loop-iteration-proof",
    prove_iteration,
    bytes_loop_iteration,
    bytes_loop_iteration_call,
    loop_result,
    "bytes_loop_iteration",
    "bytes-loop-iteration",
    8,
    120,
    "bytes_loop_spread_iteration",
    0x0721_1004
);

#[cfg(feature = "bytes-fold-summation-proof")]
fn prove_summation(root: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let name = "bytes_fold_summation";
    let stem = "bytes-fold-summation";
    let mut rng = StdRng::seed_from_u64(0x0721_1002);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, name, 7)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = contract::bytes_fold_summation(observed.circuit_context(()))?;
    let recorded = contract::recorded::bytes_fold_summation(observed.circuit_context(()))?;
    if native.result.value() != 15
        || contract::PublicStateView::from(&native)
            .fold_result()?
            .value()
            != 15
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || !native.private_transcript_outputs.is_empty()
    {
        return Err("original Bytes summation result, state or recording changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, "bytes_fold_conditional_accumulation")?;
    let facade = contract::Contract::default();
    let call = || facade.recording().bytes_fold_summation_call(&observed, ());
    if !matches!(call()?.prepare(wrong, Fr::from(0_u64)),
                 Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == name)
    {
        return Err("wrong Bytes summation verifier accepted before proving".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation || prepared.entry_point != EntryPointBuf(name.as_bytes().to_vec()) {
        return Err("prepared Bytes summation operation differs from source".into());
    }
    let standalone = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    reject_wrong_verifier_proof(
        root,
        "bytes_fold_conditional_accumulation",
        stem,
        &standalone,
    )?;
    let proof_bytes = v3::apply(root, name, stem, prepared, deploy, &state, &standalone)?;
    let mut summary = check_proof_artifacts(root, stem, proof_bytes)?;
    summary["circuit"] = name.into();
    summary["k"] = 7.into();
    summary["result_uint"] = 15.into();
    Ok(summary)
}

#[cfg(feature = "bytes-manual-update-proof")]
fn prove_manual_update(root: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let name = "bytes_manual_update_semantics";
    let stem = "bytes-manual-update";
    let expected = FixedBytes::new([1, 4, 14, 50]);
    let mut rng = StdRng::seed_from_u64(0x0721_1005);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, name, 12)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = contract::bytes_manual_update_semantics(observed.circuit_context(()))?;
    let recorded = contract::recorded::bytes_manual_update_semantics(observed.circuit_context(()))?;
    if native.result != expected
        || contract::PublicStateView::from(&native).manual_update_result()? != expected
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || !native.private_transcript_outputs.is_empty()
    {
        return Err("original Bytes manual update result, state or recording changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, "bytes_fold_boolean_reduction")?;
    let facade = contract::Contract::default();
    let call = || {
        facade
            .recording()
            .bytes_manual_update_semantics_call(&observed, ())
    };
    if !matches!(call()?.prepare(wrong, Fr::from(0_u64)),
                 Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == name)
    {
        return Err("wrong Bytes manual update verifier accepted before proving".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation || prepared.entry_point != EntryPointBuf(name.as_bytes().to_vec()) {
        return Err("prepared Bytes manual update operation differs from source".into());
    }
    let standalone = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    reject_wrong_verifier_proof(root, "bytes_fold_boolean_reduction", stem, &standalone)?;
    let proof_bytes = v3::apply(root, name, stem, prepared, deploy, &state, &standalone)?;
    let mut summary = check_proof_artifacts(root, stem, proof_bytes)?;
    summary["circuit"] = name.into();
    summary["k"] = 12.into();
    summary["result_bytes_hex"] = "01040e32".into();
    Ok(summary)
}
