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

use generated::{ledger_contract as c,runtime as r};
fn main() {
 let context=c::initial_state(r::context::ConstructorContext::new(5u64),r::FixedVector([r::Field::from(2u64);2])).unwrap().into_circuit_context(Default::default());
 let program=c::functional::advance::Program::new(r::Field::from(1u64));
 let first=program.execute_native(context).unwrap();
 assert_eq!(first.result,r::Field::from(3u64));
}
