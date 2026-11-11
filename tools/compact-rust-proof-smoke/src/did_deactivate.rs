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
//! Original DID deactivate: seeded prior state, strict call and separate Dust.
use super::did_deactivate_support::Witness;
use super::*;
use compact_rust_did_adoption_fixture::{ledger_contract as c, ledger_slots as slots, types};
use midnight_compact_runtime as runtime;
use midnight_ledger::{structure::ProofVersioned, test_utilities::tx_prove};
use midnight_onchain_state::state::IrBuf;
use midnight_storage::arena::Sp;
use midnight_transient_crypto::proofs::Zkir;
use midnight_zkir_v3::IrSource as IrSourceV3;
use serde_json::Value;
use std::{cell::RefCell, io::Write};

pub(super) enum ProofProtocol {
    V2,
    V3 { changed_input: AlignedValue },
}
pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    let capture: Value = serde_json::from_str(include_str!(
        "../../../runtime-rs/tests/fixtures/unit-composition.json"
    ))?;
    let row = capture["cases"]
        .as_array()
        .ok_or("missing DID cases")?
        .iter()
        .find(|r| r["id"] == "did/valid")
        .ok_or("missing DID valid row")?;
    run_with(
        root,
        row,
        prepare_ordinary,
        check_ordinary_final,
        ProofProtocol::V2,
    )
    .map(|_| ())
}

