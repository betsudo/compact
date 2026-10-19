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

//! Ledger8 interpreters delegate existing frames, slots and metered witnesses.
mod bindings;
mod counters;
mod kernel;
mod lists;
mod maps;
mod merkle;
mod native;
mod qualified_storage;
mod recorded;
mod sets;
mod witness;
pub use bindings::{
    CellBinding, CounterBinding, Ledger8Bindings, ListBinding, MapBinding, MerkleBinding,
    SetBinding,
};
pub use kernel::{
    ExecutionIdentity, KernelAddress, KernelClaims, ShieldedMint, ZswapInput, ZswapOutput,
};
pub use native::NativeLedger8;
pub use recorded::RecordedLedger8;
pub use witness::{ExactWitnessOutput, NoWitnesses, WitnessHandler};

pub use qualified_storage::{QualifiedCellWrite, QualifiedSetInsert};
