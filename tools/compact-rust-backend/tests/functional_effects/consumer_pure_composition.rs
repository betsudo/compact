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

// Included alongside consumer.rs so its original four-route oracle stays shared.
#[derive(Default)]
struct CompositionWitnesses(RefCell<Vec<(u64, Field)>>);
impl composition_fixture::ledger_contract::Witnesses<u64> for CompositionWitnesses {
    fn marker(
        &self,
        context: WitnessContext<'_, u64, composition_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        self.0.borrow_mut().push((*context.private_state, value));
        (*context.private_state + 1, value)
    }
}

#[test]
fn composition_constructs_and_projects_owned_products_without_losing_values() {
    use composition_fixture::ledger_contract as c;
    for seed in [1_u64, 7] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let text = || runtime::OpaqueString::from("é\0retained");
        let bytes = || runtime::OpaqueBytes(vec![1, 0, 255]);
        let seed_field = Field::from(seed);
        let result = outcomes(
            c::compose(context(), seed_field, text(), bytes()),
            c::recorded::compose(context(), seed_field, text(), bytes()),
            c::functional::compose::Program::new(seed_field, text(), bytes())
                .execute_native(context()),
            c::functional::compose::Program::new(seed_field, text(), bytes())
                .execute_recorded(context()),
            None,
        )
        .unwrap();
        let expected = Field::from((seed + 3) * (seed - 1));
        assert_eq!(result.result.total, expected);
        assert_eq!(result.result.text, text());
        assert_eq!(result.result.bytes, bytes());
        assert_eq!(result.result.values.0.len(), 2);
        for owned in &result.result.values.0 {
            assert_eq!(owned.value, seed_field);
            assert_eq!(owned.text, text());
            assert_eq!(owned.bytes, bytes());
        }
        assert_eq!(
            composition_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            expected
        );
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
    }
}

#[test]
fn composition_unsigned_failures_and_inactive_guards_suppress_later_callbacks() {
    use composition_fixture::ledger_contract as c;
    for (left, right, fails) in [
        (10_u128, 3_u128, false),
        (255, 1, false),
        (1, 2, true),
        (20, 20, true),
    ] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [CompositionWitnesses; 4] =
            std::array::from_fn(|_| CompositionWitnesses::default());
        let a = runtime::BoundedUint::<255>::new(left).unwrap();
        let b = runtime::BoundedUint::<255>::new(right).unwrap();
        let result = outcomes(
            c::checked(context(), &witnesses[0], a, b),
            c::recorded::checked(context(), &witnesses[1], a, b),
            c::functional::checked::Program::new(a, b).execute_native(context(), &witnesses[2]),
            c::functional::checked::Program::new(a, b).execute_recorded(context(), &witnesses[3]),
            if left < right {
                Some("Compact unsigned arithmetic underflow")
            } else if fails {
                Some("unsigned value 400 exceeds Compact maximum 255")
            } else {
                None
            },
        );
        let events = if fails {
            vec![]
        } else {
            vec![(7, Field::from((left * right) as u64))]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result.difference.value(), left - right);
            assert_eq!(result.result.product.value(), left * right);
            assert_eq!(result.result.narrowed.value(), left * right);
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                composition_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                Field::from((left * right) as u64)
            );
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(
                    (left * right) as u64
                ))]
            );
        }
    }
    for (selected, left, right, expected) in [
        (false, 1_u128, 2_u128, Some(9_u64)),
        (true, 1, 2, None),
        (true, 20, 20, None),
        (true, 10, 3, Some(37)),
    ] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [CompositionWitnesses; 4] =
            std::array::from_fn(|_| CompositionWitnesses::default());
        let a = runtime::BoundedUint::<255>::new(left).unwrap();
        let b = runtime::BoundedUint::<255>::new(right).unwrap();
        let result = outcomes(
            c::guarded(context(), &witnesses[0], selected, a, b),
            c::recorded::guarded(context(), &witnesses[1], selected, a, b),
            c::functional::guarded::Program::new(selected, a, b)
                .execute_native(context(), &witnesses[2]),
            c::functional::guarded::Program::new(selected, a, b)
                .execute_recorded(context(), &witnesses[3]),
            if selected && left < right {
                Some("Compact unsigned arithmetic underflow")
            } else if expected.is_none() {
                Some("unsigned value 400 exceeds Compact maximum 255")
            } else {
                None
            },
        );
        let events = if selected && expected.is_some() {
            vec![(7, Field::from(37_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result, Field::from(expected.unwrap()));
            assert_eq!(
                composition_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                result.result
            );
            assert_eq!(result.context.private_state, if selected { 8 } else { 7 });
            let private_outputs = if selected {
                vec![runtime::fab::AlignedValue::from(Field::from(37_u64))]
            } else {
                vec![]
            };
            assert_eq!(result.private_transcript_outputs, private_outputs);
        }
    }
}
