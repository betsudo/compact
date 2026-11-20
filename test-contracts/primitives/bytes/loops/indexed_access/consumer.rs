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

use indexed_access::runtime::Field;
use indexed_access::runtime::context::ConstructorContext;
use indexed_access::runtime::ledger::ContractAddress;
use indexed_access::runtime::public_state::PublicStateSource;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

#[test]
fn indexed_access_original_result_recording_and_replay() {
    let fresh = || {
        indexed_access::ledger_contract::initial_state(ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let ordinary = indexed_access::ledger_contract::bytes_loop_indexed_access(fresh()).unwrap();
    assert_eq!(ordinary.result, Field::from(32u128));
    assert_eq!(
        indexed_access::ledger_contract::PublicStateView::from(&ordinary)
            .loop_result()
            .unwrap(),
        Field::from(32u128)
    );
    assert_eq!(
        ordinary.gas_cost.read_time.into_picoseconds(),
        2_380_000_000
    );
    assert_eq!(
        ordinary.gas_cost.compute_time.into_picoseconds(),
        22_370_692_016
    );
    assert_eq!(ordinary.gas_cost.bytes_written, 290);
    assert_eq!(ordinary.gas_cost.bytes_deleted, 288);
    assert!(ordinary.private_transcript_outputs.is_empty());

    let recorded =
        indexed_access::ledger_contract::recorded::bytes_loop_indexed_access(fresh()).unwrap();
    assert_eq!(recorded.execution.result, ordinary.result);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_eq!(recorded.public.verify_ops().len(), 54);
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
}
