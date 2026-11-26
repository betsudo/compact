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

use midnight_compact_runtime::FixedBytes;
use midnight_compact_runtime::context::{CircuitContext, ConstructorContext, ConstructorResult};
use midnight_compact_runtime::ledger::{
    self, ChargedState, CoinRecipient, ContractAddress, ContractState, DefaultDB, HashOutput,
    StateValue, empty_contract_state,
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;

#[test]
fn constructor_to_circuit_transfer_preserves_private_and_ledger_state() {
    let private = vec![1_u8, 2, 3];
    let initial = empty_contract_state();
    let constructor = ConstructorContext::new(private.clone());
    let result = ConstructorResult::new(constructor, initial.clone());
    let circuit = result.into_circuit_context(ContractAddress::default());
    assert_eq!(circuit.private_state, private);
    assert_eq!(circuit.query.state, initial);
    assert_eq!(circuit.query.address, ContractAddress::default());
    assert_eq!(
        circuit.query.call_context.own_address,
        ContractAddress::default()
    );
    assert!(circuit.gas_limit.is_none());
}

#[test]
fn constructor_to_circuit_nonzero_address_preserves_wallet_and_output_plan() {
    let address = ContractAddress(HashOutput([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ]));
    let private = vec![4_u8, 5, 6];
    let initial = empty_contract_state();
    let mut constructor = ConstructorContext::new(private.clone());
    constructor.zswap_state.first_free = 17;
    let mut circuit = ConstructorResult::new(constructor, initial.clone())
        .into_circuit_context(ContractAddress::default());
    let coin =
        ledger::coin_info_from_compact(FixedBytes::new([1; 32]), FixedBytes::new([2; 32]), 42);
    let recipient = CoinRecipient::Contract(address);
    let commitment = coin.commitment(&recipient);
    circuit.zswap_state.pending_outputs =
        circuit.zswap_state.pending_outputs.insert(commitment, coin);
    circuit.set_zswap_output_start(9).unwrap();
    circuit.create_zswap_output(coin, recipient).unwrap();
    let wallet = circuit.zswap_state.clone();
    let plan = circuit.circuit_zswap().clone();

    let circuit = circuit
        .into_constructor_result()
        .into_circuit_context(address);

    assert_eq!(circuit.query.address, address);
    assert_eq!(circuit.query.call_context.own_address, address);
    assert_eq!(circuit.private_state, private);
    assert_eq!(circuit.query.state, initial);
    assert_eq!(circuit.zswap_state.coins, wallet.coins);
    assert_eq!(circuit.zswap_state.pending_spends, wallet.pending_spends);
    assert_eq!(circuit.zswap_state.pending_outputs, wallet.pending_outputs);
    assert_eq!(circuit.zswap_state.merkle_tree, wallet.merkle_tree);
    assert_eq!(circuit.zswap_state.first_free, wallet.first_free);
    assert_eq!(circuit.circuit_zswap(), &plan);
    assert_eq!(circuit.query.call_context.com_indices.size(), 1);
    assert_eq!(
        circuit.query.call_context.com_indices.get(&commitment),
        Some(&9)
    );
    assert!(circuit.gas_limit.is_none());
    assert!(circuit.own_coin_public_key().is_err());

    let read = circuit.kernel_self().unwrap();
    assert_eq!(read.result, address);
    assert_eq!(read.context.query.call_context.own_address, address);
    assert_eq!(read.context.circuit_zswap(), &plan);
}

#[test]
fn stored_contract_nonzero_address_preserves_snapshot_and_new_call_defaults() {
    let address = ContractAddress(HashOutput([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ]));
    let private = vec![7_u8, 8, 9];
    let default_contract = ContractState::<DefaultDB>::default();
    let contract = ContractState {
        data: ChargedState::new(StateValue::Array(
            vec![ledger::constructor_cell(false)].into(),
        )),
        ..default_contract
    };
    let snapshot = contract.clone();
    let wallet = ConstructorContext::<()>::new(()).zswap_state;

    let circuit = CircuitContext::from_contract_state(private.clone(), address, &contract);

    assert_eq!(circuit.query.address, address);
    assert_eq!(circuit.query.call_context.own_address, address);
    assert_eq!(circuit.private_state, private);
    assert_eq!(circuit.query.state, snapshot.data);
    assert_eq!(contract, snapshot);
    assert_eq!(circuit.zswap_state.coins, wallet.coins);
    assert_eq!(circuit.zswap_state.pending_spends, wallet.pending_spends);
    assert_eq!(circuit.zswap_state.pending_outputs, wallet.pending_outputs);
    assert_eq!(circuit.zswap_state.merkle_tree, wallet.merkle_tree);
    assert_eq!(circuit.zswap_state.first_free, wallet.first_free);
    assert!(circuit.circuit_zswap().is_empty());
    assert_eq!(circuit.circuit_zswap().next_index(), 0);
    assert!(circuit.query.call_context.com_indices.is_empty());
    assert!(circuit.events().is_empty());
    assert!(circuit.own_coin_public_key().is_err());
    assert_eq!(circuit.cost_model, INITIAL_COST_MODEL.clone());
    assert!(circuit.gas_limit.is_none());

    let read = circuit.kernel_self().unwrap();
    assert_eq!(read.result, address);
    assert_eq!(read.context.query.call_context.own_address, address);
    assert_eq!(read.context.query.state, snapshot.data);
    assert_eq!(contract, snapshot);
}
