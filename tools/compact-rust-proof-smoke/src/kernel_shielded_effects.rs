// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Call proofs preserve Kernel effects; transaction acceptance needs matching offers.

use super::local_v3_application as v3;
use super::positional_transaction::verify_separately;
use super::*;
use compact_rust_kernel_shielded_effects_oracle_fixture::ledger_contract as contract;
use midnight_compact_runtime as runtime;
use midnight_compact_runtime::transaction::OfferBackedObservedState;
use midnight_ledger::{
    error::{
        EffectsCheckError, MalformedTransaction, TransactionApplicationError, TransactionInvalid,
    },
    structure::ProofVersioned,
    test_utilities::tx_prove,
};
use midnight_serialize::{Deserializable, Tagged};
use midnight_transient_crypto::proofs::{Proof, ProverKey, Zkir};
use midnight_zswap::{Offer, Output};

type Amount = BoundedUint<18446744073709551615>;
struct Witnesses;
impl contract::Witnesses<()> for Witnesses {
    fn next_domain(
        &self,
        _: WitnessContext<'_, (), contract::LedgerView<'_>>,
    ) -> ((), FixedBytes<32>) {
        ((), FixedBytes::new([3; 32]))
    }
    fn next_amount(&self, _: WitnessContext<'_, (), contract::LedgerView<'_>>) -> ((), Amount) {
        ((), Amount::new(13).expect("13 fits Uint64"))
    }
}

fn validate_ir(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value =
        serde_json::from_str(compact_rust_kernel_shielded_effects_oracle_fixture::RETAINED_IR)?;
    let actual: serde_json::Value = serde_json::from_slice(bytes)?;
    if expected != actual {
        return Err("original full Kernel source/proof IR mismatch".into());
    }
    Ok(())
}

