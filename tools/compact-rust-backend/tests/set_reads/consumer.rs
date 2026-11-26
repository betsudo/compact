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

//! Compile-only consumer of the complete freshly generated original Set crate.
//! set_fixture must alias that actual crate; no stub or selected-circuit crate.
use set_fixture::{ledger_contract as c, runtime as r};
struct Private(u64); // Deliberately no Clone or witness implementation.
type Count = r::BoundedUint<18446744073709551615>;
fn context() -> r::context::CircuitContext<Private> {
    c::initial_state(r::context::ConstructorContext::new(Private(91)))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn member(
    context: r::context::CircuitContext<Private>,
    value: r::Field,
) -> Result<r::proof_data::RecordedProofData<Private, bool>, r::proof_data::ProofDataError> {
    c::recorded::Contract.member_proof_data(context, value)
}
fn size(
    context: r::context::CircuitContext<Private>,
) -> Result<r::proof_data::RecordedProofData<Private, Count>, r::proof_data::ProofDataError> {
    c::recorded::Contract.size_proof_data(context)
}
fn emptiness(
    context: r::context::CircuitContext<Private>,
) -> Result<r::proof_data::RecordedProofData<Private, bool>, r::proof_data::ProofDataError> {
    c::recorded::Contract.is_empty_proof_data(context)
}
fn native_member(
    context: r::context::CircuitContext<Private>,
    value: r::Field,
) -> Result<r::context::CircuitResult<Private, bool>, r::CompactError> {
    c::member(context, value)
}
fn native_size(
    context: r::context::CircuitContext<Private>,
) -> Result<r::context::CircuitResult<Private, Count>, r::CompactError> {
    c::size(context)
}
fn main() {
    let _constructor: fn() -> r::context::CircuitContext<Private> = context;
    let _member = member;
    let _size = size;
    let _emptiness = emptiness;
    let _native_member = native_member;
    let _native_size = native_size;
    assert_eq!(Private(91).0, 91);
}
