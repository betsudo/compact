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
//! Current-profile functional DID deactivation through the existing strict path.
use super::*;
use did_contract::{ledger_contract as c, runtime as r, types};
use serde_json::{Value, json};
use std::cell::RefCell;

#[path = "../../../tests-rust-backend/did-functional-lifecycle-source/support/witness.rs"]
mod call_witness;
#[allow(dead_code)]
#[path = "../../../tests-rust-backend/did-functional-lifecycle-source/support/codec.rs"]
mod codec;
#[path = "../../../tests-rust-backend/did-functional-lifecycle-source/support/lifecycle_witness.rs"]
mod constructor_witness;

const IR: &str =
    include_str!("../../../tests-rust-backend/did-functional-lifecycle-source/original-ir.json");
const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "program-0.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-0.json"
        ),
    ),
    (
        "program-1.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-1.json"
        ),
    ),
    (
        "program-10.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-10.json"
        ),
    ),
    (
        "program-11.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-11.json"
        ),
    ),
    (
        "program-2.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-2.json"
        ),
    ),
    (
        "program-3.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-3.json"
        ),
    ),
    (
        "program-4.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-4.json"
        ),
    ),
    (
        "program-5.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-5.json"
        ),
    ),
    (
        "program-6.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-6.json"
        ),
    ),
    (
        "program-7.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-7.json"
        ),
    ),
    (
        "program-8.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-8.json"
        ),
    ),
    (
        "program-9.json",
        include_bytes!(
            "../../../tests-rust-backend/did-functional-lifecycle-source/checked-programs/program-9.json"
        ),
    ),
];

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    counter_scenario::validate_ir(
        &std::fs::read(root.join("contract/compact-rust-ir.json"))?,
        IR,
    )?;
    let directory = root.join("contract/checked-programs");
    let mut names: Vec<_> = std::fs::read_dir(&directory)?
        .map(|row| row.map(|row| row.file_name()))
        .collect::<Result<_, _>>()?;
    names.sort();
    if names
        != PROGRAMS
            .iter()
            .map(|(name, _)| std::ffi::OsString::from(name))
            .collect::<Vec<_>>()
    {
        return Err("DID complete checked-program membership differs".into());
    }
    for (name, bytes) in PROGRAMS {
        if std::fs::read(directory.join(name))? != *bytes {
            return Err("DID checked-program bytes differ".into());
        }
    }
    let result_path = root.join("did-functional-lifecycle-proof-result.json");
    if result_path.exists() {
        return Err("DID functional proof result already exists".into());
    }
    let capture: Value = serde_json::from_str(include_str!(
        "../../../tests-rust-backend/did-functional-lifecycle-source/data/unit-composition.json"
    ))?;
    let row = capture["cases"]
        .as_array()
        .ok_or("missing DID cases")?
        .iter()
        .find(|r| r["id"] == "did/valid")
        .ok_or("missing DID valid row")?;
    let changed_input = AlignedValue::from((
        signature(row)?,
        r::BoundedUint::<18446744073709551615>::new(1)?,
    ));
    let mut result =
        did_deactivate::run_with(root, row, prepare, check_final, Some(changed_input))?;
    result["format"] = json!("compact-did-functional-lifecycle-proof/v1");
    result["status"] = json!("passed");
    result["circuit"] = json!("deactivate");
    result["constructor_execution_proved"] = json!(false);
    result["all_twelve_exports_proved"] = json!(false);
    result["execution"] = json!("functional-recorded");
    result["changed_unit_result_control"] = json!("not applicable: Unit has zero output atoms");
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(result_path)?
        .write_all(&serde_json::to_vec_pretty(&result)?)?;
    println!("{result}");
    Ok(())
}

