// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use reordering::runtime::{BoundedUint, CompactError, Field, FixedBytes, FixedVector};
use reordering::runtime::{context::ConstructorContext, ledger::ContractAddress};

macro_rules! check_original {
    ($contract:ident, $circuit:ident, $expected:expr) => {{
        let expected = FixedVector::new($expected);
        let context = $contract::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default());
        let ordinary = $contract::ledger_contract::$circuit(context).unwrap();
        assert_eq!(ordinary.result, expected);
        assert_eq!(
            $contract::ledger_contract::PublicStateView::from(&ordinary)
                .map_result()
                .unwrap(),
            expected,
        );

        let context = $contract::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default());
        let recorded = $contract::ledger_contract::recorded::$circuit(context).unwrap();
        assert_eq!(recorded.execution.result, expected);
        assert_eq!(
            $contract::ledger_contract::PublicStateView::from(&recorded.execution)
                .map_result()
                .unwrap(),
            expected,
        );
    }};
}

#[test]
fn original_bytes_map_reordering() {
    check_original!(
        reordering,
        bytes_map_reordering,
        [
            FixedBytes::new([3, 2]),
            FixedBytes::new([5, 4]),
            FixedBytes::new([7, 6]),
        ]
    );
}

#[test]
fn original_bytes_map_slice_composition() {
    check_original!(
        slice_composition,
        bytes_map_slice_composition,
        [
            FixedBytes::new([20]),
            FixedBytes::new([40]),
            FixedBytes::new([60]),
        ]
    );
}

#[test]
fn original_bytes_map_to_bytes() {
    check_original!(
        to_bytes,
        bytes_map_to_bytes,
        [
            FixedBytes::new([0]),
            FixedBytes::new([1]),
            FixedBytes::new([2]),
            FixedBytes::new([3]),
            FixedBytes::new([4]),
            FixedBytes::new([5]),
            FixedBytes::new([6]),
            FixedBytes::new([7]),
        ]
    );
}

#[test]
fn original_bytes_map_transformation() {
    check_original!(
        transformation,
        bytes_map_transformation,
        [
            FixedBytes::new([15]),
            FixedBytes::new([25]),
            FixedBytes::new([35]),
            FixedBytes::new([45]),
            FixedBytes::new([55]),
        ]
    );
}

#[test]
fn field_to_uint_downcast_accepts_boundary_and_refuses_overflow() {
    assert_eq!(
        field_to_uint::pure_circuits::field_to_byte(Field::from(255u128)).unwrap(),
        BoundedUint::<255>::new(255).unwrap(),
    );
    assert_eq!(
        field_to_uint::pure_circuits::field_to_byte(Field::from(256u128)),
        Err(CompactError::UnsignedOutOfRange {
            value: 256,
            max: 255,
        }),
    );
}

#[test]
fn large_bytes_index_reads_only_the_selected_byte() {
    let mut bytes = [0u8; 4096];
    bytes[4095] = 17;
    assert_eq!(
        large_index::pure_circuits::read_last(FixedBytes::new(bytes)).unwrap(),
        BoundedUint::<255>::new(17).unwrap(),
    );
}
