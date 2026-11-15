// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use to_boolean::runtime::{CompactError, Field};

#[test]
fn original_to_boolean() {
    assert!(to_boolean::pure_circuits::field_casts_to_boolean().unwrap());
}

#[test]
fn original_to_bytes() {
    assert_eq!(
        to_bytes::pure_circuits::field_casts_to_bytes()
            .unwrap()
            .into_array(),
        [1, 0, 0, 0],
    );
}

#[test]
fn original_to_enum() {
    use to_enum::types::Color;
    assert_eq!(
        to_enum::pure_circuits::field_casts_to_enum(Field::from(0u128)).unwrap(),
        Color::red
    );
    assert_eq!(
        to_enum::pure_circuits::field_casts_to_enum(Field::from(1u128)).unwrap(),
        Color::green
    );
    assert_eq!(
        to_enum::pure_circuits::field_casts_to_enum(Field::from(2u128)).unwrap(),
        Color::blue
    );
}

#[test]
fn original_to_enum_out_of_range() {
    assert!(matches!(
        to_enum_out_of_range::pure_circuits::field_casts_to_enum_out_of_range(Field::from(3u128)),
        Err(CompactError::InvalidUnsignedValue),
    ));
}

#[test]
fn original_to_uint() {
    for value in [0u128, 1, u64::MAX as u128] {
        assert_eq!(
            to_uint::pure_circuits::field_casts_to_uint(Field::from(value))
                .unwrap()
                .value(),
            value,
        );
    }
}

#[test]
fn original_to_uint_max_field() {
    let max_field = Field::from(0u128) - Field::from(1u128);
    assert!(matches!(
        to_uint_max_field::pure_circuits::field_casts_to_uint_max_field(max_field),
        Err(CompactError::InvalidUnsignedValue),
    ));
}

#[test]
fn original_to_uint_out_of_range() {
    assert!(matches!(
        to_uint_out_of_range::pure_circuits::field_casts_to_uint_out_of_range(Field::from(
            1u128 << 64
        )),
        Err(CompactError::InvalidUnsignedValue),
    ));
}

#[test]
fn checked_bytes4_boundaries() {
    for (value, expected) in [
        (0u128, [0, 0, 0, 0]),
        (0x01020304u128, [4, 3, 2, 1]),
        (0xffffffffu128, [255, 255, 255, 255]),
    ] {
        assert_eq!(
            boundary::pure_circuits::field_casts_bytes4_boundary(Field::from(value))
                .unwrap()
                .into_array(),
            expected,
        );
    }
    for value in [0x100000000u128, (1u128 << 40) + 0x01020304] {
        assert!(matches!(
            boundary::pure_circuits::field_casts_bytes4_boundary(Field::from(value)),
            Err(CompactError::InvalidUnsignedValue),
        ));
    }
}
