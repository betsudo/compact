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
use candidate::runtime::context::CircuitContext;
use candidate::{
    ledger_contract::functional::run::Program,
    runtime::{BoundedUint, FixedVector},
};
type U = BoundedUint<255>;
type Input = FixedVector<((U,), U), 1>;
fn execute(input: Input, first: CircuitContext<u64>, second: CircuitContext<u64>) {
    let program = Program::new(input);
    let _ = program.execute_native(first);
    let _ = program.execute_native(second); // expected-error
}
fn main() {}
