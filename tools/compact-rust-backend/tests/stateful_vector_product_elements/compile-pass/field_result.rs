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

use field::{ledger_contract as c, runtime as r};
use r::{BoundedUint, FixedVector};
type Input = FixedVector<(FixedVector<BoundedUint<255>, 2>, BoundedUint<65535>), 2>;
type Output = FixedVector<(FixedVector<r::Field, 2>, r::Field), 2>;
fn u<const M: u128>(v: u128) -> BoundedUint<M> {
    BoundedUint::new(v).unwrap()
}
fn input() -> Input {
    FixedVector::new([
        (FixedVector::new([u(3), u(5)]), u(31)),
        (FixedVector::new([u(4), u(6)]), u(41)),
    ])
}
fn context() -> r::context::CircuitContext<()> {
    c::initial_state(r::context::ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn main() {
    let _: Output = c::run(context(), input()).unwrap().result;
}
