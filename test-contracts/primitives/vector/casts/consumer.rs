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

fn main() {
    assert_eq!(
        empty::pure_circuits::vector_casts_empty().unwrap().into_array(),
        [] as [u8; 0],
    );
    assert_eq!(
        to_bytes::pure_circuits::vector_casts_to_bytes()
            .unwrap().into_array(),
        [10, 20, 30, 40],
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
    println!("six original Vector cast assertions passed");
}
