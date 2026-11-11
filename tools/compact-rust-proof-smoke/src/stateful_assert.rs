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

//! Prove typed short-circuit assertions with Unit and Uint64 results.
use super::local_v3_application as v3;
use super::*;
use compact_rust_stateful_assert_oracle_fixture::ledger_contract as c;
use midnight_base_crypto::hash::persistent_hash;
use midnight_compact_runtime::{BoundedUint, transaction::RecordedCall};
use midnight_zkir_v3::IrSource as IrSourceV3;

const CIRCUITS: [(&str, u8, usize); 2] = [("checked", 7, 109), ("unit_result", 6, 41)];
const SOURCE_ARTIFACTS: &str =
    include_str!("../../compact-rust-backend/tests/stateful-assert-ledger9-artifacts.json");

struct Witness;
impl c::Witnesses<Vec<u8>> for Witness {
    fn next_gate(
        &self,
        context: WitnessContext<'_, Vec<u8>, c::LedgerView<'_>>,
        tag: BoundedUint<255>,
    ) -> (Vec<u8>, bool) {
        let mut state = context.private_state.clone();
        state.push(tag.value() as u8);
        (state, true)
    }
}
fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    let pins: std::collections::BTreeMap<String, String> = serde_json::from_str(SOURCE_ARTIFACTS)?;
    for (path, expected) in pins {
        let bytes = std::fs::read(root.join(&path))?;
        if hex::encode(persistent_hash(&bytes).0) != expected {
            return Err(format!(
                "stateful assert artifact differs from the retained source: {path}"
            )
            .into());
        }
    }
    Ok(())
}

fn operation(
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
    validate_artifacts(root)?;
    for (name, k, rows) in CIRCUITS {
        let (verifier, _) = operation(root, name, k, rows)?;
        v3::verify_standalone(root, name, &verifier)?;
        println!("{name}: separate v3 verification passed and corrupted proof was refused");
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("stateful assert proof requires local proving".into());
    }
    validate_artifacts(root)?;
    for (index, (name, k, rows)) in CIRCUITS.into_iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(0x0185_0000 + index as u64);
        let initial = c::initial_state(
            ConstructorContext::new(Vec::<u8>::new()),
            true,
            BoundedUint::new(3)?,
            BoundedUint::new(7)?,
        )?;
        let expected_state = initial.ledger_state.get_ref().clone();
        let (verifier, operation) = operation(root, name, k, rows)?;
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
        let contract = c::Contract::from(Witness);
        let randomness = Fr::from(0_u64);
        let (manual, prepared) = if name == "checked" {
            let recorded = c::recorded::checked(observed.circuit_context(vec![]), &Witness, true)?;
            if recorded.execution.result.value() != 7
                || recorded.execution.context.private_state != [1, 2, 3]
            {
                return Err("checked assertion result or witness order changed".into());
            }
            if !matches!(
                c::recorded::checked(observed.circuit_context(vec![]), &Witness, false),
                Err(midnight_compact_runtime::CompactError::AssertionFailed(_))
            ) {
                return Err("checked false guard did not reject".into());
            }
            let manual = RecordedCall::new(&observed, recorded, name, true)
                .prepare(verifier.clone(), randomness)?;
            let typed = contract.recording().checked_call(&observed, vec![], true)?;
            (manual, typed.prepare(verifier.clone(), randomness)?)
        } else {
            let recorded =
                c::recorded::unit_result(observed.circuit_context(vec![]), &Witness, true)?;
            if recorded.execution.context.private_state != [1] {
                return Err("unit assertion witness order changed".into());
            }
            if !matches!(
                c::recorded::unit_result(observed.circuit_context(vec![]), &Witness, false),
                Err(midnight_compact_runtime::CompactError::AssertionFailed(_))
            ) {
                return Err("unit false guard did not reject".into());
            }
            let manual = RecordedCall::new(&observed, recorded, name, true)
                .prepare(verifier.clone(), randomness)?;
            let typed = contract
                .recording()
                .unit_result_call(&observed, vec![], true)?;
            (manual, typed.prepare(verifier.clone(), randomness)?)
        };
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err("assertion observed call differs from direct recording".into());
        }
        let (proof, inputs) =
            kernel_shielded_effects::prove_and_verify_call_with_input_control::<IrSourceV3>(
                root,
                name,
                &prepared,
                &verifier,
                Some(AlignedValue::from(false)),
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
