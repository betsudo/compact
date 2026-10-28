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

use contract::{ledger_contract as api, runtime as rt};
struct Answers;
impl api::TryWitnesses<u64> for Answers {
    fn private_offset(
        &self,
        context: rt::context::WitnessContext<'_, u64, api::LedgerView<'_>>,
        value: rt::Field,
    ) -> Result<(u64, rt::Field), rt::CompactError> {
        Ok((*context.private_state, value))
    }
}
fn context() -> Result<rt::context::CircuitContext<u64>, rt::CompactError> {
    Ok(
        api::initial_state(rt::context::ConstructorContext::new(7_u64))?
            .into_circuit_context(Default::default()),
    )
}
fn main() -> Result<(), rt::CompactError> {
    let program = api::functional::apply_offset::Program::new(rt::Field::from(42_u64));
    let _ = program.execute_native(context()?, &Answers)?;
    let _ = program.execute_native(context()?, &Answers)?; // EXPECTED_REFUSAL
    Ok(())
}
