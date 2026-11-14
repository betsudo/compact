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

use basic::runtime::Field;

fn main() {
    assert_eq!(
        basic::pure_circuits::vector_indexing_basic().unwrap(),
        (Field::from(10u128), Field::from(30u128), Field::from(40u128)),
    );
    assert_eq!(
        nested::pure_circuits::vector_indexing_nested().unwrap(),
        Field::from(60u128),
    );
    println!("two original Vector indexing assertions passed");
}
