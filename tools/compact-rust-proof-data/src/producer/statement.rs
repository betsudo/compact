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

//! Producer projection only; the verifier must not import this module.
use anyhow::{Context, Result, ensure};
use compact_rust_proof_data::{
    codec,
    input::{Bundle, Profile, hex, scalar},
};
use midnight_transient_crypto::{
    curve::Fr,
    hash::transient_commit,
    proofs::{ProofPreimage, Zkir},
};
use midnight_zkir::IrSource;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub struct PreparedCase {
    pub id: String,
    pub supplemental: bool,
    pub pre: ProofPreimage,
    pub fields: Vec<Fr>,
    pub skips: Vec<Option<usize>>,
}
pub struct Prepared {
    pub ir: IrSource,
    pub raw: Vec<u8>,
    pub tagged: Vec<u8>,
    pub k: u8,
    pub rows: usize,
    pub cases: Vec<PreparedCase>,
}
fn constant(s: &str) -> Result<Fr> {
    let (negative, s) = s.strip_prefix('-').map_or((false, s), |v| (true, v));
    let n = u64::from_str_radix(s.strip_prefix("0x").context("public constant hex")?, 16)?;
    let f = Fr::from(n);
    Ok(if negative { -f } else { f })
}
struct StatementHeader {
    binding: Fr,
    communication_commitment: Fr,
}
fn project(
    profile: Profile,
    shape: &Value,
    input: &[Fr],
    output: &[Fr],
    public_in: &[Fr],
    public_out: &[Fr],
    header: StatementHeader,
) -> Result<(Vec<Fr>, Vec<Option<usize>>)> {
    let readonly_map = matches!(
        profile,
        Profile::MapMember | Profile::MapLookup | Profile::MapIsEmpty | Profile::UnitUint64
    );
    if profile == Profile::MapLookup {
        let original: Value = serde_json::from_str(include_str!("map_lookup.json"))?;
        ensure!(shape == &original, "exact admitted readonly Lookup profile");
    }
    if profile == Profile::MapIsEmpty {
        let original: Value = serde_json::from_str(include_str!("map_is_empty.json"))?;
        ensure!(
            shape == &original,
            "exact admitted readonly Map is_empty profile"
        );
    }
    if profile == Profile::UnitUint64 {
        let original: Value = serde_json::from_str(include_str!("map_size.json"))?;
        ensure!(
            shape == &original,
            "exact admitted readonly Map size profile"
        );
    }
    ensure!(
        shape["version"]["major"] == 3 && shape["do_communications_commitment"] == true,
        "ZKIR3 communication profile"
    );
    let declarations = shape["inputs"]
        .as_array()
        .context("declared scalar inputs")?;
    ensure!(
        declarations.len() == input.len()
            && shape["outputs"]
                .as_array()
                .context("declared outputs")?
                .len()
                == output.len(),
        "complete typed IO arity"
    );
    if profile == Profile::MapIsEmpty {
        ensure!(
            input.is_empty()
                && output.len() == 1
                && (output[0] == Fr::from(0u64) || output[0] == Fr::from(1u64)),
            "readonly Map is_empty zero-input Boolean signature"
        );
    }
    if profile == Profile::UnitUint64 {
        ensure!(
            input.is_empty() && output.len() == 1,
            "readonly Map size zero-input Uint64 signature"
        );
        let bytes = output[0].as_le_bytes();
        ensure!(
            bytes.len() == 32 && bytes[8..].iter().all(|b| *b == 0),
            "readonly Map size Uint64 output range"
        );
    }
    let mut vars = BTreeMap::new();
    for (d, v) in declarations.iter().zip(input) {
        ensure!(d["type"] == "Scalar<BLS12-381>", "scalar input only");
        ensure!(
            vars.insert(d["name"].as_str().context("input name")?.to_owned(), *v)
                .is_none(),
            "unique input SSA"
        );
    }
    let (mut used_in, mut used_out, mut emitted) = (0, 0, 0);
    let mut fields = vec![header.binding, header.communication_commitment];
    let mut skips = Vec::new();
    let mut hashes = BTreeSet::new();
    let mut projected = BTreeSet::new();
    let ops = shape["instructions"].as_array().context("instructions")?;
    ensure!(
        !ops.is_empty() && ops.len() <= 512,
        "finite original instruction roster"
    );
    for op in ops {
        match op["op"].as_str().context("opcode")? {
            "public_input" => {
                ensure!(
                    op["guard"].is_null() && op["type"] == "Scalar<BLS12-381>",
                    "unguarded scalar read"
                );
                let value = *public_out
                    .get(used_out)
                    .context("actual read, no padding")?;
                used_out += 1;
                ensure!(
                    vars.insert(op["output"].as_str().context("read SSA")?.to_owned(), value)
                        .is_none(),
                    "unique read SSA"
                );
            }
            "impact" => {
                ensure!(op["guard"] == "0x01", "constant true Impact only");
                for operand in op["inputs"]
                    .as_array()
                    .context("complete Impact operands")?
                {
                    let name = operand.as_str().context("operand")?;
                    let value = if let Some(v) = vars.get(name) {
                        *v
                    } else if projected.contains(name) {
                        let v = *public_in
                            .get(used_in)
                            .context("public hash write operand")?;
                        vars.insert(name.to_owned(), v);
                        v
                    } else {
                        constant(name)?
                    };
                    ensure!(
                        public_in.get(used_in) == Some(&value),
                        "entire public operand stream"
                    );
                    used_in += 1;
                    fields.push(value);
                }
                skips.push(None);
            }
            "persistent_hash" if !readonly_map => {
                hashes.insert(op["output"].as_str().context("hash byte SSA")?.to_owned());
            }
            "bytes32_into_low_high" if !readonly_map => {
                if hashes.contains(op["bytes"].as_str().context("byte SSA")?) {
                    let out = op["outputs"].as_array().context("two public hash fields")?;
                    ensure!(out.len() == 2, "Bytes32 scalar arity");
                    for name in out {
                        projected.insert(name.as_str().context("hash field SSA")?.to_owned());
                    }
                }
            }
            "private_input" | "constrain_bits" | "test_eq" | "cond_select" | "assert"
                if !readonly_map => {}
            "output" => {
                for name in op["vals"].as_array().context("output values")? {
                    let v = vars
                        .get(name.as_str().context("output SSA")?)
                        .context("mapped circuit output")?;
                    ensure!(
                        output.get(emitted) == Some(v),
                        "actual declared output fully mapped"
                    );
                    emitted += 1;
                }
            }
            _ => anyhow::bail!("unsupported closed profile instruction"),
        }
    }
    ensure!(
        used_in == public_in.len() && used_out == public_out.len() && emitted == output.len(),
        "all public streams and outputs consumed"
    );
    if profile == Profile::MapMember {
        ensure!(
            ops.len() == 7
                && output.len() == 1
                && (output[0] == Fr::from(0u64) || output[0] == Fr::from(1u64))
                && public_out == output,
            "readonly member Boolean read"
        );
    }
    if profile == Profile::MapLookup {
        ensure!(
            input.len() == 1 && output.len() == 1 && public_out == output,
            "readonly Lookup full Field result from actual public read"
        );
    }
    if profile == Profile::MapIsEmpty {
        ensure!(
            public_out == output,
            "actual is_empty Boolean result is the complete public read"
        );
    }
    if profile == Profile::UnitUint64 {
        ensure!(
            public_out == output,
            "actual size Uint64 result is the complete public read"
        );
    }
    Ok((fields, skips))
}
pub fn prepare(bundle: &Bundle) -> Result<Prepared> {
    let raw = bundle.read(&bundle.input.raw_ir, 4 * 1024 * 1024)?;
    let shape: Value = serde_json::from_slice(&raw)?;
    let ir = IrSource::load(raw.as_slice())?;
    let tagged = codec::encode(&ir)?;
    let again: IrSource = codec::decode(&tagged)?;
    ensure!(again == ir, "canonical original IR");
    let own = ir.model();
    let k = Zkir::k(&ir);
    let rows = own.rows();
    ensure!(own.k() == k, "actual own model consistency");
    let (binding, random) = bundle.caller()?;
    let mut cases = Vec::new();
    for c in &bundle.input.cases {
        let pre: ProofPreimage = codec::decode(&bundle.read(&c.preimage, 4 * 1024 * 1024)?)?;
        let input = c
            .input
            .iter()
            .map(|s| scalar(s))
            .collect::<Result<Vec<_>>>()?;
        let output = c
            .output
            .iter()
            .map(|s| scalar(s))
            .collect::<Result<Vec<_>>>()?;
        let comm = pre
            .communications_commitment
            .context("actual communication")?;
        let mut io = input.clone();
        io.extend(&output);
        ensure!(
            pre.binding_input == binding
                && comm.1 == random
                && comm.0 == transient_commit(&io, random)
                && pre.inputs == input,
            "same explicit caller and typed input-plus-output commitment"
        );
        let name = match bundle.input.profile {
            Profile::MapMember => "member",
            Profile::MapLookup => "lookup",
            Profile::MapIsEmpty => "is_empty",
            Profile::UnitUint64 => "size",
            Profile::TinyClear => "clear",
            Profile::TinySet => "set",
        };
        ensure!(pre.key_location.0 == name, "circuit identity");
        ensure!(
            pre.private_transcript.is_empty()
                == matches!(
                    bundle.input.profile,
                    Profile::MapMember
                        | Profile::MapLookup
                        | Profile::MapIsEmpty
                        | Profile::UnitUint64
                ),
            "actual no-witness/witness profile"
        );
        let stream = bundle.stream(c)?;
        ensure!(
            stream.inputs
                == pre
                    .public_transcript_inputs
                    .iter()
                    .map(hex)
                    .collect::<Vec<_>>()
                && stream.outputs
                    == pre
                        .public_transcript_outputs
                        .iter()
                        .map(hex)
                        .collect::<Vec<_>>(),
            "public record matches actual captured codec"
        );
        let (fields, skips) = project(
            bundle.input.profile,
            &shape,
            &input,
            &output,
            &pre.public_transcript_inputs,
            &pre.public_transcript_outputs,
            StatementHeader {
                binding,
                communication_commitment: comm.0,
            },
        )?;
        let actual = pre.check(&ir)?;
        ensure!(
            actual == skips,
            "fresh public SDK check and complete independent skips"
        );
        cases.push(PreparedCase {
            id: c.id.clone(),
            supplemental: c.supplemental,
            pre,
            fields,
            skips,
        });
    }
    Ok(Prepared {
        raw,
        tagged,
        ir,
        k,
        rows,
        cases,
    })
}
pub fn report(p: &Prepared) -> Value {
    json!({"k":p.k,"rowsExcludingCustomGatesAndLookups":p.rows,"cases":p.cases.iter().map(|c|json!({"id":c.id,"fields":c.fields.len(),"skips":c.skips,"SDKCheckPassed":true})).collect::<Vec<_>>(),"beforeStateIndependentlyRegenerated":false,"SDKPIAccessorUsed":false,"proofProduced":false})
}

