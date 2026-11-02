// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use parameter::{ledger_contract as c, runtime as r};
fn context() -> r::context::CircuitContext<()> {c::initial_state(r::context::ConstructorContext::new(())).unwrap().into_circuit_context(Default::default())}
fn input() -> r::FixedVector<(r::BoundedUint<255>,r::BoundedUint<65535>),2> {r::FixedVector::new([(r::BoundedUint::new(3).unwrap(),r::BoundedUint::new(31).unwrap()),(r::BoundedUint::new(4).unwrap(),r::BoundedUint::new(41).unwrap())])}
fn main() {
let _: r::FixedVector<(r::Field,r::Field),2> = c::run(context(),input()).unwrap().result; // expected-error
}
