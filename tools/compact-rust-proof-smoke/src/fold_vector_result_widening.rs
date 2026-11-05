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

//! Complete Fold helper SDK, two selected proofs and separate strict transactions.
use super::*;
use crate::fold_vector_result_fixture::{ledger_contract as c, ledger_slots as slots};
use midnight_compact_runtime::{BoundedUint, Field, FixedVector};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};
const IR: &[u8] = include_bytes!(
    "../../compact-rust-backend/tests/fold_vector_result_widening/goldens/helper/source-ir.json"
);
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../compact-rust-backend/tests/fold_vector_result_widening/goldens/helper/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../compact-rust-backend/tests/fold_vector_result_widening/goldens/helper/checked-programs/program-1.json"
        ),
    ),
];

fn vector<const N: usize>(values: [u128; N]) -> FixedVector<BoundedUint<255>, N> {
    FixedVector::new(values.map(|x| BoundedUint::new(x).unwrap()))
}
fn result_vector(values: [u128; 2]) -> FixedVector<BoundedUint<65535>, 2> {
    FixedVector::new(values.map(|x| BoundedUint::new(x).unwrap()))
}
fn validate_bindings(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join("contract/compact-rust-ir.json"))? != IR {
        return Err("fold-vector-result complete IR bytes differ".into());
    }
    let directory = root.join("contract/checked-programs");
    let mut names = std::fs::read_dir(&directory)?
        .map(|row| row.map(|row| row.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut expected = PROGRAMS
        .iter()
        .map(|(name, _)| std::ffi::OsString::from(name))
        .collect::<Vec<_>>();
    expected.sort();
    if names != expected {
        return Err("fold-vector-result complete Program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(directory.join(name))? != *bytes {
            return Err("fold-vector-result complete Program bytes differ".into());
        }
    }
    Ok(())
}

/// Validate exact bindings and inspect selected tiny models before key generation.
pub(super) fn models(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    let mut rows = Vec::new();
    for name in ["run", "run_checked"] {
        let ir = IrSource::load(BufReader::new(File::open(
            root.join(format!("zkir/{name}.zkir")),
        )?))?;
        let k = ir.model().k();
        rows.push(serde_json::json!({"circuit":name,"k":k}));
    }
    println!(
        "{}",
        serde_json::json!({"format":"compact-fold-vector-result-models/v1","circuits":rows,
            "selected_for_proof":["run","run_checked"],"unproved":[]})
    );
    if rows
        .iter()
        .any(|row| row["k"].as_u64().is_none_or(|k| k > 8))
    {
        return Err(
            "fold-vector-result model exceeds tiny automatic k<=8; full roster retained above"
                .into(),
        );
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_bindings(root)?;
    if root.join("fold-vector-result-proof-result.json").exists() {
        return Err("Fold Vector result proof already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (name, input_values, seed_values, values, seen)) in [
        ("run", [2_u128, 3], [9_u128, 8], [3_u128, 2], 1_u64),
        ("run_checked", [1, 2], [0, 0], [3, 1], 2),
    ]
    .into_iter()
    .enumerate()
    {
        let expected = result_vector(values);
        let mut rng = StdRng::seed_from_u64(0x0164_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64))?;
        let deploy = make_deploy(root, name, initial.ledger_state.get_ref().clone(), &mut rng)?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        macro_rules! routes {($entry:ident $(,$arg:expr)*)=>{{
            (c::$entry(observed.circuit_context(7) $(,$arg)*)?,
             c::recorded::$entry(observed.circuit_context(7) $(,$arg)*)?,
             c::functional::$entry::Program::new($($arg),*).execute_native(observed.circuit_context(7))?,
             c::functional::$entry::Program::new($($arg),*).execute_recorded(observed.circuit_context(7))?)
        }};}
        let (native, recorded, functional_native, functional) = match name {
            "run" => routes!(run, vector(input_values), vector(seed_values)),
            "run_checked" => routes!(run_checked, vector(input_values), vector(seed_values)),
            _ => return Err("unknown selected helper".into()),
        };
        for actual in [
            &native,
            &recorded.execution,
            &functional_native,
            &functional.execution,
        ] {
            if actual.result != expected
                || actual.context.private_state != 7
                || actual.context.query.state != native.context.query.state
                || actual.context.query.effects != native.context.query.effects
                || actual.gas_cost != native.gas_cost
                || !actual.private_transcript_outputs.is_empty()
            {
                return Err(
                    "Fold Vector result four-route output/state/effects/private/gas mismatch"
                        .into(),
                );
            }
        }
        if recorded.public.verify_ops() != functional.public.verify_ops()
            || slots::seen.inspect(native.context.query.state.get_ref())? != Field::from(seen)
        {
            return Err("Fold Vector result recording or seen Cell differs".into());
        }
        let state = native.context.query.state.get_ref().clone();
        let input = AlignedValue::from((vector(input_values), vector(seed_values)));
        let prepared = check_generated_trace(root, name, functional, input)?;
        let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
            root.join(format!("keys/{name}.verifier")),
        )?))?;
        let facade = c::Contract::default();
        let ordinary = match name {
            "run" => facade.recording().run_call(
                &observed,
                7,
                vector(input_values),
                vector(seed_values),
            )?,
            "run_checked" => facade.recording().run_checked_call(
                &observed,
                7,
                vector(input_values),
                vector(seed_values),
            )?,
            _ => return Err("unknown selected observed helper".into()),
        }
        .prepare(verifier.clone(), Fr::from(0_u64))?;
        if format!("{prepared:?}") != format!("{ordinary:?}") {
            return Err(
                "Fold Vector result observed call differs from functional preparation".into(),
            );
        }
        let proof_bytes = prove(root, name, index, &prepared, &verifier)?;
        let transaction_bytes = apply(root, name, index, prepared, deploy, &state, rng)?;
        rows.push(serde_json::json!({"circuit":name,"selected":null,"input":input_values,"seed":seed_values,"expected":values,"seen":seen,
            "witness_count":0,"proof_bytes":proof_bytes,"transaction_bytes":transaction_bytes,
            "standalone_proof":format!("fold-vector-result-{index}.proof"),"applied_transaction":format!("fold-vector-result-{index}-transaction.bin"),
            "transaction_proof_relationship":"independently proved","corrupt_proof_rejected":true,"changed_binding_rejected":true,
            "changed_result_rejected":true,"changed_result_same_geometry":true,"wrong_verifier_rejected":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":"compact-fold-vector-result-proof/v1","status":"passed",
        "scope":"two selected Fold helper exports, fresh proofs and independently proved funded default-strict transactions",
        "constructor_execution_proved":false,"all_exports_proved":false,"helper_exports_proved":["run","run_checked"],"unproved_helper_exports":[],"calls":rows});
    write_new(
        root.join("fold-vector-result-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
fn write_new(path: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)
}

fn commitment(call: &ContractCallPrototype<DefaultDB>) -> Fr {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    transient_commit(&fields[..], call.communication_commitment_rand)
}

fn prove(
    root: &Path,
    name: &str,
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<usize, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected fold-vector-result proof version".into()),
        };
    let ir: IrSource = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("zkir/{name}.bzkir")),
    )?))?;
    let pk: ProverKey<IrSource> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
    )?))?;
    let checked = preimage.check(&ir)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, skips) = futures_executor::block_on(ir.prove(
        ChaCha20Rng::from_seed([0x74 + index as u8; 32]),
        &params,
        pk,
        &preimage,
    ))?;
    if skips != checked
        || inputs.first() != Some(&preimage.binding_input)
        || inputs.get(1) != Some(&commitment)
    {
        return Err(
            "fold-vector-result proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("fold-vector-result proof is empty".into());
    }
    let mut corrupt = proof.clone();
    corrupt.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &corrupt, inputs.iter().copied())
        .is_ok()
    {
        return Err("fold-vector-result accepted corrupted proof bytes".into());
    }
    // A genuinely different source relation, not merely a second key for the same erased circuit.
    let other = if name == "run" { "run_checked" } else { "run" };
    let wrong_path = root.join(format!("keys/{other}.verifier"));
    if std::fs::read(&wrong_path)? == std::fs::read(root.join(format!("keys/{name}.verifier")))? {
        return Err("wrong-verifier control has identical key bytes".into());
    }
    let wrong: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(wrong_path)?))?;
    if wrong
        .verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("fold-vector-result accepted wrong immutable verifier".into());
    }
    let mut changed_binding = inputs.clone();
    changed_binding[0] = if inputs[0] == Fr::from(0_u64) {
        Fr::from(1_u64)
    } else {
        Fr::from(0_u64)
    };
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_binding.into_iter())
        .is_ok()
    {
        return Err("fold-vector-result proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(result_vector([999, 998]));
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed fold-vector-result result geometry differs".into());
    }
    let changed_commitment = self::commitment(&changed_call);
    if changed_commitment == commitment {
        return Err("changed result did not change commitment".into());
    }
    let mut changed_result = inputs.clone();
    changed_result[1] = changed_commitment;
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_result.into_iter())
        .is_ok()
    {
        return Err("fold-vector-result proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed fold-vector-result proof version".into()),
    };
    if changed_preimage.check(&ir).is_ok() {
        return Err("fold-vector-result IR accepted changed result".into());
    }
    write_new(
        root.join(format!("fold-vector-result-{index}.proof")),
        &proof.0,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(
        root.join(format!("fold-vector-result-{index}.preimage")),
        &bytes,
    )?;
    println!(
        "fold-vector-result {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof.0.len())
}

fn apply(
    root: &Path,
    name: &'static str,
    index: usize,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    expected: &StateValue<DefaultDB>,
    mut rng: StdRng,
) -> Result<usize, Box<dyn Error>> {
    let address = deploy.address();
    let mut funded = qualified_coin_funding::fee_funded_state(&mut rng)?;
    funded.ledger.contract = funded.ledger.contract.insert(address, deploy.initial_state);
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, Timestamp::from_secs(funded.time.to_secs() + 3600))
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let resolver = qualified_coin_funding::fee_resolver(root, name)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let provider = LocalProvingProvider {
        rng: StdRng::seed_from_u64(0x0164_5052 + index as u64),
        resolver: &resolver,
        params: &params,
    };
    let proven = futures_executor::block_on(
        tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
    )?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0164_5345 + index as u64));
    let sealed = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(funded.balance_tx(rng, sealed, &resolver))?;
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
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
        return Err(format!("fold-vector-result strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("fold-vector-result contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("fold-vector-result applied state differs".into());
    }
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
        return Err(format!(
            "fold-vector-result replay refused incorrectly or mutated ledger: {result:?}"
        )
        .into());
    }
    write_new(
        root.join(format!("fold-vector-result-{index}-transaction.bin")),
        &transaction_bytes,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(
        root.join(format!("fold-vector-result-{index}-state.bin")),
        &bytes,
    )?;
    println!(
        "fold-vector-result {index}: funded default-strict apply; replay refused without mutation"
    );
    Ok(transaction_bytes.len())
}
