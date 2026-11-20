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

//! Prove nonempty composite return paths; retain exact empty-call refusal.
use super::local_v3_application as v3;
use super::*;
use compact_rust_stateful_struct_oracle_fixture::{RETAINED_IR, ledger_contract as c};
use midnight_compact_runtime::{self as runtime, BoundedUint, transaction::RecordedCall};
use midnight_zkir_v3::IrSource as IrSourceV3;

const CIRCUITS: [(&str, u8, usize); 3] = [
    ("snapshot", 7, 119),
    ("reverse", 7, 113),
    ("nested", 8, 131),
];

struct Witness;
impl c::Witnesses<Vec<u8>> for Witness {
    fn next_value(
        &self,
        context: WitnessContext<'_, Vec<u8>, c::LedgerView<'_>>,
        tag: BoundedUint<255>,
    ) -> (Vec<u8>, BoundedUint<18446744073709551615>) {
        let result =
            BoundedUint::new(tag.value() * 10 + context.private_state.len() as u128).unwrap();
        let mut p = context.private_state.clone();
        p.push(tag.value() as u8);
        (p, result)
    }
}
fn validate_ir(root: &Path) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value = serde_json::from_str(RETAINED_IR)?;
    let actual: serde_json::Value =
        serde_json::from_reader(File::open(root.join("contract/compact-rust-ir.json"))?)?;
    if expected != actual {
        return Err("stateful struct source/proof IR mismatch".into());
    }
    Ok(())
}

fn artifacts(
    root: &Path,
    name: &str,
    k: u8,
    rows: usize,
) -> Result<(VerifierKey, ContractOperation), Box<dyn Error>> {
    let (ir, verifier, operation) = v3::operation(root, name, k)?;
    if ir.model().rows() != rows {
        return Err(format!("{name} v3 row count differs from the modeled circuit").into());
    }
    Ok((verifier, operation))
}

pub(super) fn verify(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_ir(root)?;
    for (name, k, rows) in CIRCUITS {
        let (verifier, _) = artifacts(root, name, k, rows)?;
        v3::verify_standalone(root, name, &verifier)?;
        println!("{name}: separate v3 verification passed and corrupted proof was refused");
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("stateful struct proof requires local proving".into());
    }
    validate_ir(root)?;
    for (index, (name, k, rows)) in CIRCUITS.into_iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(0x0187_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(Vec::<u8>::new()))?;
        let expected_state = initial.ledger_state.get_ref().clone();
        let (verifier, operation) = artifacts(root, name, k, rows)?;
        let deploy = v3::deploy(expected_state.clone(), name, operation, &mut rng);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let generated = c::Contract::from(Witness);
        let randomness = Fr::from(0_u64);
        let (manual, prepared) = match name {
            "snapshot" => {
                let empty = generated
                    .recording()
                    .snapshot_call(&observed, vec![], false)?;
                if !matches!(
                    empty.prepare(verifier.clone(), randomness),
                    Err(ObservedCallError::Prepare(
                        runtime::transaction::PrepareCallError::EmptyTranscript
                    ))
                ) {
                    return Err(
                        "empty snapshot must refuse preparation without fabricated operations"
                            .into(),
                    );
                }
                println!(
                    "snapshot(false): zero operations retained; exact EmptyTranscript preparation refusal"
                );
                let native = generated.snapshot(observed.circuit_context(vec![]), true)?;
                let recorded =
                    c::recorded::snapshot(observed.circuit_context(vec![]), &Witness, true)?;
                check_trace(&native, &recorded, &[1, 2])?;
                let manual = RecordedCall::new(&observed, recorded, name, true)
                    .prepare(verifier.clone(), randomness)?;
                let typed = generated
                    .recording()
                    .snapshot_call(&observed, vec![], true)?;
                (manual, typed.prepare(verifier.clone(), randomness)?)
            }
            "reverse" => {
                let native = generated.reverse(observed.circuit_context(vec![]))?;
                let recorded = c::recorded::reverse(observed.circuit_context(vec![]), &Witness)?;
                check_trace(&native, &recorded, &[1, 2])?;
                let manual = RecordedCall::new(&observed, recorded, name, ())
                    .prepare(verifier.clone(), randomness)?;
                let typed = generated.recording().reverse_call(&observed, vec![])?;
                (manual, typed.prepare(verifier.clone(), randomness)?)
            }
            "nested" => {
                let native = generated.nested(observed.circuit_context(vec![]))?;
                let recorded = c::recorded::nested(observed.circuit_context(vec![]), &Witness)?;
                check_trace(&native, &recorded, &[1, 2, 3])?;
                if native.result.tail.value() != 32 {
                    return Err("nested helper returned an unexpected tail".into());
                }
                let manual = RecordedCall::new(&observed, recorded, name, ())
                    .prepare(verifier.clone(), randomness)?;
                let typed = generated.recording().nested_call(&observed, vec![])?;
                (manual, typed.prepare(verifier.clone(), randomness)?)
            }
            _ => unreachable!(),
        };
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err("composite observed/direct recording mismatch".into());
        }
        let (proof, inputs) =
            kernel_shielded_effects::prove_and_verify_call_with_input_control::<IrSourceV3>(
                root,
                name,
                &prepared,
                &verifier,
                (name == "snapshot").then(|| AlignedValue::from(false)),
                FetchMode::Synchronous,
            )?;
        v3::write_new(root.join(format!("{name}.proof")), &proof.0)?;
        let mut encoded_inputs = Vec::new();
        tagged_serialize(&inputs, &mut encoded_inputs)?;
        v3::write_new(root.join(format!("{name}.inputs")), &encoded_inputs)?;
        v3::apply_after_deployment(root, name, name, prepared, deploy, &expected_state, &proof)?;
    }
    Ok(())
}

fn check_trace<T: PartialEq>(
    native: &runtime::context::CircuitResult<Vec<u8>, T>,
    recorded: &RecordedCircuitResult<Vec<u8>, T>,
    witness_order: &[u8],
) -> Result<(), Box<dyn Error>> {
    let execution = &recorded.execution;
    if execution.context.private_state != witness_order
        || execution.context.private_state != native.context.private_state
        || execution.result != native.result
        || execution.context.query.state != native.context.query.state
        || execution.context.query.effects != native.context.query.effects
        || execution.gas_cost != native.gas_cost
        || execution.private_transcript_outputs != native.private_transcript_outputs
    {
        return Err("composite native result, witness order, or recorded execution differs".into());
    }
    Ok(())
}
