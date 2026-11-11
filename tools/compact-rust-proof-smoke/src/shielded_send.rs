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

//! Prove unchanged qualified sendShielded and apply an exact funded offer.
use super::local_v3_application as v3;
use super::observational_binding::ProofBoundCall;
use super::positional_transaction::verify_separately;
use super::qualified_coin_funding::{fee_funded_state_with_utxos, fee_resolver};
use super::*;
use compact_rust_shielded_send_oracle_fixture::{ledger_contract as contract, types};
use midnight_base_crypto::hash::persistent_hash;
use midnight_compact_runtime as runtime;
use midnight_ledger::{structure::ProofVersioned, test_utilities::tx_prove};
use midnight_zswap::{Input, Offer, Output};
use runtime::transaction::{ObservedCallError, OfferBackedObservedState, ZswapIntentError};

const NAME: &str = "send_to_self";

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    run_case(root, false)
}
pub(super) fn run_observational(root: &Path) -> Result<(), Box<dyn Error>> {
    run_case(root, true)
}
fn run_case(root: &Path, observation_policy: bool) -> Result<(), Box<dyn Error>> {
    // Pair the complete generated source before deployment, keys or funding.
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        compact_rust_shielded_send_oracle_fixture::RETAINED_IR,
    )?;
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("shielded send requires local proving without MIDNIGHT_PROOF_SERVER".into());
    }
    for (path, expected) in [
        (
            "contract/lib.rs",
            "0208ef2a7eac22dbb82413651501ab78d7cbd0fd1ab6fb7ec8938b2095b00922",
        ),
        (
            "zkir/send_to_self.zkir",
            "8af8fdfcd1c11a1813ea0933cde79968b634848a0164c2c417d11d6d19868fc9",
        ),
    ] {
        if hex::encode(persistent_hash(&std::fs::read(root.join(path))?).0) != expected {
            return Err(format!("shielded send source/proof artifact differs: {path}").into());
        }
    }
    let (ir, verifier, operation) = v3::operation(root, NAME, 15)?;
    if ir.model().rows() != 23_445 {
        return Err("shielded send v3 row count differs".into());
    }
    let stem = if observation_policy {
        "observational-send-full"
    } else {
        "shielded-send-full"
    };
    let mut rng = StdRng::seed_from_u64(0x0203_0001);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation,
        &mut rng,
    );
    let address = deploy.address();
    let mut fee_state = fee_funded_state_with_utxos(&mut rng, 3)?;
    let resolver = fee_resolver(root, NAME)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        stem,
        &deploy,
        &mut fee_state,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let time = fee_state.time;
    let seed_info = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([1; 32]),
        FixedBytes::new([2; 32]),
        42,
    );
    let seed_output = Output::new_contract_owned(&mut rng, &seed_info, None, address)?;
    let seed_commitment = seed_output.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![seed_output], vec![]).ok_or("empty seed")?;
    let (seeded, indices) = fee_state.ledger.zswap.try_apply(&seed_offer, None)?;
    let seed_index = *indices
        .get(&seed_commitment)
        .ok_or("missing seeded index")?;
    fee_state.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(time, fee_state.ledger.parameters.global_ttl),
    );
    let ledger = fee_state.ledger.clone();
    let input = Input::new_contract_owned(
        &mut rng,
        &seed_info.qualify(seed_index),
        None,
        address,
        &ledger.zswap.coin_coms,
    )?;
    let nullifier = input.nullifier;
    let compact_input = types::QualifiedShieldedCoinInfo {
        nonce: FixedBytes::new([1; 32]),
        color: FixedBytes::new([2; 32]),
        value: BoundedUint::new(42)?,
        mt_index: BoundedUint::new(seed_index.into())?,
    };
    let observed = super::shielded_receive::observed(address, deploy.initial_state.clone());
    let mut preview =
        contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
    preview.set_zswap_output_start(ledger.zswap.first_free)?;
    let native = contract::send_to_self(preview, compact_input.clone(), BoundedUint::new(42)?)?;
    assert!(!native.result.change.is_some);
    let sent = &native.result.sent;
    let sent_info =
        runtime::ledger::coin_info_from_compact(sent.nonce, sent.color, sent.value.value());
    let output = Output::new_contract_owned(&mut rng, &sent_info, None, address)?;
    let commitment = output.coin_com;
    let offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![input], vec![output], vec![]).ok_or("empty send")?;
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let index = *indices.get(&commitment).ok_or("missing sent index")?;
    let bound = OfferBackedObservedState::new(observed, &ledger, offer.clone())?;
    let bound_native = contract::send_to_self(
        bound.observed().circuit_context(()),
        compact_input.clone(),
        BoundedUint::new(42)?,
    )?;
    assert_eq!(bound_native.result, native.result);
    assert_eq!(
        bound_native.context.circuit_zswap().inputs(),
        native.context.circuit_zswap().inputs()
    );
    assert_eq!(
        bound_native.context.circuit_zswap().outputs(),
        native.context.circuit_zswap().outputs()
    );
    assert_eq!(
        bound_native.context.circuit_zswap().next_index(),
        native.context.circuit_zswap().next_index()
    );
    let call = contract::Contract::default()
        .recording()
        .send_to_self_call(
            bound.observed(),
            (),
            compact_input.clone(),
            BoundedUint::new(42)?,
        )?;
    assert_eq!(call.recorded().execution.result, bound_native.result);
    assert_eq!(
        call.recorded().execution.context.query.effects,
        bound_native.context.query.effects
    );
    assert_eq!(
        call.recorded().execution.context.circuit_zswap(),
        bound_native.context.circuit_zswap()
    );
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    let mut wrong = compact_input.clone();
    wrong.mt_index = BoundedUint::new(index.into())?;
    let wrong_call = contract::Contract::default()
        .recording()
        .send_to_self_call(bound.observed(), (), wrong, BoundedUint::new(42)?)?;
    assert!(matches!(
        bound.prepare(wrong_call, verifier.clone(), Fr::from(0_u64)),
        Err(ObservedCallError::ZswapIntent(
            ZswapIntentError::InputIndexMismatch
        ))
    ));
    let partial = contract::Contract::default().recording().send_to_self_call(
        bound.observed(),
        (),
        compact_input.clone(),
        BoundedUint::new(17)?,
    );
    assert!(matches!(
        partial,
        Err(runtime::CompactError::ZswapOfferOutputMismatch)
    ));
    let updated = if observation_policy {
        use runtime::transaction::{ObservationalOfferBackedState, OfferAdmission};
        let observed = super::shielded_receive::observed(address, deploy.initial_state.clone());
        let checkpoint = super::observational_binding::checkpoint(&ledger, &observed)?;
        let binding =
            ObservationalOfferBackedState::bind(observed, checkpoint, offer.clone(), None)?;
        let call = contract::Contract::default()
            .recording()
            .send_to_self_call(binding.observed(), (), compact_input, BoundedUint::new(42)?)?;
        let observed_prepared = binding.prepare(call, verifier.clone(), Fr::from(0))?;
        assert!(matches!(
            observed_prepared.admission(),
            OfferAdmission::TrustedObservation(_)
        ));
        super::observational_binding::same_preimage(
            prepared.prototype(),
            observed_prepared.prototype(),
        )?;
        prove_and_apply(
            root,
            stem,
            observed_prepared,
            &mut fee_state,
            &mut rng,
            &verifier,
            bound_native.context.query.state.get_ref(),
        )?
    } else {
        prove_and_apply(
            root,
            stem,
            prepared,
            &mut fee_state,
            &mut rng,
            &verifier,
            bound_native.context.query.state.get_ref(),
        )?
    };
    assert_eq!(
        updated.zswap.coin_coms.index(index).map(|(hash, _)| hash),
        Some(commitment.0)
    );
    assert_eq!(updated.zswap.first_free, index + 1);
    assert!(updated.zswap.nullifiers.contains_key(&nullifier));
    assert!(matches!(updated.zswap.try_apply(&offer, None),
        Err(midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(found)) if found==nullifier));
    if !observation_policy {
        partial_order_cases(
            root,
            &mut fee_state,
            address,
            deploy.initial_state.clone(),
            &verifier,
            &mut rng,
        )?;
    }
    println!(
        "qualified full self send: exact offer, call proof, default strict Dust-funded ledger apply and nullifier replay rejection passed"
    );
    Ok(())
}

