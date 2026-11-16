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

#[test]
fn binary_vector_map_keeps_order_and_original_assertions() {
    let result = semantics::pure_circuits::vector_map_semantics().unwrap();
    assert_eq!(
        result.0,
        [
            semantics::runtime::Field::from(11u128),
            semantics::runtime::Field::from(22u128),
            semantics::runtime::Field::from(33u128),
        ],
    );
}

#[test]
fn binary_vector_map_empty_tuple_stays_empty() {
    let result = empty_tuple::pure_circuits::vector_map_empty_tuple().unwrap();
    assert_eq!(result.0.len(), 0);
}

#[test]
fn binary_bytes_operands_and_widened_formals_match_compact() {
    use regressions::runtime::{BoundedUint, FixedBytes, FixedVector};

    let left = FixedBytes::new([1, 2]);
    let right = FixedBytes::new([10, 20]);
    assert_eq!(
        regressions::pure_circuits::bytes_map(left.clone(), right.clone()).unwrap(),
        FixedVector::new([
            BoundedUint::<65535>::new(12).unwrap(),
            BoundedUint::<65535>::new(24).unwrap(),
        ]),
    );
    assert_eq!(
        regressions::pure_circuits::widened_bytes_map(left, right).unwrap(),
        FixedVector::new([
            BoundedUint::<511>::new(1).unwrap(),
            BoundedUint::<511>::new(2).unwrap(),
        ]),
    );
    let a = FixedVector::new([
        BoundedUint::<255>::new(1).unwrap(),
        BoundedUint::<255>::new(2).unwrap(),
    ]);
    let b = FixedVector::new([
        BoundedUint::<255>::new(10).unwrap(),
        BoundedUint::<255>::new(20).unwrap(),
    ]);
    assert_eq!(
        regressions::pure_circuits::widened_vector_map(a, b).unwrap(),
        FixedVector::new([
            BoundedUint::<65535>::new(1).unwrap(),
            BoundedUint::<65535>::new(2).unwrap(),
        ]),
    );
}
