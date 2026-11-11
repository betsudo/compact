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

//! Exact witnessed SDK proof and funded default-strict application.
//! Constructor-state insertion and fee funding are privileged fixture setup.
use contract::{ledger_contract as api, runtime as rt};
use midnight_base_crypto::data_provider::{FetchMode, MidnightDataProvider, OutputMode};
use midnight_compact_testkit::ProofLab;
use midnight_ledger::{
    construct::{ContractCallExt, ContractCallPrototype},
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
use midnight_serialize::{Deserializable, Tagged, tagged_deserialize, tagged_serialize};
use midnight_storage::arena::Sp;
use midnight_storage::storage::HashMap;
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    curve::Fr,
    fab::AlignedValueExt,
    hash::transient_commit,
    proofs::{
        KeyLocation, PARAMS_VERIFIER, Proof, ProofPreimage, ProverKey, ProvingKeyMaterial,
        Resolver, VerifierKey, Zkir,
    },
};
use midnight_zkir::IrSource;
use midnight_zswap::prove::ZswapResolver;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rt::{
    CompactError, Field,
    context::{ConstructorContext, WitnessContext},
    fab::AlignedValue,
    ledger::DefaultDB,
    transaction::{
        CallSpec, EncodedSizeLimit, Observation, ObservedContractState, PrepareCallError,
        RecordedCall, prepare_call,
    },
};
use std::{
    cell::RefCell,
    error::Error,
    fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
};

const NAME: &str = "apply_offset";
#[derive(Default)]
struct Answers(RefCell<Vec<(u64, Field, Field)>>);
impl api::TryWitnesses<u64> for Answers {
    fn private_offset(
        &self,
        context: WitnessContext<'_, u64, api::LedgerView<'_>>,
        value: Field,
    ) -> Result<(u64, Field), CompactError> {
        self.0
            .borrow_mut()
            .push((*context.private_state, value, context.ledger.cell()?));
        Ok((*context.private_state + 1, value))
    }
}
struct Artifacts(PathBuf);
impl Resolver for Artifacts {
    async fn resolve_key(&self, location: KeyLocation) -> io::Result<Option<ProvingKeyMaterial>> {
        if location.0.as_ref() != NAME {
            return Ok(None);
        }
        Ok(Some(ProvingKeyMaterial {
            prover_key: fs::read(self.0.join("keys/apply_offset.prover"))?,
            verifier_key: fs::read(self.0.join("keys/apply_offset.verifier"))?,
            ir_source: fs::read(self.0.join("zkir/apply_offset.bzkir"))?,
        }))
    }
}
fn fee_resolver(root: &Path) -> Result<LedgerResolver, Box<dyn Error>> {
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
    let root = root.to_owned();
    Ok(LedgerResolver::new(
        zswap,
        dust,
        Box::new(move |location| {
            let a = Artifacts(root.clone());
            Box::pin(async move { a.resolve_key(location).await })
        }),
    ))
}
fn call_preimage(call: &ContractCallPrototype<DefaultDB>) -> Result<ProofPreimage, Box<dyn Error>> {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    let commitment = transient_commit(&fields[..], call.communication_commitment_rand);
    match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
        ProofPreimageVersioned::V2(value) => Ok(value.as_ref().clone()),
        _ => Err("unsupported proof version".into()),
    }
}
fn read_tagged<T: Deserializable + Tagged>(path: &Path) -> Result<T, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let mut reader = Cursor::new(&bytes);
    let value = tagged_deserialize(&mut reader)?;
    if reader.position() != bytes.len() as u64 {
        return Err(format!("trailing bytes in {}", path.display()).into());
    }
    Ok(value)
}

fn verify_standalone(root: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let verifier: VerifierKey = read_tagged(&root.join("keys/apply_offset.verifier"))?;
    let proof = Proof(fs::read(out.join("standalone.proof"))?);
    let inputs: Vec<Fr> = read_tagged(&out.join("standalone.inputs"))?;
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    Ok(())
}

fn assert_same_call(
    actual: &ContractCallPrototype<DefaultDB>,
    expected: &ContractCallPrototype<DefaultDB>,
) {
    // An exhaustive field pattern makes a new upstream field require review.
    let ContractCallPrototype {
        address,
        entry_point,
        op,
        guaranteed_public_transcript,
        fallible_public_transcript,
        private_transcript_outputs,
        input,
        output,
        communication_commitment_rand,
        key_location,
    } = actual;
    assert_eq!(address, &expected.address);
    assert_eq!(entry_point, &expected.entry_point);
    assert_eq!(op, &expected.op);
    assert_eq!(
        guaranteed_public_transcript,
        &expected.guaranteed_public_transcript
    );
    assert_eq!(
        fallible_public_transcript,
        &expected.fallible_public_transcript
    );
    assert_eq!(
        private_transcript_outputs,
        &expected.private_transcript_outputs
    );
    assert_eq!(input, &expected.input);
    assert_eq!(output, &expected.output);
    assert_eq!(
        communication_commitment_rand,
        &expected.communication_commitment_rand
    );
    assert_eq!(key_location, &expected.key_location);
}