// SHA-256 of the complete v3 raw ZKIR emitted for the original Compact fixture.
const RAW_CIRCUITS: [(&str, &str); 8] = [
    (
        "batch",
        "50cd27a172693cf9e8bff6f163b3419236dc4991b0ddf8f0cdfb719b1b04a4e1",
    ),
    (
        "claim_receive",
        "f20e321958f68a10e5c1423efb9bf54988c29970365837d7a1d692a8e23ad23d",
    ),
    (
        "mint",
        "cb1be02ca4ee3d1402c8f3c940e91f579fd4bd35077e0dc58996cd4168835b36",
    ),
    (
        "nullifier",
        "2835ffefd19d3d0510eddbd6ad88f62a053fa0f5eef7630a1d5b770cce4ba5ca",
    ),
    (
        "read_state",
        "6b4ea40ac4e1514008f85da5fdaf124aa213c7b73274abdcf7e5ffc782f4693d",
    ),
    (
        "selected",
        "d98c94cc78b45a335df8d0f690ef68bfcadce3f913c2ac0f215b1f89e9ba9551",
    ),
    (
        "spend",
        "2030c2ee0c4fa04d5bcb2257dcaa6e9465c5662578c79b5d5b9d4491ed82c942",
    ),
    (
        "witness_order",
        "23a726e55fa3342556eb8efb9f4d5a56573f64ca7cd4864ff82e5c11797540b8",
    ),
];

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_ir(&std::fs::read(root.join("contract/compact-rust-ir.json"))?)?;
    if std::fs::read(root.join("contract/lib.rs"))?
        != compact_rust_kernel_shielded_effects_oracle_fixture::RENDERED_SOURCE
    {
        return Err("Kernel generated Rust differs from the linked fixture".into());
    }
    let zkir = root.join("zkir");
    let mut raw_count = 0;
    for entry in std::fs::read_dir(&zkir)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "zkir")
        {
            raw_count += 1;
        }
    }
    if raw_count != RAW_CIRCUITS.len() {
        return Err("Kernel raw circuit inventory differs from the original fixture".into());
    }
    for (name, digest) in RAW_CIRCUITS {
        let raw = std::fs::read(zkir.join(format!("{name}.zkir")))?;
        if hex::encode(midnight_base_crypto::hash::persistent_hash(&raw).0) != digest {
            return Err(
                format!("Kernel raw circuit differs from the original fixture: {name}").into(),
            );
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "Kernel shielded effects require local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    validate_artifacts(root)?;
    for (name, selected) in [
        ("mint", false),
        ("nullifier", false),
        ("spend", false),
        ("claim_receive", false),
        ("batch", false),
        ("selected", false),
        ("selected", true),
        ("witness_order", false),
    ] {
        prove_case(root, name, selected)?;
    }
    funded_mint(root)
}

fn prove_case(root: &Path, name: &'static str, selected: bool) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::seed_from_u64(0x0184_4341);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, name)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        name,
        operation.clone(),
        &mut rng,
    );
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let value = FixedBytes::new([2; 32]);
    let seven = Amount::new(7)?;
    let eleven = Amount::new(11)?;
    let generated = contract::Contract::from(Witnesses);
    let recording = generated.recording();
    let call = match name {
        "mint" => recording.mint_call(&observed, (), value, seven)?,
        "nullifier" => recording.nullifier_call(&observed, (), value)?,
        "spend" => recording.spend_call(&observed, (), value)?,
        "claim_receive" => recording.claim_receive_call(&observed, (), value)?,
        "batch" => recording.batch_call(&observed, (), value, value, seven, eleven)?,
        "selected" => recording.selected_call(&observed, (), selected, value)?,
        "witness_order" => recording.witness_order_call(&observed, ())?,
        _ => unreachable!(),
    };
    if name == "selected" && !selected {
        if !matches!(
            call.prepare(verifier, Fr::from(0_u64)),
            Err(ObservedCallError::Prepare(
                runtime::transaction::PrepareCallError::EmptyTranscript
            ))
        ) {
            return Err(
                "zero-effect selected branch did not retain EmptyTranscript refusal".into(),
            );
        }
        println!(
            "selected(false) records/replays zero effects and retains exact EmptyTranscript preparation refusal"
        );
        return Ok(());
    }
    let prepared = call.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("Kernel observed operation differs from the installed v3 operation".into());
    }
    let stem = format!("kernel-{name}");
    v3::prove_standalone(root, name, &stem, &prepared, &verifier, None)?;
    verify_separately(root, name, &stem)?;
    if matches!(name, "nullifier" | "spend" | "claim_receive" | "batch")
        || (name == "selected" && selected)
    {
        let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
            Intent::empty(&mut rng, Timestamp::from_secs(0)).add_call::<ProofPreimage>(prepared);
        let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
        let mut ledger = LedgerState::<DefaultDB>::new("local-test");
        ledger.contract = ledger
            .contract
            .insert(deploy.address(), deploy.initial_state);
        // Isolate mandatory coin-claim semantics from this deliberately fee-unfunded call.
        let mut strictness = WellFormedStrictness::default();
        strictness.enforce_balancing = false;
        let error = tx
            .well_formed(&ledger, strictness, Timestamp::from_secs(0))
            .err()
            .ok_or("unmatched Kernel claim unexpectedly passed transaction validation")?;
        let expected = match (&error, name) {
            (
                MalformedTransaction::EffectsCheckFailure(
                    EffectsCheckError::NullifiersNEClaimedNullifiers {
                        nullifiers,
                        claimed_nullifiers,
                    },
                ),
                "nullifier" | "batch",
            ) => {
                nullifiers.is_empty()
                    && claimed_nullifiers.len() == 1
                    && claimed_nullifiers[0].1
                        == runtime::ledger::CoinNullifier(runtime::ledger::HashOutput([2; 32]))
            }
            (
                MalformedTransaction::EffectsCheckFailure(
                    EffectsCheckError::CommitmentsNEClaimedShieldedReceives {
                        commitments,
                        claimed_shielded_receives,
                    },
                ),
                "claim_receive",
            ) => {
                commitments.is_empty()
                    && claimed_shielded_receives.len() == 1
                    && claimed_shielded_receives[0].1
                        == runtime::ledger::CoinCommitment(runtime::ledger::HashOutput([2; 32]))
            }
            (
                MalformedTransaction::EffectsCheckFailure(
                    EffectsCheckError::AllCommitmentsSubsetCheckFailure(missing),
                ),
                "spend" | "selected",
            ) => {
                missing.superset.is_empty()
                    && missing.subset
                        == vec![(
                            0,
                            runtime::ledger::CoinCommitment(runtime::ledger::HashOutput([2; 32])),
                        )]
            }
            _ => false,
        };
        if !expected {
            return Err(format!("unexpected {name} unmatched-claim rejection: {error:?}").into());
        }
        println!(
            "{name} call proof verified; unmatched-offer transaction rejected exactly: {error:?}"
        );
    } else {
        println!("{name} selected={selected} call proof verified; no transaction acceptance claim");
    }
    Ok(())
}

