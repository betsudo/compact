// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! One user-owned Zswap output allocates the coin index used by writeCoin.

use super::local_v3_application as v3;
use super::positional_transaction::{check_generated_trace, make_deploy, verify_separately};
use super::*;
use compact_rust_qualified_coin_cell_oracle_fixture::{ledger_contract as contract, types};
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::OfferBackedObservedState;
use midnight_ledger::{structure::ProofVersioned, test_utilities::tx_prove};
use midnight_zswap::{Delta, Offer, Output};

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "Qualified Coin Cell requires local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    // Pair the complete compiler input before key loading or deployment.
    // This checks input identity, not authenticated proving-key provenance.
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        compact_rust_qualified_coin_cell_oracle_fixture::RETAINED_IR,
    )?;
    let name = "write_coin";
    let mut rng = StdRng::seed_from_u64(0x0176_434f_494e);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let deploy = make_deploy(root, name, initial.ledger_state.get_ref().clone(), &mut rng)?;
    let mut ledger = LedgerState::<DefaultDB>::new("local-test");
    ledger.contract = ledger
        .contract
        .insert(deploy.address(), deploy.initial_state.clone());
    let mut nonce = [0_u8; 32];
    nonce[..5].copy_from_slice(b"nonce");
    let mut color = [0_u8; 32];
    color[..5].copy_from_slice(b"color");
    let coin = types::ShieldedCoinInfo {
        nonce: FixedBytes::new(nonce),
        color: FixedBytes::new(color),
        value: BoundedUint::new(42)?,
    };
    let mut user_key = [0_u8; 32];
    user_key[0] = 7;
    let recipient = types::Either {
        is_left: true,
        left: types::ZswapCoinPublicKey {
            bytes: FixedBytes::new(user_key),
        },
        right: types::ContractAddress {
            bytes: FixedBytes::new([0; 32]),
        },
    };
    let info = runtime::ledger::coin_info_from_compact(coin.nonce, coin.color, coin.value.value());
    let output = Output::new(
        &mut rng,
        &info,
        None,
        &runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput(user_key)),
        None,
    )?;
    let offer: Offer<ProofPreimage, DefaultDB> = Offer {
        inputs: vec![].into(),
        outputs: vec![output].into(),
        transient: vec![].into(),
        deltas: vec![Delta {
            token_type: info.type_,
            value: -42,
        }]
        .into(),
    };
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let mut wrong_snapshot = deploy.initial_state.clone();
    wrong_snapshot.operations = HashMap::new();
    if OfferBackedObservedState::new(
        ObservedContractState::new(
            deploy.address(),
            wrong_snapshot,
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        ),
        &ledger,
        offer.clone(),
    )
    .is_ok()
    {
        return Err("offer observation accepted a different contract snapshot".into());
    }
    let bound = OfferBackedObservedState::new(observed, &ledger, offer)?;
    let native = contract::write_coin(
        bound.observed().circuit_context(()),
        coin.clone(),
        recipient.clone(),
    )?;
    let recorded = contract::recorded::write_coin(
        bound.observed().circuit_context(()),
        coin.clone(),
        recipient.clone(),
    )?;
    if native.context.query.state.get_ref() != recorded.execution.context.query.state.get_ref()
        || native.context.query.effects != recorded.execution.context.query.effects
        || native.gas_cost != recorded.execution.gas_cost
        || !recorded.execution.private_transcript_outputs.is_empty()
    {
        return Err("offer-backed writeCoin native and recorded differ".into());
    }
    let expected_state = native.context.query.state.get_ref().clone();
    let input = AlignedValue::concat(&[
        AlignedValue::from(coin.clone()),
        AlignedValue::from(recipient.clone()),
    ]);
    let manual = check_generated_trace(root, name, recorded, input)?;
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.verifier")),
    )?))?;
    let generated = contract::Contract::default();
    let call = generated
        .recording()
        .write_coin_call(bound.observed(), (), coin, recipient)?;
    let other_info =
        runtime::ledger::coin_info_from_compact(FixedBytes::new(nonce), FixedBytes::new(color), 43);
    let other_output = Output::new(
        &mut rng,
        &other_info,
        None,
        &runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput(user_key)),
        None,
    )?;
    let other_offer: Offer<ProofPreimage, DefaultDB> = Offer {
        inputs: vec![].into(),
        outputs: vec![other_output].into(),
        transient: vec![].into(),
        deltas: vec![Delta {
            token_type: other_info.type_,
            value: -43,
        }]
        .into(),
    };
    let other_bound = OfferBackedObservedState::new(
        ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        ),
        &ledger,
        other_offer,
    )?;
    if !matches!(
        other_bound.prepare(call, verifier.clone(), Fr::from(0_u64)),
        Err(runtime::transaction::ObservedCallError::OfferMismatch)
    ) {
        return Err("same-address different-offer call was accepted".into());
    }
    let call = generated.recording().write_coin_call(
        bound.observed(),
        (),
        types::ShieldedCoinInfo {
            nonce: FixedBytes::new(nonce),
            color: FixedBytes::new(color),
            value: BoundedUint::new(42)?,
        },
        types::Either {
            is_left: true,
            left: types::ZswapCoinPublicKey {
                bytes: FixedBytes::new(user_key),
            },
            right: types::ContractAddress {
                bytes: FixedBytes::new([0; 32]),
            },
        },
    )?;
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    if format!("{manual:?}") != format!("{:?}", prepared.prototype()) {
        return Err("offer-backed writeCoin observed and direct preparation differ".into());
    }
    let unfunded_stem = "qualified-coin-unfunded";
    let unfunded_proof = v3::prove_standalone(
        root,
        name,
        unfunded_stem,
        prepared.prototype(),
        &verifier,
        None,
    )?;
    verify_separately(root, name, unfunded_stem)?;
    let tx = prepared.into_transaction(&mut rng, "local-test", Timestamp::from_secs(0));
    let resolver = super::qualified_coin_funding::fee_resolver(root, name)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("unfunded qualified coin proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0176_5345));
    let mut calls = sealed.calls();
    if !matches!(calls.next(), Some((_, call)) if matches!(&call.proof, ProofVersioned::V3(proof) if proof != &unfunded_proof))
        || calls.next().is_some()
    {
        return Err("unfunded offer requires one transaction-bound v3 proof".into());
    }
    let validation_time = Timestamp::from_secs(0);
    let strict_error = sealed
        .well_formed(&ledger, WellFormedStrictness::default(), validation_time)
        .err()
        .ok_or("unfunded output unexpectedly passed default balancing")?;
    if !matches!(
        strict_error,
        midnight_ledger::error::MalformedTransaction::BalanceCheckOverspend {
            segment: 0,
            overspent_value: -42,
            ..
        }
    ) {
        return Err(format!("unexpected strict rejection: {strict_error:?}").into());
    }
    println!("unfunded qualified coin offer rejected by default strictness: {strict_error:?}");
    let mut strictness = WellFormedStrictness::default();
    strictness.enforce_balancing = false;
    let verified = sealed.well_formed(&ledger, strictness, validation_time)?;
    let context = TransactionContext {
        ref_state: ledger.clone(),
        block_context: BlockContext {
            tblock: validation_time,
            last_block_time: validation_time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, outcome) = ledger.apply(&verified, &context);
    if !matches!(outcome, TransactionResult::Success(_)) {
        return Err(format!("offer-backed writeCoin application failed: {outcome:?}").into());
    }
    let applied = updated
        .contract
        .get(&deploy.address())
        .ok_or("contract disappeared")?;
    if applied.data.get_ref() != &expected_state {
        return Err("offer-backed writeCoin applied state differs".into());
    }
    println!(
        "qualified coin Cell writeCoin output-backed proof verified and Ledger 9 applied with balancing disabled"
    );
    let funded_stem = "qualified-coin-funded";
    let mut fee_state = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    v3::apply_funded_deployment(
        root,
        funded_stem,
        &deploy,
        &mut fee_state,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let mut funded_ledger = fee_state.ledger.clone();
    let funded_time = fee_state.time;
    let funded = super::qualified_coin_funding::seed_and_fund(
        &mut rng,
        &mut funded_ledger,
        info,
        runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput(user_key)),
        funded_time,
    )?;
    let funded_offer = funded.offer.clone();
    fee_state.ledger = funded_ledger.clone();
    let funded_bound = OfferBackedObservedState::new(
        ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        ),
        &funded_ledger,
        funded.offer,
    )?;
    let funded_coin = types::ShieldedCoinInfo {
        nonce: FixedBytes::new(nonce),
        color: FixedBytes::new(color),
        value: BoundedUint::new(42)?,
    };
    let funded_recipient = types::Either {
        is_left: true,
        left: types::ZswapCoinPublicKey {
            bytes: FixedBytes::new(user_key),
        },
        right: types::ContractAddress {
            bytes: FixedBytes::new([0; 32]),
        },
    };
    let funded_native = contract::write_coin(
        funded_bound.observed().circuit_context(()),
        funded_coin.clone(),
        funded_recipient.clone(),
    )?;
    let funded_recorded = contract::recorded::write_coin(
        funded_bound.observed().circuit_context(()),
        funded_coin.clone(),
        funded_recipient.clone(),
    )?;
    if funded_native.context.query.state.get_ref()
        != funded_recorded.execution.context.query.state.get_ref()
        || funded_native.context.query.effects != funded_recorded.execution.context.query.effects
        || funded_native.gas_cost != funded_recorded.execution.gas_cost
        || !funded_recorded
            .execution
            .private_transcript_outputs
            .is_empty()
    {
        return Err("funded write_coin native and recorded differ".into());
    }
    let funded_expected_state = funded_native.context.query.state.get_ref().clone();
    let funded_input = AlignedValue::concat(&[
        AlignedValue::from(funded_coin.clone()),
        AlignedValue::from(funded_recipient.clone()),
    ]);
    let funded_manual = check_generated_trace(root, name, funded_recorded, funded_input)?;
    let funded_call = generated.recording().write_coin_call(
        funded_bound.observed(),
        (),
        funded_coin,
        funded_recipient,
    )?;
    let funded_prepared = funded_bound.prepare(funded_call, verifier.clone(), Fr::from(0_u64))?;
    if format!("{funded_manual:?}") != format!("{:?}", funded_prepared.prototype()) {
        return Err("funded writeCoin observed and direct preparation differ".into());
    }
    let standalone = v3::prove_standalone(
        root,
        name,
        funded_stem,
        funded_prepared.prototype(),
        &verifier,
        None,
    )?;
    verify_separately(root, name, funded_stem)?;
    let funded_tx = funded_prepared.into_transaction(
        &mut rng,
        "local-test",
        Timestamp::from_secs(funded_time.to_secs() + 3_600),
    );
    let funded_proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &funded_tx,
            &resolver,
        ))
        .map_err(|error| format!("funded qualified coin proving failed: {error:?}"))?;
    let funded_sealed = funded_proven.seal(StdRng::seed_from_u64(0x0180_5445));
    let funded_sealed =
        executor.block_on(fee_state.balance_tx(rng.clone(), funded_sealed, &resolver))?;
    let mut funded_calls = funded_sealed.calls();
    let (_, funded_call) = funded_calls
        .next()
        .ok_or("missing funded qualified coin call")?;
    let ProofVersioned::V3(funded_proof) = &funded_call.proof else {
        return Err("funded qualified coin requires a transaction-bound v3 proof".into());
    };
    if funded_proof == &standalone
        || funded_calls.next().is_some()
        || v3::positive_fee_dust_spends(&funded_sealed) != 1
    {
        return Err("funded qualified coin proof or Dust spend differs".into());
    }
    let mut bytes = Vec::new();
    tagged_serialize(&funded_sealed, &mut bytes)?;
    v3::write_new(root.join(format!("{funded_stem}-transaction.bin")), &bytes)?;
    v3::write_new(
        root.join(format!("{funded_stem}-transaction.proof")),
        &funded_proof.0,
    )?;
    let funded_verified =
        funded_sealed.well_formed(&funded_ledger, WellFormedStrictness::default(), funded_time)?;
    let funded_context = TransactionContext {
        ref_state: funded_ledger.clone(),
        block_context: BlockContext {
            tblock: funded_time,
            last_block_time: funded_time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (funded_updated, funded_outcome) = funded_ledger.apply(&funded_verified, &funded_context);
    if !matches!(funded_outcome, TransactionResult::Success(_)) {
        return Err(format!("funded writeCoin application failed: {funded_outcome:?}").into());
    }
    if !funded_updated
        .zswap
        .nullifiers
        .contains_key(&funded.spent_nullifier)
    {
        return Err("funded input nullifier was not applied".into());
    }
    if funded_updated.zswap.first_free != funded.output_index + 1 {
        return Err("funded output was not allocated at the expected index".into());
    }
    if funded_updated
        .zswap
        .coin_coms
        .index(funded.output_index)
        .map(|(hash, _)| hash)
        != Some(funded.output_commitment.0)
    {
        return Err("funded output commitment at allocated index differs".into());
    }
    let funded_applied = funded_updated
        .contract
        .get(&deploy.address())
        .ok_or("funded contract disappeared")?;
    if funded_applied.data.get_ref() != &funded_expected_state {
        return Err("funded applied state differs from native and recorded execution".into());
    }
    let cell = contract::PublicStateView::from(funded_applied).pot()?;
    if cell.nonce != FixedBytes::new(nonce)
        || cell.color != FixedBytes::new(color)
        || cell.value != BoundedUint::new(42)?
        || cell.mt_index.value() != funded.output_index as u128
    {
        return Err("funded Cell contains a different qualified coin".into());
    }
    if !matches!(
        funded_updated.zswap.try_apply(&funded_offer, None),
        Err(midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(nullifier))
            if nullifier == funded.spent_nullifier
    ) {
        return Err("replayed upstream offer did not reject the spent nullifier".into());
    }
    let (replayed, replay_result) = funded_updated.apply(
        &funded_verified,
        &TransactionContext {
            ref_state: funded_updated.clone(),
            ..funded_context
        },
    );
    if !matches!(
        replay_result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != funded_updated
    {
        return Err(
            format!("funded qualified coin replay refused incorrectly: {replay_result:?}").into(),
        );
    }
    let mut after = Vec::new();
    let mut replay = Vec::new();
    tagged_serialize(&funded_updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    if after != replay {
        return Err("funded qualified coin replay changed ledger bytes".into());
    }
    v3::write_new(root.join(format!("{funded_stem}-ledger-after.bin")), &after)?;
    v3::write_new(
        root.join(format!("{funded_stem}-ledger-replay.bin")),
        &replay,
    )?;
    println!(
        "funded qualified coin Cell writeCoin passed default strict validation and Ledger 9 application; native/recorded state, input nullifier, output index/commitment, and exact replay rejection verified"
    );
    Ok(())
}
