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

//! Circuit-only domain delegation preserves exact private outputs and VM replay.
use super::super::ledger8::{
    ExecutionIdentity, KernelAddress, KernelClaims, Ledger8Bindings, NativeLedger8,
    RecordedLedger8, ShieldedMint, ZswapInput, ZswapOutput,
};
use super::super::{Circuit, Executor, Rule};
use crate::{
    CompactError, FixedBytes,
    context::{CircuitContext, ConstructorContext, ConstructorResult, RunningCost},
    ledger::{self, ContractAddress, DefaultDB, StateValue, contract_state},
};
use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
use midnight_serialize::tagged_serialize;
fn context() -> CircuitContext<u64, DefaultDB> {
    ConstructorResult::new(ConstructorContext::new(103), contract_state(vec![]))
        .into_circuit_context(ContractAddress::default())
}
fn bindings() -> Ledger8Bindings<'static> {
    Ledger8Bindings::new(&[]).unwrap()
}
fn coin() -> ledger::CoinInfo {
    ledger::coin_info_from_compact(FixedBytes::new([1; 32]), FixedBytes::new([2; 32]), 42)
}
fn recipient() -> ledger::CoinRecipient {
    ledger::coin_recipient_from_compact(true, FixedBytes::new([3; 32]), FixedBytes::new([0; 32]))
}
fn bytes(state: &StateValue<DefaultDB>) -> Vec<u8> {
    let mut out = vec![];
    tagged_serialize(state, &mut out).unwrap();
    out
}
fn intents<E: Executor + ExecutionIdentity + KernelAddress + ZswapInput + ZswapOutput>(
    executor: E,
) -> Result<E::Outcome<[u8; 32]>, CompactError> {
    let (executor, key) = executor.own_public_key()?;
    assert_eq!(key, [7; 32]);
    let (executor, address) = executor.kernel_address()?;
    assert_eq!(address, ContractAddress::default());
    let executor = executor.zswap_input(coin().qualify(3))?;
    let executor = executor.zswap_output(coin(), recipient())?;
    let (executor, last) = executor.own_public_key()?;
    assert_eq!(key, last);
    Ok(executor.finish(last))
}
#[test]
fn identity_address_and_intents_keep_exact_transcript_and_allocation() {
    let initial = || {
        let mut context = context().with_coin_public_key_bytes([7; 32]);
        context.set_zswap_output_start(11).unwrap();
        context
    };
    let native = intents(NativeLedger8::new(initial(), bindings())).unwrap();
    let recorded = intents(RecordedLedger8::new(initial(), bindings())).unwrap();
    assert_eq!(native.result, [7; 32]);
    assert_eq!(native.result, recorded.execution.result);
    assert_eq!(native.context.private_state, 103);
    assert_eq!(recorded.execution.context.private_state, 103);
    let expected = vec![
        crate::fab::AlignedValue::from([7_u8; 32]),
        ().into(),
        ().into(),
        crate::fab::AlignedValue::from([7_u8; 32]),
    ];
    assert_eq!(native.private_transcript_outputs, expected);
    assert_eq!(recorded.execution.private_transcript_outputs, expected);
    assert_eq!(
        native.context.circuit_zswap(),
        recorded.execution.context.circuit_zswap()
    );
    assert_eq!(native.context.circuit_zswap().next_index(), 12);
    assert_eq!(native.context.circuit_zswap().inputs(), [coin().qualify(3)]);
    assert_eq!(native.context.circuit_zswap().outputs().len(), 1);
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    let address_only = context().kernel_self().unwrap();
    assert_eq!(native.gas_cost, address_only.gas_cost);
    assert_eq!(
        bytes(native.context.query.state.get_ref()),
        bytes(recorded.execution.context.query.state.get_ref())
    );
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    assert_eq!(
        bytes(replay.context.state.get_ref()),
        bytes(native.context.query.state.get_ref())
    );
    assert_eq!(replay.context.effects, native.context.query.effects);
}
#[test]
fn duplicate_claims_and_maximum_mint_keep_original_verify_programs() {
    let claims = [
        ledger::KernelClaim::Nullifier(ledger::CoinNullifier(ledger::HashOutput([4; 32]))),
        ledger::KernelClaim::CoinSpend(ledger::CoinCommitment(ledger::HashOutput([5; 32]))),
        ledger::KernelClaim::CoinReceive(ledger::CoinCommitment(ledger::HashOutput([6; 32]))),
        ledger::KernelClaim::Nullifier(ledger::CoinNullifier(ledger::HashOutput([4; 32]))),
    ];
    let mut native = NativeLedger8::new(context(), bindings());
    let mut recorded = RecordedLedger8::new(context(), bindings());
    let mut reference = crate::recording::RecordingFrame::new(context());
    for claim in claims {
        native = native.kernel_claim(claim).unwrap();
        recorded = recorded.kernel_claim(claim).unwrap();
        reference = reference.kernel_claim(claim).unwrap();
    }
    let domain = ledger::HashOutput([9; 32]);
    let native = native.mint_shielded(domain, u64::MAX).unwrap().finish(());
    let recorded = recorded.mint_shielded(domain, u64::MAX).unwrap().finish(());
    let reference = reference
        .kernel_mint_shielded(domain, u64::MAX)
        .unwrap()
        .finish(());
    assert_eq!(
        native.context.query.effects,
        recorded.execution.context.query.effects
    );
    assert_eq!(
        reference.execution.context.query.effects,
        recorded.execution.context.query.effects
    );
    assert_eq!(
        *native
            .context
            .query
            .effects
            .shielded_mints
            .get(&domain)
            .unwrap(),
        u64::MAX
    );
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(reference.execution.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        serde_json::to_value(reference.public.verify_ops()).unwrap(),
        serde_json::to_value(recorded.public.verify_ops()).unwrap()
    );
    let replay = recorded
        .public
        .initial()
        .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
        .unwrap();
    assert_eq!(replay.context.effects, native.context.query.effects);
    let mut next = context();
    next.query = native.context.query.clone();
    let mut other = context();
    other.query = native.context.query.clone();
    let error = NativeLedger8::new(next, bindings())
        .mint_shielded(domain, 1)
        .err()
        .unwrap();
    let recorded_error = RecordedLedger8::new(other, bindings())
        .mint_shielded(domain, 1)
        .err()
        .unwrap();
    assert_eq!(error, recorded_error);
    assert!(error.to_string().to_lowercase().contains("overflow"));
}
#[test]
fn missing_identity_guards_and_zero_gas_preserve_first_failure() {
    let calls = std::cell::Cell::new(0);
    let native = Circuit::begin(NativeLedger8::new(context(), bindings()))
        .own_public_key()
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish()
        .err()
        .unwrap();
    let recorded = Circuit::begin(RecordedLedger8::new(context(), bindings()))
        .own_public_key()
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::MissingCoinPublicKey);
    assert_eq!(native, recorded);
    assert_eq!(calls.get(), 0);
    let native = Circuit::begin(NativeLedger8::new(context(), bindings()))
        .require(Rule::new(false, "kernel guard"))
        .own_public_key()
        .finish()
        .err()
        .unwrap();
    assert_eq!(native, CompactError::AssertionFailed("kernel guard".into()));
    let mut first = context();
    first.gas_limit = Some(RunningCost::ZERO);
    let mut second = context();
    second.gas_limit = Some(RunningCost::ZERO);
    let native = Circuit::begin(NativeLedger8::new(first, bindings()))
        .kernel_address()
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    let recorded = Circuit::begin(RecordedLedger8::new(second, bindings()))
        .kernel_address()
        .bind(|_, c| {
            c.try_returning(|| {
                calls.set(calls.get() + 1);
                Ok(())
            })
        })
        .finish();
    assert!(matches!(native, Err(CompactError::LedgerQueryRejected(_))));
    assert!(matches!(
        recorded,
        Err(CompactError::LedgerQueryRejected(_))
    ));
    assert_eq!(calls.get(), 0);
}
#[test]
fn output_cursor_overflow_keeps_existing_error() {
    let mut first = context();
    first.set_zswap_output_start(u64::MAX).unwrap();
    let mut second = context();
    second.set_zswap_output_start(u64::MAX).unwrap();
    let native = NativeLedger8::new(first, bindings())
        .zswap_output(coin(), recipient())
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(second, bindings())
        .zswap_output(coin(), recipient())
        .err()
        .unwrap();
    assert_eq!(native, CompactError::ZswapCursorOverflow);
    assert_eq!(native, recorded);
}
#[cfg(feature = "ledger-transaction")]
#[test]
fn observed_allocation_lock_is_not_bypassed() {
    let native = NativeLedger8::new(context().lock_zswap_allocation(), bindings())
        .zswap_output(coin(), recipient())
        .err()
        .unwrap();
    let recorded = RecordedLedger8::new(context().lock_zswap_allocation(), bindings())
        .zswap_output(coin(), recipient())
        .err()
        .unwrap();
    assert_eq!(native, CompactError::ZswapAllocationLocked);
    assert_eq!(native, recorded);
}
