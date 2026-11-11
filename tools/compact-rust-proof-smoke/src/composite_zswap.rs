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

//! Original zero-output composite and exact funded nonzero composite transfer.
use super::local_v3_application as v3;
use super::qualified_coin_funding::{fee_funded_state_with_utxos, fee_resolver};
use super::*;
use compact_rust_composite_zswap_transfer_oracle_fixture::{
    ledger_contract as transfer, types as transfer_types,
};
use compact_rust_stateful_struct_oracle_fixture::{
    ledger_contract as planned, types as planned_types,
};
use midnight_base_crypto::hash::persistent_hash;
use midnight_compact_runtime as runtime;
use midnight_ledger::test_utilities::TestState;
use midnight_zswap::{Input, Offer, Output};
use runtime::transaction::{OfferBackedObservedState, PrepareCallError, ZswapIntentError};
struct Witnesses;
macro_rules! witness_impl {
    ($contract:ident) => {
        impl $contract::Witnesses<Vec<u8>> for Witnesses {
            fn next_value(
                &self,
                ctx: WitnessContext<'_, Vec<u8>, $contract::LedgerView<'_>>,
                tag: BoundedUint<255>,
            ) -> (Vec<u8>, BoundedUint<18446744073709551615>) {
                let value = tag.value() * 10 + ctx.private_state.len() as u128;
                let mut private = ctx.private_state.clone();
                private.push(tag.value() as u8);
                (private, BoundedUint::new(value).unwrap())
            }
        }
    };
}
witness_impl!(planned);
witness_impl!(transfer);
fn observation() -> Observation {
    Observation {
        transaction_hash: [0; 32],
        block_hash: [0; 32],
        block_height: 0,
    }
}
fn validate_ir(bytes: &[u8], expected: &str) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value = serde_json::from_str(expected)?;
    let actual: serde_json::Value = serde_json::from_slice(bytes)?;
    if expected != actual {
        return Err("composite source/proof IR mismatch".into());
    }
    Ok(())
}
fn validate_artifacts(root: &Path, pins: &str) -> Result<(), Box<dyn Error>> {
    let pins: std::collections::BTreeMap<String, String> = serde_json::from_str(pins)?;
    let raw_count = std::fs::read_dir(root.join("zkir"))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "zkir"))
        .count();
    if raw_count != pins.keys().filter(|path| path.ends_with(".zkir")).count() {
        return Err("composite raw circuit inventory differs from the pinned fixture".into());
    }
    for (path, expected) in pins {
        let actual = hex::encode(persistent_hash(&std::fs::read(root.join(&path))?).0);
        if actual != expected {
            return Err(format!("composite source/proof artifact differs: {path}").into());
        }
    }
    Ok(())
}

