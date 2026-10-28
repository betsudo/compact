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

use vector_fixture::{ledger_contract as c, runtime as r};

fn main() {
    let context = c::initial_state(r::context::ConstructorContext::new(7_u64))
        .unwrap()
        .into_circuit_context(Default::default());
    let fields = || r::FixedVector([r::Field::from(17_u64), r::Field::from(41_u64)]);
    let first = c::functional::choose::Program::new(fields(), true);
    let last = c::functional::choose::Program::new(fields(), false);
    let _ = first.execute_native(context);
    let _ = last.execute_native(context); // expected-error
}
