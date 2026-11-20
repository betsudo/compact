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

//! Pinned upstream bytes/loops/indexed_access proof and funded strict call.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as original;
use midnight_compact_runtime::public_state::PublicStateSource;
use midnight_compact_runtime::transaction::ObservedCallError;
use midnight_transient_crypto::proofs::Proof;

const CIRCUIT: &str = "bytes_loop_indexed_access";
const WRONG: &str = "bytes_indexed_access_wrong_verifier";
const STEM: &str = "bytes-indexed-access";
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_INDEXED_ACCESS_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_BYTES_INDEXED_ACCESS_GENERATED"),
    "/zkir/bytes_loop_indexed_access.zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json")))? != GENERATED_IR
    {
        return Err("Bytes indexed access complete Rust IR bytes differ".into());
    }
    if std::fs::read(root.join(format!("zkir/{CIRCUIT}.zkir")))? != GENERATED_ZKIR {
        return Err("Bytes indexed access raw ZKIR bytes differ".into());
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
    let wrong = v3::read_verifier(root, WRONG)?;
    let bytes = std::fs::read(root.join(format!("{STEM}.inputs")))?;
    let mut reader = bytes.as_slice();
    let inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty()
        || wrong
            .verify(&PARAMS_VERIFIER, proof, inputs.into_iter())
            .is_ok()
    {
        return Err("Bytes indexed access proof accepted independent wrong verifier".into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local Bytes indexed access proof refuses remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join("bytes-indexed-access-proof-result.json");
    if result_path.exists() {
        return Err("Bytes indexed access proof result already exists".into());
    }
    let mut rng = StdRng::seed_from_u64(0x0753_0001);
    let initial = original::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, CIRCUIT, 8)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = original::bytes_loop_indexed_access(observed.circuit_context(()))?;
    let recorded = original::recorded::bytes_loop_indexed_access(observed.circuit_context(()))?;
    let result = native.result;
    let ledger_result = original::PublicStateView::from(&native).loop_result()?;
    if result != Field::from(32_u128)
        || ledger_result != Field::from(32_u128)
        || recorded.public.verify_ops().len() != 54
        || native.gas_cost.read_time.into_picoseconds() != 2_380_000_000
        || native.gas_cost.compute_time.into_picoseconds() != 22_370_692_016
        || native.gas_cost.bytes_written != 290
        || native.gas_cost.bytes_deleted != 288
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
    {
        return Err(
            "original Bytes indexed access result, state, gas or transcript changed".into(),
        );
    }
    let state = native.context.query.state.get_ref().clone();
    let wrong = v3::read_verifier(root, WRONG)?;
    let facade = original::Contract::default();
    let call = || {
        facade
            .recording()
            .bytes_loop_indexed_access_call(&observed, ())
    };
    if !matches!(
        call()?.prepare(wrong, Fr::from(0_u64)),
        Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == CIRCUIT
    ) {
        return Err("wrong verifier accepted before Bytes indexed access proof".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation
        || prepared.entry_point != EntryPointBuf(CIRCUIT.as_bytes().to_vec())
    {
        return Err("prepared Bytes indexed access operation differs from source".into());
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
        return Err("Bytes indexed access proof or strict replay artifact changed".into());
    }
    let summary = serde_json::json!({
        "format":"compact-ledger9-bytes-indexed-access-proof/v1",
        "status":"passed",
        "selected_source_export_proved":CIRCUIT,
        "constructor_execution_proved":false,
        "all_exports_proved":true,
        "result":32,
        "ledger_result":32,
        "public_verify_ops":recorded.public.verify_ops().len(),
        "private_transcript_outputs":0,
        "gas_read_picoseconds":native.gas_cost.read_time.into_picoseconds(),
        "gas_compute_picoseconds":native.gas_cost.compute_time.into_picoseconds(),
        "gas_bytes_written":native.gas_cost.bytes_written,
        "gas_bytes_deleted":native.gas_cost.bytes_deleted,
        "k":8,
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
    v3::write_new(result_path, &serde_json::to_vec_pretty(&summary)?)?;
    println!("{summary}");
    Ok(())
}
