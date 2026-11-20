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

use compact_rust_effect_plan_public_products_source_fixture::{runtime as r, support::context};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, EntryPointBuf,
};
use r::ledger::ContractState;
use r::transaction::{Observation, ObservedContractState, VerifierKey};
use rand::{Rng, SeedableRng, rngs::StdRng};
pub fn setup() -> (ObservedContractState, VerifierKey) {
    let key: VerifierKey = StdRng::seed_from_u64(558).r#gen();
    let mut operations = midnight_storage::storage::HashMap::new();
    for name in ["run", "tuple11", "vector9", "empty"] {
        operations = operations.insert(
            EntryPointBuf(name.as_bytes().to_vec()),
            ContractOperation::new(Some(key.clone()), None),
        );
    }
    (
        ObservedContractState::new(
            Default::default(),
            ContractState::new(
                context().query.state.get_ref().clone(),
                operations,
                ContractMaintenanceAuthority::default(),
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
pub fn atoms(value: &r::fab::AlignedValue) -> Vec<Vec<u8>> {
    value.value.0.iter().map(|atom| atom.0.clone()).collect()
}
pub fn invalid() -> r::CompactError {
    r::CompactError::InvalidLedgerCell("opaque ledger input has trailing zero bytes".into())
}
