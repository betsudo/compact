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

use super::*;
use compact_rust_witness_jubjub_fixture::ledger_contract as c;
use midnight_compact_runtime as r;
use midnight_onchain_state::state::{ContractOperation, EntryPointBuf};
use midnight_serialize::{Deserializable, Serializable};
use r::transaction::{
    Observation, ObservedCallError, ObservedContractState, PrepareCallError, VerifierKey,
};
fn observed_state() -> (ObservedContractState, VerifierKey) {
    // This empty installed carrier is not a proof key; empty-public-program
    // refusal is the intended preparation boundary.
    let mut bytes = Vec::new();
    Vec::<u8>::new().serialize(&mut bytes).unwrap();
    let key = VerifierKey::deserialize(&mut bytes.as_slice(), 0).unwrap();
    let operations = r::ledger::LedgerHashMap::new().insert(
        EntryPointBuf(b"hash_scalar".to_vec()),
        ContractOperation::new(Some(key.clone()), None),
    );
    (
        ObservedContractState::new(
            Default::default(),
            r::ledger::ContractState::new(
                context().query.state.get_ref().clone(),
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
#[test]
fn hash_to_curve_records_one_scalar_witness_and_refuses_empty_preparation() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../runtime-rs/tests/fixtures/witness-jubjub.json"
    ))
    .unwrap();
    let (state, key) = observed_state();
    let seed = Field::from(2_u64);
    let a = GeneratorWitness(Default::default());
    let b = GeneratorWitness(Default::default());
    let w = GeneratorWitness(Default::default());
    let native = c::hash_scalar(state.circuit_context(7), &a, seed).unwrap();
    let recorded = c::recorded::hash_scalar(state.circuit_context(7), &b, seed).unwrap();
    let contract = c::Contract::from(w.clone());
    let call = contract
        .recording()
        .hash_scalar_call(&state, 7, seed)
        .unwrap();
    for actual in [&recorded, call.recorded()] {
        assert_oracle(
            &actual.execution,
            point_json(actual.execution.result),
            &oracle["hashScalar"],
        );
        assert_eq!(actual.execution.result, native.result);
        assert_eq!(
            actual.execution.context.query.state,
            native.context.query.state
        );
        assert_eq!(
            actual.execution.context.query.effects,
            native.context.query.effects
        );
        assert_eq!(
            actual.execution.context.private_state,
            native.context.private_state
        );
        assert_eq!(
            actual.execution.private_transcript_outputs,
            native.private_transcript_outputs
        );
        assert_eq!(actual.execution.gas_cost, native.gas_cost);
        assert!(actual.public.verify_ops().is_empty());
        let replay = actual
            .public
            .initial()
            .query(actual.public.verify_ops(), None, &native.context.cost_model)
            .unwrap();
        assert_eq!(replay.context.state, native.context.query.state);
        assert_eq!(replay.context.effects, native.context.query.effects);
    }
    assert_oracle(&native, point_json(native.result), &oracle["hashScalar"]);
    for witness in [&a, &b, &w] {
        assert_eq!(*witness.0.borrow(), vec![(seed, 7)]);
    }
    assert!(matches!(
        call.prepare(key, Field::from(0_u64)),
        Err(ObservedCallError::Prepare(
            PrepareCallError::EmptyTranscript
        ))
    ));
}
