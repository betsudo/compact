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

//! Synthetic independent Set.member controls; no model/proof outcome claims.
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
fn wide() -> Fr {
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    bytes[16] = 1;
    Fr::from_le_bytes(&bytes).unwrap()
}
fn fixture(value: Fr, result: Fr) -> (Case, Stream) {
    let operands = vec![
        f(48),
        f(80),
        f(1),
        f(1),
        f(0),
        f(16),
        f(1),
        f(1),
        -f(2),
        value,
        f(24),
        f(13),
        f(1),
        f(1),
        result,
    ];
    (
        Case {
            id: "synthetic-set-member".into(),
            supplemental: true,
            input: encoded(&[value]),
            output: encoded(&[result]),
            preimage: unread(),
            public_stream: unread(),
        },
        Stream {
            schema: "compact-rust-proof-data-public/v1".into(),
            inputs: encoded(&operands),
            outputs: encoded(&[result]),
        },
    )
}
fn project(case: &Case, stream: &Stream) -> anyhow::Result<statement::Statement> {
    statement::derive(
        &serde_json::to_vec(&statement::profile_ir(Profile::SetMember)?)?,
        Profile::SetMember,
        case,
        stream,
        f(7),
        f(11),
    )
}

#[test]
fn zero_and_wide_inputs_with_both_booleans_bind_complete_field_bool_statement() {
    for value in [f(0), f(1), wide()] {
        for result in [f(0), f(1)] {
            let (case, stream) = fixture(value, result);
            let actual = project(&case, &stream).unwrap();
            let expected = vec![
                f(7),
                transient_commit(&[value, result], f(11)),
                f(48),
                f(80),
                f(1),
                f(1),
                f(0),
                f(16),
                f(1),
                f(1),
                -f(2),
                value,
                f(24),
                f(13),
                f(1),
                f(1),
                result,
            ];
            assert_eq!(actual.fields, expected);
            assert_eq!(actual.skips, vec![None; 5]);
        }
    }
}
#[test]
fn typed_member_arity_boolean_and_canonical_scalar_guards_are_required() {
    for input in [
        vec![],
        encoded(&[f(1), f(2)]),
        vec!["01".into()],
        vec!["f".repeat(64)],
    ] {
        let (mut case, stream) = fixture(f(1), f(1));
        case.input = input;
        assert!(project(&case, &stream).is_err());
    }
    for output in [
        vec![],
        encoded(&[f(0), f(1)]),
        vec!["01".into()],
        vec!["f".repeat(64)],
    ] {
        let (mut case, stream) = fixture(f(1), f(1));
        case.output = output;
        assert!(project(&case, &stream).is_err());
    }
    // All public occurrences agree; only the explicit Boolean domain refuses 2.
    let (case, stream) = fixture(f(1), f(2));
    assert!(project(&case, &stream).is_err());
    let (mut case, stream) = fixture(f(1), f(1));
    case.input = encoded(&[f(2)]);
    assert!(project(&case, &stream).is_err());
}
#[test]
fn every_member_operand_and_its_order_are_required() {
    let (case, stream) = fixture(wide(), f(0));
    for position in 0..stream.inputs.len() {
        let mut missing = stream.clone();
        missing.inputs.remove(position);
        assert!(project(&case, &missing).is_err());
        let mut changed = stream.clone();
        changed.inputs[position] =
            input::hex(&(input::scalar(&changed.inputs[position]).unwrap() + f(1)));
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
fn member_read_return_and_stream_schema_must_match_completely() {
    let (case, stream) = fixture(f(0), f(0));
    for outputs in [vec![], encoded(&[f(0), f(1)]), encoded(&[f(1)])] {
        let mut changed = stream.clone();
        changed.outputs = outputs;
        assert!(project(&case, &changed).is_err());
    }
    let (mut different_return, stream) = fixture(f(0), f(0));
    different_return.output = encoded(&[f(1)]);
    assert!(project(&different_return, &stream).is_err());
    let mut changed = stream;
    changed.schema = "unadmitted".into();
    assert!(project(&case, &changed).is_err());
}
#[test]
fn closed_member_ir_rejects_map_ssa_substitution_and_every_shape_expansion() {
    let (case, stream) = fixture(f(1), f(1));
    for (pointer, value) in [
        ("/version/minor", json!(1)),
        ("/do_communications_commitment", json!(false)),
        ("/inputs/0/name", json!("%key.0")),
        ("/inputs/0/type", json!("unadmitted")),
        ("/outputs/0", json!("unadmitted")),
        ("/instructions/0/inputs/0", json!("0x31")),
        ("/instructions/2/guard", json!("0x00")),
        ("/instructions/2/inputs/3", json!("0x02")),
        ("/instructions/2/inputs/4", json!("%other")),
        ("/instructions/3/op", json!("output")),
        ("/instructions/4/guard", json!("0x01")),
        ("/instructions/4/output", json!("%other")),
        ("/instructions/5/inputs/3", json!("%other")),
        ("/instructions/6/vals/0", json!("%other")),
    ] {
        let mut raw = statement::profile_ir(Profile::SetMember).unwrap();
        *raw.pointer_mut(pointer).unwrap() = value;
        assert!(
            statement::derive(
                &serde_json::to_vec(&raw).unwrap(),
                Profile::SetMember,
                &case,
                &stream,
                f(7),
                f(11)
            )
            .is_err()
        );
    }
    let mut raw = statement::profile_ir(Profile::SetMember).unwrap();
    raw["inputs"][0]["name"] = json!("%key.0");
    raw["instructions"][2]["inputs"][4] = json!("%key.0");
    assert!(
        statement::derive(
            &serde_json::to_vec(&raw).unwrap(),
            Profile::SetMember,
            &case,
            &stream,
            f(7),
            f(11)
        )
        .is_err()
    );
    let mut raw = statement::profile_ir(Profile::SetMember).unwrap();
    raw["instructions"]
        .as_array_mut()
        .unwrap()
        .push(json!({"op":"private_input","output":"%ghost"}));
    assert!(
        statement::derive(
            &serde_json::to_vec(&raw).unwrap(),
            Profile::SetMember,
            &case,
            &stream,
            f(7),
            f(11)
        )
        .is_err()
    );
}
#[test]
fn member_caller_fields_are_explicit_nonzero_and_distinct() {
    let (case, stream) = fixture(f(1), f(1));
    let raw = serde_json::to_vec(&statement::profile_ir(Profile::SetMember).unwrap()).unwrap();
    for (binding, opening) in [(f(0), f(11)), (f(7), f(0)), (f(7), f(7))] {
        assert!(
            statement::derive(&raw, Profile::SetMember, &case, &stream, binding, opening).is_err()
        );
    }
    // A changed valid binding is tested at the later actual SDK verify boundary.
}
#[test]
fn set_member_label_and_template_remain_distinct_from_map_member() {
    assert_eq!(
        serde_json::to_value(Profile::SetMember).unwrap(),
        json!("set-member")
    );
    assert_ne!(
        serde_json::to_value(Profile::SetMember).unwrap(),
        serde_json::to_value(Profile::MapMember).unwrap()
    );
    assert_ne!(
        statement::profile_ir(Profile::SetMember).unwrap(),
        statement::profile_ir(Profile::MapMember).unwrap()
    );
    let (case, stream) = fixture(f(1), f(1));
    assert!(
        statement::derive(
            &serde_json::to_vec(&statement::profile_ir(Profile::MapMember).unwrap()).unwrap(),
            Profile::SetMember,
            &case,
            &stream,
            f(7),
            f(11)
        )
        .is_err()
    );
    // Typed Set artifact/fixture/API admission is external; raw alone is no ADT authority.
}