pub(super) fn prove_and_verify_call(
    root: &Path,
    name: &'static str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<(), Box<dyn Error>> {
    prove_and_verify_call_measured(root, name, call, verifier).map(|_| ())
}

pub(super) fn prove_and_verify_call_measured(
    root: &Path,
    name: &'static str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
) -> Result<usize, Box<dyn Error>> {
    let (proof, _) = prove_and_verify_call_with_input_control::<IrSource>(
        root,
        name,
        call,
        verifier,
        None,
        FetchMode::OnDemand,
    )?;
    Ok(proof.0.len())
}

// A selected same-geometry public-input control uses the original immutable
// verifier and exact communication commitment, without regenerating a proof.
pub(super) fn prove_and_verify_call_with_input_control<Ir: Zkir + Deserializable + Tagged>(
    root: &Path,
    name: &'static str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    changed_input: Option<AlignedValue>,
    fetch_mode: FetchMode,
) -> Result<(Proof, Vec<Fr>), Box<dyn Error>> {
    prove_and_verify_call_with_input_mutation::<Ir>(
        root,
        name,
        call,
        verifier,
        changed_input.map(InputMutation::Used),
        fetch_mode,
    )
}

pub(super) enum InputMutation {
    Used(AlignedValue),
    Unused(AlignedValue),
}

