// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use helper::{ledger_contract as c, runtime as r};
fn context() -> r::context::CircuitContext<()> {
    c::initial_state(r::context::ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn main() {
    let input = helper::types::Input {left:r::BoundedUint::new(0).unwrap(),right:r::BoundedUint::new(255).unwrap()};
    let program = c::functional::run::Program::new(input);
    let _first = program.execute_native(context());
    let _second = program.execute_native(context()); // expected-error
}
