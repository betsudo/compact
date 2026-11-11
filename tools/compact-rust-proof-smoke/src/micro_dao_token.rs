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
//! Original microDAO token query: real nonempty proof and ledger application.
use super::local_v3_application as v3;
use super::positional_transaction::{check_generated_trace, check_transaction, make_deploy};
use super::*;
use compact_rust_test_center_micro_dao_fixture::{ledger_contract as c, types};
use midnight_base_crypto::hash::{HashOutput, persistent_hash};
use midnight_compact_runtime::{BoundedUint, FixedBytes};
use std::collections::BTreeMap;

const ARTIFACT_PINS: &str =
    include_str!("../../compact-rust-backend/tests/micro-dao-state-ledger9-artifacts.json");

#[derive(serde::Deserialize)]
struct ArtifactPins {
    source: BTreeMap<String, String>,
    material: BTreeMap<String, BTreeMap<String, String>>,
}

fn validate_ir(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let expected: serde_json::Value =
        serde_json::from_str(compact_rust_test_center_micro_dao_fixture::RETAINED_IR)?;
    let actual: serde_json::Value = serde_json::from_slice(bytes)?;
    if expected != actual {
        return Err("original full microDAO source/proof IR mismatch".into());
    }
    Ok(())
}

pub(super) fn validate_artifacts(root: &Path, name: &str) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("MicroDAO state proof requires local proving".into());
    }
    validate_ir(&std::fs::read(root.join("contract/compact-rust-ir.json"))?)?;
    if std::fs::read(root.join("contract/lib.rs"))?
        != compact_rust_test_center_micro_dao_fixture::RENDERED_SOURCE
    {
        return Err("MicroDAO generated Rust differs from the linked fixture".into());
    }
    let pins: ArtifactPins = serde_json::from_str(ARTIFACT_PINS)?;
    let raw = std::fs::read_dir(root.join("zkir"))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "zkir"))
        .count();
    if raw
        != pins
            .source
            .keys()
            .filter(|path| path.ends_with(".zkir"))
            .count()
    {
        return Err("MicroDAO raw circuit inventory differs from the original fixture".into());
    }
    let material = pins
        .material
        .get(name)
        .ok_or("unsupported MicroDAO state circuit")?;
    for (path, expected) in pins.source.iter().chain(material) {
        if hex::encode(persistent_hash(&std::fs::read(root.join(path))?).0) != *expected {
            return Err(
                format!("MicroDAO artifact differs from the retained fixture: {path}").into(),
            );
        }
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    let name = "dao_voting_token";
    validate_artifacts(root, name)?;
    let mut rng = StdRng::seed_from_u64(0x0190_0000);
    let initial = c::initial_state(
        ConstructorContext::new(vec![77u8]),
        FixedBytes::new([4; 32]),
        types::Costs {
            seed_dust: BoundedUint::new(10)?,
            buy_in_dust: BoundedUint::new(3)?,
        },
    )?;
    let expected = initial.ledger_state.get_ref().clone();
    let deploy = make_deploy(root, name, expected.clone(), &mut rng)?;
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let recorded = c::recorded::dao_voting_token(observed.circuit_context(vec![77u8]))?;
    if recorded.public.verify_ops().is_empty()
        || !recorded.execution.private_transcript_outputs.is_empty()
        || recorded.execution.context.private_state != [77]
    {
        return Err("token query transcript/private invariant".into());
    }
    let native = c::dao_voting_token(observed.circuit_context(vec![77u8]))?;
    let mut domain = [0; 32];
    domain[..16].copy_from_slice(b"dao_voting_token");
    if recorded.execution.result != native.result
        || native.result.0
            != deploy
                .address()
                .custom_shielded_token_type(HashOutput(domain))
                .0
                .0
    {
        return Err("token derivation differs at deployed address".into());
    }
    let manual = check_generated_trace(root, name, recorded, ())?;
    let verifier = v3::read_verifier(root, name)?;
    let typed = c::recorded::Contract.dao_voting_token_call(&observed, vec![77u8])?;
    let prepared = typed.prepare(verifier, Fr::from(0u64))?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("token observed/direct mismatch".into());
    }
    check_transaction(root, name, deploy, prepared, &mut rng, |state| {
        if state.data.get_ref() != &expected {
            return Err("token query changed ledger state".into());
        }
        Ok(())
    })?;
    println!(
        "original microDAO.dao_voting_token preserved its deployed-address derivation after funded default-strict application"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_mismatched_source_before_reading_keys() {
        let source = compact_rust_test_center_micro_dao_fixture::RETAINED_IR;
        validate_ir(source.as_bytes()).unwrap();
        let mut altered: serde_json::Value = serde_json::from_str(source).unwrap();
        altered["stateful_circuits"][0]["name"] = "different".into();
        assert!(validate_ir(&serde_json::to_vec(&altered).unwrap()).is_err());
        assert!(
            validate_ir(include_bytes!(
                "../../compact-rust-backend/tests/token-query-schema20-ir.json"
            ))
            .is_err()
        );
    }
}