// Share funded default-strict application, with source-specific preparation and
// post-application checks. Historical callers do not acquire new output files.
pub(super) fn run_with(
    root: &Path,
    row: &Value,
    prepare: impl FnOnce(
        &Path,
        &ObservedContractState,
        &Value,
    ) -> Result<
        (ContractCallPrototype<DefaultDB>, StateValue<DefaultDB>),
        Box<dyn Error>,
    >,
    check_final: impl FnOnce(
        midnight_compact_runtime::ledger::ContractAddress,
        &ContractState<DefaultDB>,
        &Value,
    ) -> Result<String, Box<dyn Error>>,
    protocol: ProofProtocol,
) -> Result<Value, Box<dyn Error>> {
    let v3 = matches!(protocol, ProofProtocol::V3 { .. });
    if v3 && std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "Ledger 9 DID proof requires local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    let prior: ContractState<DefaultDB> =
        tagged_deserialize(&mut hex::decode(row["before"].as_str().unwrap())?.as_slice())?;
    let mut rng = StdRng::seed_from_u64(0x0251);
    let entry = EntryPointBuf(b"deactivate".to_vec());
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/deactivate.verifier"),
    )?))?;
    let deploy = if v3 {
        let ir = IrSourceV3::load(File::open(root.join("zkir/deactivate.zkir"))?)?;
        let binary = std::fs::read(root.join("zkir/deactivate.bzkir"))?;
        let mut expected = Vec::new();
        tagged_serialize(&ir, &mut expected)?;
        if expected != binary || ir.k() != 11 {
            return Err("DID v3 IR differs from the bound raw circuit or k11 recipe".into());
        }
        let operation =
            ContractOperation::new(Some(verifier.clone()), Some(Sp::new(IrBuf::from(binary))));
        let operations = HashMap::new().insert(entry.clone(), operation);
        let contract = ContractState::new(
            prior.data.get_ref().clone(),
            operations,
            ContractMaintenanceAuthority::default(),
        );
        ContractDeploy::new(&mut rng, contract)
    } else {
        make_deploy(root, "deactivate", prior.data.get_ref().clone(), &mut rng)?
    };
    let address = deploy.address();
    let mut state = super::qualified_coin_funding::fee_funded_state(&mut rng)?;
    state.ledger.contract = state
        .ledger
        .contract
        .insert(address, deploy.initial_state.clone());
    let observed = ObservedContractState::new(
        address,
        deploy.initial_state,
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let (prepared, expected) = prepare(root, &observed, row)?;
    let installed_operation = prepared.op.clone();
    let standalone = match protocol {
        ProofProtocol::V3 { changed_input } => Some(
            super::kernel_shielded_effects::prove_and_verify_call_with_input_control::<IrSourceV3>(
                root,
                "deactivate",
                &prepared,
                &verifier,
                Some(changed_input),
                FetchMode::Synchronous,
            )?,
        ),
        ProofProtocol::V2 => None,
    };
    let proof_bytes = if let Some((proof, _)) = &standalone {
        proof.0.len()
    } else {
        super::kernel_shielded_effects::prove_and_verify_call_measured(
            root,
            "deactivate",
            &prepared,
            &verifier,
        )?
    };
    let ttl = if v3 {
        state.time + state.ledger.parameters.global_ttl
    } else {
        Timestamp::from_secs(state.time.to_secs() + 3600)
    };
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, ttl).add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1u16, intent));
    let resolver = super::qualified_coin_funding::fee_resolver(root, "deactivate")?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let proven = if v3 {
        executor
            .block_on(tx_prove(
                StdRng::from_rng(rand::rngs::OsRng)?,
                &tx,
                &resolver,
            ))
            .map_err(|error| format!("DID transaction proving failed: {error:?}"))?
    } else {
        let params = MidnightDataProvider::new(FetchMode::OnDemand, OutputMode::Log, vec![])?;
        let provider = LocalProvingProvider {
            rng: StdRng::seed_from_u64(0x02515052),
            resolver: &resolver,
            params: &params,
        };
        futures_executor::block_on(
            tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
        )?
    };
    let sealed = proven.seal(StdRng::seed_from_u64(0x02515345));
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let transaction_proof = if let Some((standalone, _)) = &standalone {
        let mut calls = sealed.calls();
        let (_, call) = calls.next().ok_or("missing funded DID call")?;
        let ProofVersioned::V3(proof) = &call.proof else {
            return Err("expected a v3 transaction-bound DID proof".into());
        };
        if calls.next().is_some() || proof == standalone {
            return Err(
                "DID needs one fresh transaction-bound proof distinct from the standalone proof"
                    .into(),
            );
        }
        let dust_spends = sealed
            .intents()
            .filter_map(|(_, intent)| intent.dust_actions)
            .map(|actions| {
                actions
                    .spends
                    .iter()
                    .filter(|spend| !spend.proof.0.is_empty() && spend.v_fee > 0)
                    .count()
            })
            .sum::<usize>();
        if dust_spends != 1 {
            return Err("expected one proved, positive-fee Dust spend".into());
        }
        Some(proof.0.clone())
    } else {
        None
    };
    let verified =
        sealed.well_formed(&state.ledger, WellFormedStrictness::default(), state.time)?;
    let context = TransactionContext {
        ref_state: state.ledger.clone(),
        block_context: BlockContext {
            tblock: state.time,
            last_block_time: state.time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, result) = state.ledger.apply(&verified, &context);
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("DID apply failed: {result:?}").into());
    }
    let actual = updated.contract.get(&address).ok_or("DID missing")?;
    if actual.data.get_ref() != &expected
        || actual.operations.get(&entry).as_deref() != Some(&installed_operation)
    {
        return Err("DID final data or installed operation differs".into());
    }
    let replay_context = TransactionContext {
        ref_state: updated.clone(),
        block_context: BlockContext {
            tblock: state.time,
            last_block_time: state.time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (replayed, replay_result) = updated.apply(&verified, &replay_context);
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
            format!("DID replay refused incorrectly or mutated state: {replay_result:?}").into(),
        );
    }
    let stale_error = check_final(address, actual, row)?;
    let mut applied_bytes = Vec::new();
    if let (Some((proof, inputs)), Some(transaction_proof)) = (&standalone, &transaction_proof) {
        let mut after = Vec::new();
        let mut after_replay = Vec::new();
        tagged_serialize(&updated, &mut after)?;
        tagged_serialize(&replayed, &mut after_replay)?;
        if after_replay != after {
            return Err("DID replay changed serialized ledger bytes".into());
        }
        tagged_serialize(&sealed, &mut applied_bytes)?;
        let mut public_inputs = Vec::new();
        tagged_serialize(inputs, &mut public_inputs)?;
        for (name, bytes) in [
            (
                "did-functional-applied-transaction.bin",
                applied_bytes.as_slice(),
            ),
            ("deactivate-input-control.proof", proof.0.as_slice()),
            ("deactivate-input-control.inputs", public_inputs.as_slice()),
            (
                "deactivate-transaction-bound.proof",
                transaction_proof.as_slice(),
            ),
            ("did-ledger-after.bin", after.as_slice()),
            ("did-ledger-replay.bin", after_replay.as_slice()),
        ] {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(name))?
                .write_all(bytes)?;
        }
    }
    println!(
        "original DID deactivate: proof verified, changed binding rejected, default-strict separate Dust ledger apply; exact final state and stale repeat rejection; constructor-derived seeded prior state, not a full DID lifecycle"
    );
    let mut final_state = Vec::new();
    tagged_serialize(actual, &mut final_state)?;
    Ok(serde_json::json!({
        "proof_bytes":proof_bytes,"changed_binding_rejected":true,
        "changed_version_input_rejected":v3,"changed_input_same_geometry":v3,
        "strict_application":"default","replay_refusal":"IntentAlreadyExists",
        "replay_unchanged":replayed == updated,"stale_version_rejected":true,
        "stale_version_error":stale_error,
        "standalone_call_proof_file": if v3 { Some("deactivate-input-control.proof") } else { None },
        "applied_transaction_file": if v3 { Some("did-functional-applied-transaction.bin") } else { None },
        "applied_transaction_bytes":applied_bytes.len(),
        "transaction_proof_version": if v3 { 3 } else { 2 },
        "transaction_proof_bytes":transaction_proof.as_ref().map_or(0, Vec::len),
        "funded_dust_spends": if v3 { Some(1) } else { None },
        "proof_relationship":"standalone mutation-control proof; transaction independently proved and default-strictly verified",
        "final_state_hex":hex::encode(final_state)
    }))
}

