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

//! Pinned original immediate send to a distinct contract: 60+40 and three calls.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_ledger::{
    error::{EffectsCheckError, MalformedTransaction, TransactionProvingError},
    structure::ProofVersioned,
    test_utilities::{ClientProvingError, tx_prove},
};
use midnight_transient_crypto::proofs::Proof;
use midnight_zswap::{Offer, Output, Transient};
use runtime::ledger::{CoinInfo, CoinRecipient};
use runtime::transaction::{
    ContractTransientCoins, ObservedCallError, OfferBackedObservedState, OfferBindingOptions,
    PersistentOutputAllocation, WalletFundingInputs, ZswapIntentError,
};

const NAME: &str = "sendImmediateShieldedToContractTest";
const RECIPIENT: &str = "receiveShieldedTest";
const STEM: &str = "unshielded-original-immediate-shielded-contract-change-60";
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
    "/zkir/sendImmediateShieldedToContractTest.zkir"
));

fn verify_changed_binding(
    root: &Path,
    stem: &str,
    verifier: &VerifierKey,
) -> Result<(), Box<dyn Error>> {
    let proof = Proof(fs::read(root.join(format!("{stem}.proof")))?);
    let bytes = fs::read(root.join(format!("{stem}.inputs")))?;
    let mut reader = bytes.as_slice();
    let mut inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty() || inputs.len() < 2 {
        return Err("standalone binding inputs missing or trailing".into());
    }
    inputs[1] = if inputs[1] == Fr::from(1_u64) {
        Fr::from(2_u64)
    } else {
        Fr::from(1_u64)
    };
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, inputs.into_iter())
        .is_ok()
    {
        return Err(format!("{stem} changed binding was accepted").into());
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original immediate contract send requires local proving".into());
    }
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        (
            "zkir/sendImmediateShieldedToContractTest.zkir",
            GENERATED_ZKIR,
        ),
    ] {
        if fs::read(root.join(member))? != expected {
            return Err(format!("original immediate contract artifact changed: {member}").into());
        }
    }
    let mut rng = StdRng::seed_from_u64(0x0184_5454);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let (_, receive_verifier, receive_operation) = v3::load_operation(root, RECIPIENT)?;
    let recipient_deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        RECIPIENT,
        receive_operation.clone(),
        &mut rng,
    );
    let contract_state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new()
            .insert(EntryPointBuf(NAME.as_bytes().to_vec()), operation.clone())
            .insert(
                EntryPointBuf(RECIPIENT.as_bytes().to_vec()),
                receive_operation.clone(),
            ),
        ContractMaintenanceAuthority::default(),
    );
    let deploy = ContractDeploy::new(&mut rng, contract_state);
    let address = deploy.address();
    let recipient_address = recipient_deploy.address();
    if address == recipient_address {
        return Err("immediate contract recipient equals sender".into());
    }
    let compact_recipient = crate::types::ContractAddress {
        bytes: runtime::ledger::contract_address_bytes(&recipient_address),
    };
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    // The physical offer orders persistent outputs by commitment. Select a
    // deterministic original input whose sent/change order matches the
    // recorded circuit's sent-then-change intent stream.
    let mut chosen = None;
    for byte in 1_u8..=96 {
        let coin = crate::types::ShieldedCoinInfo {
            nonce: FixedBytes::new([byte; 32]),
            color: FixedBytes::new(address.custom_shielded_token_type(domain).0.0),
            value: BoundedUint::new(100)?,
        };
        let mut native_context =
            contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
        native_context.set_zswap_output_start(0)?;
        let native = contract::sendImmediateShieldedToContractTest(
            native_context,
            coin.clone(),
            compact_recipient.clone(),
            BoundedUint::new(60)?,
        )?;
        if !native.result.change.is_some || native.result.change.value.value.value() != 40 {
            return Err("original immediate contract 60+40 result differs".into());
        }
        let sent = runtime::ledger::coin_info_from_compact(
            native.result.sent.nonce,
            native.result.sent.color,
            60,
        )
        .commitment(&CoinRecipient::Contract(recipient_address));
        let change = runtime::ledger::coin_info_from_compact(
            native.result.change.value.nonce,
            native.result.change.value.color,
            40,
        )
        .commitment(&CoinRecipient::Contract(address));
        if sent < change {
            chosen = Some((coin, native));
            break;
        }
    }
    let (coin, native) = chosen.ok_or("no canonical immediate contract input nonce")?;
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 3)?;
    let resolver =
        super::qualified_coin_funding::fee_resolver_for_circuits(root, &[RECIPIENT, NAME])?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        &format!("{STEM}-recipient"),
        &recipient_deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
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
        60,
    );
    let sent_output = Output::new_contract_owned(&mut rng, &sent_info, None, recipient_address)?;
    let sent_commitment = sent_output.coin_com;
    let change = native.result.change.value.clone();
    if !native.result.change.is_some || change.value.value() != 40 {
        return Err("original immediate contract 60+40 result differs".into());
    }
    let change_info = runtime::ledger::coin_info_from_compact(change.nonce, change.color, 40);
    let change_output = Output::new_contract_owned(&mut rng, &change_info, None, address)?;
    let change_commitment = change_output.coin_com;
    let recipient_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![sent_output.clone()], vec![])
            .ok_or("recipient-only exact output offer missing")?;
    if sent_output.contract_address.as_deref() != Some(&recipient_address)
        || change_output.contract_address.as_deref() != Some(&address)
        || transient.contract_address.as_deref() != Some(&address)
        || transient.segment() != Some(0)
        || sent_output.segment() != Some(0)
        || change_output.segment() != Some(0)
    {
        return Err(format!("immediate contract offer owner or segment differs: sent={:?} change={:?} transient={:?} segments={:?}/{:?}/{:?}", sent_output.contract_address, change_output.contract_address, transient.contract_address, sent_output.segment(), change_output.segment(), transient.segment()).into());
    }
    let offer: Offer<ProofPreimage, DefaultDB> = Offer::new(
        vec![wallet_input.clone()],
        vec![sent_output, change_output],
        vec![transient.clone()],
    )
    .ok_or("immediate offer missing")?;
    let offer_control = offer.clone();
    let wallet_input_control = wallet_input.clone();
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let sent_index = *indices
        .get(&sent_commitment)
        .ok_or("sent output index missing")?;
    let change_index = *indices
        .get(&change_commitment)
        .ok_or("change output index missing")?;
    let transient_index = *indices
        .get(&transient.coin_com)
        .ok_or("transient index missing")?;
    if sent_index != ledger.zswap.first_free
        || change_index != ledger.zswap.first_free + 1
        || transient_index != ledger.zswap.first_free + 2
        || offer.outputs.len() != 2
        || offer.inputs.len() != 1
        || offer.transient.len() != 1
        || recipient_offer.outputs.len() != 1
        || recipient_offer
            .outputs
            .iter()
            .next()
            .map(|output| output.coin_com)
            != Some(sent_commitment)
        || !recipient_offer
            .outputs
            .iter()
            .next()
            .is_some_and(|output| output.contract_address.as_deref() == Some(&recipient_address))
    {
        return Err("immediate contract exact output/offer allocation differs".into());
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
        .sendImmediateShieldedToContractTest_call(
            bound.observed(),
            (),
            coin.clone(),
            compact_recipient.clone(),
            BoundedUint::new(60)?,
        )?;
    let send_only = contract::Contract::default()
        .recording
        .sendImmediateShieldedToContractTest_call(
            bound.observed(),
            (),
            coin.clone(),
            compact_recipient.clone(),
            BoundedUint::new(60)?,
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
    if native.result != call.recorded().execution.result {
        return Err("original immediate contract recorded result differs".into());
    }
    let receive_call = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(bound.observed(), (), coin.clone())?;
    let pair = bound
        .prepare_transient_receive_then_send(
            receive_call,
            receive_verifier.clone(),
            Fr::from(0_u64),
            call,
            verifier.clone(),
            Fr::from(0_u64),
        )
        .map_err(|error| format!("sender pair reconciliation: {error:?}"))?;
    if pair.offer().erase_proofs() != offer_control.erase_proofs() {
        return Err("sender pair retained offer differs from sole physical offer".into());
    }
    let wrong_recipient = crate::types::ContractAddress {
        bytes: runtime::ledger::contract_address_bytes(&address),
    };
    let wrong_send = contract::Contract::default()
        .recording
        .sendImmediateShieldedToContractTest_call(
            bound.observed(),
            (),
            coin.clone(),
            wrong_recipient,
            BoundedUint::new(60)?,
        );
    if !matches!(
        wrong_send,
        Err(runtime::CompactError::ZswapOfferOutputMismatch)
    ) {
        return Err("immediate contract accepted wrong recipient against exact offer".into());
    }
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
        return Err("original immediate contract installed operation differs".into());
    }
    let effects = &prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original immediate contract transcript missing")?
        .effects;
    let sent = &native.result.sent;
    let claimed_sent_info: CoinInfo =
        runtime::ledger::coin_info_from_compact(sent.nonce, sent.color, sent.value.value());
    let commitment = claimed_sent_info.commitment(&CoinRecipient::Contract(recipient_address));
    let claimed_change_info =
        runtime::ledger::coin_info_from_compact(change.nonce, change.color, change.value.value());
    let claimed_change = claimed_change_info.commitment(&CoinRecipient::Contract(address));
    if sent.value.value() != 60
        || change.value.value() != 40
        || commitment != sent_commitment
        || claimed_change != change_commitment
        || effects.claimed_nullifiers.size() != 1
        || !effects.claimed_nullifiers.member(&transient.nullifier)
        || effects.claimed_shielded_spends.size() != 2
        || effects.claimed_shielded_receives.size() != 1
        || !effects.claimed_shielded_spends.member(&commitment)
        || !effects.claimed_shielded_spends.member(&claimed_change)
        || !effects.claimed_shielded_receives.member(&claimed_change)
        || effects.claimed_shielded_receives.member(&commitment)
        || !effects.shielded_mints.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err("original immediate contract Effects differ".into());
    }
    let recipient_observed = ObservedContractState::new(
        recipient_address,
        recipient_deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let recipient_bound =
        OfferBackedObservedState::new(recipient_observed, &ledger, recipient_offer)?;
    let recipient_call = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(recipient_bound.observed(), (), native.result.sent.clone())?;
    let recipient_prepared = recipient_bound
        .prepare(recipient_call, receive_verifier.clone(), Fr::from(0_u64))
        .map_err(|error| format!("recipient preparation: {error:?}"))?;
    let recipient_prototype = recipient_prepared.prototype();
    let recipient_effects = &recipient_prototype
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("recipient guaranteed transcript missing")?
        .effects;
    if recipient_prototype.op != receive_operation
        || recipient_prototype.fallible_public_transcript.is_some()
        || recipient_effects.claimed_shielded_receives.size() != 1
        || !recipient_effects
            .claimed_shielded_receives
            .member(&sent_commitment)
        || !recipient_effects.claimed_nullifiers.is_empty()
        || !recipient_effects.claimed_shielded_spends.is_empty()
        || !recipient_effects.shielded_mints.is_empty()
        || !recipient_effects.unshielded_mints.is_empty()
        || !recipient_effects.unshielded_inputs.is_empty()
        || !recipient_effects.unshielded_outputs.is_empty()
        || !recipient_effects.claimed_unshielded_spends.is_empty()
        || !recipient_effects.claimed_contract_calls.is_empty()
    {
        return Err("recipient original receive Effects differ".into());
    }
    let without_offer: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(receive_prepared.clone())
            .add_call::<ProofPreimage>(prepared.clone())
            .add_call::<ProofPreimage>(recipient_prototype.clone());
    let without_offer =
        Transaction::from_intents("local-test", HashMap::new().insert(1_u16, without_offer));
    let mut refusal_strictness = WellFormedStrictness::default();
    refusal_strictness.enforce_balancing = false;
    let no_offer_refusal = without_offer
        .well_formed(&ledger, refusal_strictness, funded.time)
        .err()
        .ok_or("immediate contract three calls without offer accepted")?;
    if !matches!(
        &no_offer_refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::NullifiersNEClaimedNullifiers { nullifiers, claimed_nullifiers }
        ) if nullifiers.is_empty()
            && claimed_nullifiers.as_slice() == [(0, transient.nullifier, address)]
    ) {
        return Err(format!("no-offer refusal differs: {no_offer_refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-without-offer-refusal.txt")),
        format!("{no_offer_refusal:?}\n").as_bytes(),
    )?;
    let sender_only: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(receive_prepared.clone())
            .add_call::<ProofPreimage>(prepared.clone());
    let sender_only = Transaction::new(
        "local-test",
        HashMap::new().insert(1_u16, sender_only),
        Some(offer_control.clone()),
        HashMap::new(),
    );
    let sender_refusal = sender_only
        .well_formed(&ledger, refusal_strictness, funded.time)
        .err()
        .ok_or("immediate contract sender-only offer accepted")?;
    if !matches!(
        &sender_refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::CommitmentsNEClaimedShieldedReceives {
                commitments, claimed_shielded_receives,
            }
        ) if commitments.len() == 3
            && commitments.contains(&(0, transient.coin_com, address))
            && commitments.contains(&(0, sent_commitment, recipient_address))
            && commitments.contains(&(0, change_commitment, address))
            && claimed_shielded_receives.len() == 2
            && claimed_shielded_receives.contains(&(0, transient.coin_com, address))
            && claimed_shielded_receives.contains(&(0, change_commitment, address))
            && !claimed_shielded_receives.contains(&(0, sent_commitment, recipient_address))
    ) {
        return Err(format!("sender-only refusal differs: {sender_refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-without-recipient-refusal.txt")),
        format!("{sender_refusal:?}\n").as_bytes(),
    )?;
    let standalone = v3::prove_standalone(root, NAME, STEM, prepared, &verifier, None)?;
    super::positional_transaction::verify_separately(root, NAME, STEM)?;
    verify_changed_binding(root, STEM, &verifier)?;
    let receive_stem = format!("{STEM}-receive");
    let receive_standalone = v3::prove_standalone(
        root,
        "receiveShieldedTest",
        &receive_stem,
        receive_prepared,
        &receive_verifier,
        None,
    )?;
    super::positional_transaction::verify_separately(root, RECIPIENT, &receive_stem)?;
    verify_changed_binding(root, &receive_stem, &receive_verifier)?;
    let recipient_stem = format!("{STEM}-recipient-receive");
    let recipient_standalone = v3::prove_standalone(
        root,
        RECIPIENT,
        &recipient_stem,
        recipient_prototype,
        &receive_verifier,
        None,
    )?;
    super::positional_transaction::verify_separately(root, RECIPIENT, &recipient_stem)?;
    verify_changed_binding(root, &recipient_stem, &receive_verifier)?;
    println!(
        "{NAME}: pinned original 100→60+40 Effects and all three source-bound standalone proofs passed"
    );
    let before = funded.ledger.clone();
    let time = funded.time;
    // The sender pair was reconciled jointly against the complete offer. The
    // recipient was prepared against the exact sent output alone; here all
    // three prototypes share one guaranteed intent and one physical offer.
    let (sender_receive, sender_send) = pair.prototypes();
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(sender_receive.clone())
            .add_call::<ProofPreimage>(sender_send.clone())
            .add_call::<ProofPreimage>(recipient_prototype.clone());
    let tx = Transaction::new(
        "local-test",
        HashMap::new().insert(1_u16, intent),
        Some(offer_control.clone()),
        HashMap::new(),
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
    let (recipient_segment, recipient_tx_call) =
        calls.next().ok_or("recipient receive call missing")?;
    let ProofVersioned::V3(recipient_tx_proof) = &recipient_tx_call.proof else {
        return Err("recipient receive transaction proof is not v3".into());
    };
    if receive_segment != send_segment
        || send_segment != recipient_segment
        || receive_call.address != address
        || send_call.address != address
        || recipient_tx_call.address != recipient_address
        || receive_call.entry_point != EntryPointBuf(b"receiveShieldedTest".to_vec())
        || send_call.entry_point != EntryPointBuf(NAME.as_bytes().to_vec())
        || recipient_tx_call.entry_point != EntryPointBuf(RECIPIENT.as_bytes().to_vec())
        || receive_tx_proof == &receive_standalone
        || send_tx_proof == &standalone
        || recipient_tx_proof == &recipient_standalone
        || calls.next().is_some()
        || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err("original immediate contract three-call binding or fee differs".into());
    }
    v3::write_new(
        root.join(format!("{STEM}-receive-transaction.proof")),
        &receive_tx_proof.0,
    )?;
    v3::write_new(
        root.join(format!("{STEM}-transaction.proof")),
        &send_tx_proof.0,
    )?;
    v3::write_new(
        root.join(format!("{STEM}-recipient-receive-transaction.proof")),
        &recipient_tx_proof.0,
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
        return Err(format!("original immediate contract strict apply failed: {result:?}").into());
    }
    if updated.zswap.first_free != ledger.zswap.first_free + 3
        || updated
            .zswap
            .coin_coms
            .index(sent_index)
            .map(|(hash, _)| hash)
            != Some(sent_commitment.0)
        || updated
            .zswap
            .coin_coms
            .index(change_index)
            .map(|(hash, _)| hash)
            != Some(change_commitment.0)
        || updated
            .zswap
            .coin_coms
            .index(transient_index)
            .map(|(hash, _)| hash)
            != Some(transient.coin_com.0)
        || !updated.zswap.nullifiers.contains_key(&transient.nullifier)
        || updated.contract.get(&address) != Some(&deploy.initial_state)
        || updated.contract.get(&recipient_address) != Some(&recipient_deploy.initial_state)
    {
        return Err(
            "original immediate contract post-Ledger output/nullifier/state differs".into(),
        );
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
        .sendImmediateShieldedToContractTest_call(
            wrong_bound.observed(),
            (),
            coin,
            compact_recipient,
            BoundedUint::new(60)?,
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
        "{NAME}: real wallet-funded singleton transient, three original calls, funded strict apply and replay passed"
    );
    Ok(())
}
