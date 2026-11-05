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
type NarrowProduct = ((U8,), U8);
type WideProduct = ((U16,), U16);
type Narrow = FixedVector<NarrowProduct, 2>;
type Wide = FixedVector<WideProduct, 2>;
fn u<const M: u128>(v: u128) -> BoundedUint<M> {
    BoundedUint::new(v).unwrap()
}
fn input() -> Narrow {
    FixedVector::new([((u(3),), u(128)), ((u(255),), u(219))])
}
fn context() -> r::context::CircuitContext<u64> {
    api::initial_state(r::context::ConstructorContext::new(7_u64))
        .unwrap()
        .into_circuit_context(Default::default())
}
type Bad = [NarrowProduct; 2];
fn main() {
    let input: Bad = [((u(3),), u(128)), ((u(255),), u(219))];
    let _ = Program::new(input); // expected-error
}
