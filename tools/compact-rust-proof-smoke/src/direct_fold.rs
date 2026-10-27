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

//! Fresh source-bound Field Fold proofs and funded default-strict applications.
use super::*;
use midnight_compact_runtime::{self as r, Field, FixedVector};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};
mod cases;
mod empty_ordered;
const NAME: &str = "run";

pub(super) fn run(case: &str, root: &Path) -> Result<(), Box<dyn Error>> {
    let ir = match case {
        "single" => {
            include_str!("../../../tests-rust-backend/direct-fold-single-source/original-ir.json")
        }
        "empty-ordered" => include_str!(
            "../../../tests-rust-backend/direct-fold-empty-ordered-source/original-ir.json"
        ),
        "captured" => {
            include_str!("../../../tests-rust-backend/direct-fold-captured-source/original-ir.json")
        }
        "counter-capture" => include_str!(
            "../../../tests-rust-backend/direct-fold-counter-capture-source/original-ir.json"
        ),
        "counter-local-capture" => include_str!(
            "../../../tests-rust-backend/direct-fold-counter-local-capture-source/original-ir.json"
        ),
        "multiple-inputs" => include_str!(
            "../../../tests-rust-backend/direct-fold-multiple-inputs-source/original-ir.json"
        ),
        "effectful-reducer" => include_str!(
            "../../../tests-rust-backend/direct-fold-effectful-reducer-source/original-ir.json"
        ),
        "witness-reducer" => include_str!(
            "../../../tests-rust-backend/direct-fold-witness-reducer-source/original-ir.json"
        ),
        "inline-effectful" => include_str!(
            "../../../tests-rust-backend/direct-fold-inline-effectful-source/original-ir.json"
        ),
        "inline-branch" => include_str!(
            "../../../tests-rust-backend/direct-fold-inline-branch-source/original-ir.json"
        ),
        _ => return Err("unknown direct Fold proof case".into()),
    };
    // Pair the complete original IR before any key/state/proof action.
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        ir,
    )?;
    if root.join("fold-proof-result.json").exists() {
        return Err("Fold proof result already exists".into());
    }
    let calls = match case {
        "single" => vec![cases::single(root, 0)?],
        "empty-ordered" => vec![empty_ordered::run(root, 0)?],
        "captured" => vec![cases::captured(root, 0)?],
        "counter-capture" => vec![cases::counter_capture(root, 0)?],
        "counter-local-capture" => vec![cases::counter_local_capture(root, 0)?],
        "multiple-inputs" => vec![cases::multiple(root, 0)?],
        "effectful-reducer" => vec![cases::stateful(root, 0)?],
        "witness-reducer" => vec![cases::witnessed(root, 0)?],
        "inline-effectful" => vec![cases::inline(root, 0)?],
        "inline-branch" => vec![
            cases::branch(root, 0, true)?,
            cases::branch(root, 1, false)?,
        ],
        _ => unreachable!("case validated before key/state actions"),
    };
    let result = serde_json::json!({"format":"compact-direct-fold-proof/v1", "status":"passed", "case":case,
        "scope":"selected Field-returning source-bound functional-derived call proofs and funded default-strict local transactions",
        "constructor_execution_proved":false, "unselected_sources_proved":false, "calls":calls});
    write_new(
        root.join("fold-proof-result.json"),
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
    index: usize,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<usize, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected fold proof version".into()),
        };
    let ir: IrSource = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("zkir/run.bzkir"),
    )?))?;
    let pk: ProverKey<IrSource> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/run.prover"),
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
            "fold proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
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
        return Err("fold proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(Field::from(42_u64));
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
        || changed_call
            .output
            .value
            .0
            .iter()
            .zip(&call.output.value.0)
            .any(|(a, b)| a.0.len() != b.0.len())
    {
        return Err("changed Fold result geometry differs".into());
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
        return Err("fold proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed fold proof version".into()),
    };
    if changed_preimage.check(&ir).is_ok() {
        return Err("fold IR accepted changed result".into());
    }
    write_new(root.join(format!("fold-{index}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(root.join(format!("fold-{index}.preimage")), &bytes)?;
    println!(
        "fold {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof.0.len())
}

fn apply(
    root: &Path,
    index: usize,
    prepared: ContractCallPrototype<DefaultDB>,
    deploy: ContractDeploy<DefaultDB>,
    expected: &StateValue<DefaultDB>,
    mut rng: StdRng,
) -> Result<(), Box<dyn Error>> {
    let address = deploy.address();
    let mut funded = qualified_coin_funding::fee_funded_state(&mut rng)?;
    funded.ledger.contract = funded.ledger.contract.insert(address, deploy.initial_state);
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, Timestamp::from_secs(funded.time.to_secs() + 3600))
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let resolver = qualified_coin_funding::fee_resolver(root, NAME)?;
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let provider = LocalProvingProvider {
        rng: StdRng::seed_from_u64(0x0610_5052 + index as u64),
        resolver: &resolver,
        params: &params,
    };
    let proven = futures_executor::block_on(
        tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
    )?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0610_5345 + index as u64));
    let sealed = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(funded.balance_tx(rng, sealed, &resolver))?;
    let mut transaction_bytes = Vec::new();
    tagged_serialize(&sealed, &mut transaction_bytes)?;
    write_new(
        root.join(format!("fold-{index}-transaction.bin")),
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
        return Err(format!("fold strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("fold contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("fold applied state differs".into());
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
        return Err(
            format!("fold replay refused incorrectly or mutated ledger: {result:?}").into(),
        );
    }
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(root.join(format!("fold-{index}-state.bin")), &bytes)?;
    println!("fold {index}: funded default-strict apply; replay refused without mutation");
    Ok(())
}