pub(super) fn prove_and_verify_call_with_input_mutation<Ir: Zkir + Deserializable + Tagged>(
    root: &Path,
    name: &'static str,
    call: &ContractCallPrototype<DefaultDB>,
    verifier: &VerifierKey,
    mutation: Option<InputMutation>,
    fetch_mode: FetchMode,
) -> Result<(Proof, Vec<Fr>), Box<dyn Error>> {
    let mut fields = Vec::new();
    call.input.value_only_field_repr(&mut fields);
    call.output.value_only_field_repr(&mut fields);
    let commitment = transient_commit(&fields[..], call.communication_commitment_rand);
    let preimage =
        match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(call, commitment) {
            ProofPreimageVersioned::V2(preimage) => preimage,
            _ => return Err("unexpected Kernel call proof version".into()),
        };
    let ir: Ir = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("zkir/{name}.bzkir")),
    )?))?;
    let pk: ProverKey<Ir> = tagged_deserialize(&mut BufReader::new(File::open(
        root.join(format!("keys/{name}.prover")),
    )?))?;
    let checked_skips = preimage.check(&ir)?;
    let params = MidnightDataProvider::new(fetch_mode, OutputMode::Log, vec![])?;
    // Use upstream IR's exact public inputs, including conditional skip semantics.
    let (proof, inputs, proof_skips) = futures_executor::block_on(ir.prove(
        ChaCha20Rng::from_seed([0x84; 32]),
        &params,
        pk,
        &preimage,
    ))?;
    if proof_skips != checked_skips {
        return Err("Kernel proof/preimage skips differ".into());
    }
    verifier.verify(&PARAMS_VERIFIER, &proof, inputs.iter().copied())?;
    if let Some(mutation) = mutation {
        let (changed_input, used) = match mutation {
            InputMutation::Used(input) => (input, true),
            InputMutation::Unused(input) => (input, false),
        };
        if changed_input.alignment != call.input.alignment
            || changed_input.value.0.len() != call.input.value.0.len()
            || changed_input == call.input
        {
            return Err(
                "changed public input is not a different value with identical geometry".into(),
            );
        }
        let mut changed_call = call.clone();
        changed_call.input = changed_input;
        let mut fields = Vec::new();
        changed_call.input.value_only_field_repr(&mut fields);
        changed_call.output.value_only_field_repr(&mut fields);
        let changed_commitment =
            transient_commit(&fields[..], changed_call.communication_commitment_rand);
        if changed_commitment == commitment || inputs.get(1) != Some(&commitment) {
            return Err("changed public-input commitment geometry differs".into());
        }
        let mut changed_inputs = inputs.clone();
        changed_inputs[1] = changed_commitment;
        if verifier
            .verify(&PARAMS_VERIFIER, &proof, changed_inputs.into_iter())
            .is_ok()
        {
            return Err("proof accepted changed public input".into());
        }
        let changed_preimage = match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(
            &changed_call,
            changed_commitment,
        ) {
            ProofPreimageVersioned::V2(value) => value,
            _ => return Err("unsupported changed-input proof version".into()),
        };
        match changed_preimage.check(&ir) {
            Err(_) if used => {}
            Ok(skips) if !used && skips == checked_skips => {}
            _ => return Err("changed input disagrees with the selected IR branch".into()),
        }
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
        return Err("Kernel proof accepted changed binding".into());
    }
    println!(
        "{name} cryptographic proof verified ({} bytes), changed binding rejected",
        proof.0.len()
    );
    Ok((proof, inputs))
}

