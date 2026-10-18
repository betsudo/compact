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

//! Experimental execution projection for compiler-checked functional programs.
//!
//! A generated immutable request holds arguments; its one checked run body uses
//! these consuming carriers with a selected interpreter. Construction of a request
//! does not execute this API. Handwritten callbacks and custom executors are plain
//! Rust: this module does not certify their effects, privacy or recording support.
//! No runtime graph discovery, captured continuation storage or error recovery is
//! provided. The compiler remains responsible for slot/type/branch authority.
//!
//! # Operand evaluation
//!
//! Rust evaluates method arguments eagerly, including arguments to `require`,
//! `write`, `increment`, `witness` and `returning`. A terminal carrier suppresses
//! executor calls, not evaluation performed before those methods receive their
//! arguments. Generated programs evaluate fallible/effectful operands inside a
//! successful `bind` or `try_returning`; handwritten chaining has no such automatic
//! guarantee. Branches must likewise construct and execute only the selected arm.

mod circuit;
pub mod ledger8;
mod map;
mod set;
mod slots;
mod witness;
pub use circuit::{Circuit, Executor, Rule};
pub use map::{Map, MapRead, MapWrite};
pub use set::{Set, SetRead, SetWrite};
pub use slots::{Cell, CellRead, CellWrite, Counter, CounterIncrement, CounterRead, SlotId};
pub use witness::{Witness, WitnessCall, WitnessId};

#[cfg(test)]
mod tests;
