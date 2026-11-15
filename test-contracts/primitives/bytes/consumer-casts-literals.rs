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

use to_field::runtime::Field;

#[test]
fn original_empty() {
    assert_eq!(
        empty::pure_circuits::bytes_casts_empty()
            .unwrap()
            .into_array(),
        []
    );
}

#[test]
fn original_round_trip() {
    let actual = round_trip::pure_circuits::bytes_casts_round_trip()
        .unwrap()
        .into_array()
        .map(|value| value.value());
    assert_eq!(actual, [5, 6, 7, 8]);
}

#[test]
fn original_to_field() {
    assert_eq!(
        to_field::pure_circuits::bytes_casts_to_field().unwrap(),
        Field::from(42u128)
    );
}

#[test]
fn original_to_tuple() {
    let actual = to_tuple::pure_circuits::bytes_casts_to_tuple().unwrap();
    assert_eq!(
        actual,
        (
            Field::from(10u128),
            Field::from(20u128),
            Field::from(30u128),
            Field::from(40u128),
        )
    );
}

#[test]
fn original_to_uint() {
    assert_eq!(
        to_uint::pure_circuits::bytes_casts_to_uint()
            .unwrap()
            .value(),
        67305985
    );
}

#[test]
fn original_escape_sequences() {
    assert_eq!(
        escape_sequences::pure_circuits::bytes_literal_escape_sequences()
            .unwrap()
            .into_array(),
        [13, 10]
    );
}

#[test]
fn original_non_ascii() {
    assert_eq!(
        non_ascii::pure_circuits::bytes_literal_non_ascii()
            .unwrap()
            .into_array(),
        [10, 195, 132]
    );
}

#[test]
fn original_string() {
    assert_eq!(
        string::pure_circuits::bytes_literal_string()
            .unwrap()
            .into_array(),
        [97, 98, 99, 100, 101, 102, 103, 104]
    );
}
