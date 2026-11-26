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
    verifier::statement,
};
use midnight_transient_crypto::{curve::Fr, hash::transient_commit};
use serde_json::json;
fn f(n: u64) -> Fr { Fr::from(n) }
fn enc(values: &[Fr]) -> Vec<String> { values.iter().map(input::hex).collect() }
fn absent_private_role() -> FileRef {
    FileRef { path: "PRIVATE-NEVER-OPEN".into(), bytes: 0, sha256: "0".repeat(64) }
}
fn fixture(key: Fr, result: Fr) -> (Case, Stream) {
    let case = Case { id: "lookup".into(), supplemental: true,
        input: enc(&[key]), output: enc(&[result]),
        preimage: absent_private_role(), public_stream: absent_private_role() };
    let stream = Stream { schema: "compact-rust-proof-data-public/v1".into(),
        inputs: enc(&[f(48),f(80),f(1),f(1),f(0),f(80),f(1),-f(2),key,f(12),f(1),-f(2),result]),
        outputs: enc(&[result]) };
    (case,stream)
}
fn derive(c: &Case, s: &Stream) -> anyhow::Result<statement::Statement> {
    statement::derive(&serde_json::to_vec(&statement::profile_ir(Profile::MapLookup)?)?,
        Profile::MapLookup,c,s,f(7),f(11))
}
#[test]
fn lookup_projects_field_field_communications_and_all_four_impacts() {
    let (c,s)=fixture(f(1),f(5)); let got=derive(&c,&s).unwrap();
    assert_eq!(got.fields.len(),15); assert_eq!(got.skips,vec![None;4]);
    assert_eq!(got.fields[0],f(7));
    assert_eq!(got.fields[1],transient_commit(&[f(1),f(5)],f(11)));
    assert_eq!(got.fields[2..],s.inputs.iter().map(|x|input::scalar(x).unwrap()).collect::<Vec<_>>());
}
#[test]
fn canonical_field_result_is_neither_boolean_nor_omitted_zero() {
    for result in [f(0),f(2),-f(1)] {
        let (c,s)=fixture(f(1),result);let got=derive(&c,&s).unwrap();
        assert_eq!(got.fields[14],result);
        assert_eq!(got.fields[1],transient_commit(&[f(1),result],f(11)));
    }
}
#[test]
fn key_and_result_beyond_machine_width_are_lossless() {
    let mut wide=f(1);for _ in 0..128 {wide=wide*f(2);}wide=wide+f(1);
    let (c,s)=fixture(wide,wide+f(3));let got=derive(&c,&s).unwrap();
    assert_eq!(got.fields[10],wide);assert_eq!(got.fields[14],wide+f(3));
}
#[test]
fn typed_key_and_return_must_match_actual_public_stream() {
    let (mut c,s)=fixture(f(1),f(5));c.input=enc(&[f(2)]);assert!(derive(&c,&s).is_err());
    let (mut c,s)=fixture(f(1),f(5));c.output=enc(&[f(6)]);assert!(derive(&c,&s).is_err());
    let (c,mut s)=fixture(f(1),f(5));s.outputs=enc(&[f(6)]);assert!(derive(&c,&s).is_err());
}
#[test]
fn every_public_operand_drift_and_missing_or_extra_stream_word_refuses() {
    let (c,s)=fixture(f(1),f(5));
    for i in 0..13 {let mut bad=Stream{schema:s.schema.clone(),inputs:s.inputs.clone(),outputs:s.outputs.clone()};
        let old=input::scalar(&bad.inputs[i]).unwrap();bad.inputs[i]=input::hex(&(old+f(1)));assert!(derive(&c,&bad).is_err());}
    let (c,mut s)=fixture(f(1),f(5));s.inputs.pop();assert!(derive(&c,&s).is_err());
    let (c,mut s)=fixture(f(1),f(5));s.inputs.push(input::hex(&f(0)));assert!(derive(&c,&s).is_err());
    let (c,mut s)=fixture(f(1),f(5));s.outputs.clear();assert!(derive(&c,&s).is_err());
    let (c,mut s)=fixture(f(1),f(5));s.outputs.push(input::hex(&f(5)));assert!(derive(&c,&s).is_err());
}
#[test]
fn wrong_arity_or_noncanonical_field_is_refused_before_material() {
    let (mut c,s)=fixture(f(1),f(5));c.input.clear();assert!(derive(&c,&s).is_err());
    let (mut c,s)=fixture(f(1),f(5));c.output.clear();assert!(derive(&c,&s).is_err());
    let (mut c,s)=fixture(f(1),f(5));c.output.push(input::hex(&f(6)));assert!(derive(&c,&s).is_err());
    let (mut c,s)=fixture(f(1),f(5));c.output=vec!["ff".repeat(32)];assert!(derive(&c,&s).is_err());
}
#[test]
fn complete_lookup_ir_is_the_admission_boundary() {
    let (c,s)=fixture(f(1),f(5));
    for (pointer,value) in [("/instructions/0/inputs/0",json!("0x31")),
        ("/instructions/2/inputs/3",json!("%other")),("/instructions/3/guard",json!("0x01")),
        ("/instructions/5/vals/0",json!("%key.0")),("/do_communications_commitment",json!(false)),
        ("/inputs/0/type",json!("Boolean"))] {
        let mut ir=statement::profile_ir(Profile::MapLookup).unwrap();*ir.pointer_mut(pointer).unwrap()=value;
        assert!(statement::derive(&serde_json::to_vec(&ir).unwrap(),Profile::MapLookup,&c,&s,f(7),f(11)).is_err());
    }
}
#[test]
fn missing_public_read_cannot_be_synthesized_as_zero_or_a_proof_statement() {
    let (c,mut s)=fixture(f(1),f(0));s.outputs.clear();assert!(derive(&c,&s).is_err());
    // This is a projector refusal only. Actual ExpectedCell(null) requires a
    // future absent-map runtime call; it is never counted as SDK verify refusal.
}
