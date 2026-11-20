// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Prove and apply the Ledger 9 scoped Field Cell update and independent return.

use super::local_v3_application as v3;
use super::*;
use compact_rust_root_let_action_return_oracle_fixture::ledger_contract as contract;
use midnight_compact_runtime::transaction::RecordedCall;
use midnight_zkir_v3::IrSource as IrSourceV3;

const CIRCUIT: &str = "step";
const STEM: &str = "root-let-step";
const K: u8 = 7;
const ROWS: usize = 68;

fn validate_ir(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value =
        serde_json::from_str(compact_rust_root_let_action_return_oracle_fixture::RETAINED_IR)?;
    let actual: serde_json::Value = serde_json::from_slice(bytes)?;
    if expected != actual {
        return Err("root Let return source/proof IR mismatch".into());
    }
    Ok(())
}

fn artifacts(root: &Path) -> Result<(VerifierKey, ContractOperation), Box<dyn Error>> {
    validate_ir(&std::fs::read(root.join("contract/compact-rust-ir.json"))?)?;
    let (ir, verifier, operation) = v3::operation(root, CIRCUIT, K)?;
    if ir.model().rows() != ROWS {
        return Err("root Let v3 row count differs from the modeled circuit".into());
    }
    Ok((verifier, operation))
}

pub(super) fn verify(root: &Path) -> Result<(), Box<dyn Error>> {
    let (verifier, _) = artifacts(root)?;
    v3::verify_standalone(root, STEM, &verifier)?;
    println!("{CIRCUIT}: separate v3 verification passed and corrupted proof was refused");
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("root Let proof requires local proving".into());
    }
    let (verifier, operation) = artifacts(root)?;
    let mut rng = StdRng::seed_from_u64(0x4144_5231_3738);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation,
        &mut rng,
    );
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let echo = Fr::from(9_u64);
    let native = contract::step(observed.circuit_context(()), echo)?;
    let recorded = contract::recorded::step(observed.circuit_context(()), echo)?;
    let expected_state = native.context.query.state.get_ref().clone();
    if native.result != echo
        || recorded.execution.result != echo
        || read_cell_at_path::<Fr, _>(&expected_state, &[0])? != Fr::from(1_u64)
        || recorded.execution.context.query.state != native.context.query.state
        || recorded.execution.context.query.effects != native.context.query.effects
        || recorded.execution.gas_cost != native.gas_cost
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
    {
        return Err("root Let return, Field Cell, or recording differs".into());
    }
    let randomness = Fr::from(0_u64);
    let manual = RecordedCall::new(&observed, recorded, CIRCUIT, echo)
        .prepare(verifier.clone(), randomness)?;
    let prepared = contract::recorded::Contract
        .step_call(&observed, (), echo)?
        .prepare(verifier.clone(), randomness)?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("root Let observed call differs from direct recording".into());
    }
    let (proof, inputs) =
        kernel_shielded_effects::prove_and_verify_call_with_input_control::<IrSourceV3>(
            root,
            CIRCUIT,
            &prepared,
            &verifier,
            Some(AlignedValue::from(Fr::from(10_u64))),
            FetchMode::Synchronous,
        )?;
    v3::write_new(root.join(format!("{STEM}.proof")), &proof.0)?;
    let mut encoded_inputs = Vec::new();
    tagged_serialize(&inputs, &mut encoded_inputs)?;
    v3::write_new(root.join(format!("{STEM}.inputs")), &encoded_inputs)?;
    v3::apply_after_deployment(
        root,
        CIRCUIT,
        STEM,
        prepared,
        deploy,
        &expected_state,
        &proof,
    )?;
    Ok(())
}

#[cfg(test)]
mod source_pairing_tests {
    use super::*;
    #[test]
    fn rejects_mismatched_source_before_keys() {
        let source = compact_rust_root_let_action_return_oracle_fixture::RETAINED_IR;
        validate_ir(source.as_bytes()).unwrap();
        let mut altered: serde_json::Value = serde_json::from_str(source).unwrap();
        altered["stateful_circuits"][0]["name"] = "different".into();
        assert!(validate_ir(&serde_json::to_vec(&altered).unwrap()).is_err());
        assert!(
            validate_ir(
                compact_rust_terminal_lexical_return_oracle_fixture::RETAINED_IR.as_bytes()
            )
            .is_err()
        );
    }
}
