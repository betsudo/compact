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
use midnight_base_crypto::{
    data_provider::{FetchMode, MidnightDataProvider, OutputMode},
    signatures::Signature,
    time::Timestamp,
};
use midnight_compact_testkit::ProofLab;
use midnight_ledger::{
    construct::{ContractCallExt, ContractCallPrototype},
    dust::{DUST_EXPECTED_FILES, DustResolver},
    prove::Resolver as LedgerResolver,
    semantics::{TransactionContext, TransactionResult},
    structure::{
        ContractDeploy, INITIAL_PARAMETERS, Intent, ProofPreimageMarker, ProofPreimageVersioned,
        Transaction,
    },
    test_utilities::TestState,
    verify::WellFormedStrictness,
};
use midnight_onchain_runtime::context::BlockContext;
use midnight_onchain_state::state::{
    ContractMaintenanceAuthority, ContractOperation, ContractState, EntryPointBuf,
};
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_storage::storage::HashMap;
use midnight_transient_crypto::{
    commitment::PedersenRandomness,
    curve::Fr,
    fab::AlignedValueExt,
    hash::transient_commit,
    proofs::{
        KeyLocation, PARAMS_VERIFIER, ProofPreimage, ProvingKeyMaterial, ProvingProvider, Resolver,
        VerifierKey,
    },
};
use midnight_zswap::prove::ZswapResolver;
use rand::{SeedableRng, rngs::StdRng};
use rt::{
    CompactError, Field,
    context::{ConstructorContext, WitnessContext},
    fab::AlignedValue,
    ledger::DefaultDB,
    transaction::{CallSpec, EncodedSizeLimit, Observation, ObservedContractState, prepare_call},
};
use std::{
    cell::RefCell,
    error::Error,
    fs, io,
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
fn public_inputs(p: &ProofPreimage) -> Result<Vec<Fr>, Box<dyn Error>> {
    // Exact pinned upstream protocol for this unconditional source. Provider.check
    // below rejects skip-bearing executions; this is not a general PI interpreter.
    let mut inputs = vec![
        p.binding_input,
        p.communications_commitment.ok_or("missing commitment")?.0,
    ];
    inputs.extend(p.public_transcript_inputs.iter().copied());
    Ok(inputs)
}
fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("missing generated directory")?,
    );
    let out = PathBuf::from(
        std::env::args_os()
            .nth(2)
            .ok_or("missing output directory")?,
    );
    fs::create_dir(&out)?;
    let verifier_bytes = fs::read(root.join("keys/apply_offset.verifier"))?;
    let verifier: VerifierKey = tagged_deserialize(&mut verifier_bytes.as_slice())?;
    let mut rng = StdRng::seed_from_u64(0x6680_0042);
    let initial = api::initial_state(ConstructorContext::new(7_u64))?;
    let operations = HashMap::new().insert(
        EntryPointBuf(NAME.as_bytes().to_vec()),
        ContractOperation::new(Some(verifier.clone())),
    );
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
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(state.give_fee_token(&mut rng, 1));
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
    assert_eq!(contract::ledger_slots::cell.inspect(&expected)?, value);
    let prepared = prepare_call(
        recorded,
        CallSpec::new(NAME, verifier.clone(), value, Fr::from(0_u64)),
    )?;
    let generated = api::Contract::from(Answers::default());
    let ordinary = generated
        .recording()
        .apply_offset_call(&observed, 7_u64, value)?
        .prepare(verifier.clone(), Fr::from(0_u64))?;
    assert_eq!(format!("{prepared:?}"), format!("{ordinary:?}"));
    let lab = ProofLab::new(
        Artifacts(root.clone()),
        MidnightDataProvider::new(FetchMode::OnDemand, OutputMode::Log, vec![])?,
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
    let provider = lab.provider(StdRng::seed_from_u64(0x6680_5052));
    let skips = futures_executor::block_on(provider.check(&preimage))?;
    assert!(
        skips.iter().all(Option::is_none),
        "this selected source must have no skipped public inputs"
    );
    let proof = futures_executor::block_on(provider.prove(&preimage, None))?;
    let inputs = public_inputs(&preimage)?;
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    let mut changed_binding = inputs.clone();
    changed_binding[0] = changed_binding[0] + Fr::from(1_u64);
    assert!(
        verifier
            .verify(&PARAMS_VERIFIER, &proof, changed_binding.into_iter())
            .is_err()
    );
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
        *slot = replacement;
        let bad = call_preimage(&changed)?;
        let bad_inputs = public_inputs(&bad)?;
        assert_ne!(inputs[1], bad_inputs[1]);
        assert!(
            verifier
                .verify(&PARAMS_VERIFIER, &proof, bad_inputs.into_iter())
                .is_err()
        );
        let checked =
            futures_executor::block_on(lab.provider(StdRng::seed_from_u64(7)).check(&bad));
        // A witness has no algebraic promise to return its argument. Input 43
        // with witness/output 42 remains a valid *different* statement; only the
        // old proof must fail its changed communication commitment.
        if change_output {
            assert!(checked.is_err());
        } else {
            assert!(checked.is_ok());
        }
    }
    {
        let mut changed_private = preimage.clone();
        assert!(!changed_private.private_transcript.is_empty());
        changed_private.private_transcript[0] =
            changed_private.private_transcript[0] + Fr::from(1_u64);
        assert!(
            futures_executor::block_on(
                lab.provider(StdRng::seed_from_u64(8))
                    .check(&changed_private)
            )
            .is_err()
        );
    }
    let mut changed_proof = proof.clone();
    changed_proof.0[0] ^= 1;
    assert_eq!(changed_proof.0.len(), proof.0.len());
    assert!(
        verifier
            .verify(&PARAMS_VERIFIER, &changed_proof, inputs.iter().copied())
            .is_err()
    );
    // Recheck the unchanged positive after all controls, retaining its exact key.
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    assert_eq!(
        fs::read(root.join("keys/apply_offset.verifier"))?,
        verifier_bytes
    );
    let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
        Intent::empty(&mut rng, Timestamp::from_secs(state.time.to_secs() + 3600))
            .add_call::<ProofPreimage>(prepared);
    let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
    let fee_lab = ProofLab::new(
        fee_resolver(&root)?,
        MidnightDataProvider::new(FetchMode::OnDemand, OutputMode::Log, vec![])?,
    );
    let proven = futures_executor::block_on(tx.prove(
        fee_lab.provider(StdRng::seed_from_u64(0x6680_5458)),
        &INITIAL_PARAMETERS.cost_model.runtime_cost_model,
    ))?;
    let sealed = proven.seal(StdRng::seed_from_u64(0x6680_5345));
    let resolver = fee_resolver(&root)?;
    let sealed = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(state.balance_tx(rng, sealed, &resolver))?;
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
    assert!(
        matches!(result, TransactionResult::Success(_)),
        "{result:?}"
    );
    let actual = updated
        .contract
        .get(&address)
        .ok_or("contract missing after apply")?;
    assert_eq!(actual.data.get_ref(), &expected);
    let context = TransactionContext {
        ref_state: updated.clone(),
        ..context
    };
    let (replayed, result) = updated.apply(&verified, &context);
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
    assert!(replayed == updated);
    let mut transaction = Vec::new();
    tagged_serialize(&sealed, &mut transaction)?;
    let mut final_state = Vec::new();
    tagged_serialize(actual, &mut final_state)?;
    fs::write(out.join("standalone.proof"), &proof.0)?;
    fs::write(out.join("applied-transaction.bin"), &transaction)?;
    let receipt = serde_json::json!({"status":"passed","standalone_proof_bytes":proof.0.len(),"transaction_bytes":transaction.len(),"proof_relationship":"separate standalone proof and independently proved funded transaction","changed_binding_rejected":true,"changed_input_rejected":true,"changed_output_rejected":true,"changed_proof_rejected":true,"changed_private_witness_rejected":true,"changed_input_new_statement_check":"valid; witness return is unconstrained relative to input","same_geometry_controls":true,"strict_application":"default","replay_refusal":"IntentAlreadyExists","replay_unchanged":true,"skip_sequence":skips,"witness_private_state":[7,8],"final_state_hex":hex::encode(final_state),"scope":"selected call from privileged constructor-derived initial state and upstream fee funding; not constructor/deployment proof or wallet lifecycle"});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    println!("{receipt}");
    Ok(())
}
