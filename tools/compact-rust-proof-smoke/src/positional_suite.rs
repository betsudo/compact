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

//! Explicit execution order for the default positional proof suite.
//!
//! Parsing has succeeded before this owner reads handoff settings and seeds its
//! one RNG. Keep the calls in execution order, which differs from input order.

use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

use midnight_base_crypto::hash::persistent_hash;
use midnight_zkir_v3::IrSource as IrSourceV3;
use rand::{SeedableRng, rngs::StdRng};

use super::local_v3_application as v3;
use super::positional_inputs::PositionalInputs;
use super::{
    ContractOperation, DefaultDB, HandoffConfig, ProofMarker, PureGeneratorPedersen, Signature,
    Timestamp, Transaction, VerifierKey, positional_cells, positional_counters, positional_keys,
    positional_lists, positional_maps, positional_merkle, positional_sets, positional_unsigned,
    positional_witnesses, tagged_deserialize, write_sealed_handoff,
};

pub(super) fn baseline_operation(
    root: &Path,
    name: &str,
    rows: usize,
) -> Result<(IrSourceV3, VerifierKey, ContractOperation), Box<dyn Error>> {
    let pins: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> =
        serde_json::from_str(include_str!(
            "../../compact-rust-backend/tests/positional-baseline-ledger9-artifacts.json"
        ))?;
    for (path, expected) in pins.get(name).ok_or("missing positional artifact pins")? {
        if hex::encode(persistent_hash(&fs::read(root.join(path))?).0) != *expected {
            return Err(format!("{name} source/proof artifact differs: {path}").into());
        }
    }
    let (ir, verifier, operation) = v3::operation(root, name, 6)?;
    if ir.model().rows() != rows {
        return Err(format!("{name} v3 row count differs").into());
    }
    Ok((ir, verifier, operation))
}

