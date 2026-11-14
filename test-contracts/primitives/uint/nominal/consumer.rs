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

use overflow::pure_circuits as overflow_pure;
use same::{pure_circuits as pure, runtime as rt};

fn u<const MAX: u128>(value: u128) -> rt::BoundedUint<MAX> {
    rt::BoundedUint::new(value).unwrap()
}

#[test]
fn same_range_original_inputs() {
    assert_eq!(pure::mul_bit(u(1), u(1)).unwrap().value(), 1);
    assert_eq!(pure::mul_bit(u(1), u(0)).unwrap().value(), 0);
    assert_eq!(pure::mul_bit(u(0), u(0)).unwrap().value(), 0);
    assert_eq!(pure::mul_bit_range(u(1), u(1)).unwrap().value(), 1);
    assert_eq!(pure::mul_bit_range(u(0), u(1)).unwrap().value(), 0);
    assert_eq!(pure::add_zero(u(0), u(0)).unwrap().value(), 0);
    assert_eq!(pure::mul_zero(u(0), u(0)).unwrap().value(), 0);
    assert_eq!(pure::sub_bit(u(1), u(1)).unwrap().value(), 0);
    assert_eq!(pure::sub_bit(u(1), u(0)).unwrap().value(), 1);
}

#[test]
fn subtract_negative_refused() {
    let err = pure::sub_bit(u(0), u(1)).unwrap_err();
    assert!(matches!(err, rt::CompactError::UnsignedUnderflow));
}

#[test]
fn addition_overflow_original_boundary() {
    assert_eq!(overflow_pure::add_byte(u(200), u(55)).unwrap().value(), 255);
    let err = overflow_pure::add_byte(u(255), u(1)).unwrap_err();
    assert!(matches!(err, rt::CompactError::UnsignedOutOfRange { value: 256, max: 255 }));
}
