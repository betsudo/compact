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

//! Prove either pinned disclosed branch export with its original Boolean inputs.
use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as generated;
use midnight_compact_runtime::public_state::PublicStateSource;
use midnight_compact_runtime::transaction::ObservedCallError;
use midnight_transient_crypto::proofs::Proof;

const CASE: &str = env!("COMPACT_KEYWORD_DISCLOSE_CASE");
const CIRCUIT: &str = env!("COMPACT_KEYWORD_DISCLOSE_CIRCUIT");
const WRONG: &str = env!("COMPACT_KEYWORD_DISCLOSE_WRONG");
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_KEYWORD_DISCLOSE_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_KEYWORD_DISCLOSE_GENERATED"),
    "/zkir/",
    env!("COMPACT_KEYWORD_DISCLOSE_CIRCUIT"),
    ".zkir"
));

fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join(format!("{CIRCUIT}/contract/compact-rust-ir.json")))? != GENERATED_IR
    {
        return Err("disclose complete Rust IR bytes differ".into());
    }
    if std::fs::read(root.join(format!("zkir/{CIRCUIT}.zkir")))? != GENERATED_ZKIR {
        return Err("disclose raw ZKIR bytes differ".into());
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

fn reject_wrong_verifier_proof(
    root: &Path,
    stem: &str,
    proof: &Proof,
) -> Result<(), Box<dyn Error>> {
    let wrong = v3::read_verifier(root, WRONG)?;
    let bytes = std::fs::read(root.join(format!("{stem}.inputs")))?;
    let mut reader = bytes.as_slice();
    let inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty()
        || wrong
            .verify(&PARAMS_VERIFIER, proof, inputs.into_iter())
            .is_ok()
    {
        return Err("disclose proof accepted independent wrong verifier".into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("local disclose proof refuses remote proof server".into());
    }
    validate_bindings(root)?;
    let result_path = root.join(format!("{CIRCUIT}-proof-result.json"));
    if result_path.exists() {
        return Err("disclose proof result already exists".into());
    }
    let mut rng = StdRng::seed_from_u64(0x0721_0046);
    #[cfg(compact_keyword_disclose_branch)]
    let initial = generated::initial_state(ConstructorContext::new(()))?;
    #[cfg(compact_keyword_disclose_constructor)]
    let initial = generated::initial_state(ConstructorContext::new(()), true, false)?;
    let (_, verifier, operation) = v3::load_operation(root, CIRCUIT)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation.clone(),
        &mut rng,
    );
    let observed = observed(&deploy);
    #[cfg(compact_keyword_disclose_branch)]
    let native = generated::disclose_branch_conditions(observed.circuit_context(()), true, false)?;
    #[cfg(compact_keyword_disclose_constructor)]
    let native =
        generated::disclose_branch_conditions_in_constructor(observed.circuit_context(()))?;
    #[cfg(compact_keyword_disclose_branch)]
    let recorded =
        generated::recorded::disclose_branch_conditions(observed.circuit_context(()), true, false)?;
    #[cfg(compact_keyword_disclose_constructor)]
    let recorded = generated::recorded::disclose_branch_conditions_in_constructor(
        observed.circuit_context(()),
    )?;
    if native.result != runtime::Field::from(3_u128)
        || generated::PublicStateView::from(&native).x()? != runtime::Field::from(3_u128)
        || native.result != recorded.execution.result
        || native.public_state() != recorded.execution.public_state()
        || native.gas_cost != recorded.execution.gas_cost
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
        || recorded.public.verify_ops().is_empty()
    {
        return Err("original disclose result, state, gas or transcript changed".into());
    }
    let state = native.context.query.state.get_ref().clone();
    #[cfg(compact_keyword_disclose_branch)]
    {
        let next = generated::disclose_branch_conditions(native.context, false, true)?;
        if next.result != runtime::Field::from(4_u128)
            || generated::PublicStateView::from(&next).x()? != runtime::Field::from(4_u128)
        {
            return Err("original second disclose result changed".into());
        }
    }
    let wrong = v3::read_verifier(root, WRONG)?;
    let facade = generated::Contract::default();
    #[cfg(compact_keyword_disclose_branch)]
    let call = || {
        facade
            .recording()
            .disclose_branch_conditions_call(&observed, (), true, false)
    };
    #[cfg(compact_keyword_disclose_constructor)]
    let call = || {
        facade
            .recording()
            .disclose_branch_conditions_in_constructor_call(&observed, ())
    };
    if !matches!(
        call()?.prepare(wrong, Fr::from(0_u64)),
        Err(ObservedCallError::VerifierMismatch(ref circuit)) if circuit == CIRCUIT
    ) {
        return Err("wrong verifier accepted before disclose proof".into());
    }
    let prepared = call()?.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation
        || prepared.entry_point != EntryPointBuf(CIRCUIT.as_bytes().to_vec())
    {
        return Err("prepared disclose operation differs from source".into());
    }
    let stem = format!("keyword-disclose-{CASE}");
    let standalone = v3::prove_standalone(root, CIRCUIT, &stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, CIRCUIT, &stem)?;
    reject_wrong_verifier_proof(root, &stem, &standalone)?;
    let transaction_proof_bytes =
        v3::apply(root, CIRCUIT, &stem, prepared, deploy, &state, &standalone)?;
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
        return Err("disclose proof or strict replay artifact changed".into());
    }
    let result = serde_json::json!({
        "format":"compact-ledger9-keyword-disclose-proof/v1",
        "status":"passed",
        "case":CASE,
        "selected_source_export_proved":CIRCUIT,
        "result":3,
        "ledger_x":3,
        "original_second_result": if CASE == "branch_conditions" { Some(4) } else { None },
        "public_verify_ops":recorded.public.verify_ops().len(),
        "private_transcript_outputs":0,
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
