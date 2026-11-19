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

// Licensed under the Apache License, Version 2.0 (the "License");

//! Original shielded mint-to-contract Effects and funded Ledger 9 qualification.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::OfferBackedObservedState;
use midnight_ledger::error::{EffectsCheckError, MalformedTransaction};
use midnight_ledger::structure::ProofVersioned;
use midnight_transient_crypto::proofs::Proof;
use midnight_zswap::{Offer, Output};

const NAME: &str = "mintShieldedToContractTest";
const STEM: &str = "unshielded-original-mint-shielded-contract-100";
const RECIPIENT: &str = "receiveShieldedTest";
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
    "/zkir/mintShieldedToContractTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/mintShieldedToContractTest.zkir", GENERATED_ZKIR),
    ] {
        if fs::read(root.join(member))? != expected {
            return Err(format!("original shielded mint artifact changed: {member}").into());
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original shielded mint requires local proving".into());
    }
    validate_artifacts(root)?;
    let mut rng = StdRng::seed_from_u64(0x0184_5343);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let (_, _, recipient_operation) = v3::load_operation(root, RECIPIENT)?;
    let recipient_deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        RECIPIENT,
        recipient_operation,
        &mut rng,
    );
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    if recipient_deploy.address() == deploy.address() {
        return Err("original shielded mint recipient equals minter".into());
    }
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 3)?;
    let resolver = super::qualified_coin_funding::fee_resolver(root, NAME)?;
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
        &format!("{STEM}-minter"),
        &deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    let nonce = [0x55; 32];
    let coin = runtime::ledger::CoinInfo {
        nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput(nonce)),
        type_: deploy.address().custom_shielded_token_type(domain),
        value: 100,
    };
    let output = Output::new_contract_owned(&mut rng, &coin, None, recipient_deploy.address())?;
    if output.contract_address.as_deref() != Some(&recipient_deploy.address()) {
        return Err("original shielded mint output targets another contract".into());
    }
    let commitment = output.coin_com;
    let offer = Offer::new(vec![], vec![output], vec![]).ok_or("shielded mint output missing")?;
    let (_, indices) = funded.ledger.zswap.try_apply(&offer, None)?;
    let index = *indices
        .get(&commitment)
        .ok_or("shielded mint output index missing")?;
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let bound = OfferBackedObservedState::new(observed, &funded.ledger, offer.clone())?;
    let call = contract::Contract::default()
        .recording
        .mintShieldedToContractTest_call(
            bound.observed(),
            (),
            FixedBytes::new(domain.0),
            BoundedUint::<18446744073709551615>::new(100)?,
            FixedBytes::new(nonce),
            crate::types::ContractAddress {
                bytes: runtime::ledger::contract_address_bytes(&recipient_deploy.address()),
            },
        )?;
    let result = call.recorded().execution.result.clone();
    if result.nonce != FixedBytes::new(nonce)
        || result.color != FixedBytes::new(coin.type_.0.0)
        || result.value.value() != 100
    {
        return Err("original shielded mint coin result differs".into());
    }
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    if prepared.prototype().op != operation {
        return Err("original shielded mint operation differs from installed v3 IR".into());
    }
    let transcript = prepared
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original shielded mint guaranteed transcript missing")?;
    let effects = &transcript.effects;
    if prepared.prototype().fallible_public_transcript.is_some()
        || effects.shielded_mints.size() != 1
        || effects.shielded_mints.get(&domain).map(|value| *value) != Some(100)
        || effects.claimed_shielded_spends.size() != 1
        || !effects.claimed_shielded_receives.is_empty()
        || !effects.claimed_shielded_spends.member(&commitment)
        || !effects.claimed_nullifiers.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err(
            "original shielded mint-to-contract Effects differ from 100-unit mint without auto-receive".into(),
        );
    }
    let without_offer: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared.prototype().clone());
    let without_offer =
        Transaction::from_intents("local-test", HashMap::new().insert(1_u16, without_offer));
    let mut strictness = WellFormedStrictness::default();
    strictness.enforce_balancing = false;
    let refusal = without_offer
        .well_formed(&funded.ledger, strictness, funded.time)
        .err()
        .ok_or("original shielded mint without output offer was accepted")?;
    let expected_refusal = matches!(
        &refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::AllCommitmentsSubsetCheckFailure(failure)
        ) if failure.superset.is_empty()
            && failure.subset == [(0, commitment)]
    );
    if !expected_refusal {
        return Err(format!("original shielded mint no-offer refusal differs: {refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-without-offer-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
    println!("{NAME}: missing-output commitment refusal passed");
    let mint_only: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared.prototype().clone());
    let mint_only = Transaction::new(
        "local-test",
        HashMap::new().insert(1_u16, mint_only),
        Some(offer.clone()),
        HashMap::new(),
    );
    let refusal = mint_only
        .well_formed(&funded.ledger, strictness, funded.time)
        .err()
        .ok_or("original shielded mint without recipient receive was accepted")?;
    if !matches!(
        &refusal,
        MalformedTransaction::EffectsCheckFailure(
            EffectsCheckError::CommitmentsNEClaimedShieldedReceives {
                commitments,
                claimed_shielded_receives,
            }
        ) if commitments == &[(0, commitment, recipient_deploy.address())]
            && claimed_shielded_receives.is_empty()
    ) {
        return Err(format!("original shielded mint-only refusal differs: {refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-without-recipient-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
    println!("{NAME}: contract output without recipient receive refused");
    let recipient_observed = ObservedContractState::new(
        recipient_deploy.address(),
        recipient_deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let recipient_bound =
        OfferBackedObservedState::new(recipient_observed, &funded.ledger, offer.clone())?;
    let receive = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(recipient_bound.observed(), (), result)?;
    let (_, recipient_verifier, recipient_operation) = v3::load_operation(root, RECIPIENT)?;
    let receive = recipient_bound.prepare(receive, recipient_verifier.clone(), Fr::from(0_u64))?;
    let receive_effects = &receive
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original shielded recipient has no guaranteed transcript")?
        .effects;
    if receive.prototype().op != recipient_operation
        || receive.prototype().fallible_public_transcript.is_some()
        || receive_effects.claimed_shielded_receives.size() != 1
        || !receive_effects
            .claimed_shielded_receives
            .member(&commitment)
        || !receive_effects.claimed_shielded_spends.is_empty()
        || !receive_effects.shielded_mints.is_empty()
        || !receive_effects.claimed_nullifiers.is_empty()
        || !receive_effects.unshielded_mints.is_empty()
        || !receive_effects.unshielded_inputs.is_empty()
        || !receive_effects.unshielded_outputs.is_empty()
        || !receive_effects.claimed_unshielded_spends.is_empty()
        || !receive_effects.claimed_contract_calls.is_empty()
    {
        return Err("original shielded recipient receive Effects differ".into());
    }
    let mint_standalone =
        v3::prove_standalone(root, NAME, STEM, prepared.prototype(), &verifier, None)?;
    super::positional_transaction::verify_separately(root, NAME, STEM)?;
    let receive_stem = format!("{STEM}-recipient-receive");
    let receive_standalone = v3::prove_standalone(
        root,
        RECIPIENT,
        &receive_stem,
        receive.prototype(),
        &recipient_verifier,
        None,
    )?;
    super::positional_transaction::verify_separately(root, RECIPIENT, &receive_stem)?;
    prove_original_receive_self_color(
        root,
        &funded,
        &mut rng,
        &recipient_deploy,
        &recipient_verifier,
        &recipient_operation,
        domain,
    )?;
    let updated = apply_two_contract(
        root,
        &mut funded,
        &mut rng,
        offer,
        prepared.prototype().clone(),
        receive.prototype().clone(),
        (&deploy, &recipient_deploy),
        (&mint_standalone, &receive_standalone),
    )?;
    if updated.zswap.first_free != index + 1
        || updated.zswap.coin_coms.index(index).map(|(hash, _)| hash) != Some(commitment.0)
        || updated.contract.get(&recipient_deploy.address())
            != Some(&recipient_deploy.initial_state)
    {
        return Err("original shielded mint output commitment or index changed".into());
    }
    println!(
        "{NAME}: original mint100, matching recipient contract output without auto-receive, funded strict apply and replay passed"
    );
    Ok(())
}

fn prove_original_receive_self_color(
    root: &Path,
    funded: &midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    recipient: &ContractDeploy<DefaultDB>,
    verifier: &VerifierKey,
    operation: &ContractOperation,
    domain: runtime::ledger::HashOutput,
) -> Result<(), Box<dyn Error>> {
    let nonce = [0x56; 32];
    let coin = runtime::ledger::CoinInfo {
        nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput(nonce)),
        type_: recipient.address().custom_shielded_token_type(domain),
        value: 100,
    };
    let output = Output::new_contract_owned(rng, &coin, None, recipient.address())?;
    let commitment = output.coin_com;
    if output.contract_address.as_deref() != Some(&recipient.address()) {
        return Err("original receive self-color output recipient differs".into());
    }
    let offer = Offer::new(vec![], vec![output], vec![])
        .ok_or("original receive self-color output missing")?;
    let observed = ObservedContractState::new(
        recipient.address(),
        recipient.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let bound = OfferBackedObservedState::new(observed, &funded.ledger, offer)?;
    let call = contract::Contract::default()
        .recording
        .receiveShieldedTest_call(
            bound.observed(),
            (),
            crate::types::ShieldedCoinInfo {
                nonce: FixedBytes::new(nonce),
                color: FixedBytes::new(coin.type_.0.0),
                value: BoundedUint::<340282366920938463463374607431768211455>::new(100)?,
            },
        )?;
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    let effects = &prepared
        .prototype()
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original self-color receive transcript missing")?
        .effects;
    if &prepared.prototype().op != operation
        || prepared.prototype().fallible_public_transcript.is_some()
        || effects.claimed_shielded_receives.size() != 1
        || !effects.claimed_shielded_receives.member(&commitment)
        || !effects.claimed_shielded_spends.is_empty()
        || !effects.shielded_mints.is_empty()
        || !effects.claimed_nullifiers.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err("original self-color receive Effects differ from TypeScript fixture".into());
    }
    let stem = format!("{STEM}-recipient-self-color");
    v3::prove_standalone(root, RECIPIENT, &stem, prepared.prototype(), verifier, None)?;
    super::positional_transaction::verify_separately(root, RECIPIENT, &stem)?;
    println!("{RECIPIENT}: original self-color receive Effects and proof passed");
    Ok(())
}

fn apply_two_contract(
    root: &Path,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    offer: Offer<ProofPreimage, DefaultDB>,
    mint: ContractCallPrototype<DefaultDB>,
    receive: ContractCallPrototype<DefaultDB>,
    (minter, recipient): (&ContractDeploy<DefaultDB>, &ContractDeploy<DefaultDB>),
    (mint_standalone, receive_standalone): (&Proof, &Proof),
) -> Result<LedgerState<DefaultDB>, Box<dyn Error>> {
    let resolver =
        super::qualified_coin_funding::fee_resolver_for_circuits(root, &[NAME, RECIPIENT])?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(mint)
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
        .map_err(|error| format!("two-contract shielded mint proving failed: {error:?}"))?;
    let sealed = proved.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut calls = sealed.calls();
    let (mint_segment, mint_call) = calls
        .next()
        .ok_or("shielded mint transaction has no mint call")?;
    let ProofVersioned::V3(mint_proof) = &mint_call.proof else {
        return Err("shielded mint transaction proof is not v3".into());
    };
    if mint_call.address != minter.address()
        || mint_call.entry_point != EntryPointBuf(NAME.as_bytes().to_vec())
        || mint_proof == mint_standalone
    {
        return Err("shielded mint transaction call differs from source or standalone".into());
    }
    let (receive_segment, receive_call) = calls
        .next()
        .ok_or("shielded mint transaction has no recipient call")?;
    let ProofVersioned::V3(receive_proof) = &receive_call.proof else {
        return Err("shielded recipient transaction proof is not v3".into());
    };
    if receive_segment != mint_segment
        || receive_call.address != recipient.address()
        || receive_call.entry_point != EntryPointBuf(RECIPIENT.as_bytes().to_vec())
        || receive_proof == receive_standalone
        || calls.next().is_some()
        || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err("shielded mint transaction recipient, segment, or funded fee differs".into());
    }
    v3::write_new(
        root.join(format!("{STEM}-transaction.proof")),
        &mint_proof.0,
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
    let result = funded.apply(&sealed, strictness)?;
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("shielded mint funded strict apply failed: {result:?}").into());
    }
    let updated = funded.ledger.clone();
    if updated.contract.get(&minter.address()) != Some(&minter.initial_state)
        || updated.contract.get(&recipient.address()) != Some(&recipient.initial_state)
    {
        return Err("shielded mint changed either deployed contract state".into());
    }
    v3::check_offer_replay(
        root,
        STEM,
        &updated,
        &verified,
        TransactionContext {
            ref_state: before,
            block_context: BlockContext {
                tblock: time,
                last_block_time: time,
                ..BlockContext::default()
            },
            whitelist: None,
        },
    )?;
    Ok(updated)
}
