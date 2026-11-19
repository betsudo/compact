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

//! Pinned original qualified shielded send to a user: 60+40.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::{
    ObservedCallError, OfferBackedObservedState, ZswapIntentError,
};
use midnight_ledger::error::{EffectsCheckError, MalformedTransaction};
use midnight_zswap::{Input, Offer, Output};

const NAME: &str = "sendShieldedToUserTest";
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
    "/zkir/sendShieldedToUserTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/sendShieldedToUserTest.zkir", GENERATED_ZKIR),
    ] {
        if fs::read(root.join(member))? != expected {
            return Err(format!("original shielded user send artifact changed: {member}").into());
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original shielded user send requires local proving".into());
    }
    validate_artifacts(root)?;
    run_case(root, 60, "unshielded-original-send-shielded-user-change-60")?;
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

fn run_case(root: &Path, send_value: u128, stem: &str) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x0184_5354 + send_value as u64);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    let address = deploy.address();
    let recipient = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([0x37; 32]));
    let compact_recipient = crate::types::ZswapCoinPublicKey {
        bytes: FixedBytes::new(recipient.0.0),
    };
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    let resolver = super::qualified_coin_funding::fee_resolver(root, NAME)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        stem,
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
        let result = contract::sendShieldedToUserTest(
            preview,
            compact_coin(&info, first_free_before_seed)?,
            compact_recipient.clone(),
            BoundedUint::new(send_value)?,
        )?
        .result;
        if !result.change.is_some || result.change.value.value.value() != 40 {
            return Err("original partial user send change differs".into());
        }
        use runtime::ledger::CoinRecipient;
        let sent = output_info(&result.sent).commitment(&CoinRecipient::User(recipient));
        let change =
            output_info(&result.change.value).commitment(&CoinRecipient::Contract(address));
        if sent < change {
            chosen = Some((info, result));
            break;
        }
    }
    let (seed_info, preview_result) = chosen.ok_or("no canonical original user send nonce")?;
    let seed_output = Output::new_contract_owned(&mut rng, &seed_info, None, address)?;
    let seed_commitment = seed_output.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> = Offer::new(vec![], vec![seed_output], vec![])
        .ok_or("original user send seed output missing")?;
    let (seeded, seed_indices) = funded.ledger.zswap.try_apply(&seed_offer, None)?;
    let seed_index = *seed_indices
        .get(&seed_commitment)
        .ok_or("original user send seed index missing")?;
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
        return Err("original user send input is not contract owned".into());
    }
    let nullifier = input.nullifier;
    let qualified = compact_coin(&seed_info, seed_index)?;
    let mut preview =
        contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
    preview.set_zswap_output_start(ledger.zswap.first_free)?;
    let result = contract::sendShieldedToUserTest(
        preview,
        qualified.clone(),
        compact_recipient.clone(),
        BoundedUint::new(send_value)?,
    )?
    .result;
    if result != preview_result || result.sent.value.value() != send_value {
        return Err("original user send result changed after qualification".into());
    }
    let sent_output = Output::new(&mut rng, &output_info(&result.sent), None, &recipient, None)?;
    if sent_output.contract_address.is_some() {
        return Err("original user send output is contract owned".into());
    }
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
    let offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![input], outputs, vec![]).ok_or("original user send offer missing")?;
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let sent_index = *indices
        .get(&sent_commitment)
        .ok_or("original user send output index missing")?;
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
    let wrong_recipient = contract::Contract::default()
        .recording
        .sendShieldedToUserTest_call(
            bound.observed(),
            (),
            qualified.clone(),
            crate::types::ZswapCoinPublicKey {
                bytes: FixedBytes::new([0x38; 32]),
            },
            BoundedUint::new(send_value)?,
        );
    if !matches!(
        wrong_recipient,
        Err(runtime::CompactError::ZswapOfferOutputMismatch)
    ) {
        return Err("original user send accepted unmatched output key".into());
    }
    let call = contract::Contract::default()
        .recording
        .sendShieldedToUserTest_call(
            bound.observed(),
            (),
            qualified.clone(),
            compact_recipient.clone(),
            BoundedUint::new(send_value)?,
        )?;
    if call.recorded().execution.result != result {
        return Err("original user send recorded result differs".into());
    }
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    if prepared.prototype().op != operation
        || prepared.prototype().fallible_public_transcript.is_some()
    {
        return Err("original user send v3 operation or fallible transcript differs".into());
    }
    let effects = &prepared
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original user send transcript missing")?
        .effects;
    let expected_outputs = 2;
    if effects.claimed_nullifiers.size() != 1
        || !effects.claimed_nullifiers.member(&nullifier)
        || effects.claimed_shielded_spends.size() != expected_outputs
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
        return Err(format!("original shielded user send {send_value} Effects differ").into());
    }
    let mut wrong = qualified;
    wrong.mt_index = BoundedUint::new(sent_index.into())?;
    let wrong_call = contract::Contract::default()
        .recording
        .sendShieldedToUserTest_call(
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
        return Err("original user send accepted wrong Merkle index".into());
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
        .ok_or("original user send without output offer was accepted")?;
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
        return Err(format!("original user send no-offer refusal differs: {refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{stem}-without-offer-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
    let updated = v3::prove_and_apply_offer(
        root,
        (NAME, stem),
        prepared,
        &mut funded,
        &mut rng,
        &verifier,
        initial.ledger_state.get_ref(),
    )?;
    if updated
        .zswap
        .coin_coms
        .index(sent_index)
        .map(|(hash, owner)| (hash, owner.as_ref().map(|owner| **owner)))
        != Some((sent_commitment.0, None))
        || change_index
            .zip(change_commitment)
            .is_none_or(|(index, commitment)| {
                updated
                    .zswap
                    .coin_coms
                    .index(index)
                    .map(|(hash, owner)| (hash, owner.as_ref().map(|owner| **owner)))
                    != Some((commitment.0, Some(address)))
            })
        || !updated.zswap.nullifiers.contains_key(&nullifier)
        || updated.zswap.first_free != ledger.zswap.first_free + expected_outputs as u64
    {
        return Err("original user send post-ledger output/nullifier differs".into());
    }
    println!(
        "{NAME}: original send {send_value} from 100, one user output and 40 self change, funded strict apply and replay passed"
    );
    Ok(())
}
