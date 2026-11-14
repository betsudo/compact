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
fn original_to_tuple() {
    let result = to_tuple::pure_circuits::vector_implicit_casts_to_tuple().unwrap();
    assert_eq!([result.0.value(), result.1.value()], [1, 2]);
}

#[test]
fn original_widening() {
    let result = widening::pure_circuits::vector_implicit_casts_widening().unwrap();
    assert_eq!(result.into_array().map(|item| item.value()), [1, 2, 3, 255]);
}
