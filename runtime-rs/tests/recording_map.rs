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

use midnight_compact_runtime::context::{ConstructorContext, ConstructorResult};
use midnight_compact_runtime::fab::AlignedValue;
use midnight_compact_runtime::ledger::{
    ChargedState, ContractAddress, DefaultDB, StateValue, constructor_map,
};
use midnight_compact_runtime::recording::RecordingFrame;
use midnight_compact_runtime::slots::{MapNode, MapSlot};
use midnight_compact_runtime::{BoundedUint, Field, JubjubPoint, ec_mul_generator};
use midnight_ledger_v9::construct::{PreTranscript, partition_transcripts};
use midnight_ledger_v9::structure::INITIAL_PARAMETERS;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_onchain_vm::ops::{Key, Op};
use midnight_serialize::tagged_serialize;

const TABLE: MapSlot<bool, Field> = MapSlot::new(&[0]);

fn context() -> midnight_compact_runtime::context::CircuitContext<()> {
    let state = StateValue::Array(vec![constructor_map::<DefaultDB>()].into());
    ConstructorResult::new(ConstructorContext::new(()), ChargedState::new(state))
        .into_circuit_context(ContractAddress::default())
}

#[test]
fn map_trace_matches_native_state_gas_and_partitioned_effects() {
    let frame = TABLE
        .record_insert_default(RecordingFrame::new(context()), true)
        .unwrap();
    let (frame, default_value) = TABLE.record_lookup(frame, true).unwrap();
    assert_eq!(default_value, Field::from(0_u64));
    let frame = TABLE
        .record_insert(frame, true, Field::from(42_u64))
        .unwrap();
    let (frame, present) = TABLE.record_member(frame, true).unwrap();
    assert!(present);
    let (frame, value) = TABLE.record_lookup(frame, true).unwrap();
    assert_eq!(value, Field::from(42_u64));
    let (frame, size) = TABLE.record_size(frame).unwrap();
    assert_eq!(size, 1);
    let frame = TABLE.record_remove(frame, true).unwrap();
    let (frame, empty) = TABLE.record_is_empty(frame).unwrap();
    assert!(empty);
    let frame = TABLE
        .record_insert(frame, false, Field::from(7_u64))
        .unwrap();
    let recorded = TABLE.record_reset(frame).unwrap().finish(());

    let native = TABLE.insert_default(context(), true).unwrap();
    let mut gas = native.gas_cost;
    let native = TABLE.lookup(native.context, true).unwrap();
    gas += native.gas_cost;
    assert_eq!(native.result, Field::from(0_u64));
    let native = TABLE
        .insert(native.context, true, Field::from(42_u64))
        .unwrap();
    gas += native.gas_cost;
    let native = TABLE.member(native.context, true).unwrap();
    gas += native.gas_cost;
    assert!(native.result);
    let native = TABLE.lookup(native.context, true).unwrap();
    gas += native.gas_cost;
    assert_eq!(native.result, Field::from(42_u64));
    let native = TABLE.size(native.context).unwrap();
    gas += native.gas_cost;
    assert_eq!(native.result, 1);
    let native = TABLE.remove(native.context, true).unwrap();
    gas += native.gas_cost;
    let native = TABLE.is_empty(native.context).unwrap();
    gas += native.gas_cost;
    assert!(native.result);
    let native = TABLE
        .insert(native.context, false, Field::from(7_u64))
        .unwrap();
    gas += native.gas_cost;
    let native = TABLE.reset(native.context).unwrap();
    gas += native.gas_cost;
    assert_eq!(recorded.execution.gas_cost, gas);

    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    let mut native_bytes = Vec::new();
    tagged_serialize(native.context.query.state.get_ref(), &mut native_bytes).unwrap();
    let mut recorded_bytes = Vec::new();
    tagged_serialize(
        recorded.execution.context.query.state.get_ref(),
        &mut recorded_bytes,
    )
    .unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(recorded_bytes, native_bytes);
    assert_eq!(replay_bytes, native_bytes);
    assert_eq!(
        recorded.execution.context.query.effects,
        native.context.query.effects
    );
    assert_eq!(replay.context.effects, native.context.query.effects);

    let (initial, program) = recorded.public.into_parts();
    let partitioned = partition_transcripts(
        &[PreTranscript {
            context: initial,
            program,
            comm_comm: None,
        }],
        &INITIAL_PARAMETERS,
    )
    .unwrap();
    let (guaranteed, fallible) = &partitioned[0];
    let transcript = guaranteed.as_ref().or(fallible.as_ref()).unwrap();
    assert_eq!(transcript.effects, replay.context.effects);
}

