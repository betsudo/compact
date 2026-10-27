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

//! Source-bound composition of plain/historic insertions and root observations.
use super::*;
use compact_rust_effect_plan_merkle_source_fixture as source;
use source::{ledger_contract as contract, runtime as r, types};

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        source::RETAINED_IR,
    )?;
    let mut rng = StdRng::seed_from_u64(0x0545_4d45_524b_4c45);
    for selected in [false, true] {
        let initial = contract::initial_state(ConstructorContext::new(()))?;
        let blank = merkle_tree_view_at_path(initial.ledger_state.get_ref(), &[0])?
            .root()
            .ok_or("missing initial plain Merkle root")?;
        let digest = types::MerkleTreeDigest { field: blank.0 };
        let leaf = r::BoundedUint::<255>::new(7)?;
        let deploy = make_deploy(
            root,
            "run",
            initial.ledger_state.get_ref().clone(),
            &mut rng,
        )?;
        let recorded = contract::functional::run::Program::new(leaf, digest.clone(), selected)
            .execute_recorded(initial.into_circuit_context(deploy.address()))?;
        let native = contract::run(
            contract::initial_state(ConstructorContext::new(()))?
                .into_circuit_context(deploy.address()),
            leaf,
            digest.clone(),
            selected,
        )?;
        if recorded.execution.result != native.result
            || recorded.execution.context.query.state != native.context.query.state
            || recorded.execution.context.query.effects != native.context.query.effects
            || recorded.execution.gas_cost != native.gas_cost
            || !recorded.execution.private_transcript_outputs.is_empty()
        {
            return Err("Merkle composition native/recorded observations differ".into());
        }
        let expected_state = recorded.execution.context.query.state.get_ref().clone();
        let manual =
            check_generated_trace(root, "run", recorded, (leaf, digest.clone(), selected))?;
        let observed = pure_field_arguments::observed_state(&deploy);
        let prepared = contract::recorded::Contract
            .run_call(&observed, (), leaf, digest, selected)?
            .prepare(
                pure_field_arguments::verifier(root, "run")?,
                Fr::from(0_u64),
            )?;
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err("Merkle observed/direct preparation differs".into());
        }
        check_transaction(root, "run", deploy, manual, &mut rng, |state| {
            let plain = merkle_tree_view_at_path(state.data.get_ref(), &[0])?;
            let historic = historic_merkle_tree_view_at_path(state.data.get_ref(), &[1])?;
            if state.data.get_ref() != &expected_state
                || plain.first_free()?.value() != 1
                || plain.find_path_for_leaf(leaf).is_none()
                || historic.first_free()?.value() != u128::from(selected)
                || (selected && historic.find_path_for_leaf(leaf).is_none())
            {
                return Err("Merkle proof applied unexpected tree state".into());
            }
            Ok(())
        })?;
        println!("functional Merkle branch {} proved", selected);
    }
    println!(
        "plain/historic Merkle selected/skipped insertion branches proved and applied through ledger8"
    );
    Ok(())
}
