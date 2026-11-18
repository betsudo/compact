// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Fresh proof from the unchanged upstream unshielded-token contract.

use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as contract;
use midnight_onchain_runtime::context::ClaimedUnshieldedSpendsKey;
use midnight_transient_crypto::proofs::Proof;

const CIRCUIT: &str = "mintUnshieldedToSelfTest";
const STEM: &str = "unshielded-original-mint-self";
const BALANCE_CIRCUIT: &str = "getUnshieldedBalanceTest";
const BALANCE_STEM: &str = "unshielded-original-balance-100";
const BALANCE_OTHER_STEM: &str = "unshielded-original-balance-other-color-0";
const BALANCE_GT_CIRCUIT: &str = "getUnshieldedBalanceGtTest";
const BALANCE_LT_CIRCUIT: &str = "getUnshieldedBalanceLtTest";
const SEND_CONTRACT_CIRCUIT: &str = "sendUnshieldedToContractTest";
const SEND_CONTRACT_STEM: &str = "unshielded-original-send-contract-100";
const RECEIVE_CONTRACT_CIRCUIT: &str = "receiveUnshieldedTest";
const RECEIVE_CONTRACT_STEM: &str = "unshielded-original-receive-contract-100";
const SEND_SELF_CIRCUIT: &str = "sendUnshieldedToSelfTest";
const SEND_SELF_STEM: &str = "unshielded-original-send-self-100";
const MINT_CONTRACT_CIRCUIT: &str = "mintUnshieldedToContractTest";
const MINT_CONTRACT_STEM: &str = "unshielded-original-mint-contract-100";
const MINT_USER_CIRCUIT: &str = "mintUnshieldedToUserTest";
const MINT_USER_STEM: &str = "unshielded-original-mint-user-100";
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
    "/zkir/mintUnshieldedToSelfTest.zkir"
));
const GENERATED_BALANCE_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/getUnshieldedBalanceTest.zkir"
));
const GENERATED_BALANCE_GT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/getUnshieldedBalanceGtTest.zkir"
));
const GENERATED_BALANCE_LT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/getUnshieldedBalanceLtTest.zkir"
));
const GENERATED_SEND_CONTRACT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/sendUnshieldedToContractTest.zkir"
));
const GENERATED_RECEIVE_CONTRACT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/receiveUnshieldedTest.zkir"
));
const GENERATED_SEND_SELF_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/sendUnshieldedToSelfTest.zkir"
));
const GENERATED_MINT_CONTRACT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/mintUnshieldedToContractTest.zkir"
));
const GENERATED_MINT_USER_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/mintUnshieldedToUserTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (name, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/mintUnshieldedToSelfTest.zkir", GENERATED_ZKIR),
        ("zkir/getUnshieldedBalanceTest.zkir", GENERATED_BALANCE_ZKIR),
        (
            "zkir/getUnshieldedBalanceGtTest.zkir",
            GENERATED_BALANCE_GT_ZKIR,
        ),
        (
            "zkir/getUnshieldedBalanceLtTest.zkir",
            GENERATED_BALANCE_LT_ZKIR,
        ),
        (
            "zkir/sendUnshieldedToContractTest.zkir",
            GENERATED_SEND_CONTRACT_ZKIR,
        ),
        (
            "zkir/receiveUnshieldedTest.zkir",
            GENERATED_RECEIVE_CONTRACT_ZKIR,
        ),
        (
            "zkir/sendUnshieldedToSelfTest.zkir",
            GENERATED_SEND_SELF_ZKIR,
        ),
        (
            "zkir/mintUnshieldedToContractTest.zkir",
            GENERATED_MINT_CONTRACT_ZKIR,
        ),
        (
            "zkir/mintUnshieldedToUserTest.zkir",
            GENERATED_MINT_USER_ZKIR,
        ),
    ] {
        if std::fs::read(root.join(name))? != expected {
            return Err(format!("original unshielded artifact changed: {name}").into());
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original unshielded proof requires local proving".into());
    }
    validate_artifacts(root)?;
    let mut rng = StdRng::seed_from_u64(0x0184_554e);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, CIRCUIT)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation.clone(),
        &mut rng,
    );
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let generated = contract::Contract::default();
    let recorded = generated.recording.mintUnshieldedToSelfTest_call(
        &observed,
        (),
        FixedBytes::new([0x84; 32]),
        BoundedUint::<18446744073709551615>::new(100)?,
    )?;
    let prepared = recorded.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("original unshielded call operation differs from installed v3 IR".into());
    }
    let transcript = prepared
        .guaranteed_public_transcript
        .as_ref()
        .or(prepared.fallible_public_transcript.as_ref())
        .ok_or("original unshielded call has no effects transcript")?;
    let effects = &transcript.effects;
    let domain = midnight_compact_runtime::ledger::HashOutput([0x84; 32]);
    let token = midnight_compact_runtime::ledger::TokenType::Unshielded(
        deploy.address().custom_unshielded_token_type(domain),
    );
    let claim = ClaimedUnshieldedSpendsKey(
        token,
        midnight_compact_runtime::ledger::PublicAddress::Contract(deploy.address()),
    );
    if effects.unshielded_mints.size() != 1
        || effects.unshielded_inputs.size() != 1
        || effects.claimed_unshielded_spends.size() != 1
        || !effects.unshielded_outputs.is_empty()
        || *effects
            .unshielded_mints
            .get(&domain)
            .ok_or("mint effect missing")?
            != 100
        || *effects
            .unshielded_inputs
            .get(&token)
            .ok_or("input effect missing")?
            != 100
        || *effects
            .claimed_unshielded_spends
            .get(&claim)
            .ok_or("claim effect missing")?
            != 100
    {
        return Err("original unshielded mint effects differ from 100-unit self claim".into());
    }
    let proof: Proof = v3::prove_standalone(root, CIRCUIT, STEM, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, CIRCUIT, STEM)?;
    v3::apply(
        root,
        CIRCUIT,
        STEM,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        &proof,
    )?;
    println!("{CIRCUIT}: original source fresh proof and funded strict replay gate passed");
    run_balance(root)?;
    run_balance_comparators(root)?;
    run_send_contract(root)?;
    run_send_self(root)?;
    run_mint_destination(root, false)?;
    run_mint_destination(root, true)?;
    Ok(())
}

