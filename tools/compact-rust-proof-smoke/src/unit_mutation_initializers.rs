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

//! Source-bound Unit-initializer selected Set mutation through the ordinary prover.
use super::*;
use compact_rust_unit_mutation_initializers_source_fixture::{
    ledger_contract as c, ledger_slots as slots,
};
use midnight_compact_runtime::{self as r, Field};
use midnight_transient_crypto::proofs::{ProverKey, Zkir};

const IR: &str =
    include_str!("../../../tests-rust-backend/unit-mutation-initializers-source/original-ir.json");
const PROOF_RESULT_FORMAT: &str = "compact-unit-mutation-initializers-proof/v2";
struct UnusedWitness;
impl c::TryWitnesses<u64> for UnusedWitness {
    fn point(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
        _: Field,
        _: Field,
    ) -> Result<(u64, Field), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected point witness".into(),
        ))
    }
    fn gate(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
        _: Field,
        _: bool,
    ) -> Result<(u64, bool), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected gate witness".into(),
        ))
    }
    fn unitGuard(
        &self,
        _: WitnessContext<'_, u64, c::LedgerView<'_>>,
        _: Field,
        _: bool,
    ) -> Result<(u64, ()), r::CompactError> {
        Err(r::CompactError::AssertionFailed(
            "unexpected Unit witness".into(),
        ))
    }
}
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-1.json"
        ),
    ),
    (
        "program-2.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-2.json"
        ),
    ),
    (
        "program-3.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-3.json"
        ),
    ),
    (
        "program-4.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-4.json"
        ),
    ),
    (
        "program-5.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-5.json"
        ),
    ),
    (
        "program-6.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-6.json"
        ),
    ),
    (
        "program-7.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-7.json"
        ),
    ),
    (
        "program-8.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-8.json"
        ),
    ),
    (
        "program-9.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-9.json"
        ),
    ),
    (
        "program-10.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-10.json"
        ),
    ),
    (
        "program-11.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-11.json"
        ),
    ),
    (
        "program-12.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-12.json"
        ),
    ),
    (
        "program-13.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-13.json"
        ),
    ),
    (
        "program-14.json",
        include_bytes!(
            "../../../tests-rust-backend/unit-mutation-initializers-source/checked-programs/program-14.json"
        ),
    ),
];
fn validate_programs(root: &Path) -> Result<(), Box<dyn Error>> {
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
        return Err("Unit initializer complete checked-program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(directory.join(name))? != *bytes {
            return Err("Unit initializer checked-program bytes differ".into());
        }
    }
    Ok(())
}
pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        IR,
    )?;
    validate_programs(root)?;
    if root.join("unit-initializers-proof-result.json").exists() {
        return Err("Unit initializer proof result already exists".into());
    }
    let mut rows = Vec::new();
    for (index, (name, key_id)) in [
        ("conditional", 5_u64),
        ("conditional", 6),
        ("statement_field", 5),
        ("statement_field", 6),
    ]
    .into_iter()
    .enumerate()
    {
        let key = Field::from(key_id);
        let expected = Field::from(1_u64);
        let expected_size = if key_id == 5 { 1 } else { 2 };
        let mut rng = StdRng::seed_from_u64(0x0610_0000 + index as u64);
        let initial = c::initial_state(ConstructorContext::new(7_u64))?;
        // Establish the same real prestate for insertion and reinsertion.
        // This setup execution and the constructor are not proved.
        let seeded = c::conditional(
            initial.into_circuit_context(Default::default()),
            true,
            Field::from(5_u64),
        )?;
        let deploy = make_deploy(
            root,
            name,
            seeded.context.query.state.get_ref().clone(),
            &mut rng,
        )?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        macro_rules! execute_routes {
            ($entry:ident) => {{
                (
                    c::$entry(observed.circuit_context(7), true, key)?,
                    c::recorded::$entry(observed.circuit_context(7), true, key)?,
                    c::functional::$entry::Program::new(true, key)
                        .execute_native(observed.circuit_context(7))?,
                    c::functional::$entry::Program::new(true, key)
                        .execute_recorded(observed.circuit_context(7))?,
                )
            }};
        }
        let (native, recorded, functional_native, functional) = match name {
            "conditional" => execute_routes!(conditional),
            "statement_field" => execute_routes!(statement_field),
            _ => return Err("unqualified Unit initializer circuit".into()),
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
                    "Unit initializer native/recorded/functional value/state/effects/private/gas mismatch"
                        .into(),
                );
            }
        }
        if recorded.public.verify_ops() != functional.public.verify_ops()
            || slots::values
                .inspect(native.context.query.state.get_ref())?
                .size()?
                .value()
                != expected_size
            || !slots::values
                .inspect(native.context.query.state.get_ref())?
                .member(key)
        {
            return Err("Unit initializer mutation state or recording differs".into());
        }
        let expected_state = native.context.query.state.get_ref().clone();
        let prepared = check_generated_trace(root, name, functional, (true, key))?;
        let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
            root.join(format!("keys/{name}.verifier")),
        )?))?;
        let facade = c::Contract::from(UnusedWitness);
        let ordinary = match name {
            "conditional" => facade
                .recording()
                .conditional_call(&observed, 7, true, key)?,
            "statement_field" => facade
                .recording()
                .statement_field_call(&observed, 7, true, key)?,
            _ => return Err("unqualified Unit initializer observed call".into()),
        }
        .prepare(verifier.clone(), Fr::from(0_u64))?;
        if format!("{prepared:?}") != format!("{ordinary:?}") {
            return Err(
                "functional Unit-initializer preparation differs from ordinary observed call"
                    .into(),
            );
        }
        let proof_bytes = prove(root, name, index, &prepared, &verifier)?;
        let transaction_bytes = apply(root, name, index, prepared, deploy, &expected_state, rng)?;
        rows.push(serde_json::json!({"circuit":name,"key":key_id,"before":1,"after":expected_size,"proof_bytes":proof_bytes,"transaction_bytes":transaction_bytes,
            "standalone_proof":format!("unit-initializers-{index}.proof"),
            "applied_transaction":format!("unit-initializers-{index}-transaction.bin"),
            "transaction_proof_relationship":"independently proved",
            "changed_binding_rejected":true,"changed_result_rejected":true,"changed_result_same_geometry":true,
            "strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true}));
    }
    let result = serde_json::json!({"format":PROOF_RESULT_FORMAT,"status":"passed",
        "scope":"four fresh selected return/statement Unit-initializer Set insert functional-derived proofs and independently proved funded default-strict local transactions",
        "constructor_execution_proved":false,"unselected_branch_proved":false,"prestate_setup_proved":false,
        "all_exports_proved":false,"calls":rows});
    write_new(
        root.join("unit-initializers-proof-result.json"),
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
            _ => return Err("unexpected unit-initializers proof version".into()),
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
            "unit-initializers proof statement geometry differs from upstream binding/communication inputs"
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
        return Err("unit-initializers proof accepted changed public binding".into());
    }
    // Bind a genuinely different public result through the exact upstream
    // communication commitment, rather than changing an arbitrary input slot.
    let mut changed_call = call.clone();
    changed_call.output = AlignedValue::from(Field::from(2_u64));
    if changed_call.output.alignment != call.output.alignment
        || changed_call.output.value.0.len() != call.output.value.0.len()
    {
        return Err("changed Unit-initializer result geometry differs".into());
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
        return Err("unit-initializers proof accepted changed result commitment".into());
    }
    let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
        &changed_call,
        changed_commitment,
    ) {
        ProofPreimageVersioned::V2(value) => value,
        _ => return Err("unexpected changed unit-initializers proof version".into()),
    };
    if changed_preimage.check(&ir).is_ok() {
        return Err("unit-initializers IR accepted changed result".into());
    }
    write_new(
        root.join(format!("unit-initializers-{index}.proof")),
        &proof.0,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(preimage.as_ref(), &mut bytes)?;
    write_new(
        root.join(format!("unit-initializers-{index}.preimage")),
        &bytes,
    )?;
    println!(
        "unit-initializers {index}: {}-byte proof verified; changed binding/result refused",
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
        return Err(format!("unit-initializers strict apply failed: {result:?}").into());
    }
    let actual = updated
        .contract
        .get(&address)
        .ok_or("unit-initializers contract missing after apply")?;
    if actual.data.get_ref() != expected {
        return Err("unit-initializers applied state differs".into());
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
            "unit-initializers replay refused incorrectly or mutated ledger: {result:?}"
        )
        .into());
    }
    write_new(
        root.join(format!("unit-initializers-{index}-transaction.bin")),
        &transaction_bytes,
    )?;
    let mut bytes = Vec::new();
    tagged_serialize(actual, &mut bytes)?;
    write_new(
        root.join(format!("unit-initializers-{index}-state.bin")),
        &bytes,
    )?;
    println!(
        "unit-initializers {index}: funded default-strict apply; replay refused without mutation"
    );
    Ok(transaction_bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_ir_mismatch_refuses_before_keys_or_state() {
        let mut changed: serde_json::Value = serde_json::from_str(IR).unwrap();
        changed["contractName"] = serde_json::json!("changed-unit-initializers-source");
        assert!(counter_scenario::validate_ir(&serde_json::to_vec(&changed).unwrap(), IR).is_err());
    }
}
