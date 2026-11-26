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

//! Compile-only external consumer: no Clone or witness bound on the original read.
//! Compile against the complete freshly generated original crate as set_fixture.
use set_fixture::{ledger_contract as c, runtime as r};
struct Private(u64); // Intentionally does not implement Clone.
fn context() -> r::context::CircuitContext<Private> {
    c::initial_state(r::context::ConstructorContext::new(Private(91)))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn capture(
    context: r::context::CircuitContext<Private>,
) -> Result<r::proof_data::RecordedProofData<Private, bool>, r::proof_data::ProofDataError> {
    c::recorded::Contract.is_empty_proof_data(context)
}
fn native(
    context: r::context::CircuitContext<Private>,
) -> Result<r::context::CircuitResult<Private, bool>, r::CompactError> {
    c::is_empty(context)
}
fn main() {
    let _constructor: fn() -> r::context::CircuitContext<Private> = context;
    let _capture = capture;
    let _native = native;
    assert_eq!(Private(91).0, 91);
}
