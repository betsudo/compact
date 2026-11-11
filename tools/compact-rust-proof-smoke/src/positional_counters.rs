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

//! Counter binding, decoder, handoff, parameterized, and nested-call scenarios.
//!
//! The caller owns the shared RNG and exports the verified baseline handoff.

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use compact_rust_counter_fixture::ledger_contract as counter_contract;
use compact_rust_counter_parameter_fixture::ledger_contract as counter_parameter_contract;
use compact_rust_stateful_circuit_call_fixture::ledger_contract as nested_contract;
use midnight_compact_runtime::BoundedUint;
use midnight_compact_runtime::context::ConstructorContext;
use midnight_compact_runtime::ledger::{StateValue, read_counter};
use midnight_compact_runtime::transaction::{
    Observation, ObservedCallError, ObservedContractState, RecordedCall,
};
use midnight_onchain_state::state::{ContractMaintenanceAuthority, ContractState};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_storage::storage::HashMap;
use midnight_transient_crypto::curve::Fr;
use midnight_transient_crypto::proofs::VerifierKey;
use rand::{SeedableRng, rngs::StdRng};

use super::local_v3_application as v3;
use super::{
    ContractCallPrototype, DefaultDB, Intent, PedersenRandomness, ProofPreimage,
    ProofPreimageMarker, Signature, Timestamp, Transaction, check_counter_statement,
    check_generated_trace, check_observed_call_parity, check_transaction, make_deploy,
    positional_suite::baseline_operation,
};