pub(super) fn run(original: &Path, transfer_root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("composite Zswap requires local proving without MIDNIGHT_PROOF_SERVER".into());
    }
    // Bind both complete sources and artifacts before either scenario reads proving keys.
    validate_ir(
        &std::fs::read(original.join("contract/compact-rust-ir.json"))?,
        compact_rust_stateful_struct_oracle_fixture::RETAINED_IR,
    )?;
    validate_ir(
        &std::fs::read(transfer_root.join("contract/compact-rust-ir.json"))?,
        compact_rust_composite_zswap_transfer_oracle_fixture::RETAINED_IR,
    )?;
    validate_artifacts(
        original,
        include_str!("../../compact-rust-backend/tests/composite-planned-ledger9-artifacts.json"),
    )?;
    validate_artifacts(
        transfer_root,
        include_str!("../../compact-rust-backend/tests/composite-transfer-ledger9-artifacts.json"),
    )?;
    original_planned(original)?;
    funded_transfer(transfer_root)
}
fn deploy_funded(
    root: &Path,
    (name, stem): (&'static str, &str),
    data: StateValue<DefaultDB>,
    rng: &mut StdRng,
) -> Result<(ContractDeploy<DefaultDB>, VerifierKey, TestState<DefaultDB>), Box<dyn Error>> {
    let (_, verifier, operation) = v3::operation(root, name, 10)?;
    let deploy = v3::deploy(data, name, operation, rng);
    let mut state = fee_funded_state_with_utxos(rng, 2)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let resolver = fee_resolver(root, name)?;
    v3::apply_funded_deployment(root, stem, &deploy, &mut state, rng, &executor, &resolver)?;
    Ok((deploy, verifier, state))
}

fn original_planned(root: &Path) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x0191_0000);
    let initial = planned::initial_state(ConstructorContext::new(Vec::<u8>::new()))?;
    let circuit = ("planned", "composite-planned-zero");
    let (deploy, verifier, mut state) = deploy_funded(
        root,
        circuit,
        initial.ledger_state.get_ref().clone(),
        &mut rng,
    )?;
    let address = deploy.address();
    let generated = planned::Contract::from(Witnesses);
    let coin = planned_types::ShieldedCoinInfo {
        nonce: FixedBytes::new([7; 32]),
        color: FixedBytes::new([8; 32]),
        value: BoundedUint::new(0)?,
    };
    let recipient = planned_types::Either {
        is_left: true,
        left: planned_types::ZswapCoinPublicKey {
            bytes: FixedBytes::new([5; 32]),
        },
        right: planned_types::ContractAddress {
            bytes: FixedBytes::new([0; 32]),
        },
    };
    let native_coin = runtime::ledger::coin_info_from_compact(coin.nonce, coin.color, 0);
    let key = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([5; 32]));
    let output = Output::new(&mut rng, &native_coin, None, &key, None)?;
    let commitment = output.coin_com;
    let offer = Offer::new(vec![], vec![output], vec![]).ok_or("empty zero offer")?;
    let bound = OfferBackedObservedState::new(
        ObservedContractState::new(address, deploy.initial_state.clone(), observation()),
        &state.ledger,
        offer,
    )?;
    let call = generated.recording().planned_call(
        bound.observed(),
        vec![],
        coin.clone(),
        recipient.clone(),
    )?;
    assert_eq!(call.recorded().execution.result.first.value(), 30);
    assert_eq!(call.recorded().execution.result.after.value(), 41);
    assert_eq!(
        call.recorded().execution.result.address.bytes.0,
        address.0.0
    );
    assert_eq!(call.recorded().execution.context.private_state, vec![3, 4]);
    assert_eq!(
        call.recorded().execution.private_transcript_outputs.len(),
        3
    );
    assert_eq!(call.recorded().public.verify_ops().len(), 3);
    assert!(matches!(
        generated
            .recording()
            .planned_call(bound.observed(), vec![], coin.clone(), recipient.clone())?
            .prepare(verifier.clone(), Fr::from(0)),
        Err(ObservedCallError::Prepare(
            PrepareCallError::UnboundZswapIntents
        ))
    ));
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0))?;
    // A positive output-only amount has no source of shielded tokens.
    let nonzero = planned_types::ShieldedCoinInfo {
        value: BoundedUint::new(42)?,
        ..coin.clone()
    };
    let output = Output::new(
        &mut rng,
        &runtime::ledger::coin_info_from_compact(nonzero.nonce, nonzero.color, 42),
        None,
        &key,
        None,
    )?;
    let offer = Offer::new(vec![], vec![output], vec![]).ok_or("empty nonzero offer")?;
    let nonzero_bound = OfferBackedObservedState::new(
        ObservedContractState::new(address, deploy.initial_state.clone(), observation()),
        &state.ledger,
        offer,
    )?;
    let call = generated.recording().planned_call(
        nonzero_bound.observed(),
        vec![],
        nonzero.clone(),
        recipient.clone(),
    )?;
    let tx = nonzero_bound
        .prepare(call, verifier.clone(), Fr::from(0))?
        .into_transaction(
            &mut rng,
            "local-test",
            Timestamp::from_secs(state.time.to_secs() + 3600),
        );
    let error = tx
        .well_formed(&state.ledger, WellFormedStrictness::default(), state.time)
        .err()
        .ok_or("nonzero planned accepted")?;
    assert!(matches!(
        error,
        midnight_ledger::error::MalformedTransaction::BalanceCheckOverspend {
            segment: 0,
            overspent_value: -42,
            ..
        }
    ));
    // Ordinary wallet funding is deliberately outside the exact-input profile.
    let mut wallet_ledger = state.ledger.clone();
    let funded = super::qualified_coin_funding::seed_and_fund(
        &mut rng,
        &mut wallet_ledger,
        runtime::ledger::coin_info_from_compact(nonzero.nonce, nonzero.color, 42),
        key,
        state.time,
    )?;
    let wallet_bound = OfferBackedObservedState::new(
        ObservedContractState::new(address, deploy.initial_state.clone(), observation()),
        &wallet_ledger,
        funded.offer,
    )?;
    let call =
        generated
            .recording()
            .planned_call(wallet_bound.observed(), vec![], nonzero, recipient)?;
    assert!(matches!(
        wallet_bound.prepare(call, verifier.clone(), Fr::from(0)),
        Err(ObservedCallError::ZswapIntent(
            ZswapIntentError::InputMismatch
        ))
    ));
    // Contract recipients need an explicit receive claim even for value zero.
    let output = Output::new_contract_owned(&mut rng, &native_coin, None, address)?;
    let offer = Offer::new(vec![], vec![output], vec![]).ok_or("empty contract offer")?;
    let contract_bound = OfferBackedObservedState::new(
        ObservedContractState::new(address, deploy.initial_state.clone(), observation()),
        &state.ledger,
        offer,
    )?;
    let recipient = planned_types::Either {
        is_left: false,
        left: planned_types::ZswapCoinPublicKey {
            bytes: FixedBytes::new([0; 32]),
        },
        right: planned_types::ContractAddress {
            bytes: FixedBytes::new(address.0.0),
        },
    };
    let call =
        generated
            .recording()
            .planned_call(contract_bound.observed(), vec![], coin, recipient)?;
    let tx = contract_bound
        .prepare(call, verifier.clone(), Fr::from(0))?
        .into_transaction(
            &mut rng,
            "local-test",
            Timestamp::from_secs(state.time.to_secs() + 3600),
        );
    let error = tx
        .well_formed(&state.ledger, WellFormedStrictness::default(), state.time)
        .err()
        .ok_or("missing receive claim accepted")?;
    assert!(matches!(
        error,
        midnight_ledger::error::MalformedTransaction::EffectsCheckFailure(
            midnight_ledger::error::EffectsCheckError::CommitmentsNEClaimedShieldedReceives { .. }
        )
    ));
    println!(
        "planned negatives: exact -42 deficit, extra wallet input mismatch, missing receive claim under default strictness"
    );
    let updated = v3::prove_and_apply_offer(
        root,
        circuit,
        prepared,
        &mut state,
        &mut rng,
        &verifier,
        deploy.initial_state.data.get_ref(),
    )?;
    super::shielded_receive::check_received_output(&updated, commitment, 0)?;
    Ok(())
}
fn funded_transfer(root: &Path) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x0191_0042);
    let initial = transfer::initial_state(ConstructorContext::new(Vec::<u8>::new()))?;
    let circuit = ("transfer", "composite-transfer");
    let (deploy, verifier, mut state) = deploy_funded(
        root,
        circuit,
        initial.ledger_state.get_ref().clone(),
        &mut rng,
    )?;
    let address = deploy.address();
    let seed_coin = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([1; 32]),
        FixedBytes::new([2; 32]),
        42,
    );
    let seed_output = Output::new_contract_owned(&mut rng, &seed_coin, None, address)?;
    let seed_commitment = seed_output.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![seed_output], vec![]).ok_or("empty seed")?;
    let (seeded, indices) = state.ledger.zswap.try_apply(&seed_offer, None)?;
    let seed_index = *indices.get(&seed_commitment).ok_or("missing seed index")?;
    state.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(state.time, state.ledger.parameters.global_ttl),
    );
    let input = Input::new_contract_owned(
        &mut rng,
        &seed_coin.qualify(seed_index),
        None,
        address,
        &state.ledger.zswap.coin_coms,
    )?;
    let nullifier = input.nullifier;
    let coin = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([3; 32]),
        FixedBytes::new([2; 32]),
        42,
    );
    let key = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([5; 32]));
    let output = Output::new(&mut rng, &coin, None, &key, None)?;
    let commitment = output.coin_com;
    let offer = Offer::new(vec![input], vec![output], vec![]).ok_or("empty transfer")?;
    let bound = OfferBackedObservedState::new(
        ObservedContractState::new(address, deploy.initial_state.clone(), observation()),
        &state.ledger,
        offer.clone(),
    )?;
    let generated = transfer::Contract::from(Witnesses);
    let call = generated.recording().transfer_call(
        bound.observed(),
        vec![],
        transfer_types::QualifiedShieldedCoinInfo {
            nonce: FixedBytes::new([1; 32]),
            color: FixedBytes::new([2; 32]),
            value: BoundedUint::new(42)?,
            mt_index: BoundedUint::new(seed_index.into())?,
        },
        transfer_types::ShieldedCoinInfo {
            nonce: FixedBytes::new([3; 32]),
            color: FixedBytes::new([2; 32]),
            value: BoundedUint::new(42)?,
        },
        transfer_types::Either {
            is_left: true,
            left: transfer_types::ZswapCoinPublicKey {
                bytes: FixedBytes::new([5; 32]),
            },
            right: transfer_types::ContractAddress {
                bytes: FixedBytes::new([0; 32]),
            },
        },
        FixedBytes::new(nullifier.0.0),
        FixedBytes::new(commitment.0.0),
    )?;
    assert_eq!(call.recorded().execution.result.first.value(), 30);
    assert_eq!(call.recorded().execution.result.after.value(), 41);
    assert_eq!(
        call.recorded().execution.result.address.bytes.0,
        address.0.0
    );
    assert_eq!(call.recorded().execution.context.private_state, vec![3, 4]);
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0))?;
    let updated = v3::prove_and_apply_offer(
        root,
        circuit,
        prepared,
        &mut state,
        &mut rng,
        &verifier,
        deploy.initial_state.data.get_ref(),
    )?;
    super::shielded_receive::check_received_output(&updated, commitment, seed_index + 1)?;
    assert!(updated.zswap.nullifiers.contains_key(&nullifier));
    let error = updated
        .zswap
        .try_apply(&offer, None)
        .err()
        .ok_or("spent input replay accepted")?;
    assert!(
        matches!(error,midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(found) if found==nullifier)
    );
    println!(
        "composite transfer42: exact spent-nullifier replay rejection; same-frame Kernel.self and Unit members retained"
    );
    Ok(())
}

#[cfg(test)]
mod source_pairing_tests {
    use super::*;
    #[test]
    fn both_sources_are_bound_before_reading_keys_and_cannot_be_swapped() {
        let original = compact_rust_stateful_struct_oracle_fixture::RETAINED_IR;
        let transfer = compact_rust_composite_zswap_transfer_oracle_fixture::RETAINED_IR;
        validate_ir(original.as_bytes(), original).unwrap();
        validate_ir(transfer.as_bytes(), transfer).unwrap();
        assert!(validate_ir(original.as_bytes(), transfer).is_err());
        assert!(validate_ir(transfer.as_bytes(), original).is_err());
        for source in [original, transfer] {
            let mut changed: serde_json::Value = serde_json::from_str(source).unwrap();
            changed["stateful_circuits"][0]["name"] = "different".into();
            assert!(validate_ir(&serde_json::to_vec(&changed).unwrap(), source).is_err());
        }
    }
}