fn prove_and_apply(
    root: &Path,
    stem: &str,
    prepared: impl ProofBoundCall,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    verifier: &VerifierKey,
    expected: &StateValue<DefaultDB>,
) -> Result<LedgerState<DefaultDB>, Box<dyn Error>> {
    let prototype = prepared.prototype();
    let address = prototype.address;
    let entry = EntryPointBuf(NAME.as_bytes().to_vec());
    let ledger = funded.ledger.clone();
    let operation = ledger
        .contract
        .get(&address)
        .and_then(|state| state.operations.get(&entry))
        .ok_or("shielded send has no installed operation")?;
    if prototype.entry_point != entry || prototype.op != *operation {
        return Err("shielded send prototype differs from the installed v3 operation".into());
    }
    let standalone = v3::prove_standalone(root, NAME, stem, prototype, verifier, None)?;
    verify_separately(root, NAME, stem)?;

    let time = funded.time;
    let tx = prepared.into_test_transaction(rng, Timestamp::from_secs(time.to_secs() + 3600));
    let resolver = fee_resolver(root, NAME)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let proved = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("{stem} Ledger 9 transaction proving failed: {error:?}"))?;
    let sealed = proved.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut calls = sealed.calls();
    let (_, call) = calls
        .next()
        .ok_or("shielded send transaction has no call")?;
    let ProofVersioned::V3(transaction_proof) = &call.proof else {
        return Err("shielded send transaction proof is not v3".into());
    };
    if calls.next().is_some()
        || transaction_proof == &standalone
        || v3::positive_fee_dust_spends(&sealed) == 0
    {
        return Err("shielded send transaction proof or Dust spend differs".into());
    }
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    v3::write_new(
        root.join(format!("{stem}-transaction.bin")),
        &transaction_bytes,
    )?;
    v3::write_new(
        root.join(format!("{stem}-transaction.proof")),
        &transaction_proof.0,
    )?;

    let verified = sealed.well_formed(&ledger, WellFormedStrictness::default(), time)?;
    let context = TransactionContext {
        ref_state: ledger.clone(),
        block_context: BlockContext {
            tblock: time,
            last_block_time: time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, outcome) = ledger.apply(&verified, &context);
    if !matches!(outcome, TransactionResult::Success(_)) {
        return Err(format!("{stem} strict apply failed: {outcome:?}").into());
    }
    let state = updated
        .contract
        .get(&address)
        .ok_or("shielded send contract missing after application")?;
    if state.data.get_ref() != expected || state.operations.get(&entry) != Some(operation) {
        return Err("shielded send changed the expected state or installed operation".into());
    }
    let (replayed, replay_result) = updated.apply(
        &verified,
        &TransactionContext {
            ref_state: updated.clone(),
            ..context
        },
    );
    if !matches!(
        replay_result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != updated
    {
        return Err(format!("{stem} replay differs or mutated ledger: {replay_result:?}").into());
    }
    let mut after = Vec::new();
    let mut replay = Vec::new();
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    if replay != after {
        return Err("shielded send replay changed serialized ledger bytes".into());
    }
    v3::write_new(root.join(format!("{stem}-ledger-after.bin")), &after)?;
    v3::write_new(root.join(format!("{stem}-ledger-replay.bin")), &replay)?;
    println!("{stem}: funded default-strict apply; replay refused without mutation");
    Ok(updated)
}

fn partial_order_cases(
    root: &Path,
    fee_state: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    address: runtime::ledger::ContractAddress,
    contract_state: ContractState<DefaultDB>,
    verifier: &VerifierKey,
    rng: &mut StdRng,
) -> Result<(), Box<dyn Error>> {
    use runtime::ledger::CoinRecipient;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(fee_state.give_fee_token(rng, 10));
    let info = |byte| {
        runtime::ledger::coin_info_from_compact(
            FixedBytes::new([byte; 32]),
            FixedBytes::new([2; 32]),
            42,
        )
    };
    let mut same = None;
    let mut reversed = None;
    for byte in 3_u8..=96 {
        let qualified = types::QualifiedShieldedCoinInfo {
            nonce: FixedBytes::new([byte; 32]),
            color: FixedBytes::new([2; 32]),
            value: BoundedUint::new(42)?,
            mt_index: BoundedUint::new(0)?,
        };
        let preview =
            contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
        let result = contract::send_to_self(preview, qualified, BoundedUint::new(17)?)?.result;
        let sent = runtime::ledger::coin_info_from_compact(
            result.sent.nonce,
            result.sent.color,
            result.sent.value.value(),
        );
        let change = runtime::ledger::coin_info_from_compact(
            result.change.value.nonce,
            result.change.value.color,
            result.change.value.value.value(),
        );
        let sent_com = sent.commitment(&CoinRecipient::Contract(address));
        let change_com = change.commitment(&CoinRecipient::Contract(address));
        if sent_com < change_com && same.is_none() {
            same = Some(byte);
        }
        if sent_com > change_com && reversed.is_none() {
            reversed = Some(byte);
        }
        if same.is_some() && reversed.is_some() {
            break;
        }
    }
    let same = same.ok_or("no same-order partial nonce")?;
    let reversed = reversed.ok_or("no reverse-order partial nonce")?;
    let time = fee_state.time;
    let same_seed = Output::new_contract_owned(rng, &info(same), None, address)?;
    let same_com = same_seed.coin_com;
    let reverse_seed = Output::new_contract_owned(rng, &info(reversed), None, address)?;
    let reverse_com = reverse_seed.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![same_seed, reverse_seed], vec![]).ok_or("empty partial seed")?;
    let (seeded, seed_indices) = fee_state.ledger.zswap.try_apply(&seed_offer, None)?;
    fee_state.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(time, fee_state.ledger.parameters.global_ttl),
    );
    let ledger = fee_state.ledger.clone();
    for (byte, seed_com, same_order) in [(same, same_com, true), (reversed, reverse_com, false)] {
        let seed_index = *seed_indices
            .get(&seed_com)
            .ok_or("partial seed index missing")?;
        let input = Input::new_contract_owned(
            rng,
            &info(byte).qualify(seed_index),
            None,
            address,
            &ledger.zswap.coin_coms,
        )?;
        assert_eq!(input.contract_address.as_deref(), Some(&address));
        let nullifier = input.nullifier;
        let qualified = types::QualifiedShieldedCoinInfo {
            nonce: FixedBytes::new([byte; 32]),
            color: FixedBytes::new([2; 32]),
            value: BoundedUint::new(42)?,
            mt_index: BoundedUint::new(seed_index.into())?,
        };
        let mut preview =
            contract::initial_state(ConstructorContext::new(()))?.into_circuit_context(address);
        preview.set_zswap_output_start(ledger.zswap.first_free)?;
        let native = contract::send_to_self(preview, qualified.clone(), BoundedUint::new(17)?)?;
        assert!(native.result.change.is_some);
        assert_eq!(native.result.change.value.value.value(), 25);
        let sent = runtime::ledger::coin_info_from_compact(
            native.result.sent.nonce,
            native.result.sent.color,
            native.result.sent.value.value(),
        );
        let change = runtime::ledger::coin_info_from_compact(
            native.result.change.value.nonce,
            native.result.change.value.color,
            native.result.change.value.value.value(),
        );
        let sent_output = Output::new_contract_owned(rng, &sent, None, address)?;
        let sent_com = sent_output.coin_com;
        let change_output = Output::new_contract_owned(rng, &change, None, address)?;
        let change_com = change_output.coin_com;
        assert_eq!(sent_output.contract_address.as_deref(), Some(&address));
        assert_eq!(change_output.contract_address.as_deref(), Some(&address));
        let offer: Offer<ProofPreimage, DefaultDB> =
            Offer::new(vec![input], vec![sent_output, change_output], vec![])
                .ok_or("empty partial offer")?;
        assert_eq!(
            offer
                .outputs
                .get(0)
                .ok_or("missing normalized output")?
                .coin_com
                == sent_com,
            same_order
        );
        let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
        let sent_index = *indices.get(&sent_com).ok_or("missing sent index")?;
        let change_index = *indices.get(&change_com).ok_or("missing change index")?;
        let bound = OfferBackedObservedState::new(
            super::shielded_receive::observed(address, contract_state.clone()),
            &ledger,
            offer.clone(),
        )?;
        let call = contract::Contract::default().recording().send_to_self_call(
            bound.observed(),
            (),
            qualified,
            BoundedUint::new(17)?,
        );
        if !same_order {
            assert!(matches!(
                call,
                Err(runtime::CompactError::ZswapOfferOutputMismatch)
            ));
            println!("partial reverse-order offer rejected by exact output binding");
            continue;
        }
        let call = call?;
        assert_eq!(call.recorded().execution.result, native.result);
        assert_eq!(
            call.recorded().execution.context.circuit_zswap().outputs(),
            native.context.circuit_zswap().outputs()
        );
        let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
        let updated = prove_and_apply(
            root,
            "shielded-send-partial",
            prepared,
            fee_state,
            rng,
            verifier,
            native.context.query.state.get_ref(),
        )?;
        assert_eq!(
            updated.contract.get(&address).unwrap().data.get_ref(),
            contract_state.data.get_ref()
        );
        assert_eq!(
            updated
                .zswap
                .coin_coms
                .index(sent_index)
                .map(|(hash, _)| hash),
            Some(sent_com.0)
        );
        assert_eq!(
            updated
                .zswap
                .coin_coms
                .index(change_index)
                .map(|(hash, _)| hash),
            Some(change_com.0)
        );
        assert_eq!(updated.zswap.first_free, ledger.zswap.first_free + 2);
        assert!(updated.zswap.nullifiers.contains_key(&nullifier));
        assert!(matches!(updated.zswap.try_apply(&offer, None),
            Err(midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(found)) if found==nullifier));
        println!(
            "partial same-order send: two-output proof/default-strict apply/replay passed at {sent_index},{change_index}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod source_binding_tests {
    use super::*;

    #[test]
    fn complete_source_pairing_accepts_exact_ir() {
        let retained = compact_rust_shielded_send_oracle_fixture::RETAINED_IR;
        counter_scenario::validate_ir(retained.as_bytes(), retained).unwrap();
    }

    #[test]
    fn direct_and_observational_paths_refuse_missing_or_changed_ir_before_setup() {
        let root = std::env::temp_dir().join(format!(
            "compact-shielded-send-source-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!root.exists());
        for observational in [false, true] {
            let error = run_case(&root, observational).unwrap_err();
            assert_eq!(
                error.downcast_ref::<io::Error>().unwrap().kind(),
                io::ErrorKind::NotFound
            );
            assert!(!root.exists());
        }
        std::fs::create_dir_all(root.join("contract")).unwrap();
        let mut changed: serde_json::Value =
            serde_json::from_str(compact_rust_shielded_send_oracle_fixture::RETAINED_IR).unwrap();
        changed["stateful_circuits"][0]["name"] = serde_json::json!("wrong-source");
        let path = root.join("contract/compact-rust-ir.json");
        let bytes = serde_json::to_vec(&changed).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        for observational in [false, true] {
            let error = run_case(&root, observational).unwrap_err();
            assert_eq!(
                error.to_string(),
                "Proof artifact IR differs from the generated source fixture"
            );
            assert!(!root.join("keys").exists());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
