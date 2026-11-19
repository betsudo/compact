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

//! Pinned original qualified shielded send to a distinct contract: 60+40.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::{
    ObservedCallError, OfferBackedObservedState, ZswapIntentError,
};
use midnight_ledger::error::{EffectsCheckError, MalformedTransaction};
use midnight_ledger::structure::ProofVersioned;
use midnight_transient_crypto::proofs::Proof;
use midnight_zswap::{Input, Offer, Output};

const NAME: &str = "sendShieldedToContractTest";
const RECIPIENT: &str = "receiveShieldedTest";
const STEM: &str = "unshielded-original-send-shielded-contract-change-60";
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
    "/zkir/sendShieldedToContractTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/sendShieldedToContractTest.zkir", GENERATED_ZKIR),
    ] {
        if fs::read(root.join(member))? != expected {
            return Err(
                format!("original shielded send to contract artifact changed: {member}").into(),
            );
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original shielded send to contract requires local proving".into());
    }
    validate_artifacts(root)?;
    run_case(root)?;
    Ok(())
}

fn compact_coin(
    info: &runtime::ledger::CoinInfo,
    index: u64,
) -> Result<crate::types::QualifiedShieldedCoinInfo, Box<dyn Error>> {
    Ok(crate::types::QualifiedShieldedCoinInfo {
        nonce: FixedBytes::new(info.nonce.0.0),
        color: FixedBytes::new(info.type_.0.0),
        value: BoundedUint::new(info.value)?,
        mt_index: BoundedUint::new(index.into())?,
    })
}

fn output_info(coin: &crate::types::ShieldedCoinInfo) -> runtime::ledger::CoinInfo {
    runtime::ledger::coin_info_from_compact(coin.nonce, coin.color, coin.value.value())
}