fn run_mint_destination(root: &Path, is_user: bool) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{HashOutput, PublicAddress, TokenType, UserAddress};
    use midnight_compact_runtime::transaction::RecordedCall;

    let (name, stem, seed) = if is_user {
        (MINT_USER_CIRCUIT, MINT_USER_STEM, 0x0184_4d55)
    } else {
        (MINT_CONTRACT_CIRCUIT, MINT_CONTRACT_STEM, 0x0184_4d43)
    };
    let mut rng = StdRng::seed_from_u64(seed);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, name)?;
    let recipient_deploy = if is_user {
        None
    } else {
        let (_, _, receive_operation) = v3::load_operation(root, RECEIVE_CONTRACT_CIRCUIT)?;
        Some(v3::deploy(
            initial.ledger_state.get_ref().clone(),
            RECEIVE_CONTRACT_CIRCUIT,
            receive_operation,
            &mut rng,
        ))
    };
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let domain = FixedBytes::new([if is_user { 0x85 } else { 0x86 }; 32]);
    let recipient_bytes = recipient_deploy
        .as_ref()
        .map(|recipient| {
            midnight_compact_runtime::ledger::contract_address_bytes(&recipient.address())
        })
        .unwrap_or_else(|| FixedBytes::new([0x92; 32]));
    let recipient_contract = crate::types::ContractAddress {
        bytes: recipient_bytes,
    };
    let recipient_user = crate::types::UserAddress {
        bytes: recipient_bytes,
    };
    let contract_address =
        midnight_compact_runtime::ledger::ContractAddress(HashOutput(recipient_bytes.into_array()));
    if contract_address == deploy.address() {
        return Err("mint-to-contract recipient unexpectedly equals minter".into());
    }
    let amount = BoundedUint::<18446744073709551615>::new(100)?;
    let generated = contract::Contract::default();
    let recorded = if is_user {
        generated.recording.mintUnshieldedToUserTest(
            observed.circuit_context(()),
            domain,
            recipient_user.clone(),
            amount,
        )?
    } else {
        generated.recording.mintUnshieldedToContractTest(
            observed.circuit_context(()),
            domain,
            recipient_contract.clone(),
            amount,
        )?
    };
    let input = if is_user {
        AlignedValue::concat(&[
            AlignedValue::from(domain),
            AlignedValue::from(recipient_user),
            AlignedValue::from(amount),
        ])
    } else {
        AlignedValue::concat(&[
            AlignedValue::from(domain),
            AlignedValue::from(recipient_contract),
            AlignedValue::from(amount),
        ])
    };
    let prepared = RecordedCall::new(&observed, recorded, name, input)
        .prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err(format!("original {name} operation differs from installed v3 IR").into());
    }
    let effects = &prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original mint destination has no guaranteed effects transcript")?
        .effects;
    let raw_domain = HashOutput(domain.into_array());
    let token = TokenType::Unshielded(deploy.address().custom_unshielded_token_type(raw_domain));
    let destination = if is_user {
        PublicAddress::User(UserAddress(HashOutput(recipient_bytes.into_array())))
    } else {
        PublicAddress::Contract(contract_address)
    };
    let wrong_tag = if is_user {
        PublicAddress::Contract(contract_address)
    } else {
        PublicAddress::User(UserAddress(HashOutput(recipient_bytes.into_array())))
    };
    let claim = ClaimedUnshieldedSpendsKey(token, destination);
    let wrong_claim = ClaimedUnshieldedSpendsKey(token, wrong_tag);
    if prepared.fallible_public_transcript.is_some()
        || !effects.claimed_nullifiers.is_empty()
        || !effects.claimed_shielded_receives.is_empty()
        || !effects.claimed_shielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
        || !effects.shielded_mints.is_empty()
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_outputs.is_empty()
        || effects.unshielded_mints.size() != 1
        || effects.claimed_unshielded_spends.size() != 1
        || effects
            .unshielded_mints
            .get(&raw_domain)
            .map(|value| *value)
            != Some(100)
        || effects
            .claimed_unshielded_spends
            .get(&claim)
            .map(|value| *value)
            != Some(100)
        || effects
            .claimed_unshielded_spends
            .get(&wrong_claim)
            .is_some()
    {
        return Err(
            format!("original {name} effects or destination tag differ from fixture").into(),
        );
    }
    let proof: Proof = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    apply_mint_destination(
        root,
        (name, stem),
        prepared,
        deploy,
        recipient_deploy,
        token,
        destination,
        &proof,
    )?;
    println!(
        "{name}: original 100-unit destination proof, funded strict apply and replay gate passed"
    );
    Ok(())
}

