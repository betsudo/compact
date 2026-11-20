// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use compact_rust_counter_fixture::ledger_contract as counter_contract;
use midnight_compact_runtime::context::CircuitContext;
use midnight_compact_runtime::ledger::{ContractState, DefaultDB, StateValue, read_counter};
use midnight_ledger::structure::Signature;
use midnight_ledger::structure::{ProofMarker, Transaction};
use midnight_onchain_state::state::EntryPointBuf;
use midnight_serialize::tagged_deserialize;
use midnight_transient_crypto::commitment::PureGeneratorPedersen;

fn round(state: &StateValue<DefaultDB>) -> u64 {
    let StateValue::Array(fields) = state else {
        panic!("counter state is not an array")
    };
    read_counter(fields.get(0).expect("Counter field")).expect("valid Counter")
}

type SealedTransaction = Transaction<Signature, ProofMarker, PureGeneratorPedersen, DefaultDB>;

#[test]
fn deployed_ledger_state_starts_a_second_recorded_counter_call() {
    // The local Ledger 9 deployment and first call both passed funded default-strict application.
    let state_bytes = include_bytes!("fixtures/confirmed-counter-state-ledger9.bin");
    let contract: ContractState<DefaultDB> =
        tagged_deserialize(&mut state_bytes.as_slice()).expect("Ledger 9 ContractState bytes");
    let deploy_bytes = include_bytes!("fixtures/counter-deploy-ledger9.bin");
    let deploy: SealedTransaction =
        tagged_deserialize(&mut deploy_bytes.as_slice()).expect("Ledger 9 deploy transaction");
    let mut deployments = deploy.deploys();
    let (_, action) = deployments.next().expect("one deployment");
    assert!(deployments.next().is_none());
    let address = action.address();
    let entry = EntryPointBuf(b"increment".to_vec());
    let operation = contract
        .operations
        .get(&entry)
        .expect("deployed increment operation");
    assert!(operation.latest().is_some());
    assert_eq!(
        action.initial_state.operations.get(&entry).as_deref(),
        Some(&*operation)
    );
    assert_eq!(round(contract.data.get_ref()), 1);

    let context = CircuitContext::from_contract_state((), address, &contract);
    let recorded = counter_contract::Contract::default()
        .recording
        .increment(context)
        .expect("recorded increment from indexed state");
    assert_eq!(round(recorded.public.initial().state.get_ref()), 1);
    assert_eq!(round(recorded.execution.context.query.state.get_ref()), 2);
    assert!(!recorded.public.verify_ops().is_empty());
}

#[test]
fn malformed_confirmed_state_is_rejected_before_recording() {
    let mut bytes = include_bytes!("fixtures/confirmed-counter-state-ledger9.bin").to_vec();
    bytes.truncate(20);
    assert!(tagged_deserialize::<ContractState<DefaultDB>>(&mut bytes.as_slice()).is_err());
}

#[test]
fn historical_ledger8_artifacts_are_rejected_by_ledger9_decoders() {
    // ABI-18 files were captured from indexer 4.0.1 on the pinned Ledger 8 stack.
    let state = include_bytes!("fixtures/confirmed-counter-state-abi18.bin");
    let deploy = include_bytes!("fixtures/counter-deploy-abi18.bin");
    assert!(tagged_deserialize::<ContractState<DefaultDB>>(&mut state.as_slice()).is_err());
    assert!(tagged_deserialize::<SealedTransaction>(&mut deploy.as_slice()).is_err());
}