#[test]
fn tampered_map_lookup_read_is_rejected() {
    let frame = TABLE
        .record_insert(RecordingFrame::new(context()), true, Field::from(9_u64))
        .unwrap();
    let (frame, observed) = TABLE.record_lookup(frame, true).unwrap();
    assert_eq!(observed, Field::from(9_u64));
    let recorded = frame.finish(observed);
    let mut program = recorded.public.verify_ops().to_vec();
    *program.last_mut().unwrap() = Op::Popeq {
        cached: false,
        result: AlignedValue::from(Field::from(10_u64)),
    };
    assert!(
        recorded
            .public
            .initial()
            .query(&program, None, &INITIAL_COST_MODEL)
            .is_err()
    );
}

#[test]
fn keyed_child_map_uses_typed_paths_in_native_and_recorded_queries() {
    type InnerKey = BoundedUint<255>;
    const NESTED: MapSlot<bool, MapNode<InnerKey, JubjubPoint>> = MapSlot::new(&[0]);
    let inner = InnerKey::new(7).unwrap();
    let absent = InnerKey::new(8).unwrap();
    let point = ec_mul_generator(Field::from(42_u64)).unwrap();

    assert!(NESTED.lookup_child(context(), true, inner).is_err());
    assert!(NESTED.insert_child(context(), true, inner, point).is_err());
    let init = NESTED.init_child(context(), true).unwrap();
    assert!(NESTED.lookup_child(init.context, true, inner).is_err());
    let init = NESTED.init_child(context(), true).unwrap();
    let insert = NESTED
        .insert_child(init.context, true, inner, point)
        .unwrap();
    let read = NESTED.lookup_child(insert.context, true, inner).unwrap();
    assert_eq!(read.result, point);
    assert!(NESTED.lookup_child(read.context, true, absent).is_err());

    let init = NESTED.init_child(context(), true).unwrap();
    let insert = NESTED
        .insert_child(init.context, true, inner, point)
        .unwrap();
    let read = NESTED.lookup_child(insert.context, true, inner).unwrap();
    let frame = NESTED
        .record_init_child(RecordingFrame::new(context()), true)
        .unwrap();
    let frame = NESTED
        .record_insert_child(frame, true, inner, point)
        .unwrap();
    let (frame, result) = NESTED.record_lookup_child(frame, true, inner).unwrap();
    let recorded = frame.finish(result);
    assert_eq!(recorded.execution.result, read.result);
    assert_eq!(
        recorded.execution.gas_cost,
        init.gas_cost + insert.gas_cost + read.gas_cost
    );
    let original = recorded.public.verify_ops();
    let replay = recorded
        .public
        .initial()
        .query(original, None, &INITIAL_COST_MODEL)
        .unwrap();
    assert_eq!(
        replay.context.state.get_ref(),
        read.context.query.state.get_ref()
    );
    assert_eq!(replay.context.effects, read.context.query.effects);

    let mut program = original.to_vec();
    let outer_index = program.len() - 3;
    let Op::Idx { path, .. } = &mut program[outer_index] else {
        panic!("expected the root/outer key query path");
    };
    assert_eq!(path.len(), 2);
    let missing_outer = AlignedValue::from(false).try_into().unwrap();
    *path = vec![Key::Value(0u8.into()), Key::Value(missing_outer)].into();
    assert!(
        recorded
            .public
            .initial()
            .query(&program, None, &INITIAL_COST_MODEL)
            .is_err()
    );

    let mut program = original.to_vec();
    let inner_index = program.len() - 2;
    let Op::Idx { path, .. } = &mut program[inner_index] else {
        panic!("expected the inner key query path");
    };
    assert_eq!(path.len(), 1);
    let missing_inner = AlignedValue::from(absent).try_into().unwrap();
    *path = vec![Key::Value(missing_inner)].into();
    assert!(
        recorded
            .public
            .initial()
            .query(&program, None, &INITIAL_COST_MODEL)
            .is_err()
    );
}
