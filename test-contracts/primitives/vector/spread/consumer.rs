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

use empty::runtime::Field;

fn fields(values: &[u128]) -> Vec<Field> {
    values.iter().copied().map(Field::from).collect()
}

fn main() {
    assert_eq!(
        concatenation::pure_circuits::vector_spread_concatenation()
            .unwrap()
            .0
            .to_vec(),
        fields(&[1, 2, 3, 4])
    );
    assert_eq!(
        default::pure_circuits::vector_spread_default()
            .unwrap()
            .0
            .to_vec(),
        fields(&[0; 10])
    );
    assert_eq!(
        empty::pure_circuits::vector_spread_empty()
            .unwrap()
            .0
            .to_vec(),
        fields(&[1, 2, 3])
    );
    let mixed = mixed_types::pure_circuits::vector_spread_mixed_types().unwrap();
    assert_eq!((mixed.0, mixed.1.value(), mixed.2.value()), (true, 2, 3));
    assert_eq!(
        nested::pure_circuits::vector_spread_nested()
            .unwrap()
            .0
            .to_vec(),
        fields(&[1, 2, 3])
    );
    assert_eq!(
        with_literals::pure_circuits::vector_spread_with_literals()
            .unwrap()
            .0
            .to_vec(),
        fields(&[1, 2, 3, 4, 5])
    );
    assert_eq!(
        with_slice::pure_circuits::vector_spread_with_slice()
            .unwrap()
            .0
            .to_vec(),
        fields(&[10, 20, 40, 50])
    );
    println!("seven original vector spread assertions passed");
}