fn apply_mint_destination(
    root: &Path,
    (name, stem): (&'static str, &str),
    prepared: ContractCallPrototype<DefaultDB>,
    minter: ContractDeploy<DefaultDB>,
    recipient: Option<ContractDeploy<DefaultDB>>,
    token: midnight_compact_runtime::ledger::TokenType,
    destination: midnight_compact_runtime::ledger::PublicAddress,
    standalone: &Proof,
) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{PublicAddress, TokenType};
    use midnight_ledger::structure::{ProofVersioned, UnshieldedOffer, UtxoOutput};

    let mut rng = StdRng::seed_from_u64(if recipient.is_some() {
        0x0184_4d52
    } else {
        0x0184_4d57
    });
    let mut funded = qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 1)?;
    let minter_address = minter.address();
    funded.ledger.contract = funded
        .ledger
        .contract
        .insert(minter_address, minter.initial_state);
    let recipient_address = recipient.as_ref().map(|contract| contract.address());
    if let Some(contract) = &recipient {
        funded.ledger.contract = funded
            .ledger
            .contract
            .insert(contract.address(), contract.initial_state.clone());
    }
    let mut calls = vec![prepared];
    let mut receive_standalone = None;
    if let Some(address) = recipient_address {
        let (_, verifier, operation) = v3::load_operation(root, RECEIVE_CONTRACT_CIRCUIT)?;
        let recipient_state = funded
            .ledger
            .contract
            .get(&address)
            .ok_or("mint recipient deployment missing")?;
        let observed = ObservedContractState::new(
            address,
            recipient_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let TokenType::Unshielded(raw_type) = token else {
            return Err("mint destination token is not unshielded".into());
        };
        let receive = contract::Contract::default()
            .recording
            .receiveUnshieldedTest_call(
                &observed,
                (),
                FixedBytes::new(raw_type.0.0),
                BoundedUint::<340282366920938463463374607431768211455>::new(100)?,
            )?;
        let prepared_receive = receive.prepare(verifier.clone(), Fr::from(0_u64))?;
        if prepared_receive.op != operation {
            return Err("mint recipient receive operation differs from installed v3 IR".into());
        }
        let effects = &prepared_receive
            .guaranteed_public_transcript
            .as_ref()
            .ok_or("mint recipient has no guaranteed receive effects")?
            .effects;
        if prepared_receive.fallible_public_transcript.is_some()
            || effects.unshielded_inputs.size() != 1
            || effects.unshielded_inputs.get(&token).map(|value| *value) != Some(100)
            || !effects.claimed_nullifiers.is_empty()
            || !effects.claimed_shielded_receives.is_empty()
            || !effects.claimed_shielded_spends.is_empty()
            || !effects.claimed_contract_calls.is_empty()
            || !effects.shielded_mints.is_empty()
            || !effects.unshielded_mints.is_empty()
            || !effects.unshielded_outputs.is_empty()
            || !effects.claimed_unshielded_spends.is_empty()
        {
            return Err("mint recipient receive effects differ from 100-unit input".into());
        }
        let receive_stem = "unshielded-original-mint-contract-receive-100";
        let proof = v3::prove_standalone(
            root,
            RECEIVE_CONTRACT_CIRCUIT,
            receive_stem,
            &prepared_receive,
            &verifier,
            None,
        )?;
        positional_transaction::verify_separately(root, RECEIVE_CONTRACT_CIRCUIT, receive_stem)?;
        receive_standalone = Some(proof);
        calls.push(prepared_receive);
    }
    let offer = if let PublicAddress::User(owner) = destination {
        let TokenType::Unshielded(raw_type) = token else {
            return Err("mint-to-user token is not unshielded".into());
        };
        Some(UnshieldedOffer::<Signature, DefaultDB> {
            inputs: vec![].into(),
            outputs: vec![UtxoOutput {
                value: 100,
                owner,
                type_: raw_type,
            }]
            .into(),
            signatures: vec![].into(),
        })
    } else {
        None
    };
    let resolver_names = if recipient.is_some() {
        vec![name, RECEIVE_CONTRACT_CIRCUIT]
    } else {
        vec![name]
    };
    let resolver = qualified_coin_funding::fee_resolver_for_circuits(root, &resolver_names)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> = Intent::new(
        &mut rng,
        offer,
        None,
        calls,
        vec![],
        vec![],
        None,
        funded.time + funded.ledger.parameters.global_ttl,
    );
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let proven = executor
        .block_on(midnight_ledger::test_utilities::tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("{name} transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut tx_calls = sealed.calls();
    let (segment, mint_call) = tx_calls
        .next()
        .ok_or("mint destination transaction missing mint call")?;
    if mint_call.address != minter_address
        || mint_call.entry_point != EntryPointBuf(name.as_bytes().to_vec())
    {
        return Err("mint destination transaction changed mint call address or entry".into());
    }
    let ProofVersioned::V3(mint_proof) = &mint_call.proof else {
        return Err("mint destination transaction proof is not v3".into());
    };
    if mint_proof == standalone {
        return Err("mint destination transaction proof equals standalone proof".into());
    }
    if let Some(address) = recipient_address {
        let (other_segment, receive_call) = tx_calls
            .next()
            .ok_or("mint contract transaction missing receive call")?;
        let ProofVersioned::V3(receive_proof) = &receive_call.proof else {
            return Err("mint recipient transaction proof is not v3".into());
        };
        if other_segment != segment
            || receive_call.address != address
            || receive_call.entry_point
                != EntryPointBuf(RECEIVE_CONTRACT_CIRCUIT.as_bytes().to_vec())
            || Some(receive_proof) == receive_standalone.as_ref()
        {
            return Err("mint contract receive call differs from bound recipient".into());
        }
        v3::write_new(
            root.join("unshielded-original-mint-contract-receive-100-transaction.proof"),
            &receive_proof.0,
        )?;
    }
    if tx_calls.next().is_some() || v3::positive_fee_dust_spends(&sealed) != 1 {
        return Err("mint destination call count or positive fee funding differs".into());
    }
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    v3::write_new(
        root.join(format!("{stem}-transaction.bin")),
        &transaction_bytes,
    )?;
    v3::write_new(
        root.join(format!("{stem}-transaction.proof")),
        &mint_proof.0,
    )?;
    let time = funded.time;
    let strictness = WellFormedStrictness::default();
    let verified = sealed.well_formed(&funded.ledger, strictness, time)?;
    let result = funded.apply(&sealed, strictness)?;
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("{name} funded default-strict application failed: {result:?}").into());
    }
    let minter_after = funded
        .ledger
        .contract
        .get(&minter_address)
        .ok_or("mint minter absent after apply")?;
    let expected_data = contract::initial_state(ConstructorContext::new(()))?;
    if minter_after.data.get_ref() != expected_data.ledger_state.get_ref()
        || !minter_after.balance.is_empty()
    {
        return Err("mint destination source state or balance changed".into());
    }
    if let Some(address) = recipient_address {
        let recipient_after = funded
            .ledger
            .contract
            .get(&address)
            .ok_or("mint recipient absent after apply")?;
        if recipient_after.data.get_ref() != expected_data.ledger_state.get_ref()
            || recipient_after.balance.get(&token).map(|value| *value) != Some(100)
            || recipient_after.balance.size() != 1
        {
            return Err("mint contract recipient state or 100-unit balance differs".into());
        }
        let mut bytes = Vec::new();
        tagged_serialize(recipient_after, &mut bytes)?;
        v3::write_new(root.join(format!("{stem}-recipient-state.bin")), &bytes)?;
    } else {
        let PublicAddress::User(owner) = destination else {
            return Err("mint user recipient tag changed".into());
        };
        let TokenType::Unshielded(raw_type) = token else {
            return Err("mint user token tag changed".into());
        };
        let matches = funded
            .ledger
            .utxo
            .utxos
            .iter()
            .filter(|row| {
                let (utxo, _) = &**row;
                utxo.owner == owner && utxo.type_ == raw_type && utxo.value == 100
            })
            .count();
        if matches != 1 {
            return Err(format!("mint user 100-unit UTXO count differs: {matches}").into());
        }
    }
    let updated = funded.ledger.clone();
    let context = |state: LedgerState<DefaultDB>| TransactionContext {
        ref_state: state,
        block_context: BlockContext {
            tblock: time,
            last_block_time: time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (replayed, replay_result) = updated.apply(&verified, &context(updated.clone()));
    if !matches!(
        replay_result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != updated
    {
        return Err(
            format!("mint destination replay refused incorrectly: {replay_result:?}").into(),
        );
    }
    let mut after_bytes = Vec::new();
    let mut replay_bytes = Vec::new();
    tagged_serialize(&updated, &mut after_bytes)?;
    tagged_serialize(&replayed, &mut replay_bytes)?;
    if after_bytes != replay_bytes {
        return Err("mint destination replay changed serialized Ledger bytes".into());
    }
    v3::write_new(root.join(format!("{stem}-ledger-after.bin")), &after_bytes)?;
    v3::write_new(
        root.join(format!("{stem}-ledger-replay.bin")),
        &replay_bytes,
    )?;
    println!(
        "{name}: transaction destination matched 100-unit claim, funded default-strict apply and replay gate passed"
    );
    Ok(())
}

fn run_send_self(root: &Path) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{
        HashOutput, PublicAddress, TokenType, UnshieldedTokenType,
    };
    use midnight_compact_runtime::transaction::RecordedCall;

    let mut rng = StdRng::seed_from_u64(0x0184_5353);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, SEND_SELF_CIRCUIT)?;
    let color = FixedBytes::new([0x73; 32]);
    let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput(color.into_array())));
    let mut state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new().insert(
            EntryPointBuf(SEND_SELF_CIRCUIT.as_bytes().to_vec()),
            operation.clone(),
        ),
        ContractMaintenanceAuthority::default(),
    );
    state.balance = state.balance.insert(token, 100);
    let deploy = ContractDeploy::new(&mut rng, state);
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let amount = BoundedUint::<340282366920938463463374607431768211455>::new(100)?;
    let mut context = observed.circuit_context(());
    context.query.call_context.balance = observed.contract().balance.clone();
    let recorded = contract::Contract::default()
        .recording
        .sendUnshieldedToSelfTest(context, color, amount)?;
    let prepared = RecordedCall::new(
        &observed,
        recorded,
        SEND_SELF_CIRCUIT,
        AlignedValue::concat(&[AlignedValue::from(color), AlignedValue::from(amount)]),
    )
    .prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("original self-send operation differs from installed v3 IR".into());
    }
    let transcript = prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original self-send has no guaranteed effects transcript")?;
    let effects = &transcript.effects;
    let claim = ClaimedUnshieldedSpendsKey(token, PublicAddress::Contract(deploy.address()));
    if prepared.fallible_public_transcript.is_some()
        || !effects.claimed_nullifiers.is_empty()
        || !effects.claimed_shielded_receives.is_empty()
        || !effects.claimed_shielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
        || !effects.shielded_mints.is_empty()
        || !effects.unshielded_mints.is_empty()
        || effects.claimed_unshielded_spends.size() != 1
        || effects.unshielded_outputs.size() != 1
        || effects.unshielded_inputs.size() != 1
        || effects
            .claimed_unshielded_spends
            .get(&claim)
            .map(|value| *value)
            != Some(100)
        || effects.unshielded_outputs.get(&token).map(|value| *value) != Some(100)
        || effects.unshielded_inputs.get(&token).map(|value| *value) != Some(100)
    {
        return Err("original self-send effects differ from 100-unit auto-receive".into());
    }
    let proof: Proof = v3::prove_standalone(
        root,
        SEND_SELF_CIRCUIT,
        SEND_SELF_STEM,
        &prepared,
        &verifier,
        None,
    )?;
    positional_transaction::verify_separately(root, SEND_SELF_CIRCUIT, SEND_SELF_STEM)?;
    v3::apply_with_balance(
        root,
        SEND_SELF_CIRCUIT,
        SEND_SELF_STEM,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        token,
        100,
        &proof,
    )?;
    println!(
        "{SEND_SELF_CIRCUIT}: original 100-unit auto-receive proof, funded strict apply and replay gate passed"
    );
    Ok(())
}

