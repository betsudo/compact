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

//! Ledger 9 state-only proving for positional and named scenarios.

use std::ffi::OsString;
use std::path::Component;
use std::process::Command;

use rand::RngCore;

use super::local_v3_application as v3;
use super::*;

pub(super) fn make_deploy(
    root: &Path,
    name: &'static str,
    data: StateValue<DefaultDB>,
    rng: &mut StdRng,
) -> Result<ContractDeploy<DefaultDB>, Box<dyn Error>> {
    let (_, _, operation) = v3::load_operation(root, name)?;
    Ok(v3::deploy(data, name, operation, rng))
}

pub(super) fn check_generated_trace<
    Private,
    Output: Into<AlignedValue>,
    Input: Into<AlignedValue>,
>(
    root: &Path,
    name: &'static str,
    recorded: RecordedCircuitResult<Private, Output>,
    input: Input,
) -> Result<ContractCallPrototype<DefaultDB>, Box<dyn Error>> {
    let (_, verifier, operation) = v3::load_operation(root, name)?;
    let mut prepared = prepare_call(
        recorded,
        CallSpec::new(name, verifier, input, Fr::from(0_u64)),
    )?;
    prepared.op = operation;
    println!("generated {name} trace replayed and partitioned");
    Ok(prepared)
}

pub(super) fn check_transaction(
    root: &Path,
    name: &'static str,
    deploy: ContractDeploy<DefaultDB>,
    prepared: ContractCallPrototype<DefaultDB>,
    rng: &mut StdRng,
    check_state: impl FnOnce(&ContractState<DefaultDB>) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    // Some cases exercise the same circuit against multiple initial states.
    let stem = format!("positional-{name}-{:016x}", rng.next_u64());
    apply(root, name, &stem, deploy, prepared, check_state)?;
    Ok(())
}

pub(super) fn check_observed_call_parity(
    root: &Path,
    name: &'static str,
    deploy: &ContractDeploy<DefaultDB>,
    manual: &ContractCallPrototype<DefaultDB>,
    observed: &ContractCallPrototype<DefaultDB>,
) -> Result<(), Box<dyn Error>> {
    let preproof_bytes =
        |prepared: ContractCallPrototype<DefaultDB>| -> Result<Vec<u8>, Box<dyn Error>> {
            let mut rng = StdRng::seed_from_u64(0x0041_4452_3433);
            let intent: Intent<Signature, ProofPreimageMarker, PedersenRandomness, DefaultDB> =
                Intent::empty(&mut rng, Timestamp::from_secs(0))
                    .add_call::<ProofPreimage>(prepared);
            let tx = Transaction::from_intents("local-test", HashMap::new().insert(1_u16, intent));
            let mut bytes = Vec::new();
            tagged_serialize(&tx, &mut bytes)?;
            Ok(bytes)
        };
    let observed_bytes = preproof_bytes(observed.clone())?;
    if format!("{manual:?}") != format!("{observed:?}")
        || preproof_bytes(manual.clone())? != observed_bytes
    {
        return Err(format!("{name} observed and direct pre-proof calls differ").into());
    }
    let address = hex::encode(deploy.address().0.0);
    let direct = apply(
        root,
        name,
        &format!("positional-{name}-{address}-direct"),
        deploy.clone(),
        manual.clone(),
        |_| Ok(()),
    )?;
    let observed = apply(
        root,
        name,
        &format!("positional-{name}-{address}-observed"),
        deploy.clone(),
        observed.clone(),
        |_| Ok(()),
    )?;
    if direct != observed {
        return Err(format!("{name} independently applied contract states differ").into());
    }
    println!(
        "{name} observed call matches manual prototype, {} pre-proof bytes and independently applied state",
        observed_bytes.len()
    );
    Ok(())
}

fn apply(
    root: &Path,
    name: &'static str,
    stem: &str,
    deploy: ContractDeploy<DefaultDB>,
    prepared: ContractCallPrototype<DefaultDB>,
    check_state: impl FnOnce(&ContractState<DefaultDB>) -> Result<(), Box<dyn Error>>,
) -> Result<ContractState<DefaultDB>, Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err(
            "Ledger 9 state-call proof requires local proving without MIDNIGHT_PROOF_SERVER".into(),
        );
    }
    let (_, verifier, operation) = v3::load_operation(root, name)?;
    if prepared.op != operation {
        return Err(format!("{name} prepared operation differs from the proof artifact").into());
    }
    let proof = v3::prove_standalone(root, name, stem, &prepared, &verifier, None)?;
    let verification = Command::new(std::env::current_exe()?)
        .arg("--positional-call-verify")
        .arg(root)
        .args([name, stem])
        .status()?;
    if !verification.success() {
        return Err(format!("{name} separate v3 verification failed: {verification}").into());
    }
    v3::apply_checked(root, name, stem, prepared, deploy, &proof, check_state)
}

pub(super) fn verify(arguments: &mut impl Iterator<Item = OsString>) -> Result<(), Box<dyn Error>> {
    const USAGE: &str =
        "usage: compact-rust-proof-smoke --positional-call-verify <output> <circuit> <proof-stem>";
    let root = arguments.next().ok_or(USAGE)?;
    let name = arguments.next().ok_or(USAGE)?;
    let stem = arguments.next().ok_or(USAGE)?;
    if arguments.next().is_some()
        || [&name, &stem].into_iter().any(|value| {
            let mut components = Path::new(value).components();
            !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some()
        })
    {
        return Err(USAGE.into());
    }
    let name = name.to_str().ok_or(USAGE)?;
    let stem = stem.to_str().ok_or(USAGE)?;
    let root = Path::new(&root);
    let (_, verifier, _) = v3::load_operation(root, name)?;
    v3::verify_standalone(root, stem, &verifier)?;
    println!("{stem}: separate v3 verification passed and corrupted proof was refused");
    Ok(())
}
