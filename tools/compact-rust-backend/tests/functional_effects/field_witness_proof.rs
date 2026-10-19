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

//! Private-overlay proof of a generated typed Field witness, using existing proof helpers.
use super::*;
use functional_field_contract::ledger_contract as contract;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
struct Echo(Rc<RefCell<Vec<(u64, Field)>>>);
impl contract::TryWitnesses<u64> for Echo {
    fn private_offset(
        &self,
        context: WitnessContext<'_, u64, contract::LedgerView<'_>>,
        value: Field,
    ) -> Result<(u64, Field), midnight_compact_runtime::CompactError> {
        self.0.borrow_mut().push((*context.private_state, value));
        Ok((*context.private_state + 1, value))
    }
}
pub(super) fn arguments(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), Box<dyn Error>> {
    let root = arguments
        .next()
        .ok_or("missing functional witness proof directory")?;
    if arguments.next().is_some() {
        return Err("unexpected functional witness proof arguments".into());
    }
    run(Path::new(&root))
}
fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    const NAME: &str = "apply_offset";
    let mut rng = StdRng::seed_from_u64(0x6680_0042);
    let initial = contract::initial_state(ConstructorContext::new(7_u64))?;
    let deploy = make_deploy(root, NAME, initial.ledger_state.get_ref().clone(), &mut rng)?;
    let value = Field::from(42_u64);
    let functional_witness = Echo::default();
    let recorded = contract::functional::apply_offset::Program::new(value).execute_recorded(
        initial.into_circuit_context(deploy.address()),
        &functional_witness,
    )?;
    if recorded.execution.result != value
        || recorded.execution.context.private_state != 8
        || recorded.execution.private_transcript_outputs != [AlignedValue::from(value)]
        || *functional_witness.0.borrow() != [(7, value)]
    {
        return Err(
            "functional Field witness value, private state, transcript or event changed".into(),
        );
    }
    let expected_state = recorded.execution.context.query.state.get_ref().clone();
    let manual = check_generated_trace(root, NAME, recorded, value)?;
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
        root.join("keys/apply_offset.verifier"),
    )?))?;
    let default_witness = Echo::default();
    let generated = contract::Contract::from(default_witness.clone());
    let prepared = generated
        .recording()
        .apply_offset_call(&observed, 7_u64, value)?
        .prepare(verifier, Fr::from(0_u64))?;
    if format!("{manual:?}") != format!("{prepared:?}")
        || *default_witness.0.borrow() != [(7, value)]
    {
        return Err("functional Field preparation differs from ordinary observed call".into());
    }
    // The functional-derived input, not the ordinary comparison, reaches the prover.
    check_transaction(root, NAME, deploy, manual, &mut rng, |state| {
        if state.data.get_ref() != &expected_state {
            return Err("functional Field witness proof changed expected ledger state".into());
        }
        let StateValue::Array(fields) = state.data.get_ref() else {
            return Err("functional Field witness ledger is not an array".into());
        };
        if read_cell::<Field, _>(fields.get(0).ok_or("functional Field Cell missing")?)?
            != Field::from(42_u64)
        {
            return Err("functional Field witness proof did not store Field 42".into());
        }
        Ok(())
    })?;
    println!("functional Field witness proved, verified and ledger-applied");
    Ok(())
}