fn run_balance(root: &Path) -> Result<(), Box<dyn Error>> {
    run_balance_case(root, [0x71; 32], 100, BALANCE_STEM, 0x0184_4241)?;
    run_balance_case(root, [0x74; 32], 0, BALANCE_OTHER_STEM, 0x0184_4242)?;
    Ok(())
}

fn run_balance_case(
    root: &Path,
    query_color: [u8; 32],
    expected_result: u128,
    stem: &str,
    seed: u64,
) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{HashOutput, TokenType, UnshieldedTokenType};
    use midnight_compact_runtime::transaction::RecordedCall;

    let mut rng = StdRng::seed_from_u64(seed);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, BALANCE_CIRCUIT)?;
    let color = FixedBytes::new(query_color);
    let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput([0x71; 32])));
    let mut state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new().insert(
            EntryPointBuf(BALANCE_CIRCUIT.as_bytes().to_vec()),
            operation.clone(),
        ),
        ContractMaintenanceAuthority::default(),
    );
    state.balance = state.balance.insert(token, 100);
    let deploy = ContractDeploy::new(&mut rng, state);
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let mut context = observed.circuit_context(());
    context.query.call_context.balance = observed.contract().balance.clone();
    let recorded = contract::Contract::default()
        .recording
        .getUnshieldedBalanceTest(context, color)?;
    if recorded.execution.result.value() != expected_result {
        return Err("original balance read differs from seeded color lookup".into());
    }
    let prepared = RecordedCall::new(
        &observed,
        recorded,
        BALANCE_CIRCUIT,
        AlignedValue::from(color),
    )
    .prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("original balance call operation differs from installed v3 IR".into());
    }
    let proof: Proof =
        v3::prove_standalone(root, BALANCE_CIRCUIT, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, BALANCE_CIRCUIT, stem)?;
    v3::apply_with_balance(
        root,
        BALANCE_CIRCUIT,
        stem,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        token,
        100,
        &proof,
    )?;
    println!(
        "{BALANCE_CIRCUIT}: original {expected_result}-unit result, 100-unit seeded balance proof and funded strict replay gate passed"
    );
    Ok(())
}