pub(super) fn verify_baseline(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<(), Box<dyn Error>> {
    const USAGE: &str =
        "usage: compact-rust-proof-smoke --positional-baseline-verify <counter> <cell> <cell-read>";
    let counter = arguments.next().ok_or(USAGE)?;
    let cell = arguments.next().ok_or(USAGE)?;
    let cell_read = arguments.next().ok_or(USAGE)?;
    if arguments.next().is_some() {
        return Err(USAGE.into());
    }
    for (path, name, rows, stems) in [
        (
            counter,
            "increment",
            31,
            &["increment", "increment-direct"][..],
        ),
        (cell, "set_flag", 35, &["set_flag"][..]),
        (cell_read, "read_flag", 34, &["read_flag"][..]),
    ] {
        let root = Path::new(&path);
        let (_, verifier, _) = baseline_operation(root, name, rows)?;
        for stem in stems {
            v3::verify_standalone(root, stem, &verifier)?;
            println!("{stem}: separate v3 verification passed and corrupted proof was refused");
        }
    }
    Ok(())
}

fn export_counter_handoff(root: &Path) -> Result<(), Box<dyn Error>> {
    for (variable, artifact, label) in [
        (
            "COMPACT_RUST_WALLET_HANDOFF",
            "increment-transaction.bin",
            "call",
        ),
        (
            "COMPACT_RUST_DEPLOY_HANDOFF",
            "increment-deployment.bin",
            "deployment",
        ),
    ] {
        if let Some(path) = std::env::var_os(variable) {
            let bytes = fs::read(root.join(artifact))?;
            let mut reader = bytes.as_slice();
            let tx: Transaction<Signature, ProofMarker, PureGeneratorPedersen, DefaultDB> =
                tagged_deserialize(&mut reader)?;
            if !reader.is_empty() {
                return Err(format!("sealed {label} has trailing bytes").into());
            }
            let mut roundtrip = Vec::new();
            super::tagged_serialize(&tx, &mut roundtrip)?;
            if roundtrip != bytes {
                return Err(format!("sealed {label} bytes differ before handoff").into());
            }
            write_sealed_handoff(Path::new(&path), &tx, label)?;
        }
    }
    Ok(())
}

pub(super) fn run(inputs: PositionalInputs) -> Result<(), Box<dyn Error>> {
    let PositionalInputs {
        counter_root,
        cell_root,
        cell_read_root,
        witness_root,
        nested_root,
        expression_root,
        set_root,
        set_oracle_root,
        map_root,
        constructor_map_root,
        list_root,
        constructor_list_root,
        enum_cell_root,
        tiny_root,
        nested_map_shape_root,
        list_shapes_root,
        merkle_root,
        historic_merkle_root,
        vector_key_root,
        counter_parameter_root,
        composite_key_root,
        chunked_set_root,
        chunked_list_root,
        chunked_map_root,
        chunked_cell_root,
        uints_root,
        wide_root,
    } = inputs;
    let counter_root = Path::new(&counter_root);
    let cell_root = Path::new(&cell_root);
    let cell_read_root = Path::new(&cell_read_root);
    let handoff_config = HandoffConfig::from_env()?;
    if handoff_config.network_id != "local-test"
        || handoff_config.validation_time != Timestamp::from_secs(0)
    {
        return Err("funded positional v3 proof uses the offline Ledger 9 fixture".into());
    }
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("positional v3 proof requires local proving".into());
    }
    let mut rng = StdRng::seed_from_u64(0x434f4d50414354);
    positional_counters::check_binding(counter_root, cell_root, &mut rng)?;
    positional_cells::check_write_and_read(cell_root, cell_read_root, &mut rng)?;
    let verification = Command::new(std::env::current_exe()?)
        .arg("--positional-baseline-verify")
        .args([counter_root, cell_root, cell_read_root])
        .status()?;
    if !verification.success() {
        return Err(
            format!("separate positional baseline verification failed: {verification}").into(),
        );
    }
    export_counter_handoff(counter_root)?;

    if let Some(root) = counter_parameter_root.as_ref() {
        let root = Path::new(root);
        positional_counters::check_parameterized(root, &mut rng)?;
    }

    if let Some(witness_root) = witness_root {
        let witness_root = Path::new(&witness_root);
        positional_witnesses::check_cell_witnesses(witness_root, &mut rng)?;
    }
    if let Some(nested_root) = nested_root {
        let nested_root = Path::new(&nested_root);
        positional_counters::check_nested_calls(nested_root, &mut rng)?;
    }
    if let Some(expression_root) = expression_root {
        let expression_root = Path::new(&expression_root);
        positional_witnesses::check_nested_expressions(expression_root, &mut rng)?;
    }
    if let Some(set_root) = set_root {
        let set_root = Path::new(&set_root);
        positional_sets::check_operations(set_root, &mut rng)?;
    }
    if let Some(set_oracle_root) = set_oracle_root {
        let set_oracle_root = Path::new(&set_oracle_root);
        positional_sets::check_membership_oracle(set_oracle_root, &mut rng)?;
    }
    if let Some(map_root) = map_root {
        let map_root = Path::new(&map_root);
        positional_maps::check_operations(map_root, &mut rng)?;
    }
    if let Some(constructor_map_root) = constructor_map_root {
        let constructor_map_root = Path::new(&constructor_map_root);
        positional_maps::check_constructor_actions(constructor_map_root, &mut rng)?;
    }
    if let Some(list_root) = list_root {
        let list_root = Path::new(&list_root);
        positional_lists::check_operations(list_root, &mut rng)?;
    }
    if let Some(constructor_list_root) = constructor_list_root {
        let constructor_list_root = Path::new(&constructor_list_root);
        positional_lists::check_constructor_actions(constructor_list_root, &mut rng)?;
    }
    if let Some(list_shapes_root) = list_shapes_root.as_ref().map(Path::new) {
        positional_lists::check_witness_shapes(list_shapes_root, &mut rng)?;
    }
    if let Some(enum_cell_root) = enum_cell_root.as_ref().map(Path::new) {
        positional_cells::check_enum_value(enum_cell_root, &mut rng)?;
    }
    if let Some(tiny_root) = tiny_root.as_ref().map(Path::new) {
        positional_cells::check_optional_value(tiny_root, &mut rng)?;
    }
    if let Some(shape_root) = nested_map_shape_root.as_ref().map(Path::new) {
        positional_maps::check_nested_shape(shape_root, &mut rng)?;
    }
    if let Some(merkle_root) = merkle_root.as_ref().map(Path::new) {
        positional_merkle::check_plain(merkle_root, &mut rng)?;
    }
    if let Some(historic_root) = historic_merkle_root.as_ref().map(Path::new) {
        positional_merkle::check_historic(historic_root, &mut rng)?;
    }
    if let Some(vector_root) = vector_key_root.as_ref().map(Path::new) {
        positional_keys::check_vector(vector_root, &mut rng)?;
    }
    if let Some(composite_root) = composite_key_root.as_ref().map(Path::new) {
        positional_keys::check_composite(composite_root, &mut rng)?;
    }
    if let Some(chunked_root) = chunked_set_root.as_ref().map(Path::new) {
        positional_sets::check_chunked_layout(chunked_root, &mut rng)?;
    }
    if let Some(chunked_root) = chunked_list_root.as_ref().map(Path::new) {
        positional_lists::check_chunked_layout(chunked_root, &mut rng)?;
    }
    if let Some(chunked_root) = chunked_map_root.as_ref().map(Path::new) {
        positional_maps::check_chunked_layout(chunked_root, &mut rng)?;
    }
    if let Some(chunked_root) = chunked_cell_root.as_ref().map(Path::new) {
        positional_cells::check_chunked_layout(chunked_root, &mut rng)?;
    }
    if let Some(root) = uints_root.as_ref().map(Path::new) {
        positional_unsigned::check_bounded(root, &mut rng)?;
    }
    if let Some(root) = wide_root.as_ref().map(Path::new) {
        positional_unsigned::check_wide(root, &mut rng)?;
    }
    Ok(())
}
