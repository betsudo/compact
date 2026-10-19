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
struct CryptoWitnesses(RefCell<Vec<(u64, Field)>>);
impl crypto_fixture::ledger_contract::Witnesses<u64> for CryptoWitnesses {
    fn marker(
        &self,
        context: WitnessContext<'_, u64, crypto_fixture::ledger_contract::LedgerView<'_>>,
        value: Field,
    ) -> (u64, Field) {
        self.0.borrow_mut().push((*context.private_state, value));
        (*context.private_state + 1, value)
    }
}
macro_rules! crypto_case{
    ($entry:ident,[$($argument:expr),*],$error:expr)=>{{
        use crypto_fixture::ledger_contract as c;
        let context=||c::initial_state(ConstructorContext::new(7_u64)).unwrap().into_circuit_context(ContractAddress::default());
        let witnesses:[CryptoWitnesses;4]=std::array::from_fn(|_|CryptoWitnesses::default());
        let result=outcomes(c::$entry(context(),&witnesses[0],$($argument),*),c::recorded::$entry(context(),&witnesses[1],$($argument),*),
            c::functional::$entry::Program::new($($argument),*).execute_native(context(),&witnesses[2]),c::functional::$entry::Program::new($($argument),*).execute_recorded(context(),&witnesses[3]),$error);
        (result,witnesses)
    }};
}
#[test]
fn crypto_hashing_matches_official_primitives_for_every_intermediate() {
    for seed in [0_u64, 7] {
        let input = Field::from(seed);
        let opening = runtime::FixedBytes::new([3_u8; 32]);
        let (result, witnesses) = crypto_case!(hashing, [input, opening], None);
        let result = result.unwrap();
        let encoded = runtime::FixedBytes::<32>::new(
            input
                .as_le_bytes()
                .try_into()
                .expect("Field encoding has 32 bytes"),
        );
        let digest = runtime::persistent_hash(encoded);
        let committed = runtime::persistent_commit(encoded, opening);
        let degraded = runtime::degrade_to_transient(digest);
        let hashed = runtime::transient_hash(degraded);
        let transient = runtime::transient_commit(hashed, input);
        assert_eq!(result.result.encoded, encoded);
        assert_eq!(result.result.digest, digest);
        assert_eq!(result.result.commitment, committed);
        assert_eq!(result.result.degraded, degraded);
        assert_eq!(result.result.hashed, hashed);
        assert_eq!(result.result.transient, transient);
        assert_eq!(
            result.result.upgraded,
            runtime::upgrade_from_transient(transient)
        );
        assert_eq!(
            crypto_fixture::ledger_slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            input
        );
        assert_eq!(result.context.private_state, 8);
        assert_eq!(
            result.private_transcript_outputs,
            vec![runtime::fab::AlignedValue::from(input)]
        );
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), vec![(7, input)]);
        }
    }
}
#[test]
fn crypto_jubjub_matches_official_coordinates_reduction_and_group_operations() {
    let point = runtime::ec_mul_generator(Field::from(5_u64)).unwrap();
    for seed in [0_u64, 7] {
        let input = Field::from(seed);
        let (result, witnesses) = crypto_case!(jubjub, [input, point], None);
        let result = result.unwrap();
        let x = runtime::jubjub_point_x(point);
        let y = runtime::jubjub_point_y(point);
        let scalar = runtime::jubjub_scalar_from_native(input);
        let generator = runtime::ec_mul_generator(scalar).unwrap();
        let multiplied = runtime::ec_mul(point, scalar).unwrap();
        let added = runtime::ec_add(generator, multiplied);
        let negated = runtime::ec_neg(added);
        assert_eq!(result.result.curve, runtime::hash_to_curve(input));
        assert_eq!(result.result.x, x);
        assert_eq!(result.result.y, y);
        assert_eq!(
            result.result.reconstructed,
            runtime::construct_jubjub_point(x, y).unwrap()
        );
        assert_eq!(result.result.scalar, scalar);
        assert_eq!(result.result.generator, generator);
        assert_eq!(result.result.multiplied, multiplied);
        assert_eq!(result.result.added, added);
        assert_eq!(result.result.negated, negated);
        assert_eq!(
            crypto_fixture::ledger_slots::storedPoint
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            negated
        );
        assert_eq!(result.context.private_state, 8);
        assert_eq!(
            result.private_transcript_outputs,
            vec![runtime::fab::AlignedValue::from(input)]
        );
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), vec![(7, input)]);
        }
    }
}
#[test]
fn crypto_opaque_hash_inputs_preserve_empty_interior_and_trailing_zero_bytes() {
    use crypto_fixture::ledger_contract as c;
    for text in ["", "a\0b", "é🦀\0"] {
        let context = || {
            c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        let value = || runtime::OpaqueString::from(text);
        let result = outcomes(
            c::opaque(context(), value()),
            c::recorded::opaque(context(), value()),
            c::functional::opaque::Program::new(value()).execute_native(context()),
            c::functional::opaque::Program::new(value()).execute_recorded(context()),
            None,
        )
        .unwrap();
        assert_eq!(
            result.result.curve,
            runtime::hash_to_curve(runtime::exact_hash_input(value()))
        );
        assert_eq!(
            result.result.hashed,
            runtime::transient_hash(runtime::exact_hash_input(value()))
        );
        assert_eq!(
            result.result.committed,
            runtime::transient_commit(runtime::exact_hash_input(value()), Field::from(7_u64))
        );
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
    }
}
#[test]
fn crypto_noncanonical_scalar_failure_and_untaken_branch_preserve_callback_order() {
    let point = runtime::ec_mul_generator(Field::from(5_u64)).unwrap();
    for (selected, valid) in [(true, true), (true, false), (false, false)] {
        let scalar = if valid {
            Field::from(7_u64)
        } else {
            -Field::from(1_u64)
        };
        let error = (selected && !valid).then_some("invalid Jubjub scalar");
        let (result, witnesses) = crypto_case!(raw, [selected, point, scalar], error);
        let events = if error.is_none() {
            vec![(7, Field::from(9_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            let expected = if selected {
                runtime::ec_mul(point, scalar).unwrap()
            } else {
                point
            };
            assert_eq!(result.result, expected);
            assert_eq!(
                crypto_fixture::ledger_slots::storedPoint
                    .inspect(result.context.query.state.get_ref())
                    .unwrap(),
                expected
            );
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(9_u64))]
            );
        }
    }
}
#[test]
fn crypto_invalid_point_construction_refuses_before_later_callback() {
    for valid in [false, true] {
        // (0, 0) is an accepted Compact identity sentinel; (1, 0) is invalid.
        let x = Field::from(if valid { 0_u64 } else { 1_u64 });
        let y = Field::from(if valid { 1_u64 } else { 0_u64 });
        let error = (!valid).then_some("invalid Jubjub point");
        let (result, witnesses) = crypto_case!(construct, [x, y], error);
        let events = if valid {
            vec![(7, Field::from(8_u64))]
        } else {
            vec![]
        };
        for witness in &witnesses {
            assert_eq!(*witness.0.borrow(), events);
        }
        if let Some(result) = result {
            assert_eq!(
                result.result,
                runtime::ec_mul_generator(Field::from(0_u64)).unwrap()
            );
            assert_eq!(result.context.private_state, 8);
            assert_eq!(
                result.private_transcript_outputs,
                vec![runtime::fab::AlignedValue::from(Field::from(8_u64))]
            );
        }
    }
}
