// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

fn main() {
    assert_eq!(
        to_vector::pure_circuits::bytes_casts_to_vector()
            .unwrap().into_array().map(|value| value.value()),
        [1, 2],
    );
    assert_eq!(
        to_vector_default::pure_circuits::bytes_casts_to_vector_default()
            .unwrap().into_array().map(|value| value.value()),
        [0, 0, 0, 0],
    );
    assert_eq!(
        to_vector_wide_element::pure_circuits::bytes_casts_to_vector_wide_element()
            .unwrap().into_array().map(|value| value.value()),
        [1, 2, 3, 255],
    );
    println!("three original Bytes-to-Vector assertions passed");
}
