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
//! Original advance branches from explicit prior states with separate Dust fees.
use super::local_v3_application as v3;
use super::micro_dao_advance_support as support;
use super::positional_transaction::{check_generated_trace, check_transaction, make_deploy};
use super::*;
use compact_rust_test_center_micro_dao_fixture::{
    ledger_contract as c, ledger_slots as slots, types,
};

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    super::micro_dao_token::validate_artifacts(root, "advance")?;
    for (i, phase) in [
        types::LedgerState::commit,
        types::LedgerState::reveal,
        types::LedgerState::r#final,
    ]
    .into_iter()
    .enumerate()
    {
        let seeded = support::seeded(phase, 2, 3, 7, false, true)?;
        let private = seeded.private_state.clone();
        let mut rng = StdRng::seed_from_u64(0x0194_0000 + i as u64);
        let deploy = make_deploy(
            root,
            "advance",
            seeded.query.state.get_ref().clone(),
            &mut rng,
        )?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let native = c::advance(
            observed.circuit_context(private.clone()),
            &support::Witness::new(support::Mode::Normal),
        )?;
        let witness = support::Witness::new(support::Mode::Normal);
        let recorded = c::recorded::advance(observed.circuit_context(private.clone()), &witness)?;
        if native.gas_cost != recorded.execution.gas_cost
            || native.context.private_state != recorded.execution.context.private_state
            || native.private_transcript_outputs != recorded.execution.private_transcript_outputs
            || native.context.query.state != recorded.execution.context.query.state
            || native.context.query.effects != recorded.execution.context.query.effects
            || recorded.execution.private_transcript_outputs.len() != 1
            || *witness.calls.borrow() != ["secret"]
        {
            return Err("original advance native/recorded mismatch".into());
        }
        let expected = native.context.query.state.get_ref().clone();
        let manual = check_generated_trace(root, "advance", recorded, ())?;
        let verifier = v3::read_verifier(root, "advance")?;
        let generated = c::Contract::from(support::Witness::new(support::Mode::Normal));
        let prepared = generated
            .recording()
            .advance_call(&observed, private)?
            .prepare(verifier, Fr::from(0u64))?;
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err("advance typed/direct preparation differs".into());
        }
        check_transaction(root, "advance", deploy, prepared, &mut rng, |state| {
            if state.data.get_ref() != &expected
                || slots::pot.inspect(state.data.get_ref())? != support::pot()
                || !slots::pot_has_coin.inspect(state.data.get_ref())?
            {
                return Err("advance applied state/pot differs".into());
            }
            Ok(())
        })?;
        println!(
            "original microDAO.advance {phase:?} preserved the expected transition and pot from an explicit prior state"
        );
    }
    Ok(())
}