fn run_balance_comparators(root: &Path) -> Result<(), Box<dyn Error>> {
    for (name, stem, amount, expected, seed) in [
        (
            BALANCE_GT_CIRCUIT,
            "unshielded-original-balance-gt-50",
            50,
            true,
            0x0184_4750,
        ),
        (
            BALANCE_GT_CIRCUIT,
            "unshielded-original-balance-gt-150",
            150,
            false,
            0x0184_4751,
        ),
        (
            BALANCE_LT_CIRCUIT,
            "unshielded-original-balance-lt-50",
            50,
            false,
            0x0184_4c50,
        ),
        (
            BALANCE_LT_CIRCUIT,
            "unshielded-original-balance-lt-150",
            150,
            true,
            0x0184_4c51,
        ),
    ] {
        run_balance_comparator(root, name, stem, amount, expected, seed)?;
    }
    Ok(())
}

fn run_balance_comparator(
    root: &Path,
    name: &'static str,
    stem: &str,
    amount: u128,
    expected: bool,
    seed: u64,
) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{HashOutput, TokenType, UnshieldedTokenType};
    use midnight_compact_runtime::transaction::RecordedCall;

    let mut rng = StdRng::seed_from_u64(seed);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, name)?;
    let color = FixedBytes::new([0x71; 32]);
    let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput(color.into_array())));
    let mut state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new().insert(EntryPointBuf(name.as_bytes().to_vec()), operation.clone()),
        ContractMaintenanceAuthority::default(),
    );
    state.balance = state.balance.insert(token, 100);
    let deploy = ContractDeploy::new(&mut rng, state);
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let amount = BoundedUint::<340282366920938463463374607431768211455>::new(amount)?;
    let mut context = observed.circuit_context(());
    context.query.call_context.balance = observed.contract().balance.clone();
    let generated = contract::Contract::default();
    let recorded = if name == BALANCE_GT_CIRCUIT {
        generated
            .recording
            .getUnshieldedBalanceGtTest(context, color, amount)?
    } else {
        generated
            .recording
            .getUnshieldedBalanceLtTest(context, color, amount)?
    };
    if recorded.execution.result != expected {
        return Err(format!(
            "original {name}({}) result differs from {expected}",
            amount.value()
        )
        .into());
    }
    let prepared = RecordedCall::new(
        &observed,
        recorded,
        name,
        AlignedValue::concat(&[AlignedValue::from(color), AlignedValue::from(amount)]),
    )
    .prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err(format!("original {name} operation differs from installed v3 IR").into());
    }
    let proof: Proof = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, name, stem)?;
    v3::apply_with_balance(
        root,
        name,
        stem,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        token,
        100,
        &proof,
    )?;
    println!(
        "{name}({}): original {expected} result, funded strict application and replay gate passed",
        amount.value()
    );
    Ok(())
}

