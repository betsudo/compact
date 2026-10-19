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

#[derive(Default)]
struct LocalCallWitnesses(RefCell<Vec<(u64, &'static str, Field, Field)>>);
impl call_fixture::ledger_contract::Witnesses<u64> for LocalCallWitnesses {
    fn nextValue(
        &self,
        context: WitnessContext<'_, u64, call_fixture::ledger_contract::LedgerView<'_>>,
        seed: Field,
    ) -> (u64, Field) {
        let stored = context.ledger.stored().unwrap();
        self.0
            .borrow_mut()
            .push((*context.private_state, "next", seed, stored));
        (*context.private_state + 1, seed + Field::from(1_u64))
    }
    fn finish(
        &self,
        context: WitnessContext<'_, u64, call_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        let stored = context.ledger.stored().unwrap();
        assert_eq!(stored, value);
        self.0
            .borrow_mut()
            .push((*context.private_state, "finish", value, stored));
        (*context.private_state + 1, value + Field::from(100_u64))
    }
}

#[test]
fn local_calls_bind_fresh_formals_and_preserve_failure_prefixes() {
    use call_fixture::ledger_contract as c;
    for (selected, permit) in [(false, false), (true, true), (true, false)] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [LocalCallWitnesses; 4] =
            std::array::from_fn(|_| LocalCallWitnesses::default());
        let seed = Field::from(10_u64);
        let result = outcomes(
            c::run(context(), &witnesses[0], selected, seed, permit),
            c::recorded::run(context(), &witnesses[1], selected, seed, permit),
            c::functional::run::Program::new(selected, seed, permit)
                .execute_native(context(), &witnesses[2]),
            c::functional::run::Program::new(selected, seed, permit)
                .execute_recorded(context(), &witnesses[3]),
            (selected && !permit).then_some("failed assert: local call denied"),
        );
        let mut events = vec![(7, "next", Field::from(12_u64), Field::from(2_u64))];
        if selected {
            events.push((8, "next", Field::from(25_u64), Field::from(13_u64)));
        }
        if !selected || permit {
            events.push((
                if selected { 9 } else { 8 },
                "finish",
                Field::from(if selected { 26_u64 } else { 13_u64 }),
                Field::from(if selected { 26_u64 } else { 13_u64 }),
            ));
        }
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(
                result.result,
                Field::from(if selected { 152_u64 } else { 139_u64 })
            );
            assert_eq!(
                call_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                Field::from(if selected { 26_u64 } else { 13_u64 })
            );
            assert_eq!(result.context.private_state, if selected { 10 } else { 9 });
            let outputs = if selected {
                vec![13_u64, 26, 126]
            } else {
                vec![13_u64, 113]
            };
            assert_eq!(
                result.private_transcript_outputs,
                outputs
                    .into_iter()
                    .map(|value| runtime::fab::AlignedValue::from(Field::from(value)))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn certified_pure_fold_keeps_its_generated_stateful_caller() {
    use call_fixture::ledger_contract as c;
    for seed in [0_u64, 7] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let value = Field::from(seed);
        let result = outcomes(
            c::folded(context(), value),
            c::recorded::folded(context(), value),
            c::functional::folded::Program::new(value).execute_native(context()),
            c::functional::folded::Program::new(value).execute_recorded(context()),
            None,
        )
        .unwrap();
        assert_eq!(result.result, Field::from(seed + 5));
        assert_eq!(
            call_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            Field::from(seed + 5)
        );
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
    }
}
