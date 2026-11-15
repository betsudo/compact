// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use zero_iteration::runtime::context::ConstructorContext;
use zero_iteration::runtime::ledger::ContractAddress;
use zero_iteration::runtime::public_state::PublicStateSource;
use zero_iteration::runtime::Field;
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;

#[test]
fn original_return_in_nested_lambda() {
    assert_eq!(
        return_in_lambda_in_body::pure_circuits::for_return_in_lambda_in_body()
            .unwrap()
            .value(),
        7,
    );
}

#[test]
fn original_zero_iteration_and_recorded_replay() {
    let fresh = || zero_iteration::ledger_contract::initial_state(
        ConstructorContext::new(())
    ).unwrap().into_circuit_context(ContractAddress::default());
    let ordinary = zero_iteration::ledger_contract::for_zero_iteration(fresh()).unwrap();
    assert_eq!(ordinary.result, Field::from(0u128));
    let recorded = zero_iteration::ledger_contract::recorded::for_zero_iteration(fresh()).unwrap();
    assert_eq!(recorded.execution.result, Field::from(0u128));
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert!(!recorded.public.verify_ops().is_empty());
    let replay = recorded.public.initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL).unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
}

#[test]
fn original_generic_bound_and_recorded_replay() {
    let fresh = || generic_bound::ledger_contract::initial_state(
        ConstructorContext::new(())
    ).unwrap().into_circuit_context(ContractAddress::default());
    let ordinary = generic_bound::ledger_contract::for_generic_bound(fresh()).unwrap();
    assert_eq!(ordinary.result.value(), 5);
    assert_eq!(generic_bound::ledger_contract::PublicStateView::from(&ordinary)
               .total().unwrap().value(), 5);
    let recorded = generic_bound::ledger_contract::recorded::for_generic_bound(fresh()).unwrap();
    assert_eq!(recorded.execution.result.value(), 5);
    assert_eq!(generic_bound::ledger_contract::PublicStateView::from(&recorded.execution)
               .total().unwrap().value(), 5);
    assert_eq!(recorded.execution.public_state(), ordinary.public_state());
    assert_eq!(recorded.execution.gas_cost, ordinary.gas_cost);
    assert_eq!(ordinary.gas_cost.read_time.into_picoseconds(), 765000000);
    assert_eq!(ordinary.gas_cost.compute_time.into_picoseconds(), 7385878722);
    assert_eq!(ordinary.gas_cost.bytes_written, 72);
    assert_eq!(ordinary.gas_cost.bytes_deleted, 72);
    assert!(ordinary.private_transcript_outputs.is_empty());
    assert!(recorded.execution.private_transcript_outputs.is_empty());
    // The initial write and both ordered loop writes each retain six SDK
    // verification operations; the terminal read does not add a write.
    assert_eq!(recorded.public.verify_ops().len(), 18);
    let replay = recorded.public.initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL).unwrap();
    let mut ordinary_bytes = Vec::new();
    tagged_serialize(ordinary.public_state(), &mut ordinary_bytes).unwrap();
    let mut replay_bytes = Vec::new();
    tagged_serialize(replay.context.state.get_ref(), &mut replay_bytes).unwrap();
    assert_eq!(replay_bytes, ordinary_bytes);
}
