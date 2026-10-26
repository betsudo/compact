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

//! Concrete generated public APIs for the finite selected Fold proof family.
use super::*;
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct Witness(Rc<RefCell<Vec<(u64, Field, Field)>>>);
impl compact_rust_direct_fold_witness_reducer_source_fixture::ledger_contract::TryWitnesses<u64>
    for Witness
{
    fn step(
        &self,
        context: WitnessContext<
            '_,
            u64,
            compact_rust_direct_fold_witness_reducer_source_fixture::ledger_contract::LedgerView<
                '_,
            >,
        >,
        acc: Field,
        item: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        self.0
            .borrow_mut()
            .push((*context.private_state, acc, item));
        Ok((*context.private_state + 1, acc * Field::from(10_u64) + item))
    }
}
fn field(value: u64) -> Field {
    Field::from(value)
}
fn vector<const N: usize>(values: [u64; N]) -> FixedVector<Field, N> {
    FixedVector::new(values.map(Field::from))
}
macro_rules! witness {
    (yes, $all:ident, $index:expr) => {
        &$all[$index]
    };
}
macro_rules! scenario {
    ($function:ident, $fixture:ident, [$($parameter:ident:$type:ty),*], [$($arg:expr),*],
        [$($marker:ident)?], $expected:expr, $private:expr) => {
        pub(super) fn $function(root: &Path, index: usize, $($parameter:$type,)*) -> Result<serde_json::Value, Box<dyn Error>> {
            use $fixture::{ledger_contract as c, ledger_slots as slots};
            let mut rng = StdRng::seed_from_u64(0x0661_0000 + index as u64);
            let initial = c::initial_state(ConstructorContext::new(7_u64))?;
            let deploy = make_deploy(root, NAME, initial.ledger_state.get_ref().clone(), &mut rng)?;
            let observed = ObservedContractState::new(deploy.address(), deploy.initial_state.clone(),
                Observation{transaction_hash:[0;32],block_hash:[0;32],block_height:0});
            let witnesses: [Witness;5] = std::array::from_fn(|_| Witness::default());
            let native = c::run(observed.circuit_context(7), $(witness!($marker, witnesses, 0),)? $($arg,)*)?;
            let recorded = c::recorded::run(observed.circuit_context(7), $(witness!($marker, witnesses, 1),)? $($arg,)*)?;
            let functional_native = c::functional::run::Program::new($($arg),*)
                .execute_native(observed.circuit_context(7) $(,witness!($marker, witnesses, 2))?)?;
            let functional = c::functional::run::Program::new($($arg),*)
                .execute_recorded(observed.circuit_context(7) $(,witness!($marker, witnesses, 3))?)?;
            let expected = field($expected);
            let expected_private: u64 = $private;
            let outputs: Vec<AlignedValue> = if expected_private == 9 {
                vec![AlignedValue::from(field(12)), AlignedValue::from(field(123))]
            } else { vec![] };
            for actual in [&native, &recorded.execution, &functional_native, &functional.execution] {
                if actual.result != expected || actual.context.private_state != expected_private
                    || actual.context.query.state != native.context.query.state
                    || actual.context.query.effects != native.context.query.effects
                    || actual.gas_cost != native.gas_cost || actual.private_transcript_outputs != outputs {
                    return Err("Fold native/recorded/functional result/state/effects/private/gas mismatch".into());
                }
            }
            if recorded.public.verify_ops() != functional.public.verify_ops()
                || slots::stored.inspect(native.context.query.state.get_ref())? != expected {
                return Err("Fold recording or stored Cell differs".into());
            }
            let expected_state = native.context.query.state.get_ref().clone();
            let input = AlignedValue::concat(&[$(AlignedValue::from($arg)),*]);
            let prepared = check_generated_trace(root, NAME, functional, input)?;
            let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(root.join("keys/run.verifier"))?))?;
            let facade = c::Contract::from(witnesses[4].clone());
            let ordinary = facade.recording().run_call(&observed, 7, $($arg,)*)?
                .prepare(verifier.clone(), Fr::from(0_u64))?;
            if format!("{prepared:?}") != format!("{ordinary:?}") {
                return Err("functional Fold preparation differs from ordinary observed call".into());
            }
            let journal = if expected_private == 9 {
                vec![(7,field(1),field(2)),(8,field(12),field(3))]
            } else { vec![] };
            for witness in witnesses { if *witness.0.borrow() != journal {
                return Err("Fold witness count/order changed".into());
            }}
            let proof_bytes = prove(root, index, &prepared, &verifier)?;
            apply(root, index, prepared, deploy, &expected_state, rng)?;
            Ok(serde_json::json!({"variant":index,"expected":$expected,"private_state":$private,
                "witness_calls":journal.len(),"proof_bytes":proof_bytes,"changed_binding_rejected":true,
                "changed_result_rejected":true,"changed_result_same_geometry":true,"strict_application":"default",
                "replay_refusal":"IntentAlreadyExists","replay_unchanged":true}))
        }
    };
}
scenario!(
    captured,
    compact_rust_direct_fold_captured_source_fixture,
    [],
    [field(1), field(10), vector([2, 3])],
    [],
    123,
    7
);
scenario!(
    multiple,
    compact_rust_direct_fold_multiple_inputs_source_fixture,
    [],
    [field(1), vector([2, 3]), vector([4, 5])],
    [],
    24,
    7
);
scenario!(
    stateful,
    compact_rust_direct_fold_effectful_reducer_source_fixture,
    [],
    [field(1), vector([2, 3])],
    [],
    123,
    7
);
scenario!(
    witnessed,
    compact_rust_direct_fold_witness_reducer_source_fixture,
    [],
    [field(1), vector([2, 3])],
    [yes],
    123,
    9
);
scenario!(
    inline,
    compact_rust_direct_fold_inline_effectful_source_fixture,
    [],
    [field(1), field(10), vector([2, 3])],
    [],
    123,
    7
);
scenario!(branch, compact_rust_direct_fold_inline_branch_source_fixture, [choice:bool], [field(1),field(10),choice,vector([2,3])], [], if choice {123} else {134}, 7);

scenario!(
    single,
    compact_rust_direct_fold_single_source_fixture,
    [],
    [field(1), field(10), vector([2])],
    [],
    12,
    7
);
