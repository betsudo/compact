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
struct VectorMapWitnesses(RefCell<Vec<(u64, Field, Field)>>);
impl vector_map_fixture::ledger_contract::Witnesses<u64> for VectorMapWitnesses {
    fn marker(
        &self,
        context: WitnessContext<'_, u64, vector_map_fixture::ledger_contract::LedgerView<'_>>,
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
macro_rules! vector_map_case {
    ($entry:ident, [$($argument:expr),*], $error:expr) => {{
        use vector_map_fixture::ledger_contract as c;
        let context=||c::initial_state(ConstructorContext::new(7_u64)).unwrap().into_circuit_context(ContractAddress::default());
        let witnesses:[VectorMapWitnesses;4]=std::array::from_fn(|_|VectorMapWitnesses::default());
        let result=outcomes(c::$entry(context(),&witnesses[0],$($argument),*),c::recorded::$entry(context(),&witnesses[1],$($argument),*),
            c::functional::$entry::Program::new($($argument),*).execute_native(context(),&witnesses[2]),c::functional::$entry::Program::new($($argument),*).execute_recorded(context(),&witnesses[3]),$error);
        (result,witnesses)
    }};
}
#[test]
fn vector_map_owned_captures_conditionals_pure_calls_and_source_reuse() {
    use vector_map_fixture::ledger_contract as c;
    for selected in [false, true] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let values =
            runtime::FixedVector([Field::from(3_u64), Field::from(5_u64), Field::from(7_u64)]);
        let text = || runtime::OpaqueString::from("λ\0map");
        let bytes = || runtime::OpaqueBytes(vec![1, 0, 255]);
        let result = outcomes(
            c::owned(context(), selected, values.clone(), text(), bytes()),
            c::recorded::owned(context(), selected, values.clone(), text(), bytes()),
            c::functional::owned::Program::new(selected, values.clone(), text(), bytes())
                .execute_native(context()),
            c::functional::owned::Program::new(selected, values.clone(), text(), bytes())
                .execute_recorded(context()),
            None,
        )
        .unwrap();
        assert_eq!(result.result.original, values);
        assert_eq!(result.result.text, text());
        assert_eq!(result.result.mapped.0.len(), 3);
        for (element, source) in result.result.mapped.0.iter().zip([3_u64, 5, 7]) {
            assert_eq!(
                element.value,
                Field::from(source + if selected { 1 } else { 3 })
            );
            assert_eq!(element.text, text());
            assert_eq!(element.bytes, bytes());
        }
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
        assert_eq!(
            vector_map_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            Field::from(1_u64)
        );
    }
}
#[test]
fn vector_map_empty_input_does_not_run_the_fallible_body() {
    let values: runtime::FixedVector<runtime::BoundedUint<65535>, 0> = runtime::FixedVector([]);
    let (result, witnesses) = vector_map_case!(empty, [values.clone()], None);
    let result = result.unwrap();
    assert!(result.result.0.is_empty());
    assert_eq!(result.context.private_state, 8);
    assert_eq!(
        result.private_transcript_outputs,
        vec![runtime::fab::AlignedValue::from(Field::from(0_u64))]
    );
    for witness in &witnesses {
        assert_eq!(
            *witness.0.borrow(),
            vec![(7, Field::from(0_u64), Field::from(0_u64))]
        );
    }
}
#[test]
fn vector_map_single_input_preserves_boundary_and_failure() {
    for value in [0_u128, 255, 300] {
        let values = runtime::FixedVector([runtime::BoundedUint::<65535>::new(value).unwrap()]);
        let error = (value == 300).then_some("unsigned value 300 exceeds Compact maximum 255");
        let (result, witnesses) = vector_map_case!(single, [values.clone()], error);
        let events = if error.is_none() {
            vec![(7, Field::from(1_u64), Field::from(0_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result.0[0].value(), value);
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(1_u64))]
            );
        }
    }
}
#[test]
fn vector_map_middle_cast_failure_stops_later_marker_and_untaken_cast_is_safe() {
    for (selected, middle) in [(true, 2_u128), (true, 300), (false, 300)] {
        let values = runtime::FixedVector([
            runtime::BoundedUint::<65535>::new(1).unwrap(),
            runtime::BoundedUint::<65535>::new(middle).unwrap(),
            runtime::BoundedUint::<65535>::new(3).unwrap(),
        ]);
        let error =
            (selected && middle == 300).then_some("unsigned value 300 exceeds Compact maximum 255");
        let (result, witnesses) = vector_map_case!(checked, [selected, values.clone()], error);
        let events = if error.is_none() {
            vec![(7, Field::from(3_u64), Field::from(0_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result.original, values);
            let expected = if selected {
                [1_u128, middle, 3]
            } else {
                [7, 7, 7]
            };
            assert_eq!(result.result.mapped.0.map(|value| value.value()), expected);
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(3_u64))]
            );
        }
    }
}

#[test]
fn vector_map_failure_preserves_the_prior_witness_prefix_and_suppresses_the_later_marker() {
    for selected in [false, true] {
        let values = runtime::FixedVector([
            runtime::BoundedUint::<65535>::new(1).unwrap(),
            runtime::BoundedUint::<65535>::new(300).unwrap(),
            runtime::BoundedUint::<65535>::new(3).unwrap(),
        ]);
        let error = selected.then_some("unsigned value 300 exceeds Compact maximum 255");
        let (result, witnesses) = vector_map_case!(prefixed, [selected, values.clone()], error);
        let mut events = vec![(7, Field::from(99_u64), Field::from(0_u64))];
        if !selected {
            events.push((8, Field::from(3_u64), Field::from(99_u64)));
        }
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(result.result.original, values);
            assert_eq!(
                result.result.mapped.0.map(|value| value.value()),
                [7_u128; 3]
            );
            assert_eq!(result.context.private_state, 9);
            assert_eq!(
                result.private_transcript_outputs,
                vec![
                    runtime::fab::AlignedValue::from(Field::from(99_u64)),
                    runtime::fab::AlignedValue::from(Field::from(3_u64))
                ]
            );
            assert_eq!(
                vector_map_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                Field::from(3_u64)
            );
        }
    }
}
#[test]
fn vector_map_nested_maps_and_body_assertions_respect_the_enclosing_branch() {
    for (selected, permit) in [(false, false), (true, true), (true, false)] {
        let values = runtime::FixedVector([Field::from(2_u64), Field::from(4_u64)]);
        let error = (selected && !permit).then_some("failed assert: map body denied");
        let (result, witnesses) =
            vector_map_case!(nested, [selected, permit, values.clone()], error);
        let events = if error.is_none() {
            vec![(7, Field::from(4_u64), Field::from(0_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            let expected = if selected {
                [[3_u64, 4], [5, 6]]
            } else {
                [[7, 7], [7, 7]]
            };
            assert_eq!(
                result.result.0.map(|inner| inner.0),
                expected.map(|inner| inner.map(Field::from))
            );
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(4_u64))]
            );
            assert_eq!(
                vector_map_fixture::ledger_slots::stored
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                Field::from(4_u64)
            );
        }
    }
}
