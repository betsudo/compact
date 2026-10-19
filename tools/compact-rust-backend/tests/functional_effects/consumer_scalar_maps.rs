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
struct ScalarMapWitnesses(RefCell<Vec<(u64, Field, Field)>>);
impl map_fixture::ledger_contract::Witnesses<u64> for ScalarMapWitnesses {
    fn nextValue(
        &self,
        context: WitnessContext<'_, u64, map_fixture::ledger_contract::LedgerView<'_>>,
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
fn scalar_maps_report_post_remove_and_reset_observations_with_owned_values() {
    use map_fixture::ledger_contract as c;
    for value in [21_u64, 45] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let witnesses: [ScalarMapWitnesses; 4] =
            std::array::from_fn(|_| ScalarMapWitnesses::default());
        let input = Field::from(value);
        let owned = || map_fixture::types::Owned {
            text: runtime::OpaqueString::from("é\0map"),
            bytes: runtime::OpaqueBytes(vec![1, 0, 255]),
        };
        let result = outcomes(
            c::exercise(context(), &witnesses[0], input, owned()),
            c::recorded::exercise(context(), &witnesses[1], input, owned()),
            c::functional::exercise::Program::new(input, owned())
                .execute_native(context(), &witnesses[2]),
            c::functional::exercise::Program::new(input, owned())
                .execute_recorded(context(), &witnesses[3]),
            None,
        )
        .unwrap();
        assert_eq!(result.result.found, Field::from(value + 1));
        assert_eq!(result.result.beforeSize.value(), 2);
        assert!(!result.result.beforeEmpty);
        assert!(!result.result.removed); // member(true) after remove, not the remove return.
        assert_eq!(result.result.remainingSize.value(), 1);
        assert_eq!(result.result.defaultValue, Field::from(0_u64));
        assert!(result.result.resetEmpty);
        assert_eq!(result.result.resetSize.value(), 0);
        assert_eq!(result.result.owned, owned());
        assert_eq!(
            map_fixture::ledger_slots::stored
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
fn scalar_map_missing_keys_prior_assertions_and_inactive_branches_preserve_prefixes() {
    use map_fixture::ledger_contract as c;
    for (selected, permit, key) in [
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
        let expected_error = if selected && !permit {
            Some("failed assert: map denied".to_owned())
        } else if selected && !key {
            // Pin the independent official-query refusal class, then compare exact
            // current ordinary diagnostic text across all four generated routes.
            let reference = c::guarded(
                context(),
                &ScalarMapWitnesses::default(),
                selected,
                permit,
                key,
            )
            .err()
            .expect("missing Map key must reject the official query");
            assert!(matches!(reference, CompactError::LedgerQueryRejected(_)));
            Some(reference.to_string())
        } else {
            None
        };
        let witnesses: [ScalarMapWitnesses; 4] =
            std::array::from_fn(|_| ScalarMapWitnesses::default());
        let result = outcomes(
            c::guarded(context(), &witnesses[0], selected, permit, key),
            c::recorded::guarded(context(), &witnesses[1], selected, permit, key),
            c::functional::guarded::Program::new(selected, permit, key)
                .execute_native(context(), &witnesses[2]),
            c::functional::guarded::Program::new(selected, permit, key)
                .execute_recorded(context(), &witnesses[3]),
            expected_error.as_deref(),
        );
        let events = if selected && permit && key {
            vec![(7, Field::from(11_u64), Field::from(3_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(
                result.result,
                Field::from(if selected { 12_u64 } else { 9_u64 })
            );
            assert_eq!(result.context.private_state, if selected { 8 } else { 7 });
            let outputs = if selected {
                vec![runtime::fab::AlignedValue::from(Field::from(12_u64))]
            } else {
                vec![]
            };
            assert_eq!(result.private_transcript_outputs, outputs);
        }
    }
}
