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
const SEND_CONTRACT_CIRCUIT: &str = "sendUnshieldedToContractTest";
const SEND_CONTRACT_STEM: &str = "unshielded-original-send-contract-100";
const RECEIVE_CONTRACT_CIRCUIT: &str = "receiveUnshieldedTest";
const RECEIVE_CONTRACT_STEM: &str = "unshielded-original-receive-contract-100";
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
const GENERATED_SEND_CONTRACT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/sendUnshieldedToContractTest.zkir"
));
const GENERATED_RECEIVE_CONTRACT_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/receiveUnshieldedTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (name, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/mintUnshieldedToSelfTest.zkir", GENERATED_ZKIR),
        ("zkir/getUnshieldedBalanceTest.zkir", GENERATED_BALANCE_ZKIR),
        (
            "zkir/sendUnshieldedToContractTest.zkir",
            GENERATED_SEND_CONTRACT_ZKIR,
        ),
        (
            "zkir/receiveUnshieldedTest.zkir",
            GENERATED_RECEIVE_CONTRACT_ZKIR,
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
    run_send_contract(root)?;
    Ok(())
}

fn run_balance(root: &Path) -> Result<(), Box<dyn Error>> {
    use midnight_compact_runtime::ledger::{HashOutput, TokenType, UnshieldedTokenType};
    use midnight_compact_runtime::transaction::RecordedCall;

    let mut rng = StdRng::seed_from_u64(0x0184_4241);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, BALANCE_CIRCUIT)?;
    let color = FixedBytes::new([0x71; 32]);
    let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput(color.into_array())));
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
    if recorded.execution.result.value() != 100 {
        return Err("original balance read did not return 100".into());
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
    let proof: Proof = v3::prove_standalone(
        root,
        BALANCE_CIRCUIT,
        BALANCE_STEM,
        &prepared,
        &verifier,
        None,
    )?;
    positional_transaction::verify_separately(root, BALANCE_CIRCUIT, BALANCE_STEM)?;
    v3::apply_with_balance(
        root,
        BALANCE_CIRCUIT,
        BALANCE_STEM,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        token,
        100,
        &proof,
    )?;
    println!(
        "{BALANCE_CIRCUIT}: original 100-unit balance proof and funded strict replay gate passed"
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
