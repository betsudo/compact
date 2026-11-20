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

use empty::runtime::Field;

fn fields(values: &[u128]) -> Vec<Field> {
    values.iter().copied().map(Field::from).collect()
}

fn main() {
    assert_eq!(
        computed_index::pure_circuits::vector_slice_computed_index().unwrap().0.to_vec(),
        fields(&[20, 30]),
    );
    assert_eq!(
        default::pure_circuits::vector_slice_default().unwrap().0.to_vec(),
        fields(&[0, 0]),
    );
    assert_eq!(
        empty::pure_circuits::vector_slice_empty().unwrap().0.to_vec(),
        fields(&[]),
    );
    assert_eq!(
        from_parameter::pure_circuits::vector_slice_from_parameter((false, true, Field::from(42u128)))
            .unwrap().0,
        Field::from(42u128),
    );
    let mixed = mixed_types::pure_circuits::vector_slice_mixed_types().unwrap();
    assert_eq!(mixed, (Field::from(4u128), true));
    assert_eq!(
        nested::pure_circuits::vector_slice_nested().unwrap().0.to_vec(),
        fields(&[1200, 1200]),
    );
    assert_eq!(
        radix_bounds::pure_circuits::vector_slice_radix_bounds().unwrap().0.to_vec(),
        fields(&[30, 40]),
    );
    assert_eq!(
        semantics::pure_circuits::vector_slice_semantics().unwrap().0.to_vec(),
        fields(&[20, 30]),
    );
    assert_eq!(
        field_to_uint::pure_circuits::vector_slice_field_to_uint()
            .unwrap().0.to_vec(),
        fields(&[20]),
    );
    println!("eight original vector slice assertions passed");
    println!("local Field-to-Uint slice assertion passed");
}
