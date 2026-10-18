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
struct ScalarListWitnesses(RefCell<Vec<(u64, Field, Field)>>);
impl list_fixture::ledger_contract::Witnesses<u64> for ScalarListWitnesses {
    fn nextValue(
        &self,
        context: WitnessContext<'_, u64, list_fixture::ledger_contract::LedgerView<'_>>,
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
fn scalar_list_heads_prove_each_pop_before_reset_and_preserve_owned_values() {
    use list_fixture::ledger_contract as c;
    assert!(list_fixture::ledger_slots::items.path().len() > 1);
    assert!(list_fixture::ledger_slots::labels.path().len() > 1);
    for value in [21_u64, 45] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [ScalarListWitnesses; 4] =
            std::array::from_fn(|_| ScalarListWitnesses::default());
        let input = Field::from(value);
        let label = || runtime::OpaqueString::from("λ\0list");
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
        // Use the generated Maybe's actual is_some/value members; no Option substitute.
        assert!(!result.result.initialHead.is_some);
        assert_eq!(result.result.initialHead.value, Field::from(0_u64));
        assert!(result.result.initialEmpty);
        assert!(result.result.twoHead.is_some);
        assert_eq!(result.result.twoHead.value, Field::from(value + 2));
        assert_eq!(result.result.twoLength.value(), 2);
        assert!(result.result.oneHead.is_some);
        assert_eq!(result.result.oneHead.value, Field::from(value + 1));
        assert_eq!(result.result.oneLength.value(), 1);
        assert!(!result.result.poppedHead.is_some);
        assert_eq!(result.result.poppedHead.value, Field::from(0_u64));
        assert_eq!(result.result.poppedLength.value(), 0);
        assert!(result.result.poppedEmpty);
        assert!(!result.result.resetHead.is_some);
        assert_eq!(result.result.resetHead.value, Field::from(0_u64));
        assert!(result.result.resetEmpty);
        assert!(result.result.ownedHead.is_some);
        assert_eq!(result.result.ownedHead.value, label());
        assert!(result.result.ownedEmpty);
        assert_eq!(
            list_fixture::ledger_slots::stored
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
fn scalar_list_guards_keep_exact_first_failure_and_witness_prefixes() {
    use list_fixture::ledger_contract as c;
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
        let witnesses: [ScalarListWitnesses; 4] =
            std::array::from_fn(|_| ScalarListWitnesses::default());
        let error = if selected && !permit {
            Some("failed assert: list denied")
        } else if selected && !after {
            Some("failed assert: list after denied")
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
            error,
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
#[test]
fn scalar_list_empty_pop_keeps_the_existing_ordinary_policy() {
    use list_fixture::ledger_contract as c;
    let context = || {
        c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let reference = c::emptyPop(context()).expect("official Ledger8 empty pop succeeds");
    let result = outcomes(
        c::emptyPop(context()),
        c::recorded::emptyPop(context()),
        c::functional::emptyPop::Program::new().execute_native(context()),
        c::functional::emptyPop::Program::new().execute_recorded(context()),
        None,
    );
    if let Some(result) = result {
        assert_eq!(result.context.query.state, reference.context.query.state);
        let mut state = result.context.query.state.get_ref();
        for index in list_fixture::ledger_slots::items.path() {
            let runtime::ledger::StateValue::Array(fields) = state else {
                panic!("expected list physical-path array");
            };
            state = fields.get(*index as usize).unwrap();
        }
        assert!(matches!(state, runtime::ledger::StateValue::Null));
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
    }
}
