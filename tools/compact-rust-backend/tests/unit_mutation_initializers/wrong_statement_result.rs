// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use fixture::{ledger_contract as c, runtime as r};
fn context() -> r::context::CircuitContext<u64> {
    c::initial_state(r::context::ConstructorContext::new(7))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn main() {
    let _: r::Field = c::statement_unit(context(), true, r::Field::from(99_u64))
        .unwrap()
        .result;
}
