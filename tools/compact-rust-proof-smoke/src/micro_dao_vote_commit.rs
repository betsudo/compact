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
//! Original vote_commit from explicitly seeded commit-phase/token state.
//! Full default-strict wallet/transient segment with separate Dust funding.
#[path = "../../../tests-rust-backend/test-center-micro-dao/support/vote_commit.rs"]
mod support;
use super::local_v3_application as v3;
use super::micro_dao_buy_in::{FundedMicroDao, deploy_funded};
use super::micro_dao_token::validate_offer_artifacts;
use super::*;
use compact_rust_test_center_micro_dao_fixture::{
    ledger_contract as c, ledger_slots as slots, types,
};
use midnight_compact_runtime as runtime;
use midnight_ledger::error::MalformedTransaction;
use midnight_zswap::{Offer, Output, Transient};
use runtime::transaction::{
    ContractTransientCoins, OfferBackedObservedState, OfferBindingOptions, OfferPlacement,
    PersistentOutputAllocation, WalletFundingInputs,
};
use std::num::NonZeroU16;

fn private() -> support::Private {
    support::Private {
        phase: 0,
        vote: None,
        calls: 0,
    }
}
fn compact_coin(coin: runtime::ledger::CoinInfo) -> types::ShieldedCoinInfo {
    types::ShieldedCoinInfo {
        nonce: FixedBytes::new(coin.nonce.0.0),
        color: FixedBytes::new(coin.type_.0.0),
        value: BoundedUint::new(coin.value).unwrap(),
    }
}
pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_offer_artifacts(root, "vote_commit")?;
    for (ballot, round) in [(true, 0), (false, u64::MAX)] {
        run_case(root, ballot, round)?;
    }
    Ok(())
}
fn run_case(root: &Path, ballot: bool, round: u64) -> Result<(), Box<dyn Error>> {
    const NAME: &str = "vote_commit";
    let mut rng = StdRng::seed_from_u64(0x0214_0000 + u64::from(ballot));
    let prior = support::seeded(types::LedgerState::commit, round, 0, false, false)?;
    let stem = format!("vote-commit-{ballot}");
    let FundedMicroDao {
        deploy,
        verifier,
        state: mut fees,
    } = deploy_funded(
        root,
        (NAME, &stem),
        prior.query.state.get_ref().clone(),
        11,
        &mut rng,
    )?;
    let address = deploy.address();
    // Derive the instance-specific color in a separate native setup call. No
    // token issuance or complete voting lifecycle is being proved here.
    let token = c::dao_voting_token(runtime::context::CircuitContext::from_contract_state(
        private(),
        address,
        &deploy.initial_state,
    ))?
    .result;
    let coin = runtime::ledger::coin_info_from_compact(FixedBytes::new([3; 32]), token, 1);
    let keys = midnight_zswap::keys::SecretKeys::from_rng_seed(&mut rng);
    let padding = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([91; 32]),
        FixedBytes::new([0; 32]),
        17,
    );
    let padding_offer: Offer<ProofPreimage, DefaultDB> = Offer::new(
        vec![],
        vec![Output::new_contract_owned(
            &mut rng, &padding, None, address,
        )?],
        vec![],
    )
    .unwrap();
    let wallet_offer: Offer<ProofPreimage, DefaultDB> = Offer::new(
        vec![],
        vec![Output::new(
            &mut rng,
            &coin,
            None,
            &keys.coin_public_key(),
            Some(keys.enc_public_key()),
        )?],
        vec![],
    )
    .unwrap();
    let (state, _) = fees.ledger.zswap.try_apply(&padding_offer, None)?;
    let (state, _) = state.try_apply(&wallet_offer, None)?;
    fees.ledger.zswap = midnight_storage_core::arena::Sp::new(
        state.post_block_update(fees.time, fees.ledger.parameters.global_ttl),
    );
    let ledger = fees.ledger.clone();
    assert_eq!(ledger.zswap.first_free, 2);
    let wallet = midnight_zswap::local::State::<DefaultDB>::new()
        .apply(&keys, &padding_offer)?
        .apply(&keys, &wallet_offer)?;
    assert_eq!(wallet.merkle_tree.root(), ledger.zswap.coin_coms.root());
    let (_, wallet_input) = wallet.spend(&mut rng, &keys, &coin.qualify(1), Some(1))?;
    assert_eq!(
        wallet_input.proof.public_transcript_outputs,
        vec![Fr::from(1), Fr::from(1)]
    );
    let received = Output::new_contract_owned(&mut rng, &coin, Some(1), address)?;
    let transient =
        Transient::new_from_contract_owned_output(&mut rng, &coin.qualify(0), Some(1), received)?;
    let nullifiers = [wallet_input.nullifier, transient.nullifier];
    let mut provisional = runtime::context::CircuitContext::from_contract_state(
        private(),
        address,
        &deploy.initial_state,
    );
    provisional.set_zswap_output_start(2)?;
    let raw = c::vote_commit(
        provisional,
        &support::Witness::new("normal"),
        ballot,
        compact_coin(coin),
    )?;
    let outputs = raw.context.circuit_zswap().outputs();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].coin, coin);
    assert_eq!(outputs[0].provisional_index, 2);
    assert_eq!(outputs[1].provisional_index, 3);
    let zero_key = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([0; 32]));
    assert_eq!(
        outputs[1].recipient,
        runtime::ledger::CoinRecipient::User(zero_key)
    );
    let sent = outputs[1].coin;
    assert_eq!(sent.value, 1);
    assert_eq!(sent.type_, coin.type_);
    assert_ne!(sent.nonce, coin.nonce);
    let output = Output::new(&mut rng, &sent, Some(1), &zero_key, None)?;
    let output_commitment = output.coin_com;
    let transient_commitment = transient.coin_com;
    let offer: Offer<ProofPreimage, DefaultDB> = Offer::new(
        vec![wallet_input.clone()],
        vec![output],
        vec![transient.clone()],
    )
    .unwrap();
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    assert_eq!(indices.get(&output_commitment), Some(&2));
    assert_eq!(indices.get(&transient_commitment), Some(&3));
    let placement = OfferPlacement::Fallible(NonZeroU16::new(1).unwrap());
    let options = OfferBindingOptions::default()
        .with_output_allocation(PersistentOutputAllocation::CanonicalOfferIndices)
        .with_offer_placement(placement)
        .with_wallet_funding(
            WalletFundingInputs::from_inputs(vec![wallet_input])
                .map_err(|e| format!("wallet: {e:?}"))?,
        )
        .with_transient_coins(
            ContractTransientCoins::from_transients_for_placement(vec![transient], placement)
                .map_err(|e| format!("transient: {e:?}"))?,
        );
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let bound = OfferBackedObservedState::with_options(observed, &ledger, offer.clone(), options)?;
    let native = c::vote_commit(
        bound.observed().circuit_context(private()),
        &support::Witness::new("normal"),
        ballot,
        compact_coin(coin),
    )?;
    let generated = c::Contract::from(support::Witness::new("normal"));
    let call = generated.recording().vote_commit_call(
        bound.observed(),
        private(),
        ballot,
        compact_coin(coin),
    )?;
    let recorded = call.recorded();
    assert_eq!(
        recorded.execution.context.query.state,
        native.context.query.state
    );
    assert_eq!(
        recorded.execution.context.query.effects,
        native.context.query.effects
    );
    assert_eq!(
        recorded.execution.context.circuit_zswap(),
        native.context.circuit_zswap()
    );
    assert_eq!(
        recorded.execution.context.private_state,
        native.context.private_state
    );
    assert_eq!(
        recorded.execution.private_transcript_outputs,
        native.private_transcript_outputs
    );
    assert_eq!(recorded.execution.gas_cost, native.gas_cost);
    assert_eq!(recorded.public.verify_ops().len(), 56);
    assert_eq!(recorded.execution.private_transcript_outputs.len(), 7);
    assert_eq!(
        native.context.private_state,
        support::Private {
            phase: 1,
            vote: Some(ballot),
            calls: 4
        }
    );
    let replay = recorded.public.initial().query(
        recorded.public.verify_ops(),
        None,
        &recorded.execution.context.cost_model,
    )?;
    assert_eq!(replay.context.state, native.context.query.state);
    assert_eq!(replay.context.effects, native.context.query.effects);
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0))?;
    assert!(prepared.prototype().guaranteed_public_transcript.is_none());
    assert!(prepared.prototype().fallible_public_transcript.is_some());
    let sealed = v3::prove_funded_offer(
        root,
        (NAME, &stem),
        prepared,
        &mut fees,
        &mut rng,
        &verifier,
        |tx| {
            let Transaction::Standard(structure) = tx else {
                panic!("standard")
            };
            assert!(structure.guaranteed_coins.is_none());
            assert_eq!(structure.fallible_coins.iter().count(), 1);
            assert_eq!(*structure.fallible_coins.get(&1).unwrap(), offer);
            Ok(())
        },
    )?;
    tagged_serialize(
        &sealed,
        &mut File::create(root.join(format!("vote-commit-{ballot}-sealed.bin")))?,
    )?;
    let Transaction::Standard(structure) = &sealed else {
        panic!("standard")
    };
    assert!(structure.guaranteed_coins.is_none());
    assert_eq!(structure.fallible_coins.iter().count(), 1);
    for row in structure.intents.iter() {
        if *row.0 != 1 {
            assert!(row.1.actions.is_empty());
            assert!(row.1.dust_actions.is_some());
            assert!(row.1.guaranteed_unshielded_offer.is_none());
            assert!(row.1.fallible_unshielded_offer.is_none());
        }
    }
    let mut misplaced = structure.clone();
    misplaced.guaranteed_coins = Some(midnight_storage_core::arena::Sp::new(
        (*misplaced.fallible_coins.get(&1).unwrap()).clone(),
    ));
    misplaced.fallible_coins = HashMap::new();
    let wrong = Transaction::Standard(misplaced)
        .well_formed(&ledger, WellFormedStrictness::default(), fees.time)
        .err()
        .ok_or("wrong placement accepted")?;
    assert!(
        matches!(
            wrong,
            MalformedTransaction::Zswap(midnight_zswap::error::MalformedOffer::InvalidProof(_))
        ),
        "{wrong:?}"
    );
    let verified = sealed.well_formed(&ledger, WellFormedStrictness::default(), fees.time)?;
    let context = TransactionContext {
        ref_state: ledger.clone(),
        block_context: BlockContext {
            tblock: fees.time,
            last_block_time: fees.time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let mut changed = ledger.clone();
    let changed_data = slots::state
        .write(
            runtime::context::CircuitContext::from_contract_state(
                (),
                address,
                &deploy.initial_state,
            ),
            types::LedgerState::reveal,
        )?
        .context
        .query
        .state;
    let mut changed_contract = deploy.initial_state.clone();
    changed_contract.data = changed_data.clone();
    changed.contract = changed.contract.insert(address, changed_contract);
    let (after, result) = changed.apply(&verified, &context);
    assert!(
        matches!(result,TransactionResult::PartialSuccess(ref phases,_)
        if phases.get(&0).is_some_and(Result::is_ok)
            && matches!(phases.get(&1),Some(Err(midnight_ledger::error::TransactionInvalid::Transcript(
                midnight_onchain_runtime::error::TranscriptRejected::Execution(
                    midnight_onchain_runtime::vm_error::OnchainProgramError::ReadMismatch {..})))))),
        "{result:?}"
    );
    assert_eq!(after.zswap, changed.zswap);
    assert_eq!(after.contract.get(&address).unwrap().data, changed_data);
    assert_ne!(after.dust, changed.dust);
    assert_ne!(after.replay_protection, changed.replay_protection);
    let (updated, result) = ledger.apply(&verified, &context);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    assert_eq!(
        updated.contract.get(&address).unwrap().operations,
        deploy.initial_state.operations,
    );
    let data = &updated.contract.get(&address).unwrap().data;
    assert_eq!(*data, native.context.query.state);
    assert_eq!(
        slots::state.inspect(data.get_ref())?,
        types::LedgerState::commit
    );
    assert_eq!(slots::round.inspect(data.get_ref())?, round);
    assert!(
        slots::committed_participants
            .inspect(data.get_ref())?
            .member(support::participant(round))
    );
    let tree = slots::committed_votes.inspect(data.get_ref())?;
    assert_eq!(tree.first_free()?.value(), 1);
    assert!(
        tree.find_path_for_leaf(support::commitment(ballot, round))
            .is_some()
    );
    assert_eq!(updated.zswap.first_free, 4);
    assert_eq!(
        updated.zswap.coin_coms.index(2).unwrap().0,
        output_commitment.0
    );
    assert!(updated.zswap.coin_coms.index(2).unwrap().1.is_none());
    assert_eq!(
        updated.zswap.coin_coms.index(3).unwrap().0,
        transient_commitment.0
    );
    assert_eq!(
        updated
            .zswap
            .coin_coms
            .index(3)
            .unwrap()
            .1
            .as_ref()
            .map(|owner| **owner),
        Some(address)
    );
    for nullifier in nullifiers {
        assert!(updated.zswap.nullifiers.contains_key(&nullifier));
    }
    assert!(matches!(
        updated.zswap.try_apply(&offer, None),
        Err(midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(_))
    ));
    v3::check_offer_replay(root, &stem, &updated, &verified, context)?;
    println!(
        "original vote_commit ballot={ballot} round={round}: strict funded proof/application, exact tree/participant mutation, private witness state, canonical output, rollback and replay passed"
    );
    Ok(())
}
