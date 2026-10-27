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

//! Source-bound Tuple-wrapped selected Set mutation through the ordinary prover.
use super::*;
use compact_rust_tuple_return_wrappers_source_fixture::{
    ledger_contract as c, ledger_slots as slots,
};
use midnight_compact_runtime::{self as r, Field};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};

const IR: &str =
    include_str!("../../../tests-rust-backend/tuple-return-wrappers-source/original-ir.json");
const FOLD_INPUT: [u64; 2] = [1, 0];
const PROOF_RESULT_FORMAT: &str = "compact-tuple-return-wrappers-proof/v2";
struct UnusedWitness;
impl c::TryWitnesses<u64> for UnusedWitness {
    fn choose(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, bool), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected choose witness".into(),
        ))
    }
    fn first(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, r::BoundedUint<65535>), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected first witness".into(),
        ))
    }
    fn second(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, r::BoundedUint<65535>), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected second witness".into(),
        ))
    }
}
pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        IR,
    )?;
    if root.join("tuple-wrappers-proof-result.json").exists() {
        return Err("Tuple wrapper proof result already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (name, key_id)) in [
        ("inferred", 5_u64),
        ("inferred", 6),
        ("mixed", 6),
        ("folded", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let key = Field::from(key_id);
        let expected_size = if key_id == 5 { 1 } else { 2 };
        let expected = if name == "mixed" {
            AlignedValue::from((r::BoundedUint::<255>::new(1)?, Field::from(2_u64)))
        } else {
            AlignedValue::from((Field::from(1_u64), Field::from(2_u64)))
        };
        let mut rng = StdRng::seed_from_u64(0x0516_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64))?;
        let initial_values = slots::values.inspect(initial.ledger_state.get_ref())?;
        let before_size = initial_values.size()?.value();
        if before_size != 1 || !initial_values.member(Field::from(5_u64)) {
            return Err("tuple proof initial state differs from constructor precondition".into());
        }
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
        let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
            root.join(format!("keys/{name}.verifier")),
        )?))?;
        let facade = c::Contract::from(UnusedWitness);
        // Keep the generated typed APIs in the test. The macro only shares
        // assertions and preparation; it never interprets an operation.
        macro_rules! routes {
            ($method:ident, $call:ident, [$($arg:expr),*]) => {{
                let native = c::$method(observed.circuit_context(7), $($arg),*)?;
                let recorded = c::recorded::$method(observed.circuit_context(7), $($arg),*)?;
                let functional_native = c::functional::$method::Program::new($($arg),*)
                    .execute_native(observed.circuit_context(7))?;
                let functional = c::functional::$method::Program::new($($arg),*)
                    .execute_recorded(observed.circuit_context(7))?;
                if recorded.public.verify_ops() != functional.public.verify_ops() {
                    return Err("tuple wrapper recording differs".into());
                }
                let state = check_routes(
                    [&native, &recorded.execution, &functional_native, &functional.execution],
                    &expected, key, expected_size,
                )?;
                let prepared = check_generated_trace(root, name, functional, ($($arg,)*))?;
                let ordinary = facade.recording().$call(&observed, 7, $($arg),*)?
                    .prepare(verifier.clone(), Fr::from(0_u64))?;
                (state, prepared, ordinary)
            }};
        }
        let (expected_state, prepared, ordinary) = match name {
            "inferred" => routes!(inferred, inferred_call, [true, key]),
            "mixed" => routes!(mixed, mixed_call, [true, key]),
            "folded" => routes!(
                folded,
                folded_call,
                [r::FixedVector(FOLD_INPUT.map(Field::from))]
            ),
            _ => unreachable!("finite test scenario"),
        };
        if format!("{prepared:?}") != format!("{ordinary:?}") {
            return Err("functional tuple preparation differs from ordinary observed call".into());
        }
        let changed_results = if name == "mixed" {
            [
                AlignedValue::from((r::BoundedUint::<255>::new(3)?, Field::from(2_u64))),
                AlignedValue::from((r::BoundedUint::<255>::new(1)?, Field::from(4_u64))),
            ]
        } else {
            [
                AlignedValue::from((Field::from(3_u64), Field::from(2_u64))),
                AlignedValue::from((Field::from(1_u64), Field::from(4_u64))),
            ]
        };
        let proof_bytes = prove(root, index, name, &prepared, &verifier, changed_results)?;
        apply(root, index, name, prepared, deploy, &expected_state, rng)?;
        rows.push(serde_json::json!({"circuit":name,"key":key_id,"before":before_size,"after":expected_size,"proof_bytes":proof_bytes,
            "fold_input":if name=="folded"{Some(FOLD_INPUT)}else{None},
            "output_type":if name=="mixed"{"uint8_field"}else{"field_field"},
            "changed_binding_rejected":true,"changed_result_rejected":true,"changed_result_components_rejected":[0,1],"changed_result_same_geometry":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":PROOF_RESULT_FORMAT,"status":"passed",
        "scope":"four fresh selected inferred/mixed/Fold tuple call proofs and funded default-strict local transactions; Fold input [1,0]",
        "constructor_execution_proved":false,"inferred_false_branch_proved":false,"mixed_false_branch_proved":false,"calls":rows});
    write_new(
        root.join("tuple-wrappers-proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}

fn check_routes<O: Clone + Into<AlignedValue>>(
    executions: [&r::context::CircuitResult<u64, O>; 4],
    expected: &AlignedValue,
    key: Field,
    expected_size: u128,
) -> Result<StateValue<DefaultDB>, Box<dyn Error>> {
    let native = executions[0];
    for actual in executions {
        if actual.result.clone().into() != *expected
            || actual.context.private_state != 7
            || actual.context.query.state != native.context.query.state
            || actual.context.query.effects != Default::default()
            || actual.gas_cost != native.gas_cost
            || !actual.private_transcript_outputs.is_empty()
        {
            return Err(
                "tuple wrapper native/recorded/functional value/state/effects/private/gas mismatch"
                    .into(),
            );
        }
    }
    let state = native.context.query.state.get_ref();
    let values = slots::values.inspect(state)?;
    if values.size()?.value() != expected_size
        || !values.member(key)
        || !values.member(Field::from(5_u64))
        || slots::count.inspect(state)? != 0
        || slots::stored.inspect(state)? != Field::from(2_u64)
    {
        return Err("tuple wrapper mutation state differs from independent expected state".into());
    }
    Ok(state.clone())
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
    name: &'static str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    changed_results: [AlignedValue; 2],
) -> Result<usize, Box<dyn Error>> {
    let commitment = commitment(call);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected tuple-wrappers proof version".into()),
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
            "tuple-wrappers proof statement geometry differs from upstream binding/communication inputs"
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
        return Err("tuple-wrappers proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    for changed_output in changed_results {
        let mut changed_call = call.clone();
        changed_call.output = changed_output;
        if changed_call.output.alignment != call.output.alignment
            || changed_call.output.value.0.len() != call.output.value.0.len()
        {
            return Err("changed Tuple-wrapper result geometry differs".into());
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
            return Err("tuple-wrappers proof accepted changed result commitment".into());
        }
        let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
            &changed_call,
            changed_commitment,
        ) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unexpected changed tuple-wrappers proof version".into()),
        };
        if changed_preimage.check(&ir).is_ok() {
            return Err("tuple-wrappers IR accepted changed result".into());
        }
    }
    write_new(root.join(format!("tuple-wrappers-{index}.proof")), &proof.0)?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(
        root.join(format!("tuple-wrappers-{index}.preimage")),
        &bytes,
    )?;
    println!(
        "tuple-wrappers {index}: {}-byte proof verified; changed binding/result refused",
        proof.0.len()
    );
    Ok(proof.0.len())
}

fn apply(
    root: &Path,
    index: usize,
    name: &'static str,
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
    let resolver = qualified_coin_funding::fee_resolver(root, name)?;
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
        root.join(format!("tuple-wrappers-{index}-transaction.bin")),
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
        return Err(format!("tuple-wrappers strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("tuple-wrappers contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("tuple-wrappers applied state differs".into());
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
            "tuple-wrappers replay refused incorrectly or mutated ledger: {result:?}"
        )
        .into());
    }
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(
        root.join(format!("tuple-wrappers-{index}-state.bin")),
        &bytes,
    )?;
    println!(
        "tuple-wrappers {index}: funded default-strict apply; replay refused without mutation"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_ir_mismatch_refuses_before_keys_or_state() {
        let mut changed: serde_json::Value = serde_json::from_str(IR).unwrap();
        changed["contractName"] = serde_json::json!("changed-tuple-wrappers-source");
        assert!(counter_scenario::validate_ir(&serde_json::to_vec(&changed).unwrap(), IR).is_err());
    }
}
