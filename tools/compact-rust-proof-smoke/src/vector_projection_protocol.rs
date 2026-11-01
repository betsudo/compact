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

// Source-bound functional Vector selection; each thin module supplies its fixture and full IR.
use super::*;
use fixture::{ledger_contract as c, ledger_slots as slots};
use midnight_compact_runtime::{self as r, Field, FixedVector};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};
use std::{cell::RefCell, rc::Rc};

const NAME: &str = "witnessed";
const PROOF_RESULT_FORMAT: &str = "compact-vector-projection-proof/v1";

#[derive(Clone, Default)]
struct Witness(Rc<RefCell<Vec<(u64, Field)>>>);
impl c::TryWitnesses<u64> for Witness {
    fn values(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, FixedVector<Field, 2>), r::CompactError> {
        self.0
            .borrow_mut()
            .push((*context.private_state, context.ledger.stored()?));
        Ok((
            *context.private_state + 1,
            FixedVector([Field::from(17_u64), Field::from(41_u64)]),
        ))
    }
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        IR,
    )?;
    if root.join("vector-proof-result.json").exists() {
        return Err("vector proof destination already contains a result".into());
    }
    let mut rows = Vec::new();
    for (index, first) in [false, true].into_iter().enumerate() {
        let expected = Field::from(if first { 17_u64 } else { 41_u64 });
        let mut rng = StdRng::seed_from_u64(0x0674_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64))?;
        let deploy = make_deploy(root, NAME, initial.ledger_state.get_ref().clone(), &mut rng)?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let witnesses: [Witness; 5] = std::array::from_fn(|_| Witness::default());
        let native = c::witnessed(observed.circuit_context(7), &witnesses[0], first)?;
        let recorded = c::recorded::witnessed(observed.circuit_context(7), &witnesses[1], first)?;
        let functional_native = c::functional::witnessed::Program::new(first)
            .execute_native(observed.circuit_context(7), &witnesses[2])?;
        let functional = c::functional::witnessed::Program::new(first)
            .execute_recorded(observed.circuit_context(7), &witnesses[3])?;
        for actual in [
            &native,
            &recorded.execution,
            &functional_native,
            &functional.execution,
        ] {
            if actual.result != expected
                || actual.context.private_state != 8
                || actual.context.query.state != native.context.query.state
                || actual.context.query.effects != native.context.query.effects
                || actual.gas_cost != native.gas_cost
                || actual.private_transcript_outputs
                    != [AlignedValue::from(FixedVector([
                        Field::from(17_u64),
                        Field::from(41_u64),
                    ]))]
            {
                return Err(
                    "vector native/recorded/functional value/state/effects/private/gas mismatch"
                        .into(),
                );
            }
        }
        if recorded.public.verify_ops() != functional.public.verify_ops()
            || slots::stored.inspect(native.context.query.state.get_ref())? != expected
        {
            return Err("vector recording or stored Cell differs".into());
        }
        let expected_state = native.context.query.state.get_ref().clone();
        // This functional-derived trace is the actual prover input.
        let prepared = check_generated_trace(root, NAME, functional, first)?;
        let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
            root.join("keys/witnessed.verifier"),
        )?))?;
        let facade = c::Contract::from(witnesses[4].clone());
        let ordinary = facade
            .recording()
            .witnessed_call(&observed, 7, first)?
            .prepare(verifier.clone(), Fr::from(0_u64))?;
        if format!("{prepared:?}") != format!("{ordinary:?}") {
            return Err("functional vector preparation differs from ordinary observed call".into());
        }
        for witness in witnesses {
            if *witness.0.borrow() != [(7, Field::from(0_u64))] {
                return Err("vector witness count/order changed".into());
            }
        }
        let proof_bytes = prove(root, index, &prepared, &verifier)?;
        apply(root, index, prepared, deploy, &expected_state, rng)?;
        rows.push(serde_json::json!({"select_first":first,"expected":if first {17} else {41},
            "proof_bytes":proof_bytes,"changed_binding_rejected":true,"changed_result_rejected":true,
            "changed_input_same_geometry":true,"changed_input_rejected":true,"changed_proof_bytes_rejected":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":PROOF_RESULT_FORMAT,"status":"passed","source":SOURCE_LABEL,
        "scope":"two fresh functional-derived call proofs plus funded default-strict transactions; local fixture",
        "constructor_execution_proved":false,"calls":rows});
    write_new(
        root.join("vector-proof-result.json"),
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
            _ => return Err("unexpected vector proof version".into()),
        };
    let ir: IrSource = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("zkir/witnessed.bzkir"),
    )?))?;
    let pk: ProverKey<IrSource> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/witnessed.prover"),
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
            "vector proof statement geometry differs from upstream binding/communication inputs"
                .into(),
        );
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if proof.0.is_empty() {
        return Err("vector prover returned an empty proof".into());
    }
    let mut changed_proof = proof.clone();
    changed_proof.0[0] ^= 1;
    if verifier
        .verify(&PARAMS_VERIFIER, &changed_proof, inputs.iter().copied())
        .is_ok()
    {
        return Err("vector verifier accepted corrupted proof bytes".into());
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
        return Err("vector proof accepted changed public binding".into());
    }
    let mut changed_input_call = call.clone();
    changed_input_call.input = AlignedValue::from((index == 0,));
    if changed_input_call.input == call.input
        || changed_input_call.input.alignment != call.input.alignment
        || changed_input_call.input.value.0.len() != call.input.value.0.len()
    {
        return Err("changed Boolean input did not preserve source geometry".into());
    }
    let changed_input_commitment = self::commitment(&changed_input_call);
    if changed_input_commitment == commitment {
        return Err("changed input did not change commitment".into());
    }
    let mut changed_input = inputs.clone();
    changed_input[1] = changed_input_commitment;
    if verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_input.into_iter())
        .is_ok()
    {
        return Err("vector proof accepted changed input commitment".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(Field::from(999_u64));
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
        return Err("vector proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed vector proof version".into()),
    };
    if changed_preimage.check(&ir).is_ok() {
        return Err("vector IR accepted changed result".into());
    }
    write_new(root.join(format!("vector-{index}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(root.join(format!("vector-{index}.preimage")), &bytes)?;
    println!(
        "vector {index}: {}-byte proof verified; changed binding/result refused",
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
        rng: StdRng::seed_from_u64(0x0674_5052 + index as u64),
        resolver: &resolver,
        params: &params,
    };
    let proven = futures_executor::block_on(
        tx.prove(provider, &INITIAL_PARAMETERS.cost_model.runtime_cost_model),
    )?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x0674_5345 + index as u64));
    let sealed = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(funded.balance_tx(rng, sealed, &resolver))?;
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
        return Err(format!("vector strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("vector contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("vector applied state differs".into());
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
            format!("vector replay refused incorrectly or mutated ledger: {result:?}").into(),
        );
    }
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(root.join(format!("vector-{index}-state.bin")), &bytes)?;
    println!("vector {index}: funded default-strict apply; replay refused without mutation");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_ir_mismatch_refuses_before_keys_or_state() {
        let mut changed: serde_json::Value = serde_json::from_str(IR).unwrap();
        changed["contractName"] = serde_json::json!("changed-vector-binding");
        assert!(counter_scenario::validate_ir(&serde_json::to_vec(&changed).unwrap(), IR).is_err());
    }
}
