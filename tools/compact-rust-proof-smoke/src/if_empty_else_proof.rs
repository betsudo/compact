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

//! Pinned upstream if/empty_else_branch: both original inputs independently proved and funded.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as original;
use midnight_compact_runtime::public_state::PublicStateSource;

const CIRCUIT: &str = "if_empty_else_branch";
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_IF_EMPTY_ELSE_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_IF_EMPTY_ELSE_GENERATED"),
    "/zkir/if_empty_else_branch.zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json")))? != GENERATED_IR
        || std::fs::read(root.join(format!("zkir/{CIRCUIT}.zkir")))? != GENERATED_ZKIR
    {
        return Err("if/empty_else_branch source-bound Rust IR or raw ZKIR bytes differ".into());
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

fn prove_input(
    root: &Path,
    input: bool,
    expected: u128,
    stem: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(if input { 0x1f01_0001 } else { 0x1f01_0000 });
    let initial = original::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::operation(root, CIRCUIT, 7)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    let native = original::if_empty_else_branch(observed.circuit_context(()), input)?;
    let recorded = original::recorded::if_empty_else_branch(observed.circuit_context(()), input)?;
    if native.result != Fr::from(expected)
        || original::PublicStateView::from(&native).count()? != Fr::from(expected)
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
    {
        return Err(
            "original if/empty_else_branch result, state, gas or transcript changed".into(),
        );
    }
    let state = native.context.query.state.get_ref().clone();
    let facade = original::Contract::default();
    let call = || {
        facade
            .recording()
            .if_empty_else_branch_call(&observed, (), input)
    };
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation
        || prepared.entry_point != EntryPointBuf(CIRCUIT.as_bytes().to_vec())
    {
        return Err("prepared if/empty_else_branch operation differs from source".into());
    }
    let standalone = v3::prove_standalone(root, CIRCUIT, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, CIRCUIT, stem)?;
    let transaction_proof_bytes =
        v3::apply(root, CIRCUIT, stem, prepared, deploy, &state, &standalone)?;
    let proof = std::fs::read(root.join(format!("{stem}.proof")))?;
    let transaction = std::fs::read(root.join(format!("{stem}-transaction.proof")))?;
    let after = std::fs::read(root.join(format!("{stem}-ledger-after.bin")))?;
    let replay = std::fs::read(root.join(format!("{stem}-ledger-replay.bin")))?;
    if proof.is_empty()
        || transaction.is_empty()
        || proof == transaction
        || transaction.len() != transaction_proof_bytes
        || after != replay
    {
        return Err("if/empty_else_branch proof or strict replay artifact changed".into());
    }
    Ok(serde_json::json!({
        "input":input, "result":expected, "ledger_count":expected,
        "public_verify_ops":recorded.public.verify_ops().len(),
        "private_transcript_outputs":0,
        "standalone_proof_bytes":proof.len(),
        "transaction_proof_bytes":transaction.len(),
        "standalone_verified_separately":true,
        "corrupt_proof_rejected":true,
        "positive_fee_dust_spends":1,
        "strict_application":"default",
        "replay_refusal":"IntentAlreadyExists",
        "replay_unchanged":true,
    }))
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local if/empty_else_branch proof refuses remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join("if-empty-else-proof-result.json");
    if result_path.exists() {
        return Err("if/empty_else_branch proof result already exists".into());
    }
    let positive = prove_input(root, true, 1, "if-empty-else-true")?;
    let negative = prove_input(root, false, 0, "if-empty-else-false")?;
    let result = serde_json::json!({
        "format":"compact-ledger9-if-empty-else-proof/v1",
        "status":"passed",
        "selected_source_export_proved":CIRCUIT,
        "original_input_results":[positive,negative],
        "constructor_execution_proved":false,
        "all_exports_proved":true,
        "k":7,
        "transaction_proof_version":3,
    });
    v3::write_new(result_path, &serde_json::to_vec_pretty(&result)?)?;
    println!("{result}");
    Ok(())
}
