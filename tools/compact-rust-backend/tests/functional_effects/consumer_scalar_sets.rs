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
struct ScalarSetWitnesses(RefCell<Vec<(u64, Field, Field)>>);
impl set_fixture::ledger_contract::Witnesses<u64> for ScalarSetWitnesses {
    fn nextValue(
        &self,
        context: WitnessContext<'_, u64, set_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        self.0.borrow_mut().push((
            *context.private_state,
            value,
            context.ledger.stored().unwrap(),
        ));
        (*context.private_state + 1, value + Field::from(1_u64))
    }
}
#[test]
fn scalar_sets_check_real_absence_reset_and_owned_value_uniqueness() {
    use set_fixture::ledger_contract as c;
    for value in [21_u64, 45] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [ScalarSetWitnesses; 4] =
            std::array::from_fn(|_| ScalarSetWitnesses::default());
        let input = Field::from(value);
        let label = || runtime::OpaqueString::from("é\0set");
        let result = outcomes(
            c::exercise(context(), &witnesses[0], input, label()),
            c::recorded::exercise(context(), &witnesses[1], input, label()),
            c::functional::exercise::Program::new(input, label())
                .execute_native(context(), &witnesses[2]),
            c::functional::exercise::Program::new(input, label())
                .execute_recorded(context(), &witnesses[3]),
            None,
        )
        .unwrap();
        assert!(result.result.present);
        assert_eq!(result.result.size.value(), 2);
        assert!(!result.result.afterRemove);
        assert_eq!(result.result.remainingSize.value(), 1);
        assert!(!result.result.beforeResetEmpty);
        assert!(result.result.resetEmpty);
        assert_eq!(result.result.resetSize.value(), 0);
        assert!(result.result.ownedPresent);
        assert_eq!(result.result.ownedSize.value(), 1);
        assert!(!result.result.ownedAfterRemove);
        assert_eq!(result.result.ownedFinalSize.value(), 0);
        assert_eq!(
            set_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            Field::from(value + 1)
        );
        assert_eq!(result.context.private_state, 8);
        assert_eq!(
            result.private_transcript_outputs,
            vec![runtime::fab::AlignedValue::from(Field::from(value + 1))]
        );
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), vec![(7, input, Field::from(3_u64))]);
        }
    }
}
#[test]
fn scalar_set_guards_keep_exact_first_failure_and_witness_prefixes() {
    use set_fixture::ledger_contract as c;
    for (selected, permit, after) in [
        (false, false, false),
        (true, false, false),
        (true, true, false),
        (true, true, true),
    ] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [ScalarSetWitnesses; 4] =
            std::array::from_fn(|_| ScalarSetWitnesses::default());
        let expected = if selected && !permit {
            Some("failed assert: set denied")
        } else if selected && !after {
            Some("failed assert: set after denied")
        } else {
            None
        };
        let result = outcomes(
            c::guarded(context(), &witnesses[0], selected, permit, after),
            c::recorded::guarded(context(), &witnesses[1], selected, permit, after),
            c::functional::guarded::Program::new(selected, permit, after)
                .execute_native(context(), &witnesses[2]),
            c::functional::guarded::Program::new(selected, permit, after)
                .execute_recorded(context(), &witnesses[3]),
            expected,
        );
        let mut events = vec![];
        if selected && permit {
            events.push((7, Field::from(11_u64), Field::from(3_u64)));
            if after {
                events.push((8, Field::from(12_u64), Field::from(12_u64)));
            }
        }
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(
                result.result,
                Field::from(if selected { 13_u64 } else { 9_u64 })
            );
            assert_eq!(
                set_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                result.result
            );
            assert_eq!(result.context.private_state, if selected { 9 } else { 7 });
            let outputs = if selected {
                vec![
                    runtime::fab::AlignedValue::from(Field::from(12_u64)),
                    runtime::fab::AlignedValue::from(Field::from(13_u64)),
                ]
            } else {
                vec![]
            };
            assert_eq!(result.private_transcript_outputs, outputs);
        }
    }
}