pub(super) fn check_binding(
    counter_root: &Path,
    cell_root: &Path,
    rng: &mut StdRng,
) -> Result<(), Box<dyn Error>> {
    const CIRCUIT: &str = "increment";
    let (ir, counter_verifier, operation) = baseline_operation(counter_root, CIRCUIT, 31)?;
    let counter_initial = counter_contract::initial_state(ConstructorContext::new(()))?;
    let counter_deploy = v3::deploy(
        counter_initial.ledger_state.get_ref().clone(),
        CIRCUIT,
        operation,
        rng,
    );
    let counter_observed = ObservedContractState::new(
        counter_deploy.address(),
        counter_deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let native = counter_contract::increment(counter_observed.circuit_context(()))?;
    let counter_recorded =
        counter_contract::recorded::increment(counter_observed.circuit_context(()))?;
    let expected = native.context.query.state.get_ref();
    let StateValue::Array(fields) = expected else {
        return Err("counter contract state is not an array".into());
    };
    if read_counter(fields.get(0).ok_or("counter field missing")?)? != 1
        || counter_recorded.execution.context.query.state != native.context.query.state
        || counter_recorded.execution.context.query.effects != native.context.query.effects
        || counter_recorded.execution.gas_cost != native.gas_cost
        || !counter_recorded
            .execution
            .private_transcript_outputs
            .is_empty()
        || !native.private_transcript_outputs.is_empty()
    {
        return Err("Counter native and recorded execution differ".into());
    }
    let counter_manual = RecordedCall::new(&counter_observed, counter_recorded, CIRCUIT, ())
        .prepare(counter_verifier.clone(), Fr::from(0_u64))?;
    let mut state_bytes = Vec::new();
    tagged_serialize(counter_observed.contract(), &mut state_bytes)?;
    let decoded = ObservedContractState::decode(
        counter_observed.address(),
        &state_bytes,
        counter_observed.observation(),
    )?;
    if decoded.contract() != counter_observed.contract() {
        return Err("observed state decoder changed ledger state".into());
    }
    state_bytes.push(0);
    if ObservedContractState::decode(
        counter_observed.address(),
        &state_bytes,
        counter_observed.observation(),
    )
    .is_ok()
    {
        return Err("observed state decoder accepted trailing bytes".into());
    }
    let missing_operation = ObservedContractState::new(
        counter_observed.address(),
        ContractState::new(
            counter_deploy.initial_state.data.get_ref().clone(),
            HashMap::new(),
            ContractMaintenanceAuthority::default(),
        ),
        counter_observed.observation(),
    );
    let missing_call = counter_contract::Contract::default()
        .recording
        .increment_call(&missing_operation, ())?;
    if !matches!(
        missing_call.prepare(counter_verifier.clone(), Fr::from(0u64)),
        Err(ObservedCallError::MissingOperation(_))
    ) {
        return Err("observed call accepted a missing operation".into());
    }
    let (_, wrong_verifier, _) = baseline_operation(cell_root, "set_flag", 35)?;
    let wrong_key_call = counter_contract::Contract::default()
        .recording
        .increment_call(&counter_observed, ())?;
    if !matches!(
        wrong_key_call.prepare(wrong_verifier, Fr::from(0u64)),
        Err(ObservedCallError::VerifierMismatch(_))
    ) {
        return Err("observed call accepted a different verifier".into());
    }
    let mut bad_address = counter_observed.address();
    bad_address.0.0[0] ^= 1;
    let wrong_address = ObservedContractState::new(
        bad_address,
        counter_deploy.initial_state.clone(),
        counter_observed.observation(),
    );
    let recorded_at_address = counter_contract::Contract::default()
        .recording
        .increment(counter_observed.circuit_context(()))?;
    if !matches!(
        RecordedCall::new(&wrong_address, recorded_at_address, "increment", ())
            .prepare(counter_verifier.clone(), Fr::from(0u64)),
        Err(ObservedCallError::AddressMismatch)
    ) {
        return Err("observed call accepted a different address".into());
    }
    let recorded_for_stale = counter_contract::Contract::default()
        .recording
        .increment(counter_observed.circuit_context(()))?;
    let stale_state = ContractState::new(
        recorded_for_stale
            .execution
            .context
            .query
            .state
            .get_ref()
            .clone(),
        counter_deploy.initial_state.operations.clone(),
        ContractMaintenanceAuthority::default(),
    );
    let stale_observed = ObservedContractState::new(
        counter_observed.address(),
        stale_state,
        counter_observed.observation(),
    );
    if !matches!(
        RecordedCall::new(&stale_observed, recorded_for_stale, "increment", ())
            .prepare(counter_verifier.clone(), Fr::from(0u64)),
        Err(ObservedCallError::StateMismatch)
    ) {
        return Err("observed call accepted a stale state".into());
    }
    let counter_call = counter_contract::Contract::default()
        .recording
        .increment_call(&counter_observed, ())?
        .prepare(counter_verifier.clone(), Fr::from(0u64))?;
    let preproof_bytes =
        |call: ContractCallPrototype<DefaultDB>| -> Result<Vec<u8>, Box<dyn Error>> {
            let mut rng = StdRng::seed_from_u64(0x0041_4452_3433);
            let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
                Intent::empty(&mut rng, Timestamp::from_secs(0)).add_call::<ProofPreimage>(call);
            let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
            let mut bytes = Vec::new();
            tagged_serialize(&tx, &mut bytes)?;
            Ok(bytes)
        };
    if format!("{counter_manual:?}") != format!("{counter_call:?}")
        || preproof_bytes(counter_manual.clone())? != preproof_bytes(counter_call.clone())?
    {
        return Err("Counter observed and direct pre-proof transactions differ".into());
    }
    for (stem, call) in [
        ("increment-direct", counter_manual),
        (CIRCUIT, counter_call),
    ] {
        check_counter_statement(&ir, &call)?;
        let proof =
            v3::prove_standalone(counter_root, CIRCUIT, stem, &call, &counter_verifier, None)?;
        v3::apply_after_deployment(
            counter_root,
            CIRCUIT,
            stem,
            call,
            counter_deploy.clone(),
            expected,
            &proof,
        )?;
    }
    if std::fs::read(counter_root.join("increment-direct-state.bin"))?
        != std::fs::read(counter_root.join("increment-state.bin"))?
    {
        return Err("Counter independently applied contract states differ".into());
    }
    println!(
        "Counter direct and observed pre-proof bytes and independently applied contract state match"
    );
    Ok(())
}

pub(super) fn check_parameterized(root: &Path, rng: &mut StdRng) -> Result<(), Box<dyn Error>> {
    let initial = counter_parameter_contract::initial_state(ConstructorContext::new(()))?;
    let deploy = make_deploy(
        root,
        "increment_by",
        initial.ledger_state.get_ref().clone(),
        rng,
    )?;
    let amount = BoundedUint::<65535>::new(3)?;
    let recorded = counter_parameter_contract::Contract::default()
        .recording
        .increment_by(initial.into_circuit_context(deploy.address()), amount)?;
    let manual = check_generated_trace(root, "increment_by", recorded, amount)?;
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/increment_by.verifier"),
    )?))?;
    let call = counter_parameter_contract::Contract::default()
        .recording
        .increment_by_call(&observed, (), amount)?
        .prepare(verifier, Fr::from(0u64))?;
    check_observed_call_parity(root, "increment_by", &deploy, &manual, &call)?;
    check_transaction(root, "increment_by", deploy, call, rng, |contract| {
        let StateValue::Array(fields) = contract.data.get_ref() else {
            return Err("counter-parameter state is not an array".into());
        };
        if read_counter(fields.get(0).ok_or("counter-parameter field missing")?)? != 3 {
            return Err("proven typed counter call did not add the parameter".into());
        }
        Ok(())
    })?;

    let initial = counter_parameter_contract::initial_state(ConstructorContext::new(()))?;
    let seeded = counter_parameter_contract::increment_by(
        initial.into_circuit_context(Default::default()),
        BoundedUint::<65535>::new(3)?,
    )?;
    let reset_deploy = make_deploy(
        root,
        "reset_round",
        seeded.context.query.state.get_ref().clone(),
        rng,
    )?;
    let reset_context = midnight_compact_runtime::context::CircuitContext::from_contract_state(
        (),
        reset_deploy.address(),
        &reset_deploy.initial_state,
    );
    let recorded = counter_parameter_contract::Contract::default()
        .recording
        .reset_round(reset_context)?;
    let manual = check_generated_trace(root, "reset_round", recorded, ())?;
    let observed = ObservedContractState::new(
        reset_deploy.address(),
        reset_deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/reset_round.verifier"),
    )?))?;
    let call = counter_parameter_contract::Contract::default()
        .recording
        .reset_round_call(&observed, ())?
        .prepare(verifier, Fr::from(0u64))?;
    check_observed_call_parity(root, "reset_round", &reset_deploy, &manual, &call)?;
    check_transaction(root, "reset_round", reset_deploy, call, rng, |contract| {
        let StateValue::Array(fields) = contract.data.get_ref() else {
            return Err("reset-round state is not an array".into());
        };
        if read_counter(fields.get(0).ok_or("reset-round field missing")?)? != 0 {
            return Err("proven typed Counter reset did not clear the seeded value".into());
        }
        Ok(())
    })?;
    Ok(())
}

