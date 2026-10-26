// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Zero iterations still evaluate both typed operands, preserving zero-width output.
use super::*;
use compact_rust_direct_fold_empty_ordered_source_fixture::{
    ledger_contract as c, ledger_slots as slots,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct Witness(Rc<RefCell<Vec<(&'static str, u64)>>>);
impl c::TryWitnesses<u64> for Witness {
    fn seed(
        &self,
        ctx: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, Field), r::CompactError> {
        self.0.borrow_mut().push(("seed", *ctx.private_state));
        Ok((*ctx.private_state + 1, Field::from(9_u64)))
    }
    fn items(
        &self,
        ctx: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, FixedVector<Field, 0>), r::CompactError> {
        self.0.borrow_mut().push(("items", *ctx.private_state));
        Ok((*ctx.private_state + 1, FixedVector::new([])))
    }
    fn step(
        &self,
        _ctx: WitnessContext<'_, u64, c::LedgerView<'_>>,
        _acc: Field,
        _item: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "empty reducer must not execute".into(),
        ))
    }
}
pub(super) fn run(root: &Path, index: usize) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x0661_0100 + index as u64);
    let initial = c::initial_state(ConstructorContext::new(7_u64))?;
    let deploy = make_deploy(root, NAME, initial.ledger_state.get_ref().clone(), &mut rng)?;
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let witnesses: [Witness; 5] = std::array::from_fn(|_| Witness::default());
    let native = c::run(observed.circuit_context(7), &witnesses[0])?;
    let recorded = c::recorded::run(observed.circuit_context(7), &witnesses[1])?;
    let functional_native = c::functional::run::Program::new()
        .execute_native(observed.circuit_context(7), &witnesses[2])?;
    let functional = c::functional::run::Program::new()
        .execute_recorded(observed.circuit_context(7), &witnesses[3])?;
    let expected = Field::from(9_u64);
    let outputs = vec![
        AlignedValue::from(expected),
        AlignedValue::from(FixedVector::<Field, 0>::new([])),
    ];
    if outputs.len() != 2 || !outputs[1].value.0.is_empty() {
        return Err("empty output shape changed".into());
    }
    for actual in [
        &native,
        &recorded.execution,
        &functional_native,
        &functional.execution,
    ] {
        if actual.result != expected
            || actual.context.private_state != 9
            || actual.context.query.state != native.context.query.state
            || actual.context.query.effects != native.context.query.effects
            || actual.gas_cost != native.gas_cost
            || actual.private_transcript_outputs != outputs
        {
            return Err(
                "empty Fold native/recorded/functional result/state/effects/private/gas mismatch"
                    .into(),
            );
        }
    }
    if recorded.public.verify_ops() != functional.public.verify_ops()
        || slots::stored.inspect(native.context.query.state.get_ref())? != expected
    {
        return Err("empty Fold recording or stored Cell differs".into());
    }
    let expected_state = native.context.query.state.get_ref().clone();
    let prepared = check_generated_trace(root, NAME, functional, AlignedValue::from(()))?;
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/run.verifier"),
    )?))?;
    let ordinary = c::Contract::from(witnesses[4].clone())
        .recording()
        .run_call(&observed, 7)?
        .prepare(verifier.clone(), Fr::from(0_u64))?;
    if format!("{prepared:?}") != format!("{ordinary:?}") {
        return Err("empty Fold preparation differs from ordinary observed call".into());
    }
    for witness in witnesses {
        if *witness.0.borrow() != [("seed", 7), ("items", 8)] {
            return Err("empty Fold operands not evaluated exactly once in order".into());
        }
    }
    let proof_bytes = prove(root, index, &prepared, &verifier)?;
    apply(root, index, prepared, deploy, &expected_state, rng)?;
    Ok(
        serde_json::json!({"variant":index,"expected":9,"private_state":9,"witness_calls":2,"proof_bytes":proof_bytes,"changed_binding_rejected":true,"changed_result_rejected":true,"changed_result_same_geometry":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}),
    )
}
