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
use r::{Field, FixedVector, context::WitnessContext};
struct Bad;
impl empty_ordered::ledger_contract::TryWitnesses<u64> for Bad {
    fn seed(
        &self,
        _ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
    ) -> Result<(u64, Field), r::CompactError> {
        unreachable!()
    }
    fn items(
        &self,
        _ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
    ) -> Result<(u64, ()), r::CompactError> {
        unreachable!()
    }
    fn step(
        &self,
        _ctx: WitnessContext<'_, u64, empty_ordered::ledger_contract::LedgerView<'_>>,
        _acc: Field,
        _item: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        unreachable!()
    }
}
fn main() {}