pub(super) fn check_nested_calls(
    nested_root: &Path,
    rng: &mut StdRng,
) -> Result<(), Box<dyn Error>> {
    let nested_initial = nested_contract::initial_state(ConstructorContext::new(()))?;
    let nested_deploy = make_deploy(
        nested_root,
        "bump_twice",
        nested_initial.ledger_state.get_ref().clone(),
        rng,
    )?;
    let context = nested_initial.into_circuit_context(nested_deploy.address());
    let recorded = nested_contract::Contract::default()
        .recording
        .bump_twice(context)?;
    let nested_call = check_generated_trace(nested_root, "bump_twice", recorded, ())?;
    check_transaction(
        nested_root,
        "bump_twice",
        nested_deploy,
        nested_call,
        rng,
        |contract| {
            let StateValue::Array(fields) = contract.data.get_ref() else {
                return Err("nested Counter state is not an array".into());
            };
            if read_counter(fields.get(0).ok_or("nested Counter missing")?)? != 2 {
                return Err("proven nested call did not increment twice".into());
            }
            Ok(())
        },
    )?;

    let parameterized_initial = nested_contract::initial_state(ConstructorContext::new(()))?;
    let parameterized_deploy = make_deploy(
        nested_root,
        "add_twice",
        parameterized_initial.ledger_state.get_ref().clone(),
        rng,
    )?;
    let context = parameterized_initial.into_circuit_context(parameterized_deploy.address());
    let amount = BoundedUint::<65535>::new(3)?;
    let recorded = nested_contract::Contract::default()
        .recording
        .add_twice(context, amount)?;
    let parameterized_manual = check_generated_trace(nested_root, "add_twice", recorded, amount)?;
    let parameterized_observed = ObservedContractState::new(
        parameterized_deploy.address(),
        parameterized_deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let parameterized_verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        nested_root.join("keys/add_twice.verifier"),
    )?))?;
    let parameterized_call = nested_contract::Contract::default()
        .recording
        .add_twice_call(&parameterized_observed, (), amount)?
        .prepare(parameterized_verifier, Fr::from(0u64))?;
    check_observed_call_parity(
        nested_root,
        "add_twice",
        &parameterized_deploy,
        &parameterized_manual,
        &parameterized_call,
    )?;
    check_transaction(
        nested_root,
        "add_twice",
        parameterized_deploy,
        parameterized_call,
        rng,
        |contract| {
            let StateValue::Array(fields) = contract.data.get_ref() else {
                return Err("parameterized Counter state is not an array".into());
            };
            if read_counter(fields.get(0).ok_or("parameterized Counter missing")?)? != 6 {
                return Err("proven parameterized call did not increment twice".into());
            }
            Ok(())
        },
    )?;
    Ok(())
}
