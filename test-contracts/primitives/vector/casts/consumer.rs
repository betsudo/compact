// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

fn main() {
    assert_eq!(
        empty::pure_circuits::vector_casts_empty().unwrap().into_array(),
        [] as [u8; 0],
    );
    assert_eq!(
        to_bytes_default::pure_circuits::vector_casts_to_bytes_default()
            .unwrap().into_array(),
        [0, 0, 0, 0],
    );
    assert_eq!(
        to_bytes_narrow_element::pure_circuits::vector_casts_to_bytes_narrow_element()
            .unwrap().into_array(),
        [1, 2, 3, 127],
    );
    let tuple = to_tuple::pure_circuits::vector_casts_to_tuple().unwrap();
    assert_eq!((tuple.0.value(), tuple.1.value()), (100, 200));
    assert_eq!(
        to_vector::pure_circuits::vector_casts_to_vector()
            .unwrap().into_array().map(|value| value.value()),
        [1, 2, 3, 4],
    );
    println!("five original Vector cast assertions passed");
}
