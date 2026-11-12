// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Source-bound ordinary snapshot proof with a separately applied deployment.
use super::*;
use crate::local_v3_application as v3;
use compact_rust_literal_bytes_field_oracle_fixture::ledger_slots as slots;
use midnight_compact_runtime::{
    CompactError, recording::RecordingFrame, transaction::RecordedCall,
};
const RETAINED_IR: &str =
    include_str!("../../../compact-rust-backend/tests/literal-bytes-field-schema20-ir.json");
const CIRCUIT: &str = "snapshot";

pub(crate) fn run(root: &Path, verify: bool) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        RETAINED_IR,
    )?;
    let (ir, verifier, operation) = v3::operation(root, CIRCUIT, 6)?;
    if ir.model().rows() != 47 {
        return Err("literal Bytes-to-Field v3 row count differs".into());
    }
    if verify {
        v3::verify_standalone(root, CIRCUIT, &verifier)?;
        println!("{CIRCUIT}: separate v3 verification passed and corrupted proof was refused");
        return Ok(());
    }
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("literal Bytes-to-Field proof requires local proving".into());
    }
    let mut rng = StdRng::seed_from_u64(0x0606_534e_4150);
    let expected = Fr::from(16961_u64);
    let initial = contract::initial_state(ConstructorContext::new(7_u64))?;
    let initial_state = initial.ledger_state.get_ref().clone();
    let deploy = v3::deploy(initial_state.clone(), CIRCUIT, operation, &mut rng);
    let observed = ObservedContractState::new(
        deploy.address(),
        deploy.initial_state.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 0,
        },
    );
    let native = contract::snapshot(observed.circuit_context(7_u64))?;
    let recorded = contract::recorded::snapshot(observed.circuit_context(7_u64))?;
    let generated = contract::Contract::default();
    let call = generated.recording().snapshot_call(&observed, 7_u64)?;
    let (frame, first) =
        slots::value.record_read(RecordingFrame::new(observed.circuit_context(7_u64)))?;
    if first != expected {
        return Err(CompactError::AssertionFailed("constructor cast".into()).into());
    }
    let (frame, result) = slots::value.record_read(frame)?;
    let manual = frame.finish(result);
    if native.result != expected || native.context.query.state.get_ref() != &initial_state {
        return Err("snapshot native value or unchanged constructor state differs".into());
    }
    let manual_replay = manual.public.initial().query(
        manual.public.verify_ops(),
        None,
        &native.context.cost_model,
    )?;
    for actual in [&recorded, call.recorded(), &manual] {
        if actual.execution.result != expected
            || actual.execution.context.private_state != 7
            || !actual.execution.private_transcript_outputs.is_empty()
            || actual.execution.context.query.state != native.context.query.state
            || actual.execution.context.query.effects != native.context.query.effects
            || actual.execution.gas_cost != native.gas_cost
            || actual.public.verify_ops() != manual.public.verify_ops()
        {
            return Err("ordinary snapshot execution/recording/observation/manual mismatch".into());
        }
        let replay = actual.public.initial().query(
            actual.public.verify_ops(),
            None,
            &native.context.cost_model,
        )?;
        if replay.context.state != native.context.query.state
            || replay.context.effects != native.context.query.effects
            || replay.gas_cost != manual_replay.gas_cost
        {
            return Err("ordinary snapshot replay mismatch".into());
        }
    }
    if native.context.private_state != 7 || !native.private_transcript_outputs.is_empty() {
        return Err("snapshot native private state differs".into());
    }
    let randomness = Fr::from(0_u64);
    let direct = RecordedCall::new(&observed, recorded, CIRCUIT, ())
        .prepare(verifier.clone(), randomness)?;
    let prepared = call.prepare(verifier.clone(), randomness)?;
    if prepared.input != AlignedValue::from(())
        || prepared.output != AlignedValue::from(expected)
        || format!("{direct:?}") != format!("{prepared:?}")
    {
        return Err("snapshot preparation binding differs".into());
    }
    let proof = v3::prove_standalone(root, CIRCUIT, CIRCUIT, &prepared, &verifier, None)?;
    v3::apply_after_deployment(
        root,
        CIRCUIT,
        CIRCUIT,
        prepared,
        deploy,
        &initial_state,
        &proof,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_complete_ir_refuses_before_keys() {
        counter_scenario::validate_ir(RETAINED_IR.as_bytes(), RETAINED_IR).unwrap();
        let mut changed: serde_json::Value = serde_json::from_str(RETAINED_IR).unwrap();
        changed["independent_snapshot_control"] = true.into();
        assert!(
            counter_scenario::validate_ir(&serde_json::to_vec(&changed).unwrap(), RETAINED_IR)
                .is_err()
        );
    }
    #[test]
    fn absent_source_refuses_before_constructor_or_keys() {
        let root = std::env::temp_dir().join(format!(
            "compact-snapshot-absent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!root.exists());
        assert_eq!(
            run(&root, false)
                .unwrap_err()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert!(!root.exists());
    }
}
