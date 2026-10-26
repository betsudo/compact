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

use direct::runtime as r;
use r::{Field, FixedVector, context::CircuitContext};
fn invalid(a: CircuitContext<u64>, b: CircuitContext<u64>) {
    let p = inline_branch::ledger_contract::functional::run::Program::new(
        Field::from(1u64),
        Field::from(10u64),
        true,
        FixedVector::new([Field::from(2u64), Field::from(3u64)]),
    );
    let _ = p.execute_native(a);
    let _ = p.execute_recorded(b);
}
fn main() {}
