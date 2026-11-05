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

#![allow(dead_code)]
use api::functional::parameter::Program;
use family::{ledger_contract as api, runtime as r};
use r::{BoundedUint, FixedVector};
type U8 = BoundedUint<255>;
type U16 = BoundedUint<65535>;
type U32 = BoundedUint<4294967295>;
type Narrow = FixedVector<(FixedVector<(FixedVector<U8, 2>, U8), 1>, U16), 1>;
type Wide = FixedVector<(FixedVector<(FixedVector<U16, 2>, U16), 1>, U32), 1>;
fn u<const M: u128>(value: u128) -> BoundedUint<M> {
    BoundedUint::new(value).unwrap()
}
fn input() -> Narrow {
    FixedVector::new([(
        FixedVector::new([(FixedVector::new([u(3), u(128)]), u(255))]),
        u(65535),
    )])
}
fn context() -> r::context::CircuitContext<u64> {
    api::initial_state(r::context::ConstructorContext::new(7_u64))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn ordinary(value: FixedVector<(FixedVector<((U8, U8), U8), 1>, U16), 1>) {
    let _ = Program::new(value); // expected-error
}
fn main() {}
