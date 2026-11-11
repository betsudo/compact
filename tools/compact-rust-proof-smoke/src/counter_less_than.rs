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

//! Prove an explicit Counter lessThan query without changing the VM transcript.

use super::kernel_shielded_effects::InputMutation;
use super::local_v3_application as v3;
use super::*;
use compact_rust_counter_less_than_oracle_fixture::ledger_contract as contract;
use midnight_compact_runtime::transaction::RecordedCall;

const CIRCUIT: &str = "compare";
const CASES: [(u64, u64, &str); 2] = [(3, 4, "compare-equal"), (4, 3, "compare-above")];
const RETAINED_IR: &str =
    include_str!("../../compact-rust-backend/tests/counter-less-than-schema15-ir.json");

pub(super) fn run(root: &Path, verify: bool) -> Result<(), Box<dyn Error>> {
    // The current compiler emits the same contract in schema 20. Keep the
    // schema-15 fixture for its original tests and compare every current field.
    let mut expected: serde_json::Value = serde_json::from_str(RETAINED_IR)?;
    if expected["schema_version"] != 15 {
        return Err("unexpected Counter lessThan retained IR schema".into());
    }
    expected["schema_version"] = 20.into();
    let actual: serde_json::Value =
        serde_json::from_reader(File::open(root.join("contract/compact-rust-ir.json"))?)?;
    if actual != expected {
        return Err("Counter lessThan source/proof IR mismatch".into());
    }
    let (ir, verifier, operation) = v3::operation(root, CIRCUIT, 7)?;
    if ir.model().rows() != 78 {
        return Err("Counter lessThan v3 row count differs".into());
    }
    if verify {
        for (_, _, stem) in CASES {
            v3::verify_standalone(root, stem, &verifier)?;
            println!("{stem}: separate v3 verification passed and corrupted proof was refused");
        }
        return Ok(());
    }
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("Counter lessThan proof requires local proving".into());
    }
    for (value, changed_value, stem) in CASES {
        prove_case(
            root,
            value,
            changed_value,
            stem,
            &verifier,
            operation.clone(),
        )?;
    }
    Ok(())
}

fn prove_case(
    root: &Path,
    value: u64,
    changed_value: u64,
    stem: &str,
    verifier: &VerifierKey,
    operation: ContractOperation,
) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x4144_5231_3731 + value);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let expected_state = initial.ledger_state.get_ref().clone();
    let deploy = v3::deploy(expected_state.clone(), CIRCUIT, operation, &mut rng);
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let threshold = BoundedUint::<18446744073709551615>::new(value as u128)?;
    let native = contract::compare(observed.circuit_context(()), threshold)?;
    let recorded = contract::recorded::compare(observed.circuit_context(()), threshold)?;
    if native.result != (value > 3)
        || recorded.execution.result != native.result
        || !native.private_transcript_outputs.is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
        || native.context.query.state.get_ref() != &expected_state
        || recorded.execution.context.query.state != native.context.query.state
        || recorded.execution.context.query.effects != native.context.query.effects
        || recorded.execution.gas_cost != native.gas_cost
        || read_cell_at_path::<u64, _>(&expected_state, &[0])? != 3
    {
        return Err("Counter lessThan native and recorded execution differs".into());
    }
    let randomness = Fr::from(0_u64);
    let manual = RecordedCall::new(&observed, recorded, CIRCUIT, threshold)
        .prepare(verifier.clone(), randomness)?;
    let typed = contract::recorded::Contract.compare_call(&observed, (), threshold)?;
    let prepared = typed.prepare(verifier.clone(), randomness)?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("Counter lessThan observed call differs from direct recording".into());
    }

    let other = BoundedUint::<18446744073709551615>::new(changed_value as u128)?;
    let proof = v3::prove_standalone(
        root,
        CIRCUIT,
        stem,
        &prepared,
        verifier,
        Some(InputMutation::Used(other.into())),
    )?;
    v3::apply_after_deployment(
        root,
        CIRCUIT,
        stem,
        prepared,
        deploy,
        &expected_state,
        &proof,
    )?;
    Ok(())
}