fn prove_and_apply(root: PathBuf, out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir(out)?;
    let verifier_path = root.join("keys/apply_offset.verifier");
    let verifier_bytes = fs::read(&verifier_path)?;
    let verifier: VerifierKey = read_tagged(&verifier_path)?;
    let prover: ProverKey<IrSource> = read_tagged(&root.join("keys/apply_offset.prover"))?;
    let ir = IrSource::load(fs::File::open(root.join("zkir/apply_offset.zkir"))?)?;
    let binary_ir = fs::read(root.join("zkir/apply_offset.bzkir"))?;
    let mut expected_ir = Vec::new();
    tagged_serialize(&ir, &mut expected_ir)?;
    assert_eq!(binary_ir, expected_ir);
    assert_eq!(ir.k(), 7);
    assert!(ir.do_communications_commitment);
    let mut rng = StdRng::seed_from_u64(0x6680_0042);
    let initial = api::initial_state(ConstructorContext::new(7_u64))?;
    let entry = EntryPointBuf(NAME.as_bytes().to_vec());
    let operation = ContractOperation::new(
        Some(verifier.clone()),
        Some(Sp::new(IrBuf::from(binary_ir))),
    );
    let operations = HashMap::new().insert(entry.clone(), operation.clone());
    let initial_contract = ContractState::new(
        initial.ledger_state.get_ref().clone(),
        operations,
        ContractMaintenanceAuthority::default(),
    );
    let deploy = ContractDeploy::new(&mut rng, initial_contract);
    let address = deploy.address();
    // Upstream privileged Night reward/Dust registration fixture. These effects
    // are declared setup, not a proved constructor/deployment or wallet lifecycle.
    std::env::var("MIDNIGHT_LEDGER_TEST_STATIC_DIR")?;
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
    let answers = Answers::default();
    let value = Field::from(42_u64);
    // Fresh private randomness for this one call. Both routes below prepare the
    // same statement; a different call needs a fresh value. The seeded RNGs used
    // elsewhere in this executable are deterministic test-fixture setup only.
    // Non-submitted refusal/mutation controls deliberately retain this value to
    // isolate their changed input/output identities; never submit those controls.
    let communication_randomness: Fr = rand::rngs::OsRng.r#gen();
    let mut preparation_refusals = Vec::new();
    for change_state in [true, false] {
        let mut malformed = api::functional::apply_offset::Program::new(value)
            .execute_recorded(observed.circuit_context(7_u64), &Answers::default())?;
        if change_state {
            // Replace only the execution's final state, leaving its genuine
            // generated public replay program and effects unchanged.
            malformed.execution.context.query.state = initial.ledger_state.clone();
        } else {
            malformed.execution.private_transcript_outputs =
                vec![rt::exact_private_output(rt::OpaqueBytes(vec![9, 0]))];
        }
        let error = match prepare_call(
            malformed,
            CallSpec::new(NAME, verifier.clone(), value, communication_randomness),
        ) {
            Err(error) => error,
            Ok(_) => return Err("malformed recording prepared before proof".into()),
        };
        let refusal = match (change_state, error) {
            (true, PrepareCallError::ReplayStateMismatch) => "ReplayStateMismatch",
            (
                false,
                PrepareCallError::NonCanonicalPrivateOutput {
                    output_index: 0,
                    atom_index: 0,
                },
            ) => "NonCanonicalPrivateOutput(0,0)",
            (_, error) => return Err(format!("unrelated preparation refusal: {error}").into()),
        };
        preparation_refusals.push(refusal);
    }
    // prepare_call takes only an owned recording and CallSpec. These controls
    // invoke no ledger application; they do not test rollback or shared stores.
    let recorded = api::functional::apply_offset::Program::new(value)
        .execute_recorded(observed.circuit_context(7_u64), &answers)?;
    assert_eq!(recorded.execution.result, value);
    assert_eq!(recorded.execution.context.private_state, 8);
    assert_eq!(
        recorded.execution.private_transcript_outputs,
        [AlignedValue::from(value)]
    );
    assert_eq!(*answers.0.borrow(), [(7, value, Field::from(0_u64))]);
    let expected = recorded.execution.context.query.state.get_ref().clone();
    let witness_private_state = [
        initial.private_state,
        recorded.execution.context.private_state,
    ];
    assert_eq!(contract::ledger_slots::cell.inspect(&expected)?, value);
    let prepared = RecordedCall::new(&observed, recorded, NAME, value)
        .prepare(verifier.clone(), communication_randomness)?;
    assert_eq!(prepared.op, operation);
    let generated = api::Contract::from(Answers::default());
    let ordinary = generated
        .recording()
        .apply_offset_call(&observed, 7_u64, value)?
        .prepare(verifier.clone(), communication_randomness)?;
    assert_same_call(&prepared, &ordinary);
    let lab = ProofLab::new(
        Artifacts(root.clone()),
        MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?,
    );
    let resolved = futures_executor::block_on(lab.verifier(
        KeyLocation(NAME.into()),
        EncodedSizeLimit::new(verifier_bytes.len()),
    ))?;
    assert_eq!(resolved, verifier);
    assert!(matches!(
        futures_executor::block_on(
            lab.verifier(KeyLocation("absent".into()), EncodedSizeLimit::default())
        ),
        Err(midnight_compact_testkit::ProofError::Unavailable)
    ));
    assert!(matches!(
        futures_executor::block_on(lab.verifier(
            KeyLocation(NAME.into()),
            EncodedSizeLimit::new(verifier_bytes.len() - 1)
        )),
        Err(midnight_compact_testkit::ProofError::Decode(_))
    ));
    let preimage = call_preimage(&prepared)?;
    let skips = ir.check(&preimage)?;
    assert_eq!(skips, vec![None; 6]);
    let params = MidnightDataProvider::new(FetchMode::Synchronous, OutputMode::Log, vec![])?;
    let (proof, inputs, proved_skips) =
        futures_executor::block_on(ir.prove(rand::rngs::OsRng, &params, prover, &preimage))?;
    assert_eq!(proved_skips, skips);
    assert_eq!(inputs[0], preimage.binding_input);
    assert_eq!(
        inputs[1],
        preimage
            .communications_commitment
            .ok_or("missing commitment")?
            .0
    );
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    let mut changed_binding = inputs.clone();
    changed_binding[0] = changed_binding[0] + Fr::from(1_u64);
    let changed_binding_rejected = verifier
        .verify(&PARAMS_VERIFIER, &proof, changed_binding.into_iter())
        .is_err();
    assert!(changed_binding_rejected);
    let mut changed_communication_rejected = Vec::new();
    let mut changed_input_new_statement_valid = false;
    let mut same_geometry_controls = true;
    for change_output in [false, true] {
        let mut changed = prepared.clone();
        let slot = if change_output {
            &mut changed.output
        } else {
            &mut changed.input
        };
        let replacement = AlignedValue::from(Field::from(43_u64));
        assert_eq!(slot.alignment, replacement.alignment);
        assert_eq!(slot.value.0.len(), replacement.value.0.len());
        assert_ne!(*slot, replacement);
        same_geometry_controls &= slot.alignment == replacement.alignment
            && slot.value.0.len() == replacement.value.0.len();
        *slot = replacement;
        let bad = call_preimage(&changed)?;
        let mut bad_inputs = inputs.clone();
        bad_inputs[1] = bad
            .communications_commitment
            .ok_or("missing changed commitment")?
            .0;
        assert_ne!(inputs[1], bad_inputs[1]);
        let rejected = verifier
            .verify(&PARAMS_VERIFIER, &proof, bad_inputs.into_iter())
            .is_err();
        assert!(rejected);
        changed_communication_rejected.push(rejected);
        let checked = ir.check(&bad);
        // A witness has no algebraic promise to return its argument. Input 43
        // with witness/output 42 remains a valid *different* statement; only the
        // old proof must fail its changed communication commitment.
        if change_output {
            assert!(checked.is_err());
        } else {
            changed_input_new_statement_valid = checked.is_ok();
            assert!(changed_input_new_statement_valid);
        }
    }
    let changed_private_witness_rejected = {
        let mut changed_private = preimage.clone();
        assert!(!changed_private.private_transcript.is_empty());
        changed_private.private_transcript[0] =
            changed_private.private_transcript[0] + Fr::from(1_u64);
        ir.check(&changed_private).is_err()
    };
    assert!(changed_private_witness_rejected);
    let mut changed_proof = proof.clone();
    changed_proof.0[0] ^= 1;
    assert_eq!(changed_proof.0.len(), proof.0.len());
    same_geometry_controls &= changed_proof.0.len() == proof.0.len();
    let changed_proof_rejected = verifier
        .verify(&PARAMS_VERIFIER, &changed_proof, inputs.iter().copied())
        .is_err();
    assert!(changed_proof_rejected);
    // Recheck the unchanged positive after all controls, retaining its exact key.
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    assert_eq!(
        fs::read(root.join("keys/apply_offset.verifier"))?,
        verifier_bytes
    );
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, state.time + state.ledger.parameters.global_ttl)
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let resolver = fee_resolver(&root)?;
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("contract proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x6680_5345));
    let sealed = executor.block_on(state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, call) = calls.next().ok_or("missing funded contract call")?;
    assert!(calls.next().is_none());
    let ProofVersioned::V3(tx_proof) = &call.proof else {
        return Err("expected transaction-bound v3 proof".into());
    };
    assert_ne!(tx_proof.0, proof.0);
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
    let mut context = state.context();
    context.block_context.last_block_time = state.time;
    let (updated, result) = state.ledger.apply(&verified, &context);
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("contract missing after apply")?;
    assert_eq!(actual.data.get_ref(), &expected);
    assert_eq!(actual.operations.get(&entry).as_deref(), Some(&operation));
    context.ref_state = updated.clone();
    let (replayed, result) = updated.apply(&verified, &context);
    let replay_refusal = match result {
        TransactionResult::Failure(
            midnight_ledger::error::TransactionInvalid::ReplayProtectionViolation(
                midnight_ledger::error::TransactionApplicationError::IntentAlreadyExists,
            ),
        ) => "IntentAlreadyExists",
        other => return Err(format!("unexpected replay result: {other:?}").into()),
    };
    let replay_unchanged = replayed == updated;
    assert!(replay_unchanged);
    let mut updated_bytes = Vec::new();
    let mut replayed_bytes = Vec::new();
    tagged_serialize(&updated, &mut updated_bytes)?;
    tagged_serialize(&replayed, &mut replayed_bytes)?;
    assert_eq!(replayed_bytes, updated_bytes);
    let mut transaction = Vec::new();
    tagged_serialize(&sealed, &mut transaction)?;
    let mut final_state = Vec::new();
    tagged_serialize(actual, &mut final_state)?;
    let mut final_data = Vec::new();
    tagged_serialize(actual.data.get_ref(), &mut final_data)?;
    let mut expected_data = Vec::new();
    tagged_serialize(&expected, &mut expected_data)?;
    let final_cell = contract::ledger_slots::cell.inspect(actual.data.get_ref())?;
    fs::write(out.join("standalone.proof"), &proof.0)?;
    let mut public_inputs = Vec::new();
    tagged_serialize(&inputs, &mut public_inputs)?;
    fs::write(out.join("standalone.inputs"), public_inputs)?;
    fs::write(out.join("transaction-bound.proof"), &tx_proof.0)?;
    fs::write(out.join("applied-transaction.bin"), &transaction)?;
    let receipt = serde_json::json!({
        "status":"passed", "standalone_proof_bytes":proof.0.len(),
        "transaction_bytes":transaction.len(),
        "transaction_proof_version":3,
        "transaction_proof_bytes":tx_proof.0.len(),
        "funded_dust_spends":dust_spends,
        "proof_relationship":"separate standalone proof and independently proved funded transaction",
        "changed_binding_rejected":changed_binding_rejected,
        "changed_input_rejected":changed_communication_rejected[0],
        "changed_output_rejected":changed_communication_rejected[1],
        "changed_proof_rejected":changed_proof_rejected,
        "changed_private_witness_rejected":changed_private_witness_rejected,
        "changed_input_new_statement_valid":changed_input_new_statement_valid,
        "changed_input_new_statement_check":if changed_input_new_statement_valid {
            "valid; witness return is unconstrained relative to input"
        } else { "invalid" },
        "same_geometry_controls":same_geometry_controls,
        "strict_application":"default", "replay_refusal":replay_refusal,
        "replay_unchanged":replay_unchanged, "skip_sequence":skips,
        "witness_private_state":witness_private_state,
        "preparation_refusals":preparation_refusals,
        "final_state_hex":hex::encode(final_state),
        "final_data_hex":hex::encode(final_data),
        "expected_data_hex":hex::encode(expected_data),
        "final_cell_le_hex":hex::encode(final_cell.0.to_bytes_le()),
        "scope":"selected call from privileged constructor-derived initial state and upstream fee funding; not constructor/deployment proof or wallet lifecycle"
    });
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    println!("{receipt}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [root, out] => prove_and_apply(PathBuf::from(root), Path::new(out)),
        [mode, root, out] if mode == "--verify" => {
            verify_standalone(Path::new(root), Path::new(out))
        }
        _ => Err("usage: functional-sdk-proof [--verify] GENERATED PROOF_OUTPUT".into()),
    }
}
