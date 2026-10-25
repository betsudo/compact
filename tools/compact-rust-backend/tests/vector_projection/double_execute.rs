// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use vector_fixture::{ledger_contract as c, runtime as r};
fn context() -> r::context::CircuitContext<u64> {
    c::initial_state(r::context::ConstructorContext::new(7))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn main() {
    let program = c::functional::choose::Program::new(
        r::FixedVector([r::Field::from(1_u64), r::Field::from(2_u64)]),
        true,
    );
    let _ = program.execute_native(context());
    let _ = program.execute_native(context());
}
