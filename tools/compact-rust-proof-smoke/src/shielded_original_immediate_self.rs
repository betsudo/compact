// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pinned original immediate self-send: direct Effects and source-bound proof.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_ledger::{
    error::TransactionProvingError,
    structure::ProofVersioned,
    test_utilities::{ClientProvingError, tx_prove},
};
use midnight_zswap::{Offer, Output, Transient};
use runtime::ledger::{CoinInfo, CoinRecipient};
use runtime::transaction::{
    ContractTransientCoins, ObservedCallError, OfferBackedObservedState, OfferBindingOptions,
    PersistentOutputAllocation, WalletFundingInputs, ZswapIntentError,
};

const NAME: &str = "sendImmediateShieldedToSelfTest";
const STEM: &str = "unshielded-original-immediate-shielded-self-100";
const GENERATED_RUST: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/contract/lib.rs"
));
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/sendImmediateShieldedToSelfTest.zkir"
));

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original immediate self-send requires local proving".into());
    }
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/sendImmediateShieldedToSelfTest.zkir", GENERATED_ZKIR),
    ] {
        if fs::read(root.join(member))? != expected {
            return Err(format!("original immediate self-send artifact changed: {member}").into());
        }
    }
    let mut rng = StdRng::seed_from_u64(0x0184_5453);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let (_, receive_verifier, receive_operation) = v3::load_operation(root, "receiveShieldedTest")?;
    let contract_state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new()
            .insert(EntryPointBuf(NAME.as_bytes().to_vec()), operation.clone())
            .insert(
                EntryPointBuf(b"receiveShieldedTest".to_vec()),
                receive_operation.clone(),
            ),
        ContractMaintenanceAuthority::default(),
    );
    let deploy = ContractDeploy::new(&mut rng, contract_state);
    let address = deploy.address();
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    let coin = crate::types::ShieldedCoinInfo {
        nonce: FixedBytes::new([0x65; 32]),
        color: FixedBytes::new(address.custom_shielded_token_type(domain).0.0),
        value: BoundedUint::new(100)?,
    };
    let mut native_context = initial.into_circuit_context(address);
    native_context.set_zswap_output_start(0)?;
    let native = contract::sendImmediateShieldedToSelfTest(
        native_context,
        coin.clone(),
        BoundedUint::new(100)?,
    )?;
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    let resolver = super::qualified_coin_funding::fee_resolver_for_circuits(
        root,
        &["receiveShieldedTest", NAME],
    )?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        STEM,
        &deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let info = runtime::ledger::coin_info_from_compact(coin.nonce, coin.color, 100);
    let keys = midnight_zswap::keys::SecretKeys::from_rng_seed(&mut rng);
    let wallet_coins = [
        CoinInfo {
            nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput([0x91; 32])),
            type_: info.type_,
            value: 100,
        },
        CoinInfo {
            nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput([0x92; 32])),
            type_: info.type_,
            value: 1,
        },
    ];
    let wallet_outputs = wallet_coins
        .iter()
        .map(|coin| {
            Output::new(
                &mut rng,
                coin,
                None,
                &keys.coin_public_key(),
                Some(keys.enc_public_key()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let seed_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], wallet_outputs, vec![]).ok_or("wallet seed offer missing")?;
    let (seeded, _) = funded.ledger.zswap.try_apply(&seed_offer, None)?;
    funded.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(funded.time, funded.ledger.parameters.global_ttl),
    );
    let ledger = funded.ledger.clone();
    if ledger.zswap.first_free <= 1 {
        return Err("immediate transient trial lacks historical-index guard".into());
    }
    let wallet = midnight_zswap::local::State::<DefaultDB>::new().apply(&keys, &seed_offer)?;
    if wallet.merkle_tree.root() != ledger.zswap.coin_coms.root() {
        return Err("immediate wallet and Ledger Merkle roots differ".into());
    }
    let wallet_coin = *wallet
        .coins
        .iter()
        .find(|(_, coin)| coin.value == 100)
        .ok_or("wallet 100 coin missing")?
        .1;
    let (_, wallet_input) = wallet.spend(&mut rng, &keys, &wallet_coin, None)?;
    let receive_output = Output::new_contract_owned(&mut rng, &info, None, address)?;
    let transient = Transient::new_from_contract_owned_output(
        &mut rng,
        &info.qualify(0),
        None,
        receive_output.clone(),
    )?;
    let sent_info = runtime::ledger::coin_info_from_compact(
        native.result.sent.nonce,
        native.result.sent.color,
        100,
    );
    let sent_output = Output::new_contract_owned(&mut rng, &sent_info, None, address)?;
    let sent_commitment = sent_output.coin_com;
    let offer: Offer<ProofPreimage, DefaultDB> = Offer::new(
        vec![wallet_input.clone()],
        vec![sent_output],
        vec![transient.clone()],
    )
    .ok_or("immediate offer missing")?;
    let offer_control = offer.clone();
    let wallet_input_control = wallet_input.clone();
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    if indices.get(&sent_commitment).copied() != Some(ledger.zswap.first_free)
        || indices.get(&transient.coin_com).copied() != Some(ledger.zswap.first_free + 1)
    {
        return Err("immediate canonical offer allocation differs".into());
    }
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let options = OfferBindingOptions::default()
        .with_output_allocation(PersistentOutputAllocation::CanonicalOfferIndices)
        .with_wallet_funding(
            WalletFundingInputs::from_inputs(vec![wallet_input])
                .map_err(|error| format!("wallet funding: {error:?}"))?,
        )
        .with_transient_coins(
            ContractTransientCoins::from_transients(vec![transient.clone()])
                .map_err(|error| format!("transient selection: {error:?}"))?,
        );
    let bound = OfferBackedObservedState::with_options(observed, &ledger, offer, options)?;
    let call = contract::Contract::default()
        .recording
        .sendImmediateShieldedToSelfTest_call(
            bound.observed(),
            (),
            coin.clone(),
            BoundedUint::new(100)?,
        )?;
    let send_only = contract::Contract::default()
        .recording
        .sendImmediateShieldedToSelfTest_call(
            bound.observed(),
            (),
            coin.clone(),
            BoundedUint::new(100)?,
        )?;
    if !matches!(
        bound.prepare(send_only, verifier.clone(), Fr::from(0_u64)),
        Err(ObservedCallError::ZswapIntent(
            ZswapIntentError::TransientInputMismatch
        ))
    ) {
        return Err("immediate send alone was not refused by the offer binder".into());
    }
    let receive_only = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(bound.observed(), (), coin.clone())?;
    if !matches!(
        bound.prepare(receive_only, receive_verifier.clone(), Fr::from(0_u64)),
        Err(ObservedCallError::ZswapIntent(
            ZswapIntentError::TransientInputMismatch
        ))
    ) {
        return Err("transient receive alone was not refused by the offer binder".into());
    }
    if native.result != call.recorded().execution.result || native.result.change.is_some {
        return Err("original immediate self-send result or no-change differs".into());
    }
    let receive_call = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(bound.observed(), (), coin.clone())?;
    let pair = bound.prepare_transient_receive_then_send(
        receive_call,
        receive_verifier.clone(),
        Fr::from(0_u64),
        call,
        verifier.clone(),
        Fr::from(0_u64),
    )?;
    let (receive_prepared, prepared) = pair.prototypes();
    if receive_prepared.op != receive_operation
        || receive_prepared.fallible_public_transcript.is_some()
    {
        return Err("original immediate receive operation differs".into());
    }
    let receive_effects = &receive_prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original immediate receive transcript missing")?
        .effects;
    if receive_effects.claimed_shielded_receives.size() != 1
        || !receive_effects
            .claimed_shielded_receives
            .member(&transient.coin_com)
        || !receive_effects.claimed_nullifiers.is_empty()
        || !receive_effects.claimed_shielded_spends.is_empty()
        || !receive_effects.shielded_mints.is_empty()
        || !receive_effects.unshielded_mints.is_empty()
        || !receive_effects.unshielded_inputs.is_empty()
        || !receive_effects.unshielded_outputs.is_empty()
        || !receive_effects.claimed_unshielded_spends.is_empty()
        || !receive_effects.claimed_contract_calls.is_empty()
    {
        return Err("original immediate receive Effects differ".into());
    }
    if prepared.op != operation || prepared.fallible_public_transcript.is_some() {
        return Err("original immediate self-send installed operation differs".into());
    }
    let effects = &prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original immediate self-send transcript missing")?
        .effects;
    let sent = &native.result.sent;
    let claimed_sent_info: CoinInfo =
        runtime::ledger::coin_info_from_compact(sent.nonce, sent.color, sent.value.value());
    let commitment = claimed_sent_info.commitment(&CoinRecipient::Contract(address));
    if sent.value.value() != 100
        || commitment != sent_commitment
        || effects.claimed_nullifiers.size() != 1
        || !effects.claimed_nullifiers.member(&transient.nullifier)
        || effects.claimed_shielded_spends.size() != 1
        || effects.claimed_shielded_receives.size() != 1
        || !effects.claimed_shielded_spends.member(&commitment)
        || !effects.claimed_shielded_receives.member(&commitment)
        || !effects.shielded_mints.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err("original immediate self-send Effects differ".into());
    }
    let standalone = v3::prove_standalone(root, NAME, STEM, prepared, &verifier, None)?;
    v3::verify_standalone(root, STEM, &verifier)?;
    let receive_stem = format!("{STEM}-receive");
    let receive_standalone = v3::prove_standalone(
        root,
        "receiveShieldedTest",
        &receive_stem,
        receive_prepared,
        &receive_verifier,
        None,
    )?;
    v3::verify_standalone(root, &receive_stem, &receive_verifier)?;
    println!(
        "{NAME}: pinned original 100→100 Effects and both source-bound standalone proofs passed"
    );
    let before = funded.ledger.clone();
    let time = funded.time;
    let tx = pair.into_transaction(
        &mut rng,
        "local-test",
        time + funded.ledger.parameters.global_ttl,
    );
    let proved = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("original immediate pair proving failed: {error:?}"))?;
    let sealed = proved.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut calls = sealed.calls();
    let (receive_segment, receive_call) = calls.next().ok_or("transient receive call missing")?;
    let (send_segment, send_call) = calls.next().ok_or("immediate send call missing")?;
    let ProofVersioned::V3(receive_tx_proof) = &receive_call.proof else {
        return Err("transient receive transaction proof is not v3".into());
    };
    let ProofVersioned::V3(send_tx_proof) = &send_call.proof else {
        return Err("immediate send transaction proof is not v3".into());
    };
    if receive_segment != send_segment
        || receive_call.address != address
        || send_call.address != address
        || receive_call.entry_point != EntryPointBuf(b"receiveShieldedTest".to_vec())
        || send_call.entry_point != EntryPointBuf(NAME.as_bytes().to_vec())
        || receive_tx_proof == &receive_standalone
        || send_tx_proof == &standalone
        || calls.next().is_some()
        || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err("original immediate pair transaction binding or fee differs".into());
    }
    v3::write_new(
        root.join(format!("{STEM}-receive-transaction.proof")),
        &receive_tx_proof.0,
    )?;
    v3::write_new(
        root.join(format!("{STEM}-transaction.proof")),
        &send_tx_proof.0,
    )?;
    let mut tx_bytes = Vec::new();
    tagged_serialize(&sealed, &mut tx_bytes)?;
    v3::write_new(root.join(format!("{STEM}-transaction.bin")), &tx_bytes)?;
    let strictness = WellFormedStrictness::default();
    let verified = sealed.well_formed(&before, strictness, time)?;
    let context = TransactionContext {
        ref_state: before.clone(),
        block_context: BlockContext {
            tblock: time,
            last_block_time: time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, result) = before.apply(&verified, &context);
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("original immediate pair strict apply failed: {result:?}").into());
    }
    if updated.zswap.first_free != ledger.zswap.first_free + 2
        || updated
            .zswap
            .coin_coms
            .index(ledger.zswap.first_free)
            .map(|(hash, _)| hash)
            != Some(sent_commitment.0)
        || updated
            .zswap
            .coin_coms
            .index(ledger.zswap.first_free + 1)
            .map(|(hash, _)| hash)
            != Some(transient.coin_com.0)
        || !updated.zswap.nullifiers.contains_key(&transient.nullifier)
        || updated.contract.get(&address) != Some(&deploy.initial_state)
    {
        return Err("original immediate pair post-Ledger output/nullifier/state differs".into());
    }
    v3::check_offer_replay(root, STEM, &updated, &verified, context)?;
    // Same public coin commitment and nullifier with a private singleton
    // path at index one must fail the actual transaction prover. Index zero
    // is the only permitted immediate transient path, even though historical
    // commitments exist at indices zero and one in the seeded Ledger.
    let wrong = Transient::new_from_contract_owned_output(
        &mut rng,
        &info.qualify(1),
        None,
        receive_output,
    )?;
    if wrong.coin_com != transient.coin_com || wrong.nullifier != transient.nullifier {
        return Err("wrong singleton transient changed public identity".into());
    }
    let mut wrong_offer = offer_control;
    wrong_offer.transient = vec![wrong.clone()].into_iter().collect();
    let wrong_observed = ObservedContractState::new(
        address,
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let wrong_options = OfferBindingOptions::default()
        .with_output_allocation(PersistentOutputAllocation::CanonicalOfferIndices)
        .with_wallet_funding(
            WalletFundingInputs::from_inputs(vec![wallet_input_control])
                .map_err(|error| format!("wrong-path funding: {error:?}"))?,
        )
        .with_transient_coins(
            ContractTransientCoins::from_transients(vec![wrong])
                .map_err(|error| format!("wrong-path selection: {error:?}"))?,
        );
    let wrong_bound = OfferBackedObservedState::with_options(
        wrong_observed,
        &ledger,
        wrong_offer,
        wrong_options,
    )?;
    let wrong_receive = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(wrong_bound.observed(), (), coin.clone())?;
    let wrong_send = contract::Contract::default()
        .recording
        .sendImmediateShieldedToSelfTest_call(
            wrong_bound.observed(),
            (),
            coin,
            BoundedUint::new(100)?,
        )?;
    let wrong_pair = wrong_bound.prepare_transient_receive_then_send(
        wrong_receive,
        receive_verifier,
        Fr::from(0_u64),
        wrong_send,
        verifier,
        Fr::from(0_u64),
    )?;
    let wrong_tx = wrong_pair.into_transaction(
        &mut rng,
        "local-test",
        time + funded.ledger.parameters.global_ttl,
    );
    let failure = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &wrong_tx,
            &resolver,
        ))
        .err()
        .ok_or("wrong singleton index transaction unexpectedly proved")?;
    let diagnostic = format!("{failure:?}");
    if !matches!(
        failure,
        ClientProvingError::Local(TransactionProvingError::Proving(_))
    ) || !diagnostic.contains("Public transcript input mismatch for input 13;")
    {
        return Err(format!("wrong singleton index refusal differs: {diagnostic}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-wrong-index-refusal.txt")),
        format!("{diagnostic}\n").as_bytes(),
    )?;
    println!(
        "{NAME}: real wallet-funded singleton transient, two original calls, strict apply and replay passed"
    );
    Ok(())
}
