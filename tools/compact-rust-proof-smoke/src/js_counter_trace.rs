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

//! Test-local JS transport qualification using official Ledger8 types.
//! `run` only decodes; `run_counter` proves and applies the JS-derived call.

use std::{error::Error, fs::File, io::Read, path::Path};

use midnight_serialize::tagged_deserialize;
use midnight_transient_crypto::proofs::ProofPreimage;

const MAX_PREIMAGE_BYTES: u64 = 16 * 1024 * 1024;

fn decode_preimage(bytes: &[u8]) -> Result<ProofPreimage, Box<dyn Error>> {
    if bytes.len() as u64 > MAX_PREIMAGE_BYTES {
        return Err("JS proof preimage exceeds 16 MiB bound".into());
    }
    let mut remaining = bytes;
    let preimage = tagged_deserialize::<ProofPreimage>(&mut remaining)?;
    if !remaining.is_empty() {
        return Err("JS proof preimage has trailing bytes".into());
    }
    Ok(preimage)
}

pub(super) fn read_preimage(path: &Path) -> Result<ProofPreimage, Box<dyn Error>> {
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err("JS proof preimage must be a regular non-symlink file".into());
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_PREIMAGE_BYTES {
        return Err("JS proof preimage is not a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_PREIMAGE_BYTES + 1).read_to_end(&mut bytes)?;
    decode_preimage(&bytes)
}

pub(super) fn run(path: &Path) -> Result<(), Box<dyn Error>> {
    let preimage = read_preimage(path)?;
    println!(
        "{}",
        serde_json::json!({
            "format": "compact-js-preimage-compatibility/v1",
            "scope": "official tagged decoding only; no proof, verification or ledger application",
            "inputs": preimage.inputs.len(),
            "private_transcript": preimage.private_transcript.len(),
            "public_transcript_inputs": preimage.public_transcript_inputs.len(),
            "public_transcript_outputs": preimage.public_transcript_outputs.len(),
            "key_location": preimage.key_location.0,
        })
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use midnight_serialize::tagged_serialize;
    use midnight_transient_crypto::proofs::KeyLocation;

    fn official_bytes() -> Vec<u8> {
        let value = ProofPreimage {
            inputs: vec![0u8.into()],
            private_transcript: vec![],
            public_transcript_inputs: vec![],
            public_transcript_outputs: vec![],
            binding_input: 0u8.into(),
            communications_commitment: None,
            key_location: KeyLocation("serializer-test".into()),
        };
        let mut bytes = Vec::new();
        tagged_serialize(&value, &mut bytes).unwrap();
        bytes
    }

    #[test]
    fn official_roundtrip_and_trailing_bytes() {
        let mut bytes = official_bytes();
        assert_eq!(
            decode_preimage(&bytes).unwrap().key_location.0,
            "serializer-test"
        );
        bytes.push(0);
        // The official compressed decoder may refuse before our remaining-byte
        // guard; either owner must reject the actual trailing input.
        assert!(decode_preimage(&bytes).is_err());
    }

    #[test]
    fn invalid_truncated_and_oversized_inputs_are_refused() {
        assert!(decode_preimage(b"not a tagged preimage").is_err());
        let bytes = official_bytes();
        assert!(decode_preimage(&bytes[..bytes.len() - 1]).is_err());
        assert!(decode_preimage(&vec![0; MAX_PREIMAGE_BYTES as usize + 1]).is_err());
    }
}

fn exact_fields(value: &serde_json::Value, expected: &[&str]) -> Result<(), Box<dyn Error>> {
    let object = value.as_object().ok_or("expected transport object")?;
    if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
        return Err("unknown or missing JS counter transport field".into());
    }
    Ok(())
}

fn aligned(value: &serde_json::Value) -> Result<super::AlignedValue, Box<dyn Error>> {
    exact_fields(value, &["value", "alignment"])?;
    let atoms = value["value"].as_array().ok_or("expected atom array")?;
    let alignment = value["alignment"]
        .as_array()
        .ok_or("expected alignment array")?;
    if atoms.len() > 8 || alignment.len() != atoms.len() {
        return Err("counter aligned atom count exceeds finite transport".into());
    }
    for (atom, segment) in atoms.iter().zip(alignment) {
        let bytes = atom.as_array().ok_or("expected byte atom")?;
        if bytes.len() > 32 || bytes.iter().any(|v| v.as_u64().is_none_or(|n| n > 255)) {
            return Err("invalid counter byte atom".into());
        }
        exact_fields(segment, &["tag", "value"])?;
        if segment["tag"] != "atom" {
            return Err("counter transport permits only atomic alignment".into());
        }
        let kind = &segment["value"];
        match kind["tag"].as_str() {
            Some("field") => exact_fields(kind, &["tag"])?,
            Some("bytes") => {
                exact_fields(kind, &["tag", "length"])?;
                if kind["length"].as_u64().is_none_or(|n| n > 32) {
                    return Err("invalid counter byte alignment".into());
                }
            }
            _ => return Err("unsupported counter alignment".into()),
        }
    }
    Ok(serde_json::from_value(value.clone())?)
}

fn operations(value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    let rows = value.as_array().ok_or("expected public transcript array")?;
    if rows.is_empty() || rows.len() > 256 {
        return Err("counter public transcript exceeds finite bound".into());
    }
    for op in rows {
        if op == "lt" {
            continue;
        }
        let object = op.as_object().ok_or("unsupported counter operation")?;
        if object.len() != 1 {
            return Err("invalid counter operation fields".into());
        }
        let (name, body) = object.iter().next().ok_or("empty counter operation")?;
        match name.as_str() {
            "idx" => {
                exact_fields(body, &["cached", "pushPath", "path"])?;
                let path = body["path"].as_array().ok_or("expected counter path")?;
                if path.len() != 1 {
                    return Err("counter path must have one component".into());
                }
                exact_fields(&path[0], &["tag", "value"])?;
                if path[0]["tag"] != "value" {
                    return Err("unsupported counter key".into());
                }
                aligned(&path[0]["value"])?;
            }
            "addi" | "subi" => {
                exact_fields(body, &["immediate"])?;
                if body["immediate"]
                    .as_u64()
                    .is_none_or(|n| n > u16::MAX.into())
                {
                    return Err("counter immediate exceeds fixture bound".into());
                }
            }
            "ins" => exact_fields(body, &["cached", "n"])?,
            "dup" => exact_fields(body, &["n"])?,
            "popeq" => {
                exact_fields(body, &["cached", "result"])?;
                aligned(&body["result"])?;
            }
            "push" => {
                exact_fields(body, &["storage", "value"])?;
                exact_fields(&body["value"], &["tag", "content"])?;
                if body["value"]["tag"] != "cell" {
                    return Err("counter push must be cell".into());
                }
                aligned(&body["value"]["content"])?;
            }
            _ => return Err("unsupported counter operation".into()),
        }
    }
    Ok(())
}

fn counter_state(value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    exact_fields(value, &["tag", "content"])?;
    if value["tag"] != "array" {
        return Err("JS counter state must be an encoded array".into());
    }
    let cells = value["content"]
        .as_array()
        .ok_or("expected encoded counter cells")?;
    if cells.len() != 2 {
        return Err("JS counter state must contain exactly two cells".into());
    }
    for cell in cells {
        exact_fields(cell, &["tag", "content"])?;
        if cell["tag"] != "cell" {
            return Err("JS counter state supports only cells".into());
        }
        aligned(&cell["content"])?;
    }
    Ok(())
}

// Decode from original JS bytes, never from sorted serde_json::Value objects:
// the pinned official StateValue visitor requires tag before content, including
// StateValue operands nested inside push operations. Shape checks above retain
// the finite fixture boundary; this decoder owns official types and invariants.
#[derive(serde::Deserialize)]
struct OfficialCounterTrace {
    #[serde(rename = "publicTranscript")]
    program: Vec<super::Op<midnight_onchain_vm::result_mode::ResultModeVerify, super::DefaultDB>>,
    #[serde(rename = "initialState")]
    initial: super::StateValue<super::DefaultDB>,
    #[serde(rename = "finalState")]
    final_state: super::StateValue<super::DefaultDB>,
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err("counter transport must be regular non-symlink file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_PREIMAGE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_PREIMAGE_BYTES {
        return Err("counter transport exceeds bound".into());
    }
    Ok(bytes)
}

pub(super) fn run_counter(root: &Path, directory: &Path) -> Result<(), Box<dyn Error>> {
    use super::*;
    use compact_rust_effect_plan_counter_only_source_fixture as source;
    use midnight_ledger::construct::{PreTranscript, partition_transcripts};
    counter_scenario::validate_ir(
        &read_bounded(&root.join("contract/compact-rust-ir.json"))?,
        source::RETAINED_IR,
    )?;
    for (branch, amount, remove, expected) in [(0u64, 3u64, 2u64, 1u64), (1, 10, 65535, 0)] {
        let bytes = read_bounded(&directory.join(format!("branch-{branch}.json")))?;
        let trace: serde_json::Value = serde_json::from_slice(&bytes)?;
        exact_fields(
            &trace,
            &[
                "branch",
                "amount",
                "remove",
                "expected",
                "input",
                "output",
                "publicTranscript",
                "privateTranscriptOutputs",
                "initialState",
                "finalState",
            ],
        )?;
        let (amount_text, remove_text, expected_text) =
            (amount.to_string(), remove.to_string(), expected.to_string());
        if trace["branch"] != branch
            || trace["amount"].as_str() != Some(amount_text.as_str())
            || trace["remove"].as_str() != Some(remove_text.as_str())
            || trace["expected"].as_str() != Some(expected_text.as_str())
            || trace["privateTranscriptOutputs"] != serde_json::json!([])
        {
            return Err("JS counter fixture inputs/private outputs differ".into());
        }
        let input = aligned(&trace["input"])?;
        let output = aligned(&trace["output"])?;
        let expected_input = AlignedValue::concat(&[
            AlignedValue::from(BoundedUint::<65535>::new(amount.into())?),
            AlignedValue::from(BoundedUint::<65535>::new(remove.into())?),
        ]);
        if input != expected_input || output != AlignedValue::from(Field::from(expected)) {
            return Err("JS counter aligned values differ from independent expectations".into());
        }
        operations(&trace["publicTranscript"])?;
        counter_state(&trace["initialState"])?;
        counter_state(&trace["finalState"])?;
        let OfficialCounterTrace {
            program,
            initial: js_initial,
            final_state: js_final,
        } = serde_json::from_slice(&bytes)?;
        let initial = source::ledger_contract::initial_state(ConstructorContext::new(7u64))?;
        if &js_initial != initial.ledger_state.get_ref() {
            return Err("JS initial state differs from constructor baseline".into());
        }
        let StateValue::Array(counters) = &js_final else {
            return Err("counter state must be array".into());
        };
        if counters.len() != 2
            || read_counter(counters.get(0).ok_or("missing JS round counter")?)? != expected
            || read_counter(counters.get(1).ok_or("missing JS peer counter")?)? != 10
        {
            return Err("JS final counter state differs from independent expectations".into());
        }
        let mut rng = StdRng::seed_from_u64(0x4a53_0000 + branch);
        let deploy = make_deploy(root, "adjust", js_initial, &mut rng)?;
        let observed = ObservedContractState::new(
            deploy.address(),
            deploy.initial_state.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 0,
            },
        );
        let context = observed.circuit_context(7u64);
        let replay = context
            .query
            .query(&program, None, &context.cost_model)
            .map_err(|e| format!("JS public trace replay failed: {e:?}"))?;
        if replay.context.state.get_ref() != &js_final {
            return Err("JS public trace replay differs from JS final state".into());
        }
        let partitioned = partition_transcripts(
            &[PreTranscript {
                context: context.query,
                program,
                comm_comm: None,
            }],
            &INITIAL_PARAMETERS,
        )?;
        if partitioned.len() != 1 {
            return Err("JS counter partition count changed".into());
        }
        let (guaranteed, fallible) = partitioned
            .into_iter()
            .next()
            .ok_or("missing JS counter partition")?;
        if guaranteed.is_some() == fallible.is_some() {
            return Err("JS counter requires exactly one transcript partition".into());
        }
        let transcript = guaranteed
            .as_ref()
            .or(fallible.as_ref())
            .ok_or("empty JS counter transcript")?;
        if transcript.effects != replay.context.effects {
            return Err("JS replay/partition effects differ".into());
        }
        let verifier: VerifierKey =
            tagged_deserialize(&mut read_bounded(&root.join("keys/adjust.verifier"))?.as_slice())?;
        let call = ContractCallPrototype {
            address: deploy.address(),
            entry_point: EntryPointBuf(b"adjust".to_vec()),
            op: ContractOperation::new(Some(verifier)),
            guaranteed_public_transcript: guaranteed,
            fallible_public_transcript: fallible,
            private_transcript_outputs: vec![],
            input,
            output,
            communication_commitment_rand: Fr::from(0u8),
            key_location: KeyLocation("adjust".into()),
        };
        let js_preimage = read_preimage(&directory.join(format!("branch-{branch}.preimage")))?;
        let mut value_fields = Vec::new();
        call.input.value_only_field_repr(&mut value_fields);
        call.output.value_only_field_repr(&mut value_fields);
        let comm = transient_commit(&value_fields, Fr::from(0u8));
        let constructed =
            match <ProofPreimage as ContractCallExt<DefaultDB>>::construct_proof(&call, comm) {
                ProofPreimageVersioned::V2(value) => value,
                _ => return Err("unsupported JS counter proof version".into()),
            };
        if js_preimage != *constructed {
            return Err("JS official preimage differs from replayed call proof inputs".into());
        }
        check_transaction(root, "adjust", deploy, call, &mut rng, |state| {
            if state.data.get_ref() != &js_final {
                return Err("JS-derived proof applied unexpected ledger state".into());
            }
            Ok(())
        })?;
        println!(
            "JS-derived Counter branch {branch} proved, verified, and ledger-applied under shared unbalanced smoke policy"
        );
    }
    Ok(())
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    use serde_json::json;

    fn scalar() -> serde_json::Value {
        json!({"value": [[1]], "alignment": [{"tag":"atom", "value":{"tag":"bytes", "length":1}}]})
    }

    #[test]
    fn counter_scalar_and_finite_operations_use_official_deserializers() {
        assert!(aligned(&scalar()).is_ok());
        assert!(
            operations(&json!([{"addi":{"immediate":3}}, {"dup":{"n":0}},
            {"popeq":{"cached":true,"result":scalar()}}, "lt"]))
            .is_ok()
        );
    }

    #[test]
    fn unknown_fields_ops_and_oversized_atoms_are_refused() {
        let mut value = scalar();
        value["unexpected"] = true.into();
        assert!(aligned(&value).is_err());
        assert!(
            aligned(&json!({"value":[[256]],"alignment":[{"tag":"atom","value":{"tag":"field"}}]}))
                .is_err()
        );
        assert!(operations(&json!([{"addi":{"immediate":3,"extra":true}}])).is_err());
        assert!(operations(&json!(["ckpt"])).is_err());
        assert!(operations(&json!([])).is_err());
        assert!(operations(&json!([{"subi":{"immediate":65536}}])).is_err());
    }

    #[test]
    fn encoded_counter_state_accepts_only_exact_array_cell_shape() {
        let state = json!({"tag":"array","content":[
            {"tag":"cell","content":scalar()},
            {"tag":"cell","content":scalar()}
        ]});
        assert!(counter_state(&state).is_ok());
        let mut extra = state.clone();
        extra["extra"] = true.into();
        assert!(counter_state(&extra).is_err());
        let mut wrong = state.clone();
        wrong["content"][0]["tag"] = "null".into();
        assert!(counter_state(&wrong).is_err());
        let mut short = state;
        short["content"].as_array_mut().unwrap().pop();
        assert!(counter_state(&short).is_err());
    }
}
