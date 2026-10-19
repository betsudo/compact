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

fn nested_child(base: u64) -> runtime::ledger::StateValue<runtime::ledger::DefaultDB> {
    use runtime::ledger::{DefaultDB, LedgerHashMap, StateValue, constructor_cell};
    StateValue::Map(
        LedgerHashMap::new()
            .insert(
                runtime::fab::AlignedValue::from(Field::from(7_u64)),
                constructor_cell::<_, DefaultDB>(Field::from(base)),
            )
            .insert(
                runtime::fab::AlignedValue::from(Field::from(8_u64)),
                constructor_cell::<_, DefaultDB>(Field::from(base + 1)),
            ),
    )
}
fn nested_seeded_context() -> runtime::context::CircuitContext<u64> {
    use runtime::ledger::{DefaultDB, LedgerHashMap, StateValue, constructor_cell, contract_state};
    let mut fields = (0..16)
        .map(|_| constructor_cell::<_, DefaultDB>(Field::from(0_u64)))
        .collect::<Vec<_>>();
    fields.push(constructor_cell::<_, DefaultDB>(Field::from(3_u64)));
    fields.push(StateValue::Map(
        LedgerHashMap::new()
            .insert(runtime::fab::AlignedValue::from(true), nested_child(10))
            .insert(runtime::fab::AlignedValue::from(false), nested_child(90)),
    ));
    runtime::context::ConstructorResult::new(ConstructorContext::new(7_u64), contract_state(fields))
        .into_circuit_context(ContractAddress::default())
}
fn nested_assert_surviving_child(context: &runtime::context::CircuitContext<u64>, removed: bool) {
    use runtime::ledger::StateValue;
    let mut state = context.query.state.get_ref();
    let path = nested_fixture::ledger_slots::outer.path();
    assert!(path.len() > 1, "fixture must retain full chunked Map path");
    for index in path {
        let StateValue::Array(fields) = state else {
            panic!("expected path array")
        };
        state = fields.get(*index as usize).unwrap();
    }
    let StateValue::Map(entries) = state else {
        panic!("expected outer Map")
    };
    assert!(
        entries
            .get(&runtime::fab::AlignedValue::from(removed))
            .is_none()
    );
    let expected = nested_child(if removed { 90 } else { 10 });
    assert_eq!(
        &*entries
            .get(&runtime::fab::AlignedValue::from(!removed))
            .unwrap(),
        &expected
    );
}
#[derive(Default)]
struct NestedWitnesses(RefCell<Vec<(u64, Field, Field)>>);
impl nested_fixture::ledger_contract::Witnesses<u64> for NestedWitnesses {
    fn mark(
        &self,
        context: WitnessContext<'_, u64, nested_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        self.0.borrow_mut().push((
            *context.private_state,
            value,
            context.ledger.stored().unwrap(),
        ));
        (*context.private_state + 1, value)
    }
}
#[test]
fn nested_map_structural_remove_preserves_the_entire_other_child() {
    use nested_fixture::ledger_contract as c;
    for key in [false, true] {
        let witnesses: [NestedWitnesses; 4] = std::array::from_fn(|_| NestedWitnesses::default());
        let result = outcomes(
            c::inspect(nested_seeded_context(), &witnesses[0], key, !key),
            c::recorded::inspect(nested_seeded_context(), &witnesses[1], key, !key),
            c::functional::inspect::Program::new(key, !key)
                .execute_native(nested_seeded_context(), &witnesses[2]),
            c::functional::inspect::Program::new(key, !key)
                .execute_recorded(nested_seeded_context(), &witnesses[3]),
            None,
        )
        .unwrap();
        assert_eq!(result.result.before.value(), 2);
        assert!(result.result.present);
        assert!(!result.result.removed);
        assert!(result.result.sibling);
        assert_eq!(result.result.after.value(), 1);
        assert!(!result.result.empty);
        nested_assert_surviving_child(&result.context, key);
        assert_eq!(result.context.private_state, 8);
        assert_eq!(
            result.private_transcript_outputs,
            vec![runtime::fab::AlignedValue::from(Field::from(17_u64))]
        );
        for witness in &witnesses {
            assert_eq!(
                *witness.0.borrow(),
                vec![(7, Field::from(17_u64), Field::from(3_u64))]
            );
        }
    }
}
#[test]
fn nested_map_guards_preserve_inactivity_and_exact_witness_prefixes() {
    use nested_fixture::ledger_contract as c;
    for (selected, permit, after) in [
        (false, false, false),
        (true, false, false),
        (true, true, false),
        (true, true, true),
    ] {
        let witnesses: [NestedWitnesses; 4] = std::array::from_fn(|_| NestedWitnesses::default());
        let error = if selected && !permit {
            Some("failed assert: nested denied")
        } else if selected && !after {
            Some("failed assert: nested after denied")
        } else {
            None
        };
        let result = outcomes(
            c::guarded(
                nested_seeded_context(),
                &witnesses[0],
                selected,
                permit,
                after,
                true,
            ),
            c::recorded::guarded(
                nested_seeded_context(),
                &witnesses[1],
                selected,
                permit,
                after,
                true,
            ),
            c::functional::guarded::Program::new(selected, permit, after, true)
                .execute_native(nested_seeded_context(), &witnesses[2]),
            c::functional::guarded::Program::new(selected, permit, after, true)
                .execute_recorded(nested_seeded_context(), &witnesses[3]),
            error,
        );
        let mut events = vec![];
        if selected && permit {
            events.push((7, Field::from(17_u64), Field::from(3_u64)));
            if after {
                events.push((8, Field::from(18_u64), Field::from(17_u64)));
            }
        }
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result, !selected);
            assert_eq!(result.context.private_state, if selected { 9 } else { 7 });
            if selected {
                nested_assert_surviving_child(&result.context, true);
            } else {
                assert_eq!(
                    result.context.query.state,
                    nested_seeded_context().query.state
                );
            }
            let outputs = if selected {
                vec![
                    runtime::fab::AlignedValue::from(Field::from(17_u64)),
                    runtime::fab::AlignedValue::from(Field::from(18_u64)),
                ]
            } else {
                vec![]
            };
            assert_eq!(result.private_transcript_outputs, outputs);
        }
    }
}
