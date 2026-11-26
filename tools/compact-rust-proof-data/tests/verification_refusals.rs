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

use compact_rust_proof_data::{
    codec::FileRef,
    input::{self, Case, Profile, Stream},
    verifier::{self, statement},
};
use midnight_transient_crypto::{curve::Fr, hash::transient_commit};
use serde_json::json;
fn f(n: i64) -> Fr {
    if n < 0 {
        -Fr::from(n.unsigned_abs())
    } else {
        Fr::from(n as u64)
    }
}
fn encoded(values: &[i64]) -> Vec<String> {
    values.iter().map(|n| input::hex(&f(*n))).collect()
}
fn dummy() -> FileRef {
    FileRef {
        path: "PRIVATE-NEVER-OPEN".into(),
        bytes: 0,
        sha256: "0".repeat(64),
    }
}
fn case(inputs: &[i64], outputs: &[i64]) -> Case {
    Case {
        id: "test".into(),
        supplemental: true,
        input: encoded(inputs),
        output: encoded(outputs),
        preimage: dummy(),
        public_stream: dummy(),
    }
}
fn stream(inputs: &[i64], outputs: &[i64]) -> Stream {
    Stream {
        schema: "compact-rust-proof-data-public/v1".into(),
        inputs: encoded(inputs),
        outputs: encoded(outputs),
    }
}
fn derive(profile: Profile, case: &Case, stream: &Stream) -> anyhow::Result<statement::Statement> {
    statement::derive(
        &serde_json::to_vec(&statement::profile_ir(profile)?)?,
        profile,
        case,
        stream,
        f(7),
        f(11),
    )
}
fn member() -> (Case, Stream) {
    (
        case(&[1], &[1]),
        stream(&[48, 80, 1, 1, 0, 16, 1, 1, -2, 1, 24, 13, 1, 1, 1], &[1]),
    )
}
fn set() -> (Case, Stream) {
    (
        case(&[5], &[]),
        stream(
            &[
                48, 80, 1, 1, 2, 12, 1, 1, 0, 16, 1, 1, 1, 0, 17, 1, 1, 32, 101, 202, 145, 16, 1,
                1, 1, 1, 17, 1, 1, -2, 5, 145, 16, 1, 1, 1, 2, 17, 1, 1, 1, 1, 145,
            ],
            &[0],
        ),
    )
}
fn clear() -> (Case, Stream) {
    (
        case(&[], &[]),
        stream(
            &[
                48, 80, 1, 1, 2, 12, 1, 1, 1, 48, 80, 1, 1, 0, 12, 1, 32, 101, 202, 16, 1, 1, 1, 0,
                17, 1, 1, 32, 0, 0, 145, 16, 1, 1, 1, 1, 17, 1, 1, -2, 0, 145, 16, 1, 1, 1, 2, 17,
                1, 1, 1, 0, 145,
            ],
            &[1, 101, 202],
        ),
    )
}
#[test]
fn all_profiles_reconstruct_complete_streams_and_value_only_communications() {
    for (profile, (case, stream), fields, skips) in [
        (Profile::MapMember, member(), 17, 5),
        (Profile::TinySet, set(), 45, 12),
        (Profile::TinyClear, clear(), 55, 15),
    ] {
        let result = derive(profile, &case, &stream).unwrap();
        assert_eq!(result.fields.len(), fields);
        assert_eq!(result.skips, vec![None; skips]);
        let io = case
            .input
            .iter()
            .chain(&case.output)
            .map(|x| input::scalar(x).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(result.fields[1], transient_commit(&io, f(11)));
    }
}
#[test]
fn false_boolean_is_a_real_zero_field_not_an_omitted_value() {
    let (mut c, mut s) = member();
    c.output = encoded(&[0]);
    s.outputs = encoded(&[0]);
    s.inputs[14] = input::hex(&f(0));
    let result = derive(Profile::MapMember, &c, &s).unwrap();
    assert_eq!(result.fields[1], transient_commit(&[f(1), f(0)], f(11)));
    assert_eq!(result.fields[16], f(0));
}
#[test]
fn wide_field_input_is_lossless() {
    let (mut c, mut s) = member();
    let mut wide = f(1);
    for _ in 0..128 {
        wide = wide * f(2);
    }
    wide = wide + f(1);
    c.input = vec![input::hex(&wide)];
    s.inputs[9] = input::hex(&wide);
    let result = derive(Profile::MapMember, &c, &s).unwrap();
    assert_eq!(result.fields[11], wide);
}
#[test]
fn every_extra_or_missing_public_operand_and_read_is_refused() {
    for (profile, (c, s)) in [
        (Profile::MapMember, member()),
        (Profile::TinySet, set()),
        (Profile::TinyClear, clear()),
    ] {
        let mut extra = stream(&[], &[]);
        extra.inputs = s.inputs.clone();
        extra.outputs = s.outputs.clone();
        extra.inputs.push(input::hex(&f(0)));
        assert!(derive(profile, &c, &extra).is_err());
        extra.inputs = s.inputs[..s.inputs.len() - 1].to_vec();
        assert!(derive(profile, &c, &extra).is_err());
        extra.inputs = s.inputs.clone();
        extra.outputs.push(input::hex(&f(0)));
        assert!(derive(profile, &c, &extra).is_err());
        extra.outputs.clear();
        assert!(derive(profile, &c, &extra).is_err());
    }
}
#[test]
fn typed_io_state_and_public_constant_drift_are_refused() {
    let (c, mut s) = member();
    s.inputs[0] = input::hex(&f(49));
    assert!(derive(Profile::MapMember, &c, &s).is_err());
    let (mut c, s) = member();
    c.output = encoded(&[2]);
    assert!(derive(Profile::MapMember, &c, &s).is_err());
    let (mut c, s) = set();
    c.input = encoded(&[6]);
    assert!(derive(Profile::TinySet, &c, &s).is_err());
    let (c, mut s) = set();
    s.outputs = encoded(&[1]);
    s.inputs[8] = input::hex(&f(1));
    assert!(derive(Profile::TinySet, &c, &s).is_err());
    let (c, mut s) = clear();
    s.outputs[0] = input::hex(&f(0));
    s.inputs[8] = input::hex(&f(0));
    assert!(derive(Profile::TinyClear, &c, &s).is_err());
}
#[test]
fn private_ir_and_unknown_symbol_changes_cannot_expand_admission() {
    let (c, s) = set();
    let mut ir = statement::profile_ir(Profile::TinySet).unwrap();
    for (pointer, value) in [
        ("/instructions/7/bits", json!(9)),
        ("/instructions/13/inputs/4", json!("%foreign")),
        ("/do_communications_commitment", json!(false)),
    ] {
        let old = ir.pointer(pointer).unwrap().clone();
        *ir.pointer_mut(pointer).unwrap() = value;
        assert!(
            statement::derive(
                &serde_json::to_vec(&ir).unwrap(),
                Profile::TinySet,
                &c,
                &s,
                f(7),
                f(11)
            )
            .is_err()
        );
        *ir.pointer_mut(pointer).unwrap() = old;
    }
}
#[test]
fn public_authority_words_are_consumed_without_a_private_hash_or_preimage_reader() {
    let (c, mut s) = set();
    s.inputs[18] = input::hex(&f(303));
    s.inputs[19] = input::hex(&f(404));
    let result = derive(Profile::TinySet, &c, &s).unwrap();
    assert_eq!(&result.fields[20..22], &[f(303), f(404)]);
    // This projects admitted public operands; the actual SDK verification must
    // still bind them to a valid proof. No private-witness truth is inferred.
}
#[test]
fn byte_literals_and_lane_limits_are_strict() {
    assert_eq!(statement::constant("0x0001").unwrap(), f(256));
    assert_eq!(statement::constant("-0x0001").unwrap(), -f(256));
    for literal in ["0x1", "0X01", "0xGG", ""] {
        assert!(statement::constant(literal).is_err());
    }
    assert!(input::policy(7, 8, false).is_ok());
    assert!(input::policy(13, 8, false).is_err());
    assert!(input::policy(13, 13, false).is_err());
    assert!(input::policy(13, 13, true).is_ok());
    assert!(input::policy(14, 14, true).is_err());
}
#[test]
fn only_an_actual_call_return_is_a_refusal_and_first_unexpected_result_stops() {
    let mut calls = Vec::new();
    let mut invoked = 0;
    verifier::verify_call(&mut calls, "one", "negative", false, || {
        invoked += 1;
        Err::<(), ()>(())
    })
    .unwrap();
    assert_eq!(invoked, 1);
    assert_eq!(calls[0]["status"], "RETURNED");
    assert_eq!(calls[0]["accepted"], false);
    assert!(
        verifier::verify_call(&mut calls, "two", "negative", false, || Ok::<(), ()>(())).is_err()
    );
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1]["accepted"], true);
}
