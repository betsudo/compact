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

use parameter::{ledger_contract as c, runtime as r};
use r::{BoundedUint, FixedVector};
type Input = FixedVector<
    (
        FixedVector<(BoundedUint<255>, BoundedUint<255>), 2>,
        BoundedUint<65535>,
    ),
    2,
>;
type Output = FixedVector<
    (
        FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
        BoundedUint<4294967295>,
    ),
    2,
>;
fn u<const M: u128>(v: u128) -> BoundedUint<M> {
    BoundedUint::new(v).unwrap()
}
fn pair<const M: u128>(v: u128) -> (BoundedUint<M>, BoundedUint<M>) {
    (u(v), u((v + 17) % 256))
}
fn input() -> Input {
    FixedVector::new([
        (FixedVector::new([pair(3), pair(5)]), u(31)),
        (FixedVector::new([pair(4), pair(6)]), u(41)),
    ])
}
fn context() -> r::context::CircuitContext<()> {
    c::initial_state(r::context::ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn main() {
    let _: Output = c::run(context(), input()).unwrap().result;
    let _: FixedVector<
        (
            FixedVector<(BoundedUint<65535>, BoundedUint<65535>), 2>,
            BoundedUint<4294967295>,
        ),
        3,
    > = c::run(context(), input()).unwrap().result; // expected-error
}
