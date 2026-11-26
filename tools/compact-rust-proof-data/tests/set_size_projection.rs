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

// Independent Set.size synthetic controls; no SDK model/proof measurements.
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
fn encoded(values: &[Fr]) -> Vec<String> {
    values.iter().map(input::hex).collect()
}
fn unread() -> FileRef {
    FileRef {
        path: "PRIVATE-NEVER-OPEN".into(),
        bytes: 0,
        sha256: "0".repeat(64),
    }
}
fn fixture(value: Fr) -> (Case, Stream) {
    let mut operands = [48, 80, 1, 1, 0, 4, 13, 1, 8].map(f).to_vec();
    operands.push(value);
    (
        Case {
            id: "synthetic-size".into(),
            supplemental: true,
            input: vec![],
            output: encoded(&[value]),
            preimage: unread(),
            public_stream: unread(),
        },
        Stream {
            schema: "compact-rust-proof-data-public/v1".into(),
            inputs: encoded(&operands),
            outputs: encoded(&[value]),
        },
    )
}
fn project(case: &Case, stream: &Stream) -> anyhow::Result<statement::Statement> {
    statement::derive(
        &serde_json::to_vec(&statement::profile_ir(Profile::SetSize)?)?,
        Profile::SetSize,
        case,
        stream,
        f(7),
        f(11),
    )
}
#[test]
fn uint64_boundaries_have_exact_value_only_communication_and_operand_order() {
    for n in [0, 1, 2, u64::MAX] {
        let value = f(n);
        let (case, stream) = fixture(value);
        let actual = project(&case, &stream).unwrap();
        let mut expected = vec![f(7), transient_commit(&[value], f(11))];
        expected.extend([48, 80, 1, 1, 0, 4, 13, 1, 8].into_iter().map(f));
        expected.push(value);
        assert_eq!(actual.fields, expected);
        assert_eq!(actual.skips, vec![None; 4]);
    }
}
#[test]
fn only_unit_and_single_canonical_uint64_output_are_admitted() {
    let (mut case, stream) = fixture(f(1));
    case.input = encoded(&[f(0)]);
    assert!(project(&case, &stream).is_err());
    for output in [
        vec![],
        encoded(&[f(0), f(1)]),
        vec!["01".into()],
        vec!["f".repeat(64)],
    ] {
        let (mut case, stream) = fixture(f(1));
        case.output = output;
        assert!(project(&case, &stream).is_err());
    }
    // All public occurrences agree: only the explicit Uint64 range gate refuses 2^64.
    let (case, stream) = fixture(f(u64::MAX) + f(1));
    assert!(project(&case, &stream).is_err());
}
#[test]
fn every_public_operand_is_required_in_exact_order() {
    let (case, stream) = fixture(f(2));
    for i in 0..stream.inputs.len() {
        let mut missing = stream.clone();
        missing.inputs.remove(i);
        assert!(project(&case, &missing).is_err());
        let mut changed = stream.clone();
        changed.inputs[i] = input::hex(&(input::scalar(&changed.inputs[i]).unwrap() + f(1)));
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
fn actual_read_and_typed_result_must_be_single_and_identical() {
    let (case, stream) = fixture(f(2));
    for outputs in [vec![], encoded(&[f(2), f(0)]), encoded(&[f(1)])] {
        let mut wrong = stream.clone();
        wrong.outputs = outputs;
        assert!(project(&case, &wrong).is_err());
    }
    let (mut case, stream) = fixture(f(2));
    case.output = encoded(&[f(1)]);
    assert!(project(&case, &stream).is_err());
    let (case, mut stream) = fixture(f(2));
    stream.schema = "unadmitted".into();
    assert!(project(&case, &stream).is_err());
}
#[test]
fn complete_raw_ir_gate_refuses_profile_version_shape_guard_and_width_drift() {
    let (case, stream) = fixture(f(2));
    for (pointer, value) in [
        ("/version/minor", json!(1)),
        ("/do_communications_commitment", json!(false)),
        (
            "/inputs",
            json!([{"name":"%ghost","type":"Scalar<BLS12-381>"}]),
        ),
        ("/outputs/0", json!("unadmitted")),
        ("/instructions/0/inputs/0", json!("0x31")),
        ("/instructions/2/guard", json!("0x00")),
        ("/instructions/3/output", json!("%other")),
        ("/instructions/4/inputs/2", json!("0x01")),
        ("/instructions/4/inputs/3", json!("%other")),
        ("/instructions/5/vals/0", json!("%other")),
    ] {
        let mut ir = statement::profile_ir(Profile::SetSize).unwrap();
        *ir.pointer_mut(pointer).unwrap() = value;
        assert!(
            statement::derive(
                &serde_json::to_vec(&ir).unwrap(),
                Profile::SetSize,
                &case,
                &stream,
                f(7),
                f(11)
            )
            .is_err()
        );
    }
    // Released3.1 hoists the read; it is reference DATA, not the admitted current3.0 profile.
    let mut other = statement::profile_ir(Profile::SetSize).unwrap();
    other["version"]["minor"] = json!(1);
    let instructions = other["instructions"].as_array_mut().unwrap();
    let read = instructions.remove(3);
    instructions.insert(0, read);
    assert!(
        statement::derive(
            &serde_json::to_vec(&other).unwrap(),
            Profile::SetSize,
            &case,
            &stream,
            f(7),
            f(11)
        )
        .is_err()
    );
}
#[test]
fn caller_binding_and_randomness_remain_explicit_nonzero_and_distinct() {
    let (case, stream) = fixture(f(2));
    let raw = serde_json::to_vec(&statement::profile_ir(Profile::SetSize).unwrap()).unwrap();
    for (binding, opening) in [(f(0), f(11)), (f(7), f(0)), (f(7), f(7))] {
        assert!(
            statement::derive(&raw, Profile::SetSize, &case, &stream, binding, opening).is_err()
        );
    }
    // Genuine changed-binding crypto refusal is still a later SDK verify_call, not this admission test.
}

#[test]
fn set_size_label_requires_separate_set_admission_despite_equal_raw_program() {
    assert_eq!(
        serde_json::to_value(Profile::SetSize).unwrap(),
        json!("set-size")
    );
    assert_ne!(
        serde_json::to_value(Profile::SetSize).unwrap(),
        serde_json::to_value(Profile::UnitUint64).unwrap()
    );
    assert_eq!(
        statement::profile_ir(Profile::SetSize).unwrap(),
        statement::profile_ir(Profile::UnitUint64).unwrap()
    );
    // Root must bind the typed Set<Field> fixture and fresh member/size helper APIs;
    // this byte equality transfers no Map source/keys/model/proof qualification.
}
