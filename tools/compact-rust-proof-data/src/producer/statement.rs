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
            "persistent_hash" if profile != Profile::MapMember => {
                hashes.insert(op["output"].as_str().context("hash byte SSA")?.to_owned());
            }
            "bytes32_into_low_high" if profile != Profile::MapMember => {
                if hashes.contains(op["bytes"].as_str().context("byte SSA")?) {
                    let out = op["outputs"].as_array().context("two public hash fields")?;
                    ensure!(out.len() == 2, "Bytes32 scalar arity");
                    for name in out {
                        projected.insert(name.as_str().context("hash field SSA")?.to_owned());
                    }
                }
            }
            "private_input" | "constrain_bits" | "test_eq" | "cond_select" | "assert"
                if profile != Profile::MapMember => {}
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
            Profile::TinyClear => "clear",
            Profile::TinySet => "set",
        };
        ensure!(pre.key_location.0 == name, "circuit identity");
        ensure!(
            pre.private_transcript.is_empty() == (bundle.input.profile == Profile::MapMember),
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