#[cfg(test)]
mod lookup_tests {
    use super::*;
    fn wide(bits: usize, tail: u64) -> Fr {
        let mut n = Fr::from(1u64);
        for _ in 0..bits {
            n = n + n;
        }
        n + Fr::from(tail)
    }
    fn shape() -> Value {
        serde_json::from_str(include_str!("map_lookup.json")).unwrap()
    }
    // Public unit operands from the admitted actual IR, not a forged VM trace.
    fn operands(key: Fr, result: Fr) -> Vec<Fr> {
        vec![
            Fr::from(0x30u64),
            Fr::from(0x50u64),
            Fr::from(1u64),
            Fr::from(1u64),
            Fr::from(0u64),
            Fr::from(0x50u64),
            Fr::from(1u64),
            -Fr::from(2u64),
            key,
            Fr::from(0x0cu64),
            Fr::from(1u64),
            -Fr::from(2u64),
            result,
        ]
    }
    fn projection(
        shape: &Value,
        key: &[Fr],
        result: &[Fr],
        operands: &[Fr],
        reads: &[Fr],
    ) -> Result<(Vec<Fr>, Vec<Option<usize>>)> {
        project(
            Profile::MapLookup,
            shape,
            key,
            result,
            operands,
            reads,
            StatementHeader {
                binding: Fr::from(17u64),
                communication_commitment: Fr::from(31u64),
            },
        )
    }
    #[test]
    fn lookup_projects_zero_nonboolean_and_wide_field_outputs() {
        for (key, result) in [
            (Fr::from(1u64), Fr::from(0u64)),
            (Fr::from(1u64), Fr::from(7u64)),
            (wide(128, 1), wide(129, 7)),
        ] {
            let words = operands(key, result);
            let (fields, skips) =
                projection(&shape(), &[key], &[result], &words, &[result]).unwrap();
            let mut expected = vec![Fr::from(17u64), Fr::from(31u64)];
            expected.extend_from_slice(&words);
            assert_eq!(fields, expected);
            let expected = shape()["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|op| op["op"] == "impact")
                .map(|_| None)
                .collect::<Vec<Option<usize>>>();
            assert_eq!(skips, expected);
        }
    }
    #[test]
    fn lookup_refuses_operand_read_output_arity_and_unconsumed_data() {
        let key = Fr::from(1u64);
        let result = Fr::from(7u64);
        let words = operands(key, result);
        let mut wrong = words.clone();
        wrong[8] = Fr::from(2u64);
        assert!(projection(&shape(), &[key], &[result], &wrong, &[result]).is_err());
        assert!(projection(&shape(), &[key], &[Fr::from(8u64)], &words, &[result]).is_err());
        assert!(projection(&shape(), &[key], &[result], &words, &[]).is_err());
        assert!(projection(&shape(), &[key], &[result], &words, &[result, result]).is_err());
        assert!(
            projection(
                &shape(),
                &[key],
                &[result],
                &words[..words.len() - 1],
                &[result]
            )
            .is_err()
        );
        let mut extra = words.clone();
        extra.push(Fr::from(0u64));
        assert!(projection(&shape(), &[key], &[result], &extra, &[result]).is_err());
        assert!(projection(&shape(), &[], &[result], &words, &[result]).is_err());
        assert!(projection(&shape(), &[key], &[], &words, &[result]).is_err());
    }
    #[test]
    fn lookup_refuses_private_hash_writes_guards_and_foreign_ssa_shapes() {
        let key = Fr::from(1u64);
        let result = Fr::from(7u64);
        let words = operands(key, result);
        let mut shapes = Vec::new();
        for opcode in ["private_input", "persistent_hash", "bytes32_into_low_high"] {
            let mut s = shape();
            s["instructions"][3]["op"] = json!(opcode);
            shapes.push(s);
        }
        let mut s = shape();
        s["instructions"][2]["inputs"][0] = json!("0x70");
        shapes.push(s);
        let mut s = shape();
        s["instructions"][0]["guard"] = json!("0x00");
        shapes.push(s);
        let mut s = shape();
        s["instructions"][2]["inputs"][3] = json!("%foreign");
        shapes.push(s);
        let mut s = shape();
        s["outputs"][0] = json!("Boolean");
        shapes.push(s);
        for s in shapes {
            assert!(projection(&s, &[key], &[result], &words, &[result]).is_err());
        }
    }
}

#[cfg(test)]
mod is_empty_tests {
    use super::*;
    fn shape() -> Value {
        serde_json::from_str(include_str!("map_is_empty.json")).unwrap()
    }
    // Literal operands from stopped fresh819f/nativef793 public streams.
    // This is a projection unit input, not an invented VM execution or proof.
    fn words(bit: Fr) -> Vec<Fr> {
        let mut words = [
            48u64, 80u64, 1u64, 1u64, 0u64, 4u64, 16u64, 1u64, 1u64, 8u64, 0u64, 2u64, 13u64, 1u64,
            1u64,
        ]
        .into_iter()
        .map(Fr::from)
        .collect::<Vec<_>>();
        words.push(bit);
        words
    }
    fn projection(
        shape: &Value,
        input: &[Fr],
        output: &[Fr],
        words: &[Fr],
        reads: &[Fr],
    ) -> Result<(Vec<Fr>, Vec<Option<usize>>)> {
        project(
            Profile::MapIsEmpty,
            shape,
            input,
            output,
            words,
            reads,
            StatementHeader {
                binding: Fr::from(17u64),
                communication_commitment: Fr::from(31u64),
            },
        )
    }
    #[test]
    fn native_true_false_streams_project_every_operand_with_zero_declared_inputs() {
        for bit in [Fr::from(0u64), Fr::from(1u64)] {
            let words = words(bit);
            let (fields, skips) = projection(&shape(), &[], &[bit], &words, &[bit]).unwrap();
            let mut expected = vec![Fr::from(17u64), Fr::from(31u64)];
            expected.extend(&words);
            assert_eq!(fields, expected);
            let impacts = shape()["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|x| x["op"] == "impact")
                .count();
            assert_eq!(skips, vec![None; impacts]);
        }
    }
    #[test]
    fn ghost_unit_input_nonboolean_output_and_read_return_mismatch_refuse() {
        let bit = Fr::from(1u64);
        let w = words(bit);
        assert!(projection(&shape(), &[Fr::from(0u64)], &[bit], &w, &[bit]).is_err());
        assert!(projection(&shape(), &[], &[], &w, &[bit]).is_err());
        assert!(projection(&shape(), &[], &[bit, bit], &w, &[bit]).is_err());
        let two = Fr::from(2u64);
        assert!(projection(&shape(), &[], &[two], &words(two), &[two]).is_err());
        assert!(projection(&shape(), &[], &[Fr::from(0u64)], &w, &[bit]).is_err());
        assert!(projection(&shape(), &[], &[bit], &w, &[Fr::from(0u64)]).is_err());
    }
    #[test]
    fn every_ordered_operand_and_public_read_is_consumed_without_padding() {
        let bit = Fr::from(1u64);
        let w = words(bit);
        for i in 0..w.len() {
            let mut changed = w.clone();
            changed[i] = changed[i] + Fr::from(1u64);
            assert!(projection(&shape(), &[], &[bit], &changed, &[bit]).is_err());
            let mut missing = w.clone();
            missing.remove(i);
            assert!(projection(&shape(), &[], &[bit], &missing, &[bit]).is_err());
        }
        let mut reordered = w.clone();
        reordered.swap(0, 1);
        assert!(projection(&shape(), &[], &[bit], &reordered, &[bit]).is_err());
        let mut extra = w.clone();
        extra.push(Fr::from(0u64));
        assert!(projection(&shape(), &[], &[bit], &extra, &[bit]).is_err());
        assert!(projection(&shape(), &[], &[bit], &w, &[]).is_err());
        assert!(projection(&shape(), &[], &[bit], &w, &[bit, bit]).is_err());
    }
    #[test]
    fn complete_raw_profile_refuses_new_private_effects_ssa_guards_and_io() {
        let bit = Fr::from(1u64);
        let w = words(bit);
        for (path, replacement) in [
            ("/do_communications_commitment", json!(false)),
            ("/version/minor", json!(1)),
            (
                "/inputs",
                json!([{"name":"%ghost","type":"Scalar<BLS12-381>"}]),
            ),
            ("/outputs/0", json!("unadmitted")),
            ("/instructions/0/op", json!("private_input")),
            ("/instructions/0/inputs/0", json!("0x31")),
            ("/instructions/2/guard", json!("0x00")),
            ("/instructions/5/output", json!("%foreign")),
            ("/instructions/6/inputs/3", json!("%foreign")),
            ("/instructions/7/vals/0", json!("%foreign")),
        ] {
            let mut modified = shape();
            *modified.pointer_mut(path).unwrap() = replacement;
            assert!(
                projection(&modified, &[], &[bit], &w, &[bit]).is_err(),
                "{path}"
            );
        }
    }
}

#[cfg(test)]
mod unit_uint64_tests {
    use super::*;
    fn shape() -> Value {
        serde_json::from_str(include_str!("map_size.json")).unwrap()
    }
    // Synthetic projection inputs derived only from the exact public raw IR.
    // No VM execution, captured cardinality, model or proof is claimed here.
    fn words(value: Fr) -> Vec<Fr> {
        let mut words = [48u64, 80, 1, 1, 0, 4, 13, 1, 8]
            .into_iter()
            .map(Fr::from)
            .collect::<Vec<_>>();
        words.push(value);
        words
    }
    fn projection(
        shape: &Value,
        input: &[Fr],
        output: &[Fr],
        words: &[Fr],
        reads: &[Fr],
    ) -> Result<(Vec<Fr>, Vec<Option<usize>>)> {
        project(
            Profile::UnitUint64,
            shape,
            input,
            output,
            words,
            reads,
            StatementHeader {
                binding: Fr::from(17u64),
                communication_commitment: Fr::from(31u64),
            },
        )
    }
    #[test]
    fn zero_one_two_and_uint64_max_project_every_public_operand() {
        for n in [0, 1, 2, u64::MAX] {
            let value = Fr::from(n);
            let w = words(value);
            let (fields, skips) = projection(&shape(), &[], &[value], &w, &[value]).unwrap();
            let mut expected = vec![Fr::from(17u64), Fr::from(31u64)];
            expected.extend(&w);
            assert_eq!(fields, expected);
            let impacts = shape()["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|op| op["op"] == "impact")
                .count();
            assert_eq!(skips, vec![None; impacts]);
        }
    }
    #[test]
    fn uint64_domain_zero_arity_and_actual_read_result_must_agree() {
        let value = Fr::from(2u64); // Valid Uint64; deliberately not a Boolean.
        let w = words(value);
        assert!(projection(&shape(), &[Fr::from(0u64)], &[value], &w, &[value]).is_err());
        assert!(projection(&shape(), &[], &[], &w, &[value]).is_err());
        assert!(projection(&shape(), &[], &[value, value], &w, &[value]).is_err());
        let overflow = Fr::from(u64::MAX) + Fr::from(1u64);
        assert!(projection(&shape(), &[], &[overflow], &words(overflow), &[overflow]).is_err());
        assert!(projection(&shape(), &[], &[value], &w, &[Fr::from(1u64)]).is_err());
        assert!(projection(&shape(), &[], &[Fr::from(1u64)], &w, &[value]).is_err());
    }
    #[test]
    fn every_ordered_operand_and_read_is_required_without_padding() {
        let value = Fr::from(2u64);
        let w = words(value);
        for i in 0..w.len() {
            let mut changed = w.clone();
            changed[i] = changed[i] + Fr::from(1u64);
            assert!(projection(&shape(), &[], &[value], &changed, &[value]).is_err());
            let mut missing = w.clone();
            missing.remove(i);
            assert!(projection(&shape(), &[], &[value], &missing, &[value]).is_err());
        }
        let mut reordered = w.clone();
        reordered.swap(0, 1);
        assert!(projection(&shape(), &[], &[value], &reordered, &[value]).is_err());
        let mut extra = w.clone();
        extra.push(Fr::from(0u64));
        assert!(projection(&shape(), &[], &[value], &extra, &[value]).is_err());
        assert!(projection(&shape(), &[], &[value], &w, &[]).is_err());
        assert!(projection(&shape(), &[], &[value], &w, &[value, value]).is_err());
    }
    #[test]
    fn complete_raw_template_refuses_effects_ssa_guards_io_and_release_version() {
        let value = Fr::from(2u64);
        let w = words(value);
        for (pointer, replacement) in [
            ("/do_communications_commitment", json!(false)),
            ("/version/minor", json!(1)),
            (
                "/inputs",
                json!([{"name":"%ghost","type":"Scalar<BLS12-381>"}]),
            ),
            ("/outputs/0", json!("Boolean")),
            ("/instructions/0/op", json!("private_input")),
            ("/instructions/0/inputs/0", json!("0x31")),
            ("/instructions/1/guard", json!("0x00")),
            ("/instructions/3/output", json!("%foreign")),
            ("/instructions/4/inputs/3", json!("%foreign")),
            ("/instructions/5/vals/0", json!("%foreign")),
        ] {
            let mut changed = shape();
            *changed.pointer_mut(pointer).unwrap() = replacement;
            assert!(
                projection(&changed, &[], &[value], &w, &[value]).is_err(),
                "{pointer}"
            );
        }
        let mut reordered = shape();
        reordered["instructions"].as_array_mut().unwrap().swap(2, 3);
        assert!(projection(&reordered, &[], &[value], &w, &[value]).is_err());
    }
}
