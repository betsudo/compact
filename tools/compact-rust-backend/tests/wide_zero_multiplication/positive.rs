// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use wide_zero_fixture::{pure_circuits as p, runtime as r};
fn main() {
    let wide = r::WideUint::<4722366482869645213695, { u128::MAX }>::from_le_bytes(&[1]).unwrap();
    let zero = r::BoundedUint::<0>::new(0).unwrap();
    let _: r::BoundedUint<0> = p::rightZero(wide).unwrap();
    let _: r::BoundedUint<0> = p::rightParameter(wide, zero).unwrap();
}