fn run_case(root: &Path) -> Result<(), Box<dyn Error>> {
    let send_value = 60_u128;
    let stem = STEM;
    let mut rng = StdRng::seed_from_u64(0x0184_5351);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let (_, recipient_verifier, recipient_operation) = v3::load_operation(root, RECIPIENT)?;
    let recipient_deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        RECIPIENT,
        recipient_operation.clone(),
        &mut rng,
    );
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    let address = deploy.address();
    let recipient_address = recipient_deploy.address();
    if address == recipient_address {
        return Err("original send recipient equals sender".into());
    }
    let compact_recipient = crate::types::ContractAddress {
        bytes: runtime::ledger::contract_address_bytes(&recipient_address),
    };
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 3)?;
    let resolver =
        super::qualified_coin_funding::fee_resolver_for_circuits(root, &[NAME, RECIPIENT])?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        &format!("{stem}-recipient"),
        &recipient_deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
    v3::apply_funded_deployment(
        root,
        &format!("{stem}-sender"),
        &deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    let first_free_before_seed = funded.ledger.zswap.first_free;
    let mut chosen = None;
    // An offer sorts outputs by commitment. Choose a fixed corpus nonce whose
    // generated sent/change order equals that canonical order.
    for byte in 1_u8..=96 {
        let info = runtime::ledger::CoinInfo {
            nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput([byte; 32])),
            type_: address.custom_shielded_token_type(domain),
            value: 100,
        };
        let mut preview =
            contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
        preview.set_zswap_output_start(first_free_before_seed + 1)?;
        let result = contract::sendShieldedToContractTest(
            preview,
            compact_coin(&info, first_free_before_seed)?,
            compact_recipient.clone(),
            BoundedUint::new(send_value)?,
        )?
        .result;
        if !result.change.is_some || result.change.value.value.value() != 40 {
            return Err("original partial send to contract change differs".into());
        }
        use runtime::ledger::CoinRecipient;
        let sent =
            output_info(&result.sent).commitment(&CoinRecipient::Contract(recipient_address));
        let change =
            output_info(&result.change.value).commitment(&CoinRecipient::Contract(address));
        if sent < change {
            chosen = Some((info, result));
            break;
        }
    }
    let (seed_info, preview_result) =
        chosen.ok_or("no canonical original send to contract nonce")?;
    let seed_output = Output::new_contract_owned(&mut rng, &seed_info, None, address)?;
    let seed_commitment = seed_output.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> = Offer::new(vec![], vec![seed_output], vec![])
        .ok_or("original send to contract seed output missing")?;
    let (seeded, seed_indices) = funded.ledger.zswap.try_apply(&seed_offer, None)?;
    let seed_index = *seed_indices
        .get(&seed_commitment)
        .ok_or("original send to contract seed index missing")?;
    funded.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(funded.time, funded.ledger.parameters.global_ttl),
    );
    let ledger = funded.ledger.clone();
    let input = Input::new_contract_owned(
        &mut rng,
        &seed_info.qualify(seed_index),
        None,
        address,
        &ledger.zswap.coin_coms,
    )?;
    if input.contract_address.as_deref() != Some(&address) {
        return Err("original send to contract input is not contract owned".into());
    }
    let nullifier = input.nullifier;
    let qualified = compact_coin(&seed_info, seed_index)?;
    let mut preview =
        contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
    preview.set_zswap_output_start(ledger.zswap.first_free)?;
    let result = contract::sendShieldedToContractTest(
        preview,
        qualified.clone(),
        compact_recipient.clone(),
        BoundedUint::new(send_value)?,
    )?
    .result;
    if result != preview_result || result.sent.value.value() != send_value {
        return Err("original send to contract result changed after qualification".into());
    }
    let sent_output = Output::new_contract_owned(
        &mut rng,
        &output_info(&result.sent),
        None,
        recipient_address,
    )?;
    if sent_output.contract_address.as_deref() != Some(&recipient_address) {
        return Err("original sent output does not belong to recipient contract".into());
    }
    let recipient_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![sent_output.clone()], vec![])
            .ok_or("recipient-only output offer missing")?;
    let sent_commitment = sent_output.coin_com;
    let mut outputs = vec![sent_output];
    let change_commitment = if result.change.is_some {
        let change_output = Output::new_contract_owned(
            &mut rng,
            &output_info(&result.change.value),
            None,
            address,
        )?;
        let commitment = change_output.coin_com;
        outputs.push(change_output);
        Some(commitment)
    } else {
        None
    };
    let offer: Offer<ProofPreimage, DefaultDB> = Offer::new(vec![input], outputs, vec![])
        .ok_or("original send to contract offer missing")?;
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let sent_index = *indices
        .get(&sent_commitment)
        .ok_or("original send to contract output index missing")?;
    let change_index = change_commitment
        .map(|commitment| {
            indices
                .get(&commitment)
                .copied()
                .ok_or("change output index missing")
        })
        .transpose()?;
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let bound = OfferBackedObservedState::new(observed, &ledger, offer.clone())?;
    let call = contract::Contract::default()
        .recording
        .sendShieldedToContractTest_call(
            bound.observed(),
            (),
            qualified.clone(),
            compact_recipient.clone(),
            BoundedUint::new(send_value)?,
        )?;
    if call.recorded().execution.result != result {
        return Err("original send to contract recorded result differs".into());
    }
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    if prepared.prototype().op != operation
        || prepared.prototype().fallible_public_transcript.is_some()
    {
        return Err("original send to contract v3 operation or fallible transcript differs".into());
    }
    let effects = &prepared
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original send to contract transcript missing")?
        .effects;
    if effects.claimed_nullifiers.size() != 1
        || !effects.claimed_nullifiers.member(&nullifier)
        || effects.claimed_shielded_spends.size() != 2
        || effects.claimed_shielded_receives.size() != 1
        || !effects.claimed_shielded_spends.member(&sent_commitment)
        || effects.claimed_shielded_receives.member(&sent_commitment)
        || change_commitment.is_none_or(|commitment| {
            !effects.claimed_shielded_spends.member(&commitment)
                || !effects.claimed_shielded_receives.member(&commitment)
        })
        || !effects.shielded_mints.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err(
            format!("original shielded send to contract {send_value} Effects differ").into(),
        );
    }
    let mut wrong = qualified;
    wrong.mt_index = BoundedUint::new(sent_index.into())?;
    let wrong_call = contract::Contract::default()
        .recording
        .sendShieldedToContractTest_call(
            bound.observed(),
            (),
            wrong,
            compact_recipient,
            BoundedUint::new(send_value)?,
        )?;
    if !matches!(
        bound.prepare(wrong_call, verifier.clone(), Fr::from(0_u64)),
        Err(ObservedCallError::ZswapIntent(
            ZswapIntentError::InputIndexMismatch
        ))
    ) {
        return Err("original send to contract accepted wrong Merkle index".into());
    }
    let without_offer: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared.prototype().clone());
    let without_offer =
        Transaction::from_intents("local-test", HashMap::new().insert(1_u16, without_offer));
    let mut strictness = WellFormedStrictness::default();
    strictness.enforce_balancing = false;
    let refusal = without_offer
        .well_formed(&ledger, strictness, funded.time)
        .err()
        .ok_or("original send to contract without output offer was accepted")?;
    let expected_refusal = matches!(
        &refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::NullifiersNEClaimedNullifiers {
                nullifiers,
                claimed_nullifiers,
            }
        ) if nullifiers.is_empty()
            && claimed_nullifiers.as_slice() == [(0, nullifier, address)]
    );
    if !expected_refusal {
        return Err(
            format!("original send to contract no-offer refusal differs: {refusal:?}").into(),
        );
    }
    v3::write_new(
        root.join(format!("{stem}-without-offer-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
    let sender_only: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared.prototype().clone());
    let sender_only = Transaction::new(
        "local-test",
        HashMap::new().insert(1_u16, sender_only),
        Some(offer.clone()),
        HashMap::new(),
    );
    let refusal = sender_only
        .well_formed(&ledger, strictness, funded.time)
        .err()
        .ok_or("original sender-only contract output was accepted")?;
    let expected_refusal = matches!(
        &refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::CommitmentsNEClaimedShieldedReceives {
                commitments,
                claimed_shielded_receives,
            }
        ) if commitments.len() == 2
            && commitments.contains(&(0, sent_commitment, recipient_address))
            && commitments.contains(&(0, change_commitment.ok_or("change missing")?, address))
            && claimed_shielded_receives.as_slice()
                == [(0, change_commitment.ok_or("change missing")?, address)]
    );
    if !expected_refusal {
        return Err(format!("original sender-only refusal differs: {refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{stem}-without-recipient-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
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
    let receive = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(recipient_bound.observed(), (), result.sent.clone())?;
    let receive = recipient_bound.prepare(receive, recipient_verifier.clone(), Fr::from(0_u64))?;
    let receive_effects = &receive
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original recipient receive transcript missing")?
        .effects;
    if receive.prototype().op != recipient_operation
        || receive.prototype().fallible_public_transcript.is_some()
        || receive_effects.claimed_shielded_receives.size() != 1
        || !receive_effects
            .claimed_shielded_receives
            .member(&sent_commitment)
        || !receive_effects.claimed_shielded_spends.is_empty()
        || !receive_effects.shielded_mints.is_empty()
        || !receive_effects.claimed_nullifiers.is_empty()
        || !receive_effects.unshielded_mints.is_empty()
        || !receive_effects.unshielded_inputs.is_empty()
        || !receive_effects.unshielded_outputs.is_empty()
        || !receive_effects.claimed_unshielded_spends.is_empty()
        || !receive_effects.claimed_contract_calls.is_empty()
    {
        return Err("original recipient receive Effects differ".into());
    }
    let sender_standalone =
        v3::prove_standalone(root, NAME, stem, prepared.prototype(), &verifier, None)?;
    super::positional_transaction::verify_separately(root, NAME, stem)?;
    let receive_stem = format!("{stem}-recipient-receive");
    let receive_standalone = v3::prove_standalone(
        root,
        RECIPIENT,
        &receive_stem,
        receive.prototype(),
        &recipient_verifier,
        None,
    )?;
    super::positional_transaction::verify_separately(root, RECIPIENT, &receive_stem)?;
    let updated = apply_two_contract(
        root,
        &mut funded,
        &mut rng,
        offer,
        prepared.prototype().clone(),
        receive.prototype().clone(),
        (&deploy, &recipient_deploy),
        (&sender_standalone, &receive_standalone),
    )?;
    if updated
        .zswap
        .coin_coms
        .index(sent_index)
        .map(|(hash, _)| hash)
        != Some(sent_commitment.0)
        || change_index
            .zip(change_commitment)
            .is_some_and(|(index, commitment)| {
                updated.zswap.coin_coms.index(index).map(|(hash, _)| hash) != Some(commitment.0)
            })
        || !updated.zswap.nullifiers.contains_key(&nullifier)
        || updated.zswap.first_free != ledger.zswap.first_free + 2
        || updated.contract.get(&recipient_address) != Some(&recipient_deploy.initial_state)
    {
        return Err("original send to contract post-ledger output/nullifier differs".into());
    }
    println!(
        "{NAME}: original send 60 from 100, one recipient and one change output, funded strict apply and replay passed"
    );
    Ok(())
}

fn apply_two_contract(
    root: &Path,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    offer: Offer<ProofPreimage, DefaultDB>,
    send: ContractCallPrototype<DefaultDB>,
    receive: ContractCallPrototype<DefaultDB>,
    (sender, recipient): (&ContractDeploy<DefaultDB>, &ContractDeploy<DefaultDB>),
    (send_standalone, receive_standalone): (&Proof, &Proof),
) -> Result<LedgerState<DefaultDB>, Box<dyn Error>> {
    let resolver =
        super::qualified_coin_funding::fee_resolver_for_circuits(root, &[NAME, RECIPIENT])?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(send)
            .add_call::<ProofPreimage>(receive);
    let tx = Transaction::new(
        "local-test",
        HashMap::new().insert(1_u16, intent),
        Some(offer),
        HashMap::new(),
    );
    let proved = executor
        .block_on(midnight_ledger::test_utilities::tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("two-contract shielded send proving failed: {error:?}"))?;
    let sealed = proved.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut calls = sealed.calls();
    let (send_segment, send_call) = calls
        .next()
        .ok_or("shielded contract-send transaction has no send call")?;
    let ProofVersioned::V3(send_proof) = &send_call.proof else {
        return Err("shielded contract-send transaction proof is not v3".into());
    };
    if send_call.address != sender.address()
        || send_call.entry_point != EntryPointBuf(NAME.as_bytes().to_vec())
        || send_proof == send_standalone
    {
        return Err(
            "shielded contract-send transaction call differs from source or standalone".into(),
        );
    }
    let (receive_segment, receive_call) = calls
        .next()
        .ok_or("shielded contract-send transaction has no recipient call")?;
    let ProofVersioned::V3(receive_proof) = &receive_call.proof else {
        return Err("shielded recipient transaction proof is not v3".into());
    };
    if receive_segment != send_segment
        || receive_call.address != recipient.address()
        || receive_call.entry_point != EntryPointBuf(RECIPIENT.as_bytes().to_vec())
        || receive_proof == receive_standalone
        || calls.next().is_some()
        || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err(
            "shielded contract-send transaction recipient, segment, or funded fee differs".into(),
        );
    }
    v3::write_new(
        root.join(format!("{STEM}-transaction.proof")),
        &send_proof.0,
    )?;
    v3::write_new(
        root.join(format!("{STEM}-recipient-receive-transaction.proof")),
        &receive_proof.0,
    )?;
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    v3::write_new(
        root.join(format!("{STEM}-transaction.bin")),
        &transaction_bytes,
    )?;
    let before = funded.ledger.clone();
    let time = funded.time;
    let strictness = WellFormedStrictness::default();
    let verified = sealed.well_formed(&before, strictness, time)?;
    // The historical input coin was inserted directly into the authoritative
    // ledger and block sealed. TestState's wallet view did not observe that
    // fixture insertion, so apply through the ledger rather than its wallet
    // event replay helper.
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
        return Err(format!("shielded send funded strict apply failed: {result:?}").into());
    }
    if updated.contract.get(&sender.address()) != Some(&sender.initial_state)
        || updated.contract.get(&recipient.address()) != Some(&recipient.initial_state)
    {
        return Err("shielded contract-send changed either deployed contract state".into());
    }
    v3::check_offer_replay(root, STEM, &updated, &verified, context)?;
    Ok(updated)
}
