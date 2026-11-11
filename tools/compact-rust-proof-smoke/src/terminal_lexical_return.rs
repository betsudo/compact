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
//! Extracted lexical results with strict ledger application and separate Dust.
use super::local_v3_application as v3;
use super::positional_transaction::{check_generated_trace, check_transaction, make_deploy};
use super::terminal_lexical_return_support as support;
use super::*;
use compact_rust_terminal_lexical_return_oracle_fixture::{
    ledger_contract as c, ledger_slots as slots,
};

fn validate_ir(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value =
        serde_json::from_str(compact_rust_terminal_lexical_return_oracle_fixture::RETAINED_IR)?;
    let actual: serde_json::Value = serde_json::from_slice(bytes)?;
    if expected != actual {
        return Err("terminal lexical return source/proof IR mismatch".into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_ir(&std::fs::read(root.join("contract/compact-rust-ir.json"))?)?;
    v3::validate_pinned_artifacts(
        root,
        include_str!("../../compact-rust-backend/tests/terminal-lexical-ledger9-artifacts.json"),
    )?;
    for (i, name) in ["two", "three", "nested", "observed"]
        .into_iter()
        .enumerate()
    {
        // Native echo calls seed the source's Cell. These are prior local state,
        // not a claim of proved or funded earlier ledger transactions.
        let seeded = support::seeded(4);
        let private = seeded.private_state.clone();
        let mut rng = StdRng::seed_from_u64(0x0202_0000 + i as u64);
        let deploy = make_deploy(root, name, seeded.query.state.get_ref().clone(), &mut rng)?;
        let address = deploy.address();
        let observed = ObservedContractState::new(
            address,
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let native_witness = support::Witnesses::new(4, 2);
        let ctx = observed.circuit_context(private.clone());
        let native = match name {
            "two" => c::two(ctx),
            "three" => c::three(ctx),
            "nested" => c::nested(ctx),
            "observed" => c::observed(ctx, &native_witness, false),
            _ => unreachable!(),
        }?;
        let witness = support::Witnesses::new(4, 2);
        let ctx = observed.circuit_context(private.clone());
        let recorded = match name {
            "two" => c::recorded::two(ctx),
            "three" => c::recorded::three(ctx),
            "nested" => c::recorded::nested(ctx),
            "observed" => c::recorded::observed(ctx, &witness, false),
            _ => unreachable!(),
        }?;
        let expected_result = Field::from(match name {
            "three" => 7u64,
            "observed" => 6,
            _ => 5,
        });
        if native.result != expected_result
            || recorded.execution.result != expected_result
            || native.gas_cost != recorded.execution.gas_cost
            || native.context.private_state != recorded.execution.context.private_state
            || native.private_transcript_outputs != recorded.execution.private_transcript_outputs
            || native.context.query.state != recorded.execution.context.query.state
            || native.context.query.effects != recorded.execution.context.query.effects
            || *native_witness.trace.borrow() != *witness.trace.borrow()
            || witness.trace.borrow().len() != usize::from(name == "observed")
        {
            return Err(format!("{name}: native/recorded lexical result or effects differ").into());
        }
        let expected = native.context.query.state.get_ref().clone();
        let input = if name == "observed" {
            AlignedValue::from(false)
        } else {
            AlignedValue::from(())
        };
        let manual = check_generated_trace(root, name, recorded, input)?;
        let verifier = v3::read_verifier(root, name)?;
        let generated = c::Contract::from(support::Witnesses::new(4, 2));
        let call = match name {
            "two" => generated.recording().two_call(&observed, private)?,
            "three" => generated.recording().three_call(&observed, private)?,
            "nested" => generated.recording().nested_call(&observed, private)?,
            "observed" => generated
                .recording()
                .observed_call(&observed, private, false)?,
            _ => unreachable!(),
        };
        let prepared = call.prepare(verifier, Fr::from(0u64))?;
        if format!("{manual:?}") != format!("{prepared:?}")
            || prepared.output != AlignedValue::from(expected_result)
        {
            return Err(
                format!("{name}: typed/direct preparation or returned output differs").into(),
            );
        }
        check_transaction(root, name, deploy, prepared, &mut rng, |actual| {
            if actual.data.get_ref() != &expected
                || slots::stored.inspect(actual.data.get_ref())? != expected_result
            {
                return Err(format!("{name}: applied state or Field value differs").into());
            }
            Ok(())
        })?;
        println!(
            "terminal {name}: exact returned Field and applied state checked from an explicit native-seeded prior state"
        );
    }
    Ok(())
}

#[cfg(test)]
mod source_pairing_tests {
    use super::*;
    #[test]
    fn rejects_mismatched_source_before_keys() {
        let source = compact_rust_terminal_lexical_return_oracle_fixture::RETAINED_IR;
        validate_ir(source.as_bytes()).unwrap();
        let mut altered: serde_json::Value = serde_json::from_str(source).unwrap();
        altered["stateful_circuits"][0]["name"] = "different".into();
        assert!(validate_ir(&serde_json::to_vec(&altered).unwrap()).is_err());
        assert!(
            validate_ir(compact_rust_root_let_action_return_oracle_fixture::RETAINED_IR.as_bytes())
                .is_err()
        );
    }
}
