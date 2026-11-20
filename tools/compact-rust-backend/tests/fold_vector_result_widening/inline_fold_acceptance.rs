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

use inline_fold::runtime::{BoundedUint, FixedVector};

#[test]
fn inline_fold_widens_result() {
    let input = FixedVector::new([
        BoundedUint::<255>::new(3).unwrap(),
        BoundedUint::<255>::new(4).unwrap(),
    ]);
    let seed = FixedVector::new([
        BoundedUint::<255>::new(1).unwrap(),
        BoundedUint::<255>::new(2).unwrap(),
    ]);
    let result = inline_fold::pure_circuits::value(input, seed)
        .unwrap()
        .into_array()
        .map(|value| value.value());
    assert_eq!(result, [4, 3]);
}
