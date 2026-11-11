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

//! Local Ledger 9 v3 call installation, verification and funded application.
use super::observational_binding::ProofBoundCall;
use super::positional_transaction::verify_separately;
use super::*;
use midnight_ledger::{
    structure::{ProofVersioned, VerifiedTransaction},
    test_utilities::{TxBound, tx_prove},
};
use midnight_onchain_state::state::IrBuf;
use midnight_storage::arena::Sp;
use midnight_transient_crypto::proofs::{Proof, Zkir};
use midnight_zkir_v3::IrSource as IrSourceV3;

pub(super) fn validate_pinned_artifacts(root: &Path, pins: &str) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("Ledger 9 artifact qualification requires local proving".into());
    }
    let pins: std::collections::BTreeMap<String, String> = serde_json::from_str(pins)?;
    let expected_raw = pins
        .keys()
        .filter_map(|path| path.strip_prefix("zkir/"))
        .filter(|name| name.ends_with(".zkir"))
        .map(std::ffi::OsString::from)
        .collect::<std::collections::BTreeSet<_>>();
    let actual_raw = std::fs::read_dir(root.join("zkir"))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let actual_raw = actual_raw
        .into_iter()
        .filter(|name| Path::new(name).extension().is_some_and(|ext| ext == "zkir"))
        .collect::<std::collections::BTreeSet<_>>();
    if actual_raw != expected_raw {
        return Err("Ledger 9 raw circuit inventory differs from the pinned fixture".into());
    }
    for (path, digest) in pins {
        let bytes = std::fs::read(root.join(&path))?;
        if hex::encode(midnight_base_crypto::hash::persistent_hash(&bytes).0) != digest {
            return Err(
                format!("Ledger 9 artifact differs from the pinned fixture: {path}").into(),
            );
        }
    }
    Ok(())
}

pub(super) fn validate_programs(
    root: &Path,
    expected_programs: &[&[u8]],
) -> Result<(), Box<dyn Error>> {
    let programs = root.join("contract/checked-programs");
    let names = std::fs::read_dir(&programs)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let expected_names = (0..expected_programs.len())
        .map(|index| std::ffi::OsString::from(format!("program-{index}.json")))
        .collect();
    if names != expected_names {
        return Err("checked-program inventory differs from the linked fixture".into());
    }
    for (index, expected) in expected_programs.iter().enumerate() {
        if std::fs::read(programs.join(format!("program-{index}.json")))? != *expected {
            return Err("checked program differs from the linked fixture".into());
        }
    }
    Ok(())
}

pub(super) fn read_verifier(root: &Path, name: &str) -> Result<VerifierKey, Box<dyn Error>> {
    Ok(tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.verifier")),
    )?))?)
}

pub(super) fn operation(
    root: &Path,
    name: &str,
    k: u8,
) -> Result<(IrSourceV3, VerifierKey, ContractOperation), Box<dyn Error>> {
    let (ir, verifier, operation) = load_operation(root, name)?;
    if ir.k() != k {
        return Err("v3 IR differs from the bound parameter recipe".into());
    }
    Ok((ir, verifier, operation))
}

pub(super) fn load_operation(
    root: &Path,
    name: &str,
) -> Result<(IrSourceV3, VerifierKey, ContractOperation), Box<dyn Error>> {
    let ir = IrSourceV3::load(File::open(root.join(format!("zkir/{name}.zkir")))?)?;
    let binary = std::fs::read(root.join(format!("zkir/{name}.bzkir")))?;
    let mut expected = Vec::new();
    tagged_serialize(&ir, &mut expected)?;
    if binary != expected {
        return Err("v3 binary IR differs from the bound raw circuit".into());
    }
    let verifier = read_verifier(root, name)?;
    let operation =
        ContractOperation::new(Some(verifier.clone()), Some(Sp::new(IrBuf::from(binary))));
    Ok((ir, verifier, operation))
}

pub(super) fn verify_standalone(
    root: &Path,
    stem: &str,
    verifier: &VerifierKey,
) -> Result<(), Box<dyn Error>> {
    let proof = Proof(std::fs::read(root.join(format!("{stem}.proof")))?);
    let bytes = std::fs::read(root.join(format!("{stem}.inputs")))?;
    let mut reader = bytes.as_slice();
    let inputs: Vec<Fr> = tagged_deserialize(&mut reader)?;
    if !reader.is_empty() {
        return Err("v3 proof inputs have trailing bytes".into());
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    let mut corrupted = proof;
    *corrupted.0.first_mut().ok_or("empty v3 proof")? ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupted, inputs.into_iter())
        .is_ok()
    {
        return Err("v3 verifier accepted corrupt proof".into());
    }
    Ok(())
}

