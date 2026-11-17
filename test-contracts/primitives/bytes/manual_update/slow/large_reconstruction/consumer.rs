// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use large_reconstruction::runtime::context::ConstructorContext;
use large_reconstruction::runtime::ledger::ContractAddress;
use large_reconstruction::runtime::public_state::PublicStateSource;
use large_reconstruction::runtime::FixedBytes;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

fn expected_gas(name: &str) -> u128 {
    std::env::var(name).unwrap().parse().unwrap()
}

#[test]
fn untouched_original_ordinary_recorded_and_replay() {
    let fresh = || {
        large_reconstruction::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let expected = FixedBytes::new([1, 4, 9, 16, 25, 36, 49, 14, 31, 50]);
    let ordinary =
        large_reconstruction::ledger_contract::bytes_manual_update_large_reconstruction(fresh())
            .unwrap();
    assert_eq!(ordinary.result, expected);
    assert_eq!(
        large_reconstruction::ledger_contract::PublicStateView::from(&ordinary)
            .manual_update_large_result()
            .unwrap(),
        expected
    );
    let recorded =
        large_reconstruction::ledger_contract::recorded::bytes_manual_update_large_reconstruction(
            fresh(),
        )
        .unwrap();
    assert_eq!(recorded.execution.result, expected);
    assert_eq!(
        large_reconstruction::ledger_contract::PublicStateView::from(&recorded.execution)
            .manual_update_large_result()
            .unwrap(),
        expected
    );
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_eq!(
        u128::from(ordinary.gas_cost.read_time.into_picoseconds()),
        expected_gas("TS_GAS_READ_TIME")
    );
    assert_eq!(
        u128::from(ordinary.gas_cost.compute_time.into_picoseconds()),
        expected_gas("TS_GAS_COMPUTE_TIME")
    );
    assert_eq!(
        u128::from(ordinary.gas_cost.bytes_written),
        expected_gas("TS_GAS_BYTES_WRITTEN")
    );
    assert_eq!(
        u128::from(ordinary.gas_cost.bytes_deleted),
        expected_gas("TS_GAS_BYTES_DELETED")
    );
    assert!(ordinary.private_transcript_outputs.is_empty());
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    assert_eq!(recorded.public.verify_ops().len(), 315);
    let transcript_path = std::env::var("RUST_TRANSCRIPT_PATH").unwrap();
    std::fs::write(
        transcript_path,
        serde_json::to_vec(recorded.public.verify_ops()).unwrap(),
    )
    .unwrap();

    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
}
