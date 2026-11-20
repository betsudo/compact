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

//! Shared two-branch proof scenario; adapters select their actual generated crate.
//! Each branch uses the exact installed v3 operation and funded ledger validation.
use super::kernel_shielded_effects::InputMutation;
use super::local_v3_application as v3;
use super::*;
use midnight_compact_runtime::CompactError;
use midnight_compact_runtime::context::{CircuitContext, CircuitResult};
use midnight_compact_runtime::transaction::RecordedCall;

const CIRCUIT: &str = "choose";
const CASES: [(bool, u64, u64, u64, &str); 2] = [
    (false, 9, 0, 0, "choose-unprimed"),
    (true, 8, 109, 1, "choose-primed"),
];

pub(super) trait ChooseScenario {
    fn initial_context() -> Result<CircuitContext<u64>, CompactError>;
    fn native(
        context: CircuitContext<u64>,
        next: Field,
    ) -> Result<CircuitResult<u64, Field>, CompactError>;
    fn recorded(
        context: CircuitContext<u64>,
        next: Field,
    ) -> Result<RecordedCircuitResult<u64, Field>, CompactError>;
    fn prepared(
        observed: &ObservedContractState,
        next: Field,
        verifier: VerifierKey,
    ) -> Result<ContractCallPrototype<DefaultDB>, Box<dyn Error>>;
}

pub(super) fn run<S: ChooseScenario>(root: &Path, verify: bool) -> Result<(), Box<dyn Error>> {
    let (ir, verifier, operation) = v3::operation(root, CIRCUIT, 7)?;
    if ir.model().rows() != 107 {
        return Err("effectful choose v3 row count differs".into());
    }
    if verify {
        for (_, _, _, _, stem) in CASES {
            v3::verify_standalone(root, stem, &verifier)?;
            println!("{stem}: separate v3 verification passed and corrupted proof was refused");
        }
        return Ok(());
    }
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("effectful choose proof requires local proving".into());
    }
    for (index, (primed, next, expected, private, stem)) in CASES.into_iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(0x0182_0000 + index as u64);
        let initial = S::initial_context()?;
        let state = if primed {
            S::native(initial, Fr::from(9_u64))?
                .context
                .query
                .state
                .get_ref()
                .clone()
        } else {
            initial.query.state.get_ref().clone()
        };
        let deploy = v3::deploy(state, CIRCUIT, operation.clone(), &mut rng);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let native = S::native(observed.circuit_context(0_u64), Fr::from(next))?;
        let recorded = S::recorded(observed.circuit_context(0_u64), Fr::from(next))?;
        if recorded.execution.result != Fr::from(expected)
            || recorded.execution.result != native.result
            || recorded.execution.context.private_state != private
            || recorded.execution.context.private_state != native.context.private_state
            || recorded.execution.private_transcript_outputs.len() != private as usize
            || recorded.execution.private_transcript_outputs != native.private_transcript_outputs
            || recorded.execution.gas_cost != native.gas_cost
            || recorded.execution.context.query.state != native.context.query.state
            || recorded.execution.context.query.effects != native.context.query.effects
        {
            return Err(
                format!("{stem}: branch result, state, gas, effects or witness mismatch").into(),
            );
        }
        let expected_state = native.context.query.state.get_ref().clone();
        let manual = RecordedCall::new(&observed, recorded, CIRCUIT, Fr::from(next))
            .prepare(verifier.clone(), Fr::from(0_u64))?;
        let prepared = S::prepared(&observed, Fr::from(next), verifier.clone())?;
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err(format!("{stem}: observed call differs from direct recording").into());
        }
        let changed_input = AlignedValue::from(Fr::from(next + 1));
        // The primed branch never reads `next`. Its preimage still checks,
        // while the proof must refuse any changed communication commitment.
        let mutation = if primed {
            InputMutation::Unused(changed_input)
        } else {
            InputMutation::Used(changed_input)
        };
        let proof =
            v3::prove_standalone(root, CIRCUIT, stem, &prepared, &verifier, Some(mutation))?;
        v3::apply_after_deployment(
            root,
            CIRCUIT,
            stem,
            prepared,
            deploy,
            &expected_state,
            &proof,
        )?;
    }
    Ok(())
}