fn prepare(
    root: &Path,
    observed: &ObservedContractState,
    row: &Value,
) -> Result<(ContractCallPrototype<DefaultDB>, StateValue<DefaultDB>), Box<dyn Error>> {
    let constructor = constructor_witness::Witness::new(json!({"timestamp":"100"}));
    let initial = c::initial_state(ConstructorContext::new(4_u64), &constructor)?;
    if initial.private_state != 7
        || initial.ledger_state.get_ref() != observed.contract().data.get_ref()
    {
        return Err("DID proof prior state differs from real constructor execution".into());
    }
    if json!(*constructor.calls.borrow())
        != json!([
            {"name":"controller"},{"name":"recovery"},{"name":"timestamp"}
        ])
    {
        return Err("DID constructor witness order differs".into());
    }
    let signature = signature(row)?;
    let version = r::BoundedUint::<18446744073709551615>::new(0)?;
    let witnesses: Vec<_> = (0..4)
        .map(|_| call_witness::Witness {
            options: row["options"].clone(),
            calls: RefCell::default(),
        })
        .collect();
    let native = c::deactivate(
        observed.circuit_context(7_u64),
        &witnesses[0],
        signature.clone(),
        version,
    )?;
    let ordinary = c::recorded::deactivate(
        observed.circuit_context(7_u64),
        &witnesses[1],
        signature.clone(),
        version,
    )?;
    let functional_native = c::functional::deactivate::Program::new(signature.clone(), version)
        .execute_native(observed.circuit_context(7_u64), &witnesses[2])?;
    let functional = c::functional::deactivate::Program::new(signature.clone(), version)
        .execute_recorded(observed.circuit_context(7_u64), &witnesses[3])?;
    for actual in [
        &ordinary.execution,
        &functional_native,
        &functional.execution,
    ] {
        if actual.context.query.state != native.context.query.state
            || actual.context.query.effects != native.context.query.effects
            || actual.context.private_state != native.context.private_state
            || actual.gas_cost != native.gas_cost
            || actual.private_transcript_outputs != native.private_transcript_outputs
        {
            return Err("DID four-route state/effects/private/gas mismatch".into());
        }
    }
    if ordinary.public.verify_ops() != functional.public.verify_ops()
        || witnesses
            .iter()
            .any(|w| json!(*w.calls.borrow()) != row["witnessCalls"])
        || native.context.private_state
            != row["privateAfter"]
                .as_u64()
                .ok_or("missing private state")?
    {
        return Err("DID recording or witness order differs".into());
    }
    let expected = native.context.query.state.get_ref().clone();
    let stale = c::functional::deactivate::Program::new(signature.clone(), version)
        .execute_recorded(native.context, &witnesses[0]);
    if !matches!(stale, Err(r::CompactError::AssertionFailed(ref message))
        if message == "Controller authorization version is stale")
    {
        return Err("DID functional stale version did not refuse first".into());
    }
    let prepared =
        check_generated_trace(root, "deactivate", functional, (signature.clone(), version))?;
    let verifier: VerifierKey = tagged_deserialize(&mut BufReader::new(File::open(
        root.join("keys/deactivate.verifier"),
    )?))?;
    let facade = c::Contract::from(call_witness::Witness {
        options: row["options"].clone(),
        calls: RefCell::default(),
    });
    let ordinary = facade
        .recording()
        .deactivate_call(observed, 7_u64, signature, version)?
        .prepare(verifier, Fr::from(0_u64))?;
    if format!("{prepared:?}") != format!("{ordinary:?}") {
        return Err("DID functional/ordinary prepared statements differ".into());
    }
    Ok((prepared, expected))
}

fn signature(row: &Value) -> Result<types::SchnorrSignature, Box<dyn Error>> {
    Ok(types::SchnorrSignature {
        announcement: r::ec_mul_generator(Field::from(2_u64))?,
        response: Field::from_le_bytes(&hex::decode(
            row["responseHex"].as_str().ok_or("missing signature")?,
        )?)
        .ok_or("invalid DID response")?,
    })
}

fn check_final(
    address: midnight_compact_runtime::ledger::ContractAddress,
    actual: &ContractState<DefaultDB>,
    row: &Value,
) -> Result<String, Box<dyn Error>> {
    if did_contract::ledger_slots::active.inspect(actual.data.get_ref())?
        || !did_contract::ledger_slots::deactivated.inspect(actual.data.get_ref())?
    {
        return Err("functional DID final flags differ".into());
    }
    let witness = call_witness::Witness {
        options: row["options"].clone(),
        calls: RefCell::default(),
    };
    let second = c::functional::deactivate::Program::new(signature(row)?, r::BoundedUint::new(0)?)
        .execute_recorded(
            r::context::CircuitContext::from_contract_state(9_u64, address, actual),
            &witness,
        );
    match second {
        Err(r::CompactError::AssertionFailed(message))
            if message == "Controller authorization version is stale"
                && witness.calls.borrow().is_empty() =>
        {
            Ok(message)
        }
        _ => Err(
            "functional DID applied-state stale version did not refuse first without witnesses"
                .into(),
        ),
    }
}
