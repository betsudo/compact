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

//! Prove and apply the original test-center Counter's ordered public/private effects.

use super::positional_transaction::{check_generated_trace, check_transaction, make_deploy};
use super::*;
use compact_rust_test_center_counter_fixture::ledger_contract as contract;
use midnight_base_crypto::hash::persistent_hash;

struct AdvancingWitness;

impl contract::Witnesses<u64> for AdvancingWitness {
    fn private_increment(
        &self,
        context: WitnessContext<'_, u64, contract::LedgerView<'_>>,
    ) -> (u64, ()) {
        let round = context.ledger.round().expect("Counter witness read");
        assert_eq!(round.value(), 1);
        (*context.private_state + 1, ())
    }
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    for (path, expected) in [
        (
            "contract/compact-rust-ir.json",
            "0a649c2ab82955c45e0d75640732246c839b75fb4c4a7c85b1d827c64197a4e6",
        ),
        (
            "contract/lib.rs",
            "796517c4f63d33370cb41a3ea60d8120387767b9cea454151a1ee73872eda627",
        ),
        (
            "zkir/increment.zkir",
            "56ab0643bca927981a73605c349d8f6c437019136007ad59dcf8e929a57b7923",
        ),
    ] {
        if hex::encode(persistent_hash(&std::fs::read(root.join(path))?).0) != expected {
            return Err(
                format!("test-center Counter source/proof artifact differs: {path}").into(),
            );
        }
    }
    let mut rng = StdRng::seed_from_u64(0x0107_434f_554e_5445);
    let initial = contract::initial_state(ConstructorContext::new(7_u64))?;
    let deploy = make_deploy(
        root,
        "increment",
        initial.ledger_state.get_ref().clone(),
        &mut rng,
    )?;
    let generated = contract::Contract::from(AdvancingWitness);
    let recorded = generated
        .recording()
        .increment(initial.into_circuit_context(deploy.address()))?;
    if recorded.execution.context.private_state != 8
        || recorded.execution.private_transcript_outputs.len() != 1
        || !recorded.execution.private_transcript_outputs[0]
            .value
            .0
            .is_empty()
        || recorded.public.verify_ops().len() != 3
    {
        return Err("test-center Counter recording changed its public/private effects".into());
    }
    let expected_state = recorded.execution.context.query.state.get_ref().clone();
    let manual = check_generated_trace(root, "increment", recorded, ())?;

    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/increment.verifier"),
    )?))?;
    let typed = generated.recording().increment_call(&observed, 7_u64)?;
    let prepared = typed.prepare(verifier, Fr::from(0_u64))?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("test-center Counter observed call differs from direct recording".into());
    }

    check_transaction(root, "increment", deploy, prepared, &mut rng, |state| {
        if state.data.get_ref() != &expected_state {
            return Err("test-center Counter proof changed unexpected public state".into());
        }
        let StateValue::Array(fields) = state.data.get_ref() else {
            return Err("test-center Counter state is not an array".into());
        };
        if read_counter(fields.get(0).ok_or("Counter missing")?)? != 1 {
            return Err("test-center Counter proof did not increment the ledger".into());
        }
        Ok(())
    })?;
    println!("test-center Counter Unit witness proof verified and applied through Ledger 9");
    Ok(())
}
