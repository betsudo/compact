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

#[test]
fn original_cast_to_bytes() {
    assert_eq!(
        to_bytes::pure_circuits::uint_casts_to_bytes()
            .unwrap()
            .into_array(),
        [2, 1, 0, 0],
    );
}

#[test]
fn original_cast_to_uint_different_width() {
    assert_eq!(
        to_uint_different_width::pure_circuits::uint_casts_to_uint_different_width()
            .unwrap()
            .value(),
        1,
    );
}

#[test]
fn original_range_addition() {
    assert_eq!(
        addition::pure_circuits::uint_ranges_addition()
            .unwrap()
            .value(),
        8,
    );
}

#[test]
fn original_range_multiplication() {
    assert_eq!(
        multiplication::pure_circuits::uint_ranges_multiplication()
            .unwrap()
            .value(),
        15,
    );
}
