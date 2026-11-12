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

use compact_rust_effect_plan_unit_keccak_source_fixture::{ledger_contract as c, runtime as r};
use midnight_onchain_state::state::{ContractOperation, EntryPointBuf};
use r::context::ConstructorContext;
use r::transaction::{Observation, ObservedContractState, VerifierKey};
use rand::{Rng, SeedableRng, rngs::StdRng};
pub fn setup() -> (ObservedContractState, VerifierKey) {
    let key: VerifierKey = StdRng::seed_from_u64(581).r#gen();
    let mut operations = midnight_storage::storage::HashMap::new();
    for name in ["close", "lazy_close", "digest"] {
        operations = operations.insert(
            EntryPointBuf(name.as_bytes().to_vec()),
            ContractOperation::new(Some(key.clone()), None),
        );
    }
    let context = c::initial_state(ConstructorContext::new(5_u64))
        .unwrap()
        .into_circuit_context(Default::default());
    (
        ObservedContractState::new(
            Default::default(),
            r::ledger::ContractState::new(
                context.query.state.get_ref().clone(),
                operations,
                Default::default(),
            ),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        ),
        key,
    )
}