pub(super) fn prove_standalone(
    root: &Path,
    name: &'static str,
    stem: &str,
    prepared: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    mutation: Option<kernel_shielded_effects::InputMutation>,
) -> Result<Proof, Box<dyn Error>> {
    let (proof, inputs) =
        kernel_shielded_effects::prove_and_verify_call_with_input_mutation::<IrSourceV3>(
            root,
            name,
            prepared,
            verifier,
            mutation,
            FetchMode::Synchronous,
        )?;
    write_new(root.join(format!("{stem}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(&inputs, &mut bytes)?;
    write_new(root.join(format!("{stem}.inputs")), &bytes)?;
    Ok(proof)
}

pub(super) fn deploy(
    data: StateValue<DefaultDB>,
    name: &str,
    operation: ContractOperation,
    rng: &mut StdRng,
) -> ContractDeploy<DefaultDB> {
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let contract = ContractState::new(
        data,
        HashMap::new().insert(entry, operation),
        ContractMaintenanceAuthority::default(),
    );
    ContractDeploy::new(rng, contract)
}

pub(super) fn write_new(path: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)
}

pub(super) fn apply(
    root: &Path,
    name: &'static str,
    stem: &str,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    expected: &StateValue<DefaultDB>,
    standalone: &Proof,
) -> Result<usize, Box<dyn Error>> {
    apply_call(
        root,
        name,
        stem,
        prepared,
        (deploy, false),
        standalone,
        |state| check_data(state, expected),
    )
    .map(|(proof_bytes, _)| proof_bytes)
}

pub(super) fn apply_after_deployment(
    root: &Path,
    name: &'static str,
    stem: &str,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    expected: &StateValue<DefaultDB>,
    standalone: &Proof,
) -> Result<usize, Box<dyn Error>> {
    apply_call(
        root,
        name,
        stem,
        prepared,
        (deploy, true),
        standalone,
        |state| check_data(state, expected),
    )
    .map(|(proof_bytes, _)| proof_bytes)
}

pub(super) fn apply_checked(
    root: &Path,
    name: &'static str,
    stem: &str,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    standalone: &Proof,
    check_state: impl FnOnce(&ContractState<DefaultDB>) -> Result<(), Box<dyn Error>>,
) -> Result<ContractState<DefaultDB>, Box<dyn Error>> {
    apply_call(
        root,
        name,
        stem,
        prepared,
        (deploy, true),
        standalone,
        check_state,
    )
    .map(|(_, state)| state)
}

fn check_data(
    state: &ContractState<DefaultDB>,
    expected: &StateValue<DefaultDB>,
) -> Result<(), Box<dyn Error>> {
    if state.data.get_ref() != expected {
        return Err("v3 call applied state differs".into());
    }
    Ok(())
}

fn apply_call(
    root: &Path,
    name: &'static str,
    stem: &str,
    prepared: ContractCallPrototype<DefaultDB>,
    deployment: (ContractDeploy<DefaultDB>, bool),
    standalone: &Proof,
    check_state: impl FnOnce(&ContractState<DefaultDB>) -> Result<(), Box<dyn Error>>,
) -> Result<(usize, ContractState<DefaultDB>), Box<dyn Error>> {
    let (deploy, apply_deployment) = deployment;
    let mut rng = StdRng::from_rng(rand::rngs::OsRng)?;
    let address = deploy.address();
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let installed = deploy
        .initial_state
        .operations
        .get(&entry)
        .ok_or("missing v3 call operation")?;
    if prepared.address != address || prepared.entry_point != entry || prepared.op != *installed {
        return Err("v3 prepared call differs from the installed operation".into());
    }
    let resolver = qualified_coin_funding::fee_resolver(root, name)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut funded = qualified_coin_funding::fee_funded_state_with_utxos(
        &mut rng,
        if apply_deployment { 2 } else { 1 },
    )?;
    if apply_deployment {
        apply_funded_deployment(
            root,
            stem,
            &deploy,
            &mut funded,
            &mut rng,
            &executor,
            &resolver,
        )?;
    } else {
        funded.ledger.contract = funded.ledger.contract.insert(address, deploy.initial_state);
    }
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("v3 call transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(funded.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, call) = calls.next().ok_or("missing funded v3 call call")?;
    let ProofVersioned::V3(proof) = &call.proof else {
        return Err("expected v3 transaction-bound v3 call proof".into());
    };
    if calls.next().is_some() || proof == standalone {
        return Err(
            "expected one transaction-bound v3 call proof distinct from the standalone proof"
                .into(),
        );
    }
    if positive_fee_dust_spends(&sealed) != 1 {
        return Err("expected one proved, positive-fee v3 call Dust spend".into());
    }
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    write_new(
        root.join(format!("{stem}-transaction.bin")),
        &transaction_bytes,
    )?;
    let verified =
        sealed.well_formed(&funded.ledger, WellFormedStrictness::default(), funded.time)?;
    let context = |state: LedgerState<DefaultDB>| TransactionContext {
        ref_state: state,
        block_context: BlockContext {
            tblock: funded.time,
            last_block_time: funded.time,
            ..BlockContext::default()
        },
        whitelist: None,
    };
    let (updated, result) = funded
        .ledger
        .apply(&verified, &context(funded.ledger.clone()));
    if !matches!(result, TransactionResult::Success(_)) {
        return Err(format!("v3 call strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("v3 call contract missing after apply")?;
    if actual.operations.get(&entry) != Some(installed) {
        return Err("v3 call changed the installed operation".into());
    }
    check_state(actual)?;
    let (replayed, result) = updated.apply(&verified, &context(updated.clone()));
    if !matches!(
        result,
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists
            )
        )
    ) || replayed != updated
    {
        return Err(
            format!("v3 call replay refused incorrectly or mutated ledger: {result:?}").into(),
        );
    }
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(root.join(format!("{stem}-state.bin")), &bytes)?;
    let mut after = Vec::new();
    let mut replay = Vec::new();
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    if after != replay {
        return Err("v3 call replay changed serialized ledger bytes".into());
    }
    write_new(root.join(format!("{stem}-ledger-after.bin")), &after)?;
    write_new(root.join(format!("{stem}-ledger-replay.bin")), &replay)?;
    write_new(root.join(format!("{stem}-transaction.proof")), &proof.0)?;
    println!("{stem}: funded default-strict apply; replay refused without mutation");
    Ok((proof.0.len(), actual.clone()))
}

pub(super) fn prove_and_apply_offer(
    root: &Path,
    (name, stem): (&'static str, &str),
    prepared: impl ProofBoundCall,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    verifier: &VerifierKey,
    expected: &StateValue<DefaultDB>,
) -> Result<LedgerState<DefaultDB>, Box<dyn Error>> {
    let address = prepared.prototype().address;
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let ledger = funded.ledger.clone();
    let sealed = prove_funded_offer(root, (name, stem), prepared, funded, rng, verifier, |_| {
        Ok(())
    })?;
    let time = funded.time;
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
        .ok_or("v3 offer contract missing after application")?;
    let original = ledger
        .contract
        .get(&address)
        .and_then(|state| state.operations.get(&entry));
    if state.data.get_ref() != expected || state.operations.get(&entry) != original {
        return Err("v3 offer changed the expected state or installed operation".into());
    }
    check_offer_replay(root, stem, &updated, &verified, context)?;
    Ok(updated)
}

pub(super) fn prove_funded_offer(
    root: &Path,
    (name, stem): (&'static str, &str),
    prepared: impl ProofBoundCall,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    verifier: &VerifierKey,
    check_preproof: impl FnOnce(
        &Transaction<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB>,
    ) -> Result<(), Box<dyn Error>>,
) -> Result<TxBound<Signature, DefaultDB>, Box<dyn Error>> {
    let prototype = prepared.prototype();
    let address = prototype.address;
    let entry = EntryPointBuf(name.as_bytes().to_vec());
    let operation = funded
        .ledger
        .contract
        .get(&address)
        .and_then(|state| state.operations.get(&entry))
        .ok_or("v3 offer has no installed operation")?;
    if prototype.entry_point != entry || prototype.op != *operation {
        return Err("v3 offer prototype differs from the installed v3 operation".into());
    }
    let standalone = prove_standalone(root, name, stem, prototype, verifier, None)?;
    verify_separately(root, name, stem)?;

    let time = funded.time;
    let tx = prepared.into_test_transaction(rng, Timestamp::from_secs(time.to_secs() + 3600));
    check_preproof(&tx)?;
    let resolver = qualified_coin_funding::fee_resolver(root, name)?;
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
    let transaction_proof = {
        let mut calls = sealed.calls();
        let (_, call) = calls.next().ok_or("v3 offer transaction has no call")?;
        let ProofVersioned::V3(transaction_proof) = call.proof else {
            return Err("v3 offer transaction proof is not v3".into());
        };
        if calls.next().is_some()
            || transaction_proof == standalone
            || positive_fee_dust_spends(&sealed) == 0
        {
            return Err("v3 offer transaction proof or Dust spend differs".into());
        }
        transaction_proof
    };
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    write_new(
        root.join(format!("{stem}-transaction.bin")),
        &transaction_bytes,
    )?;
    write_new(
        root.join(format!("{stem}-transaction.proof")),
        &transaction_proof.0,
    )?;

    Ok(sealed)
}

pub(super) fn check_offer_replay(
    root: &Path,
    stem: &str,
    updated: &LedgerState<DefaultDB>,
    verified: &VerifiedTransaction<DefaultDB>,
    context: TransactionContext<DefaultDB>,
) -> Result<(), Box<dyn Error>> {
    let (replayed, replay_result) = updated.apply(
        verified,
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
    ) || &replayed != updated
    {
        return Err(format!("{stem} replay differs or mutated ledger: {replay_result:?}").into());
    }
    let mut after = Vec::new();
    let mut replay = Vec::new();
    tagged_serialize(updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    if replay != after {
        return Err("v3 offer replay changed serialized ledger bytes".into());
    }
    write_new(root.join(format!("{stem}-ledger-after.bin")), &after)?;
    write_new(root.join(format!("{stem}-ledger-replay.bin")), &replay)?;
    println!("{stem}: funded default-strict apply; replay refused without mutation");
    Ok(())
}

pub(super) fn apply_funded_deployment(
    root: &Path,
    stem: &str,
    deploy: &ContractDeploy<DefaultDB>,
    funded: &mut midnight_ledger::test_utilities::TestState<DefaultDB>,
    rng: &mut StdRng,
    executor: &tokio::runtime::Runtime,
    resolver: &midnight_ledger::prove::Resolver,
) -> Result<(), Box<dyn Error>> {
    let address = deploy.address();
    let deploy_intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(rng, funded.time + funded.ledger.parameters.global_ttl)
            .add_deploy(deploy.clone());
    let deploy_tx =
        Transaction::from_intents("local-test", HashMap::new().insert(1_u16, deploy_intent));
    let proved = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &deploy_tx,
            resolver,
        ))
        .map_err(|error| format!("v3 deployment proving failed: {error:?}"))?;
    let sealed = proved.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let balanced = executor.block_on(funded.balance_tx(
        StdRng::from_rng(rand::rngs::OsRng)?,
        sealed,
        resolver,
    ))?;
    if balanced
        .deploys()
        .map(|(_, action)| action.address())
        .collect::<Vec<_>>()
        != [address]
        || positive_fee_dust_spends(&balanced) != 1
    {
        return Err("v3 deployment address or proved Dust spend differs".into());
    }
    let result = funded.apply(&balanced, WellFormedStrictness::default())?;
    if !matches!(result, TransactionResult::Success(_))
        || funded.ledger.contract.get(&address) != Some(&deploy.initial_state)
    {
        return Err(
            format!("v3 deployment strict apply or installed state differs: {result:?}").into(),
        );
    }
    let mut bytes = Vec::new();
    tagged_serialize(&balanced, &mut bytes)?;
    write_new(root.join(format!("{stem}-deployment.bin")), &bytes)?;
    let mut ledger_bytes = Vec::new();
    tagged_serialize(&funded.ledger, &mut ledger_bytes)?;
    write_new(
        root.join(format!("{stem}-ledger-after-deploy.bin")),
        &ledger_bytes,
    )?;
    println!("{stem}: funded default-strict deployment applied");
    Ok(())
}

pub(super) fn positive_fee_dust_spends(
    tx: &Transaction<Signature, ProofMarker, PureGeneratorPedersen, DefaultDB>,
) -> usize {
    tx.intents()
        .filter_map(|(_, intent)| intent.dust_actions)
        .map(|actions| {
            actions
                .spends
                .iter()
                .filter(|spend| !spend.proof.0.is_empty() && spend.v_fee > 0)
                .count()
        })
        .sum()
}
