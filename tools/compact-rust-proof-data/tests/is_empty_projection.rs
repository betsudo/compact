// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Synthetic public projection tests, not SDK model/proof measurements.
use compact_rust_proof_data::{
    codec::FileRef,
    input::{self, Case, Profile, Stream},
    verifier::statement,
};
use midnight_transient_crypto::{curve::Fr, hash::transient_commit};
use serde_json::json;

fn f(n: u64) -> Fr {
    Fr::from(n)
}
fn encoded(values: &[u64]) -> Vec<String> {
    values.iter().map(|n| input::hex(&f(*n))).collect()
}
fn unread() -> FileRef {
    FileRef {
        path: "PRIVATE-NEVER-OPEN".into(),
        bytes: 0,
        sha256: "0".repeat(64),
    }
}
fn fixture(value: u64) -> (Case, Stream) {
    (
        Case {
            id: "synthetic-is-empty".into(),
            supplemental: true,
            input: vec![],
            output: encoded(&[value]),
            preimage: unread(),
            public_stream: unread(),
        },
        Stream {
            schema: "compact-rust-proof-data-public/v1".into(),
            inputs: encoded(&[48, 80, 1, 1, 0, 4, 16, 1, 1, 8, 0, 2, 13, 1, 1, value]),
            outputs: encoded(&[value]),
        },
    )
}
fn project(case: &Case, stream: &Stream) -> anyhow::Result<statement::Statement> {
    statement::derive(
        &serde_json::to_vec(&statement::profile_ir(Profile::MapIsEmpty)?)?,
        Profile::MapIsEmpty,
        case,
        stream,
        f(7),
        f(11),
    )
}

#[test]
fn both_boolean_values_project_complete_unit_communications_and_public_operands() {
    for value in [0, 1] {
        let (case, stream) = fixture(value);
        let actual = project(&case, &stream).unwrap();
        let mut expected = vec![f(7), transient_commit(&[f(value)], f(11))];
        expected.extend(
            [48, 80, 1, 1, 0, 4, 16, 1, 1, 8, 0, 2, 13, 1, 1, value]
                .into_iter()
                .map(f),
        );
        assert_eq!(actual.fields, expected);
        assert_eq!(actual.skips, vec![None; 6]);
    }
}

#[test]
fn non_unit_input_and_non_boolean_or_noncanonical_output_are_refused() {
    let (mut case, stream) = fixture(1);
    case.input = encoded(&[0]);
    assert!(project(&case, &stream).is_err());
    for output in [
        vec![],
        encoded(&[0, 1]),
        vec!["01".into()],
        vec!["f".repeat(64)],
    ] {
        let (mut case, stream) = fixture(1);
        case.output = output;
        assert!(project(&case, &stream).is_err());
    }
    // Every public occurrence agrees, so only the Boolean domain guard can admit/refuse 2.
    let (case, stream) = fixture(2);
    assert!(project(&case, &stream).is_err());
}

#[test]
fn every_public_operand_is_required_and_checked_in_order() {
    let (case, stream) = fixture(1);
    for position in 0..stream.inputs.len() {
        let mut missing = stream.clone();
        missing.inputs.remove(position);
        assert!(project(&case, &missing).is_err());
        let mut changed = stream.clone();
        let value = input::scalar(&changed.inputs[position]).unwrap() + f(1);
        changed.inputs[position] = input::hex(&value);
        assert!(project(&case, &changed).is_err());
    }
    let mut extra = stream.clone();
    extra.inputs.push(input::hex(&f(0)));
    assert!(project(&case, &extra).is_err());
    let mut reordered = stream;
    reordered.inputs.swap(0, 1);
    assert!(project(&case, &reordered).is_err());
}

#[test]
fn missing_extra_or_mismatched_read_and_typed_return_are_refused() {
    let (case, stream) = fixture(1);
    let mut missing = stream.clone();
    missing.outputs.clear();
    assert!(project(&case, &missing).is_err());
    let mut extra = stream.clone();
    extra.outputs.push(input::hex(&f(0)));
    assert!(project(&case, &extra).is_err());
    let mut changed = stream.clone();
    changed.outputs = encoded(&[0]);
    assert!(project(&case, &changed).is_err());
    let (mut wrong_return, stream) = fixture(1);
    wrong_return.output = encoded(&[0]);
    assert!(project(&wrong_return, &stream).is_err());
    let mut wrong_schema = stream;
    wrong_schema.schema = "unadmitted".into();
    assert!(project(&case, &wrong_schema).is_err());
}

#[test]
fn the_entire_profile_guard_prevents_ir_or_shape_expansion() {
    let (case, stream) = fixture(1);
    for (pointer, value) in [
        ("/do_communications_commitment", json!(false)),
        (
            "/inputs",
            json!([{"name":"%ghost","type":"Scalar<BLS12-381>"}]),
        ),
        ("/outputs/0", json!("unadmitted")),
        ("/instructions/0/inputs/0", json!("0x31")),
        ("/instructions/2/guard", json!("0x00")),
        ("/instructions/3/inputs/3", json!("0x09")),
        ("/instructions/5/output", json!("%foreign")),
        ("/instructions/6/inputs/3", json!("%foreign")),
        ("/instructions/7/vals/0", json!("%foreign")),
    ] {
        let mut ir = statement::profile_ir(Profile::MapIsEmpty).unwrap();
        *ir.pointer_mut(pointer).unwrap() = value;
        assert!(
            statement::derive(
                &serde_json::to_vec(&ir).unwrap(),
                Profile::MapIsEmpty,
                &case,
                &stream,
                f(7),
                f(11),
            )
            .is_err()
        );
    }
}

#[test]
fn caller_fields_stay_explicit_nonzero_and_distinct() {
    let (case, stream) = fixture(1);
    let raw = serde_json::to_vec(&statement::profile_ir(Profile::MapIsEmpty).unwrap()).unwrap();
    for (binding, opening) in [(f(0), f(11)), (f(7), f(0)), (f(7), f(7))] {
        assert!(
            statement::derive(&raw, Profile::MapIsEmpty, &case, &stream, binding, opening).is_err()
        );
    }
    // A changed valid binding remains an actual SDK-negative test after proof creation;
    // projection admission is not substituted for the existing verify_call boundary.
}
