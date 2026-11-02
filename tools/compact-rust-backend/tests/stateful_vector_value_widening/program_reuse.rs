// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use parameter_unsigned::{ledger_contract as c, runtime as r};
fn context() -> r::context::CircuitContext<()> {
    c::initial_state(r::context::ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn input() -> r::FixedVector<r::BoundedUint<255>, 2> {
    r::FixedVector::new([
        r::BoundedUint::new(0).unwrap(),
        r::BoundedUint::new(255).unwrap(),
    ])
}
fn main() {
    let p = c::functional::run::Program::new(input());
    let _ = p.execute_native(context());
    let _ = p.execute_native(context()); // expected-error
}
