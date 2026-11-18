// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Fresh proof from the unchanged upstream unshielded-token contract.

use super::{local_v3_application as v3, positional_transaction, *};
use crate::ledger_contract as contract;
use midnight_onchain_runtime::context::ClaimedUnshieldedSpendsKey;
use midnight_transient_crypto::proofs::Proof;

const CIRCUIT: &str = "mintUnshieldedToSelfTest";
const STEM: &str = "unshielded-original-mint-self";
const GENERATED_RUST: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/contract/lib.rs"
));
const GENERATED_IR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/contract/compact-rust-ir.json"
));
const GENERATED_ZKIR: &[u8] = include_bytes!(concat!(
    env!("COMPACT_UNSHIELDED_ORIGINAL_GENERATED"),
    "/zkir/mintUnshieldedToSelfTest.zkir"
));

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    for (name, expected) in [
        ("contract/lib.rs", GENERATED_RUST),
        ("contract/compact-rust-ir.json", GENERATED_IR),
        ("zkir/mintUnshieldedToSelfTest.zkir", GENERATED_ZKIR),
    ] {
        if std::fs::read(root.join(name))? != expected {
            return Err(format!("original unshielded artifact changed: {name}").into());
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("original unshielded proof requires local proving".into());
    }
    validate_artifacts(root)?;
    let mut rng = StdRng::seed_from_u64(0x0184_554e);
    let initial = contract::initial_state(ConstructorContext::new(()))?;
    let (_, verifier, operation) = v3::load_operation(root, CIRCUIT)?;
    let deploy = v3::deploy(
        initial.ledger_state.get_ref().clone(),
        CIRCUIT,
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
    let generated = contract::Contract::default();
    let recorded = generated.recording.mintUnshieldedToSelfTest_call(
        &observed,
        (),
        FixedBytes::new([0x84; 32]),
        BoundedUint::<18446744073709551615>::new(100)?,
    )?;
    let prepared = recorded.prepare(verifier.clone(), Fr::from(0_u64))?;
    if prepared.op != operation {
        return Err("original unshielded call operation differs from installed v3 IR".into());
    }
    let transcript = prepared
        .guaranteed_public_transcript
        .as_ref()
        .or(prepared.fallible_public_transcript.as_ref())
        .ok_or("original unshielded call has no effects transcript")?;
    let effects = &transcript.effects;
    let domain = midnight_compact_runtime::ledger::HashOutput([0x84; 32]);
    let token = midnight_compact_runtime::ledger::TokenType::Unshielded(
        deploy.address().custom_unshielded_token_type(domain),
    );
    let claim = ClaimedUnshieldedSpendsKey(
        token,
        midnight_compact_runtime::ledger::PublicAddress::Contract(deploy.address()),
    );
    if effects.unshielded_mints.size() != 1
        || effects.unshielded_inputs.size() != 1
        || effects.claimed_unshielded_spends.size() != 1
        || !effects.unshielded_outputs.is_empty()
        || *effects
            .unshielded_mints
            .get(&domain)
            .ok_or("mint effect missing")?
            != 100
        || *effects
            .unshielded_inputs
            .get(&token)
            .ok_or("input effect missing")?
            != 100
        || *effects
            .claimed_unshielded_spends
            .get(&claim)
            .ok_or("claim effect missing")?
            != 100
    {
        return Err("original unshielded mint effects differ from 100-unit self claim".into());
    }
    let proof: Proof = v3::prove_standalone(root, CIRCUIT, STEM, &prepared, &verifier, None)?;
    positional_transaction::verify_separately(root, CIRCUIT, STEM)?;
    v3::apply(
        root,
        CIRCUIT,
        STEM,
        prepared,
        deploy,
        initial.ledger_state.get_ref(),
        &proof,
    )?;
    println!("{CIRCUIT}: original source fresh proof and funded strict replay gate passed");
    Ok(())
}
