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

use compact_contract_return_if_else_either::{pure_circuits, runtime};
#[test]
fn original_branches() {
    let left = pure_circuits::return_if_else_either(runtime::Field::from(2u128)).unwrap();
    assert_eq!(
        (left.is_left, left.left, left.right),
        (true, runtime::Field::from(1u128), false)
    );
    let right = pure_circuits::return_if_else_either(runtime::Field::from(0u128)).unwrap();
    assert_eq!(
        (right.is_left, right.left, right.right),
        (false, runtime::Field::from(0u128), true)
    );
}