fn run_send_contract(root: &Path) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{
        HashOutput, PublicAddress, TokenType, UnshieldedTokenType,
    };
    use midnight_compact_runtime::transaction::RecordedCall;

    let mut rng = StdRng::seed_from_u64(0x0184_5343);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, SEND_CONTRACT_CIRCUIT)?;
    let (_, recipient_verifier, recipient_operation) =
        v3::load_operation(root, RECEIVE_CONTRACT_CIRCUIT)?;
    let recipient = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        RECEIVE_CONTRACT_CIRCUIT,
        recipient_operation.clone(),
        &mut rng,
    );
    let recipient_address = recipient.address();
    let color = FixedBytes::new([0x72; 32]);
    let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput(color.into_array())));
    let mut source_state = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        HashMap::new().insert(
            EntryPointBuf(SEND_CONTRACT_CIRCUIT.as_bytes().to_vec()),
            operation.clone(),
        ),
        ContractMaintenanceAuthority::default(),
    );
    source_state.balance = source_state.balance.insert(token, 100);
    let source = ContractDeploy::new(&mut rng, source_state);
    let source_address = source.address();
    if source_address == recipient_address {
        return Err("send-to-contract source and recipient addresses coincide".into());
    }
    let observed = ObservedContractState::new(
        source_address,
        source.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let recipient_observed = ObservedContractState::new(
        recipient_address,
        recipient.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let recipient_argument = crate::types::ContractAddress {
        bytes: midnight_compact_runtime::ledger::contract_address_bytes(&recipient_address),
    };
    let amount = BoundedUint::<340282366920938463463374607431768211455>::new(100)?;
    let mut context = observed.circuit_context(());
    context.query.call_context.balance = observed.contract().balance.clone();
    let recorded = contract::Contract::default()
        .recording
        .sendUnshieldedToContractTest(context, color, amount, recipient_argument.clone())?;
    let prepared = RecordedCall::new(
        &observed,
        recorded,
        SEND_CONTRACT_CIRCUIT,
        AlignedValue::concat(&[
            AlignedValue::from(color),
            AlignedValue::from(amount),
            AlignedValue::from(recipient_argument),
        ]),
    )
    .prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("original send-to-contract operation differs from installed v3 IR".into());
    }
    let transcript = prepared
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original send-to-contract call has no guaranteed effects transcript")?;
    if prepared.fallible_public_transcript.is_some() {
        return Err("send-to-contract has an unexpected fallible effects transcript".into());
    }
    let effects = &transcript.effects;
    let claim = ClaimedUnshieldedSpendsKey(token, PublicAddress::Contract(recipient_address));
    if !effects.claimed_nullifiers.is_empty()
        || !effects.claimed_shielded_receives.is_empty()
        || !effects.claimed_shielded_spends.is_empty()
        || !effects.claimed_contract_calls.is_empty()
        || !effects.shielded_mints.is_empty()
        || effects.unshielded_outputs.size() != 1
        || effects.claimed_unshielded_spends.size() != 1
        || !effects.unshielded_inputs.is_empty()
        || !effects.unshielded_mints.is_empty()
        || *effects
            .unshielded_outputs
            .get(&token)
            .ok_or("send-to-contract output effect missing")?
            != 100
        || *effects
            .claimed_unshielded_spends
            .get(&claim)
            .ok_or("send-to-contract claim effect missing")?
            != 100
    {
        return Err("original send-to-contract effects differ from 100-unit transfer".into());
    }
    let proof: Proof = v3::prove_standalone(
        root,
        SEND_CONTRACT_CIRCUIT,
        SEND_CONTRACT_STEM,
        &prepared,
        &verifier,
        None,
    )?;
    positional_transaction::verify_separately(root, SEND_CONTRACT_CIRCUIT, SEND_CONTRACT_STEM)?;
    let receive_recorded = contract::Contract::default()
        .recording
        .receiveUnshieldedTest_call(&recipient_observed, (), color, amount)?;
    let prepared_receive = receive_recorded.prepare(recipient_verifier.clone(), Fr::from(0_u64))?;
    if prepared_receive.op != recipient_operation {
        return Err("original receive operation differs from installed v3 IR".into());
    }
    let receive_transcript = prepared_receive
        .guaranteed_public_transcript
        .as_ref()
        .ok_or("original receive call has no guaranteed effects transcript")?;
    let receive_effects = &receive_transcript.effects;
    if prepared_receive.fallible_public_transcript.is_some()
        || !receive_effects.claimed_nullifiers.is_empty()
        || !receive_effects.claimed_shielded_receives.is_empty()
        || !receive_effects.claimed_shielded_spends.is_empty()
        || !receive_effects.claimed_contract_calls.is_empty()
        || !receive_effects.shielded_mints.is_empty()
        || receive_transcript.effects.unshielded_inputs.size() != 1
        || *receive_transcript
            .effects
            .unshielded_inputs
            .get(&token)
            .ok_or("receive input effect missing")?
            != 100
        || !receive_transcript.effects.unshielded_outputs.is_empty()
        || !receive_transcript
            .effects
            .claimed_unshielded_spends
            .is_empty()
        || !receive_transcript.effects.unshielded_mints.is_empty()
    {
        return Err("original receive effects differ from 100-unit input".into());
    }
    let receive_proof: Proof = v3::prove_standalone(
        root,
        RECEIVE_CONTRACT_CIRCUIT,
        RECEIVE_CONTRACT_STEM,
        &prepared_receive,
        &recipient_verifier,
        None,
    )?;
    positional_transaction::verify_separately(
        root,
        RECEIVE_CONTRACT_CIRCUIT,
        RECEIVE_CONTRACT_STEM,
    )?;
    let mut funded = qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 1)?;
    funded.ledger.contract = funded
        .ledger
        .contract
        .insert(source_address, source.initial_state)
        .insert(recipient_address, recipient.initial_state);
    let resolver = qualified_coin_funding::fee_resolver_for_circuits(
        root,
        &[SEND_CONTRACT_CIRCUIT, RECEIVE_CONTRACT_CIRCUIT],
    )?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared)
            .add_call::<ProofPreimage>(prepared_receive);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let proven = executor
        .block_on(midnight_ledger::test_utilities::tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("two-contract transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        &resolver,
    ))?;
    let mut calls = sealed.calls();
    let (send_segment, send_call) = calls.next().ok_or("two-contract send call missing")?;
    let (receive_segment, receive_call) =
        calls.next().ok_or("two-contract receive call missing")?;
    if calls.next().is_some()
        || send_segment != receive_segment
        || send_call.address != source_address
        || receive_call.address != recipient_address
        || send_call.entry_point != EntryPointBuf(SEND_CONTRACT_CIRCUIT.as_bytes().to_vec())
        || receive_call.entry_point != EntryPointBuf(RECEIVE_CONTRACT_CIRCUIT.as_bytes().to_vec())
    {
        return Err("two-contract transaction call sequence or segment differs".into());
    }
    let midnight_ledger::structure::ProofVersioned::V3(transaction_send_proof) = &send_call.proof
    else {
        return Err("two-contract send proof is not v3".into());
    };
    let midnight_ledger::structure::ProofVersioned::V3(transaction_receive_proof) =
        &receive_call.proof
    else {
        return Err("two-contract receive proof is not v3".into());
    };
    if transaction_send_proof == &proof
        || transaction_receive_proof == &receive_proof
        || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err("two-contract proofs or positive fee funding differ".into());
    }
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-transaction.bin")),
        &transaction_bytes,
    )?;
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-transaction.proof")),
        &transaction_send_proof.0,
    )?;
    v3::write_new(
        root.join(format!("{RECEIVE_CONTRACT_STEM}-transaction.proof")),
        &transaction_receive_proof.0,
    )?;
    let time = funded.time;
    let strictness = WellFormedStrictness::default();
    let verified = sealed.well_formed(&funded.ledger, strictness, time)?;
    let result = funded.apply(&sealed, strictness)?;
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("two-contract default-strict application failed: {result:?}").into());
    }
    let source_after = funded
        .ledger
        .contract
        .get(&source_address)
        .ok_or("two-contract sender missing after apply")?;
    let recipient_after = funded
        .ledger
        .contract
        .get(&recipient_address)
        .ok_or("two-contract recipient missing after apply")?;
    if source_after.data.get_ref() != initial.ledger_state.get_ref()
        || recipient_after.data.get_ref() != initial.ledger_state.get_ref()
        || source_after
            .balance
            .get(&token)
            .map(|balance| *balance)
            .unwrap_or(0)
            != 0
        || recipient_after.balance.get(&token).map(|balance| *balance) != Some(100)
        || source_after
            .operations
            .get(&EntryPointBuf(SEND_CONTRACT_CIRCUIT.as_bytes().to_vec()))
            .map(|installed| (*installed).clone())
            != Some(operation.clone())
        || recipient_after
            .operations
            .get(&EntryPointBuf(RECEIVE_CONTRACT_CIRCUIT.as_bytes().to_vec()))
            .map(|installed| (*installed).clone())
            != Some(recipient_operation.clone())
    {
        return Err("two-contract post-application states or balances differ".into());
    }
    let updated = funded.ledger.clone();
    let context = |state: LedgerState<DefaultDB>| TransactionContext {
        ref_state: state,
        block_context: BlockContext {
            tblock: time,
            last_block_time: time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (replayed, replay_result) = updated.apply(&verified, &context(updated.clone()));
    if !matches!(
        replay_result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != updated
    {
        return Err(format!("two-contract replay refused incorrectly: {replay_result:?}").into());
    }
    let mut after_bytes = Vec::new();
    let mut replay_bytes = Vec::new();
    tagged_serialize(&updated, &mut after_bytes)?;
    tagged_serialize(&replayed, &mut replay_bytes)?;
    if after_bytes != replay_bytes {
        return Err("two-contract replay changed serialized Ledger bytes".into());
    }
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-ledger-after.bin")),
        &after_bytes,
    )?;
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-ledger-replay.bin")),
        &replay_bytes,
    )?;
    let mut source_bytes = Vec::new();
    let mut recipient_bytes = Vec::new();
    tagged_serialize(source_after, &mut source_bytes)?;
    tagged_serialize(recipient_after, &mut recipient_bytes)?;
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-source-state.bin")),
        &source_bytes,
    )?;
    v3::write_new(
        root.join(format!("{SEND_CONTRACT_STEM}-recipient-state.bin")),
        &recipient_bytes,
    )?;
    println!(
        "{SEND_CONTRACT_CIRCUIT}+{RECEIVE_CONTRACT_CIRCUIT}: original 100-unit two-contract transaction, funded strict apply and replay gate passed"
    );
    Ok(())
}
