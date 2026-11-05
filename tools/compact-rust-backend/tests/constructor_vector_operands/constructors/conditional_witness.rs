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

impl c::TryWitnesses<u64> for Log {
    fn first(
        &self,
        context: r::context::WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, r::FixedVector<Field, 2>), r::CompactError> {
        let private = self.observe("first", *context.private_state, context.ledger.stored()?);
        self.accept("first")?;
        Ok((private, inputs()))
    }
    fn second(
        &self,
        context: r::context::WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, r::FixedVector<Field, 2>), r::CompactError> {
        let private = self.observe("second", *context.private_state, context.ledger.stored()?);
        self.accept("second")?;
        Ok((
            private,
            r::FixedVector([Field::from(7u64), Field::from(8u64)]),
        ))
    }
}
fn initialize(case: &Value, log: &Log) -> Result<ConstructorResult<u64>, r::CompactError> {
    c::initial_state(
        ConstructorContext::new(5),
        log,
        case["flag"].as_bool().unwrap(),
    )
}
fn check_extra(_state: &ConstructorResult<u64>) {}
