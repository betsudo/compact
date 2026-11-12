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
//! Exact persistent allocation with both source/offer orderings and stored change.
use super::local_v3_application as v3;
use super::qualified_coin_funding::{fee_funded_state_with_utxos, fee_resolver};
use super::*;
use compact_rust_canonical_output_order_oracle_fixture::{ledger_contract as c, types};
use midnight_base_crypto::hash::persistent_hash;
use midnight_compact_runtime as runtime;
use midnight_zswap::{Input, Offer, Output};
use runtime::transaction::{
    OfferBackedObservedState, OfferBindingOptions, PersistentOutputAllocation,
};
use std::ffi::OsString;

const NAME: &str = "distribute";

fn compact_coin(coin: runtime::ledger::CoinInfo) -> types::ShieldedCoinInfo {
    types::ShieldedCoinInfo {
        nonce: FixedBytes::new(coin.nonce.0.0),
        color: FixedBytes::new(coin.type_.0.0),
        value: BoundedUint::new(coin.value).unwrap(),
    }
}
pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        compact_rust_canonical_output_order_oracle_fixture::RETAINED_IR,
    )?;
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("canonical output order requires local proving".into());
    }
    let pins: std::collections::BTreeMap<&str, &str> = serde_json::from_str(include_str!(
        "../../compact-rust-backend/tests/canonical-output-order-ledger9-artifacts.json"
    ))?;
    let mut raw_circuits = Vec::new();
    for entry in std::fs::read_dir(root.join("zkir"))? {
        let entry = entry?;
        if Path::new(&entry.file_name())
            .extension()
            .is_some_and(|ext| ext == "zkir")
        {
            raw_circuits.push(entry.file_name());
        }
    }
    raw_circuits.sort_unstable();
    if raw_circuits
        != [
            OsString::from("distribute.zkir"),
            OsString::from("read_coin.zkir"),
        ]
    {
        return Err("canonical output order raw circuit inventory differs".into());
    }
    for (path, expected) in pins {
        if hex::encode(persistent_hash(&std::fs::read(root.join(path))?).0) != expected {
            return Err(
                format!("canonical output order source/proof artifact differs: {path}").into(),
            );
        }
    }
    let (ir, verifier, operation) = v3::operation(root, NAME, 13)?;
    if ir.model().rows() != 7_085 {
        return Err("canonical output order v3 row count differs".into());
    }
    for reversed in [false, true] {
        run_order(root, reversed, &verifier, &operation)?;
    }
    Ok(())
}
fn run_order(
    root: &Path,
    reversed: bool,
    verifier: &VerifierKey,
    operation: &ContractOperation,
) -> Result<(), Box<dyn Error>> {
    let stem = if reversed {
        "canonical-output-reversed"
    } else {
        "canonical-output-source-order"
    };
    let mut rng = StdRng::seed_from_u64(0x0205_0000 + u64::from(reversed));
    let initial = c::initial_state(ConstructorContext::new(()))?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    let address = deploy.address();
    let mut fees = fee_funded_state_with_utxos(&mut rng, 2)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let resolver = fee_resolver(root, NAME)?;
    v3::apply_funded_deployment(
        root, stem, &deploy, &mut fees, &mut rng, &executor, &resolver,
    )?;
    let seed = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([1; 32]),
        FixedBytes::new([2; 32]),
        42,
    );
    // Explicit offline genesis prerequisite, not a claimed previous transaction.
    let seed_output = Output::new_contract_owned(&mut rng, &seed, None, address)?;
    let seed_com = seed_output.coin_com;
    let seed_offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![seed_output], vec![]).unwrap();
    let (seeded, seed_indices) = fees.ledger.zswap.try_apply(&seed_offer, None)?;
    let seed_index = *seed_indices.get(&seed_com).unwrap();
    fees.ledger.zswap = midnight_storage_core::arena::Sp::new(
        seeded.post_block_update(fees.time, fees.ledger.parameters.global_ttl),
    );
    let ledger = fees.ledger.clone();
    let start = ledger.zswap.first_free;
    assert!(start > 0);
    let input = Input::new_contract_owned(
        &mut rng,
        &seed.qualify(seed_index),
        None,
        address,
        &ledger.zswap.coin_coms,
    )?;
    let nullifier = input.nullifier;
    let user = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([7; 32]));
    let recipient = runtime::ledger::CoinRecipient::User(user);
    let self_recipient = runtime::ledger::CoinRecipient::Contract(address);
    let change = runtime::ledger::coin_info_from_compact(
        FixedBytes::new([4; 32]),
        FixedBytes::new([2; 32]),
        25,
    );
    let change_com = change.commitment(&self_recipient);
    let sent = (3..=255)
        .map(|n| {
            runtime::ledger::coin_info_from_compact(
                FixedBytes::new([n; 32]),
                FixedBytes::new([2; 32]),
                17,
            )
        })
        .find(|coin| {
            (coin.commitment(&recipient) > change_com) == reversed
                && coin.commitment(&recipient) != change_com
        })
        .ok_or("no deterministic nonce for requested ordering")?;
    let sent_output = Output::new(&mut rng, &sent, None, &user, None)?;
    let sent_com = sent_output.coin_com;
    let change_output = Output::new_contract_owned(&mut rng, &change, None, address)?;
    let offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![input], vec![sent_output, change_output], vec![]).unwrap();
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let sent_index = *indices.get(&sent_com).unwrap();
    let change_index = *indices.get(&change_com).unwrap();
    assert_eq!(
        (sent_index, change_index),
        if reversed {
            (start + 1, start)
        } else {
            (start, start + 1)
        }
    );
    let observe = || {
        ObservedContractState::new(
            address,
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        )
    };
    let bound = OfferBackedObservedState::with_options(
        observe(),
        &ledger,
        offer.clone(),
        OfferBindingOptions::default()
            .with_output_allocation(PersistentOutputAllocation::CanonicalOfferIndices),
    )?;
    let qualified = types::QualifiedShieldedCoinInfo {
        nonce: FixedBytes::new(seed.nonce.0.0),
        color: FixedBytes::new(seed.type_.0.0),
        value: BoundedUint::new(42)?,
        mt_index: BoundedUint::new(seed_index.into())?,
    };
    let sent_compact = compact_coin(sent);
    let change_compact = compact_coin(change);
    let user_compact = types::Either {
        is_left: true,
        left: types::ZswapCoinPublicKey {
            bytes: FixedBytes::new(user.0.0),
        },
        right: types::ContractAddress {
            bytes: FixedBytes::new([0; 32]),
        },
    };
    let self_compact = types::Either {
        is_left: false,
        left: types::ZswapCoinPublicKey {
            bytes: FixedBytes::new([0; 32]),
        },
        right: types::ContractAddress {
            bytes: FixedBytes::new(address.0.0),
        },
    };
    let nul = FixedBytes::new(nullifier.0.0);
    let sent_claim = FixedBytes::new(sent_com.0.0);
    let change_claim = FixedBytes::new(change_com.0.0);
    let native = c::distribute(
        bound.observed().circuit_context(()),
        qualified.clone(),
        sent_compact.clone(),
        change_compact.clone(),
        user_compact.clone(),
        self_compact.clone(),
        nul,
        sent_claim,
        change_claim,
    )?;
    let call = c::recorded::Contract.distribute_call(
        bound.observed(),
        (),
        qualified.clone(),
        sent_compact.clone(),
        change_compact.clone(),
        user_compact.clone(),
        self_compact.clone(),
        nul,
        sent_claim,
        change_claim,
    )?;
    let recorded = call.recorded();
    assert_eq!(
        native.context.query.state,
        recorded.execution.context.query.state
    );
    assert_eq!(
        native.context.query.effects,
        recorded.execution.context.query.effects
    );
    assert_eq!(
        native.context.circuit_zswap(),
        recorded.execution.context.circuit_zswap()
    );
    assert_eq!(native.gas_cost, recorded.execution.gas_cost);
    assert_eq!(
        native.private_transcript_outputs,
        recorded.execution.private_transcript_outputs
    );
    assert_eq!(
        native.private_transcript_outputs,
        vec![AlignedValue::from(()); 3]
    );
    let plan = recorded.execution.context.circuit_zswap();
    assert_eq!(
        plan.outputs().iter().map(|o| o.coin).collect::<Vec<_>>(),
        vec![sent, change]
    );
    assert_eq!(
        plan.outputs()
            .iter()
            .map(|o| o.provisional_index)
            .collect::<Vec<_>>(),
        vec![sent_index, change_index]
    );
    assert_eq!(plan.next_index(), start + 2);
    assert_eq!(
        recorded.execution.context.query.call_context.com_indices,
        indices
    );
    let replay = recorded.public.initial().query(
        recorded.public.verify_ops(),
        None,
        &recorded.execution.context.cost_model,
    )?;
    assert_eq!(replay.context.state, native.context.query.state);
    assert_eq!(replay.context.effects, native.context.query.effects);
    let expected = native.context.query.state.get_ref().clone();
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0u64))?;
    // The documented default remains exact-order; it must not be silently upgraded.
    let exact = OfferBackedObservedState::new(observe(), &ledger, offer.clone())?;
    let result = c::recorded::Contract.distribute_call(
        exact.observed(),
        (),
        qualified,
        sent_compact,
        change_compact.clone(),
        user_compact,
        self_compact,
        nul,
        sent_claim,
        change_claim,
    );
    if reversed {
        assert!(matches!(
            result,
            Err(runtime::CompactError::ZswapOfferOutputMismatch)
        ));
    } else {
        assert!(result.is_ok());
    }
    let updated = v3::prove_and_apply_offer(
        root,
        (NAME, stem),
        prepared,
        &mut fees,
        &mut rng,
        verifier,
        &expected,
    )?;
    assert_eq!(updated.zswap.first_free, start + 2);
    assert_eq!(
        updated
            .zswap
            .coin_coms
            .index(sent_index)
            .map(|(hash, owner)| (hash, owner.as_ref().map(|o| **o))),
        Some((sent_com.0, None))
    );
    assert_eq!(
        updated
            .zswap
            .coin_coms
            .index(change_index)
            .map(|(hash, owner)| (hash, owner.as_ref().map(|o| **o))),
        Some((change_com.0, Some(address)))
    );
    assert!(updated.zswap.nullifiers.contains_key(&nullifier));
    let mut read = bound.observed().circuit_context(());
    read.query.state = runtime::ledger::ChargedState::new(expected);
    let stored = c::read_coin(read)?.result;
    assert_eq!(stored.mt_index.value(), u128::from(change_index));
    assert_eq!(stored.nonce, change_compact.nonce);
    assert_eq!(stored.value, change_compact.value);
    assert_eq!(stored.color, change_compact.color);
    assert!(
        matches!(updated.zswap.try_apply(&offer,None),Err(midnight_zswap::error::TransactionInvalid::NullifierAlreadyPresent(found)) if found==nullifier)
    );
    println!(
        "canonical reversed={reversed}: sent and change indices [{sent_index}, {change_index}], frontier {}, stored change index {change_index}; funded 42 → 17 + 25 and exact nullifier refusal",
        start + 2
    );
    Ok(())
}
