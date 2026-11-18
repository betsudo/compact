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

//! Original shielded mint-to-self Effects and funded Ledger 9 qualification.

use super::{local_v3_application as v3, *};
use crate::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::OfferBackedObservedState;
use midnight_ledger::error::{EffectsCheckError, MalformedTransaction};
use midnight_zswap::{Offer, Output};

const NAME: &str = "mintShieldedToSelfTest";
const STEM: &str = "unshielded-original-mint-shielded-self-100";
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
    "/zkir/mintShieldedToSelfTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (member, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/mintShieldedToSelfTest.zkir", GENERATED_ZKIR),
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
    let mut rng = StdRng::seed_from_u64(0x0184_534d);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    let mut funded = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    let resolver = super::qualified_coin_funding::fee_resolver(root, NAME)?;
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
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    let nonce = [0x55; 32];
    let coin = runtime::ledger::CoinInfo {
        nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput(nonce)),
        type_: deploy.address().custom_shielded_token_type(domain),
        value: 100,
    };
    let output = Output::new_contract_owned(&mut rng, &coin, None, deploy.address())?;
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
    let bound = OfferBackedObservedState::new(observed, &funded.ledger, offer)?;
    let call = contract::Contract::default()
        .recording
        .mintShieldedToSelfTest_call(
            bound.observed(),
            (),
            FixedBytes::new(domain.0),
            BoundedUint::<18446744073709551615>::new(100)?,
            FixedBytes::new(nonce),
        )?;
    let result = &call.recorded().execution.result;
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
        || effects.claimed_shielded_receives.size() != 1
        || !effects.claimed_shielded_spends.member(&commitment)
        || !effects.claimed_shielded_receives.member(&commitment)
        || !effects.claimed_nullifiers.is_empty()
        || !effects.unshielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || !effects.claimed_unshielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
    {
        return Err(
            "original shielded mint-to-self Effects differ from 100-unit auto-receive".into(),
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
            EffectsCheckError::CommitmentsNEClaimedShieldedReceives {
                commitments,
                claimed_shielded_receives,
            }
        ) if commitments.is_empty()
            && claimed_shielded_receives.len() == 1
            && claimed_shielded_receives[0].1 == commitment
            && claimed_shielded_receives[0].2 == deploy.address()
    );
    if !expected_refusal {
        return Err(format!("original shielded mint no-offer refusal differs: {refusal:?}").into());
    }
    v3::write_new(
        root.join(format!("{STEM}-without-offer-refusal.txt")),
        format!("{refusal:?}\n").as_bytes(),
    )?;
    println!("{NAME}: unmatched-output commitment refusal passed");
    let updated = v3::prove_and_apply_offer(
        root,
        (NAME, STEM),
        prepared,
        &mut funded,
        &mut rng,
        &verifier,
        initial.ledger_state.get_ref(),
    )?;
    if updated.zswap.first_free != index + 1
        || updated.zswap.coin_coms.index(index).map(|(hash, _)| hash) != Some(commitment.0)
    {
        return Err("original shielded mint output commitment or index changed".into());
    }
    println!(
        "{NAME}: original mint100, matching self spend/receive commitment, funded strict apply and replay passed"
    );
    Ok(())
}
