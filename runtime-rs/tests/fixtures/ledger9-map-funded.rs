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

use mapcontract::{ledger_contract as contract, runtime as r};
use midnight_base_crypto::data_provider::{FetchMode, MidnightDataProvider, OutputMode};
use midnight_ledger::{
    construct::{ContractCallExt, communication_commitment},
    dust::{DUST_EXPECTED_FILES, DustResolver},
    prove::Resolver as LedgerResolver,
    semantics::TransactionResult,
    structure::{
        ContractDeploy, Intent, ProofPreimageMarker, ProofPreimageVersioned, ProofVersioned,
        Signature, Transaction,
    },
    test_utilities::{TestState, tx_prove},
    verify::WellFormedStrictness,
};
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf, IrBuf,
};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_storage::{arena::Sp, storage::HashMap};
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    proofs::{Proof, ProofPreimage, ProvingKeyMaterial, VerifierKey, Zkir},
};
use midnight_zkir::IrSource;
use midnight_zswap::prove::ZswapResolver;
use r::{
    Field,
    context::ConstructorContext,
    ledger::{ContractAddress, DefaultDB},
    public_state::PublicStateSource,
    transaction::{Observation, ObservedCallError, ObservedContractState},
};
use rand::{SeedableRng, rngs::StdRng};
use serde_json::json;
use std::{env, error::Error, fs, io, path::Path, sync::Arc};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let generated = args.next().ok_or("missing generated root")?;
    let capture = args.next().ok_or("missing preimage root")?;
    let proof = args.next().ok_or("missing proof root")?;
    let wrong_key = args.next().ok_or("missing same-k wrong verifier")?;
    let output = args.next().ok_or("missing output root")?;
    assert!(args.next().is_none());
    let generated = Path::new(&generated);
    let capture = Path::new(&capture);
    let proof = Path::new(&proof);
    let output = Path::new(&output);
    fs::create_dir(output)?;

    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let initial_state = initial.ledger_state.get_ref().clone();
    let native = contract::put(
        initial.into_circuit_context(ContractAddress::default()),
        true,
        Field::from(42u128),
    )?;
    let expected_state = native.public_state().clone();
    assert_ne!(initial_state, expected_state);

    let prover_bytes = fs::read(proof.join("statement.prover"))?;
    let verifier_bytes = fs::read(proof.join("statement.verifier"))?;
    let verifier: VerifierKey = tagged_deserialize(verifier_bytes.as_slice())?;
    let wrong: VerifierKey = tagged_deserialize(fs::File::open(&wrong_key)?)?;
    assert_ne!(verifier, wrong);
    let standalone: Proof = tagged_deserialize(fs::File::open(proof.join("statement.proof"))?)?;
    let raw = generated.join("zkir/put.zkir");
    let ir = IrSource::load(fs::File::open(&raw)?)?;
    assert_eq!(ir.k(), 7);
    let mut binary = Vec::new();
    tagged_serialize(&ir, &mut binary)?;
    assert_eq!(binary, fs::read(raw.with_extension("bzkir"))?);
    let op = ContractOperation::new(
        Some(verifier.clone()),
        Some(Sp::new(IrBuf::from(binary.clone()))),
    );
    let entry = EntryPointBuf(b"put".to_vec());
    let initial_contract = ContractState::new(
        initial_state,
        HashMap::new().insert(entry.clone(), op.clone()),
        ContractMaintenanceAuthority::default(),
    );
    let mut rng = StdRng::from_rng(rand::rngs::OsRng)?;
    let deploy = ContractDeploy::new(&mut rng, initial_contract);
    let address = deploy.address();
    env::var("MIDNIGHT_LEDGER_TEST_STATIC_DIR")?;
    let mut state: TestState<DefaultDB> = TestState::new(&mut rng);
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(state.give_fee_token(&mut rng, 1));
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
    let typescript: ProofPreimage =
        tagged_deserialize(fs::File::open(capture.join("honest.preimage"))?)?;
    let (commitment, randomness) = typescript
        .communications_commitment
        .ok_or("missing commitment")?;
    let contract = contract::Contract::default();
    let call = || {
        contract
            .recording()
            .put_call(&observed, (), true, Field::from(42u128))
    };
    assert!(matches!(call()?.prepare(wrong, randomness),
        Err(ObservedCallError::VerifierMismatch(ref name)) if name == "put"));
    let recorded = call()?;
    assert_eq!(recorded.recorded().execution.result, native.result);
    assert_eq!(
        recorded.recorded().execution.private_transcript_outputs,
        native.private_transcript_outputs
    );
    assert_eq!(
        recorded.recorded().execution.public_state(),
        &expected_state
    );
    let prepared = recorded.prepare(verifier, randomness)?;
    assert_eq!(prepared.address, address);
    assert_eq!(prepared.entry_point, entry);
    assert_eq!(prepared.op, op);
    let computed_commitment =
        communication_commitment(prepared.input.clone(), prepared.output.clone(), randomness);
    assert_eq!(computed_commitment, commitment);

    let ProofPreimageVersioned::V2(rust) =
        ProofPreimage::construct_proof(&prepared, computed_commitment)
    else {
        return Err("expected current ledger preimage".into());
    };
    assert_eq!(rust.inputs, typescript.inputs);
    assert_eq!(rust.private_transcript, typescript.private_transcript);
    assert_eq!(
        rust.public_transcript_inputs,
        typescript.public_transcript_inputs
    );
    assert_eq!(
        rust.public_transcript_outputs,
        typescript.public_transcript_outputs
    );
    assert_eq!(rust.key_location, typescript.key_location);
    assert_eq!(
        rust.communications_commitment,
        typescript.communications_commitment
    );
    assert_eq!(rust.binding_input, typescript.binding_input);

    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, state.time + state.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1u16, intent));
    let materials = Arc::new((prover_bytes, verifier_bytes, binary));
    let zswap = ZswapResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        midnight_zswap::ZSWAP_EXPECTED_FILES.to_owned(),
    )?);
    let dust = DustResolver(MidnightDataProvider::new(
        FetchMode::Synchronous,
        OutputMode::Log,
        DUST_EXPECTED_FILES.to_owned(),
    )?);
    let resolver = LedgerResolver::new(
        zswap,
        dust,
        Box::new(move |location| {
            let materials = materials.clone();
            Box::pin(async move {
                Ok::<_, io::Error>((location.0.as_ref() == "put").then(|| ProvingKeyMaterial {
                    prover_key: materials.0.clone(),
                    verifier_key: materials.1.clone(),
                    ir_source: materials.2.clone(),
                }))
            })
        }),
    );
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("Map.put funded proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, applied) = calls.next().ok_or("missing funded call")?;
    let ProofVersioned::V3(transaction_proof) = &applied.proof else {
        return Err("expected v3 transaction proof".into());
    };
    assert!(calls.next().is_none());
    assert_ne!(transaction_proof, &standalone);
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
    assert_eq!(dust_spends, 1);
    let verified =
        sealed.well_formed(&state.ledger, WellFormedStrictness::default(), state.time)?;
    let mut ctx = state.context();
    ctx.block_context.last_block_time = state.time;
    let (updated, result) = state.ledger.apply(&verified, &ctx);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("missing applied state")?;
    assert_eq!(actual.data.get_ref(), &expected_state);
    assert_eq!(actual.operations.get(&entry).as_deref(), Some(&op));
    let strict_recheck_refused = sealed
        .well_formed(&updated, WellFormedStrictness::default(), state.time)
        .is_err();
    ctx.ref_state = updated.clone();
    let (replayed, result) = updated.apply(&verified, &ctx);
    assert!(
        matches!(
            result,
            TransactionResult::Failure(
                midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                    midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
                )
            )
        ),
        "{result:?}"
    );
    assert_eq!(replayed, updated);
    let (mut after, mut replay, mut tx_bytes) = (Vec::new(), Vec::new(), Vec::new());
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    tagged_serialize(&sealed, &mut tx_bytes)?;
    assert_eq!(after, replay);
    fs::write(output.join("applied-transaction.bin"), &tx_bytes)?;
    fs::write(output.join("transaction-bound.proof"), &transaction_proof.0)?;
    fs::write(output.join("standalone.proof.raw"), &standalone.0)?;
    fs::write(output.join("ledger-after.bin"), &after)?;
    fs::write(output.join("ledger-replay.bin"), &replay)?;
    println!(
        "{}",
        json!({
            "status":"passed","circuit":"put","k":ir.k(),
            "ledger_matches_native":true,"typescript_preimage_fields_match":true,
            "communication_commitment_recomputed":true,
            "positive_fee_dust_spends":dust_spends,"default_strict":true,
            "wrong_verifier_refused":true,"replay_refusal":"IntentAlreadyExists",
            "replay_unchanged":true,"transaction_proof_distinct":true,
            "strict_recheck_refused":strict_recheck_refused,
            "transaction_proof_bytes":transaction_proof.0.len(),
            "standalone_proof_bytes":standalone.0.len(),"transaction_bytes":tx_bytes.len()
        })
    );
    Ok(())
}