fn funded_mint(root: &Path) -> Result<(), Box<dyn Error>> {
    const NAME: &str = "mint";
    let mut rng = StdRng::seed_from_u64(0x0184_4d49);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    const STEM: &str = "kernel-funded-mint";
    let (_, verifier, operation) = v3::load_operation(root, NAME)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        NAME,
        operation.clone(),
        &mut rng,
    );
    let mut fee_state = super::qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    let resolver = super::qualified_coin_funding::fee_resolver(root, NAME)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        STEM,
        &deploy,
        &mut fee_state,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let ledger = fee_state.ledger.clone();
    let time = fee_state.time;
    let domain = runtime::ledger::HashOutput([0x84; 32]);
    let coin = runtime::ledger::CoinInfo {
        nonce: runtime::ledger::CoinNonce(runtime::ledger::HashOutput([1; 32])),
        type_: deploy.address().custom_shielded_token_type(domain),
        value: 42,
    };
    let recipient = runtime::ledger::CoinPublicKey(runtime::ledger::HashOutput([7; 32]));
    let output = Output::new(&mut rng, &coin, None, &recipient, None)?;
    let commitment = output.coin_com;
    let offer: Offer<ProofPreimage, DefaultDB> =
        Offer::new(vec![], vec![output], vec![]).ok_or("empty minted output offer")?;
    let (_, indices) = ledger.zswap.try_apply(&offer, None)?;
    let index = *indices
        .get(&commitment)
        .ok_or("minted output not allocated")?;
    let bound = OfferBackedObservedState::new(
        ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        ),
        &ledger,
        offer,
    )?;
    let call = contract::recorded::Contract.mint_call(
        bound.observed(),
        (),
        FixedBytes::new(domain.0),
        Amount::new(42)?,
    )?;
    let prepared = bound.prepare(call, verifier.clone(), Fr::from(0_u64))?;
    if prepared.prototype().op != operation {
        return Err("funded Kernel mint operation differs from installed v3 operation".into());
    }
    let standalone = v3::prove_standalone(root, NAME, STEM, prepared.prototype(), &verifier, None)?;
    verify_separately(root, NAME, STEM)?;
    let tx = prepared.into_transaction(&mut rng, "local-test", time + ledger.parameters.global_ttl);
    let proven = executor
        .block_on(tx_prove(
            StdRng::from_rng(rand::rngs::OsRng)?,
            &tx,
            &resolver,
        ))
        .map_err(|error| format!("funded Kernel mint transaction proving failed: {error:?}"))?;
    let sealed = proven.seal(StdRng::from_rng(rand::rngs::OsRng)?);
    let sealed = executor.block_on(fee_state.balance_tx(rng, sealed, &resolver))?;
    let mut calls = sealed.calls();
    let (_, call) = calls.next().ok_or("missing funded Kernel mint call")?;
    let ProofVersioned::V3(proof) = &call.proof else {
        return Err("funded Kernel mint requires a transaction-bound v3 proof".into());
    };
    if proof == &standalone || calls.next().is_some() || v3::positive_fee_dust_spends(&sealed) != 1
    {
        return Err("funded Kernel mint proof or Dust spend differs".into());
    }
    let mut tx_bytes = Vec::new();
    tagged_serialize(&sealed, &mut tx_bytes)?;
    v3::write_new(root.join(format!("{STEM}-transaction.bin")), &tx_bytes)?;
    v3::write_new(root.join(format!("{STEM}-transaction.proof")), &proof.0)?;
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
        return Err(format!("funded mint application failed: {outcome:?}").into());
    }
    if updated.contract.get(&deploy.address()) != Some(&deploy.initial_state) {
        return Err("Kernel mint changed the installed operation or public contract state".into());
    }
    if updated.zswap.first_free != index + 1
        || updated.zswap.coin_coms.index(index).map(|(hash, _)| hash) != Some(commitment.0)
    {
        return Err("minted output ledger index/commitment differs".into());
    }
    let (replayed, replay_outcome) = updated.apply(
        &verified,
        &TransactionContext {
            ref_state: updated.clone(),
            ..context
        },
    );
    if !matches!(
        replay_outcome,
        TransactionResult::Failure(TransactionInvalid::ReplayProtectionViolation(
            TransactionApplicationError::IntentAlreadyExists
        ))
    ) || replayed != updated
    {
        return Err(format!(
            "funded Kernel mint replay was refused incorrectly: {replay_outcome:?}"
        )
        .into());
    }
    let mut after = Vec::new();
    let mut replay = Vec::new();
    tagged_serialize(&updated, &mut after)?;
    tagged_serialize(&replayed, &mut replay)?;
    if after != replay {
        return Err("funded Kernel mint replay changed serialized ledger bytes".into());
    }
    v3::write_new(root.join(format!("{STEM}-ledger-after.bin")), &after)?;
    v3::write_new(root.join(format!("{STEM}-ledger-replay.bin")), &replay)?;
    println!(
        "nonzero Kernel mint42 + matching minted output + Night-backed Dust passed default strict validation and ledger application at output index {index}; replay refused without mutation"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_mismatched_source_before_reading_keys() {
        let source = compact_rust_kernel_shielded_effects_oracle_fixture::RETAINED_IR;
        validate_ir(source.as_bytes()).unwrap();
        let mut altered: serde_json::Value = serde_json::from_str(source).unwrap();
        altered["stateful_circuits"][0]["name"] = "different".into();
        assert!(validate_ir(&serde_json::to_vec(&altered).unwrap()).is_err());
        assert!(
            validate_ir(include_bytes!(
                "../../compact-rust-backend/tests/effect-plan-kernel-schema20-ir.json"
            ))
            .is_err()
        );
    }
}
