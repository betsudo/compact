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
//! Read-only composites: real nonempty proofs and exact empty-call refusal.
use super::kernel_shielded_effects::InputMutation;
use super::local_v3_application as v3;
use super::*;
use compact_rust_field_observation_oracle_fixture::{GENERATED_RUST, ledger_contract as c};
use midnight_base_crypto::hash::persistent_hash;
use midnight_compact_runtime as runtime;

const ARTIFACTS: &str =
    include_str!("../../compact-rust-backend/tests/field-observation-ledger9-artifacts.json");
const CASES: [(&str, bool, u8); 8] = [
    ("direct", false, 6),
    ("snapshot", false, 6),
    ("via_field_helper", false, 6),
    ("via_snapshot_helper", false, 6),
    ("pair", false, 7),
    ("selected", false, 7),
    ("selected", true, 7),
    ("optional", true, 6),
];

fn validate_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::fs::read(root.join("contract/lib.rs"))? != GENERATED_RUST {
        return Err("field observation generated Rust differs from the linked fixture".into());
    }
    let pins: std::collections::BTreeMap<String, String> = serde_json::from_str(ARTIFACTS)?;
    for (path, expected) in pins {
        let bytes = std::fs::read(root.join(&path))?;
        if hex::encode(persistent_hash(&bytes).0) != expected {
            return Err(format!(
                "field observation artifact differs from the retained source: {path}"
            )
            .into());
        }
    }
    Ok(())
}

pub(super) fn verify(root: &Path) -> Result<(), Box<dyn Error>> {
    validate_artifacts(root)?;
    for (name, selected, k) in CASES {
        let (_, verifier, _) = v3::operation(root, name, k)?;
        let stem = format!("field-observation-{name}-{selected}");
        v3::verify_standalone(root, &stem, &verifier)?;
        println!("{stem}: separate v3 verification passed and corrupted proof was refused");
    }
    Ok(())
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "field observation requires local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    validate_artifacts(root)?;
    for (index, (name, selected, k)) in CASES.into_iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(0x0201_0000 + index as u64);
        let initial = c::initial_state(
            ConstructorContext::new(Vec::<u8>::new()),
            runtime::Field::from(7u64),
            runtime::Field::from(19u64),
        )?;
        let expected_state = initial.ledger_state.get_ref().clone();
        let (_, verifier, operation) = v3::operation(root, name, k)?;
        let deploy = v3::deploy(expected_state.clone(), name, operation.clone(), &mut rng);
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        macro_rules! calls { ($circuit:ident, $call:ident, $input:expr $(,$arg:expr)*) => {{
            let recorded = c::recorded::$circuit(observed.circuit_context(Vec::<u8>::new()) $(,$arg)*)?;
            let mut manual = prepare_call(recorded, CallSpec::new(name, verifier.clone(), $input, Fr::from(0u64)))?;
            manual.op = operation.clone();
            let typed = c::Contract::default().recording.$call(&observed, Vec::<u8>::new() $(,$arg)*)?;
            (manual, typed.prepare(verifier.clone(), Fr::from(0u64))?)
        }}; }
        let (manual, prepared) = match name {
            "direct" => calls!(direct, direct_call, ()),
            "snapshot" => calls!(snapshot, snapshot_call, ()),
            "via_field_helper" => calls!(via_field_helper, via_field_helper_call, ()),
            "via_snapshot_helper" => calls!(via_snapshot_helper, via_snapshot_helper_call, ()),
            "pair" => calls!(pair, pair_call, ()),
            "selected" => calls!(selected, selected_call, selected, selected),
            "optional" => {
                let empty = c::Contract::default().recording.optional_call(
                    &observed,
                    Vec::<u8>::new(),
                    false,
                )?;
                if !matches!(
                    empty.prepare(verifier.clone(), Fr::from(0u64)),
                    Err(ObservedCallError::Prepare(
                        runtime::transaction::PrepareCallError::EmptyTranscript
                    ))
                ) {
                    return Err("optional(false) must retain exact empty-transcript refusal".into());
                }
                println!("optional(false): exact EmptyTranscript, no fabricated operation");
                calls!(optional, optional_call, true, true)
            }
            _ => unreachable!(),
        };
        if format!("{manual:?}") != format!("{prepared:?}") {
            return Err("manual/observed preparation differs".into());
        }
        let stem = format!("field-observation-{name}-{selected}");
        let mutation = matches!(name, "selected" | "optional")
            .then(|| InputMutation::Used(AlignedValue::from(!selected)));
        let proof = v3::prove_standalone(root, name, &stem, &prepared, &verifier, mutation)?;
        v3::apply_after_deployment(root, name, &stem, prepared, deploy, &expected_state, &proof)?;
    }
    Ok(())
}