fn check_ordinary_final(
    address: midnight_compact_runtime::ledger::ContractAddress,
    actual: &ContractState<DefaultDB>,
    row: &Value,
) -> Result<String, Box<dyn Error>> {
    if slots::active.inspect(actual.data.get_ref())?
        || !slots::deactivated.inspect(actual.data.get_ref())?
    {
        return Err("DID final flags differ".into());
    }
    let second = c::recorded::deactivate(
        runtime::context::CircuitContext::from_contract_state(9u64, address, actual),
        &Witness {
            options: row["options"].clone(),
            calls: RefCell::default(),
        },
        types::SchnorrSignature {
            announcement: runtime::ec_mul_generator(Field::from(2u64))?,
            response: Field::from_le_bytes(&hex::decode(row["responseHex"].as_str().unwrap())?)
                .ok_or("invalid fixture response")?,
        },
        runtime::BoundedUint::new(0)?,
    );
    match second {
        Err(runtime::CompactError::AssertionFailed(message))
            if message == "Controller authorization version is stale" =>
        {
            Ok(message)
        }
        _ => Err("DID repeat did not reject stale version first".into()),
    }
}

fn prepare_ordinary(
    root: &Path,
    observed: &ObservedContractState,
    row: &Value,
) -> Result<(ContractCallPrototype<DefaultDB>, StateValue<DefaultDB>), Box<dyn Error>> {
    let signature = types::SchnorrSignature {
        announcement: runtime::ec_mul_generator(Field::from(2u64))?,
        response: Field::from_le_bytes(&hex::decode(row["responseHex"].as_str().unwrap())?)
            .ok_or("invalid fixture response")?,
    };
    let witness = || Witness {
        options: row["options"].clone(),
        calls: RefCell::default(),
    };
    let native = c::deactivate(
        observed.circuit_context(7u64),
        &witness(),
        signature.clone(),
        runtime::BoundedUint::new(0)?,
    )?;
    let recorded = c::recorded::deactivate(
        observed.circuit_context(7u64),
        &witness(),
        signature.clone(),
        runtime::BoundedUint::new(0)?,
    )?;
    if native.context.query.state != recorded.execution.context.query.state
        || native.gas_cost != recorded.execution.gas_cost
        || native.private_transcript_outputs != recorded.execution.private_transcript_outputs
    {
        return Err("DID native/recorded mismatch".into());
    }
    let expected = native.context.query.state.get_ref().clone();
    let input = AlignedValue::from((
        signature.clone(),
        runtime::BoundedUint::<18446744073709551615>::new(0)?,
    ));
    let manual = check_generated_trace(root, "deactivate", recorded, input)?;
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/deactivate.verifier"),
    )?))?;
    let generated = c::Contract::from(witness());
    let prepared = generated
        .recording()
        .deactivate_call(observed, 7u64, signature, runtime::BoundedUint::new(0)?)?
        .prepare(verifier.clone(), Fr::from(0u64))?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("DID typed/manual preparation mismatch".into());
    }
    Ok((prepared, expected))
}
