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

//! Independent finite public SSA projection; no producer PI/preimage access.
use crate::input::{self, Case, Profile, Stream};
use anyhow::{Context, Result, bail, ensure};
use midnight_transient_crypto::{curve::Fr, hash::transient_commit};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct Statement {
    pub fields: Vec<Fr>,
    pub skips: Vec<Option<usize>>,
}
pub fn profile_ir(profile: Profile) -> Result<Value> {
    Ok(serde_json::from_str(match profile {
        Profile::MapMember => include_str!("map_member.json"),
        Profile::TinyClear => include_str!("tiny_clear.json"),
        Profile::TinySet => include_str!("tiny_set.json"),
    })?)
}
/// IR literals encode little-endian bytes, including signed operands.
pub fn constant(text: &str) -> Result<Fr> {
    let (negative, text) = text.strip_prefix('-').map_or((false, text), |x| (true, x));
    let digits = text.strip_prefix("0x").context("IR byte literal")?;
    ensure!(
        !digits.is_empty()
            && digits.len() <= 64
            && digits.len().is_multiple_of(2)
            && digits
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "bounded canonical IR literal"
    );
    let mut n = Fr::from(0u64);
    let mut power = Fr::from(1u64);
    for i in (0..digits.len()).step_by(2) {
        n = n + Fr::from(u64::from(u8::from_str_radix(&digits[i..i + 2], 16)?)) * power;
        power = power * Fr::from(256u64);
    }
    Ok(if negative { -n } else { n })
}
pub fn derive(
    raw: &[u8],
    profile: Profile,
    case: &Case,
    stream: &Stream,
    binding: Fr,
    opening: Fr,
) -> Result<Statement> {
    let ir: Value = serde_json::from_slice(raw)?;
    // Complete definitions also bound all ignored private instructions. This is
    // an exact finite cohort, not a general evaluator or private-hash oracle.
    ensure!(
        ir == profile_ir(profile)?,
        "unadmitted complete public IR profile"
    );
    ensure!(
        stream.schema == "compact-rust-proof-data-public/v1",
        "public stream schema"
    );
    ensure!(
        binding != Fr::from(0u64) && opening != Fr::from(0u64) && binding != opening,
        "explicit distinct nonzero caller"
    );
    let typed_in = case
        .input
        .iter()
        .map(|x| input::scalar(x))
        .collect::<Result<Vec<_>>>()?;
    let typed_out = case
        .output
        .iter()
        .map(|x| input::scalar(x))
        .collect::<Result<Vec<_>>>()?;
    match profile {
        Profile::MapMember => ensure!(
            typed_in.len() == 1
                && typed_out.len() == 1
                && (typed_out[0] == Fr::from(0u64) || typed_out[0] == Fr::from(1u64)),
            "Field to Boolean typed IO"
        ),
        Profile::TinyClear => ensure!(typed_in.is_empty() && typed_out.is_empty(), "Unit clear IO"),
        Profile::TinySet => ensure!(
            typed_in.len() == 1 && typed_out.is_empty(),
            "Field to Unit set IO"
        ),
    }
    // Official ValueReprAlignedValue is value-only: Field followed by Boolean,
    // when present. Fr slices add no length/alignment prefix.
    let mut io = typed_in.clone();
    io.extend(&typed_out);
    let mut fields = vec![binding, transient_commit(&io, opening)];
    let public_in = stream
        .inputs
        .iter()
        .map(|x| input::scalar(x))
        .collect::<Result<Vec<_>>>()?;
    let public_out = stream
        .outputs
        .iter()
        .map(|x| input::scalar(x))
        .collect::<Result<Vec<_>>>()?;
    let mut memory = BTreeMap::new();
    for (decl, value) in ir["inputs"]
        .as_array()
        .context("input declarations")?
        .iter()
        .zip(&typed_in)
    {
        memory.insert(
            decl["name"].as_str().context("input SSA")?.to_owned(),
            *value,
        );
    }
    let mut skips = Vec::new();
    let mut reads = 0;
    let mut operands = 0;
    let mut returned = Vec::new();
    let mut authority = 0;
    for (position, instruction) in ir["instructions"]
        .as_array()
        .context("IR instructions")?
        .iter()
        .enumerate()
    {
        match instruction["op"].as_str().context("opcode")? {
            "public_input" => {
                ensure!(
                    instruction["guard"].is_null() && instruction["type"] == "Scalar<BLS12-381>",
                    "unconditional scalar read"
                );
                let value = *public_out
                    .get(reads)
                    .context("missing actual public read")?;
                reads += 1;
                let name = instruction["output"].as_str().context("read SSA")?;
                ensure!(
                    memory.insert(name.to_owned(), value).is_none(),
                    "unique public definition"
                );
            }
            "impact" => {
                ensure!(instruction["guard"] == "0x01", "constant true Impact guard");
                for (index, value) in instruction["inputs"]
                    .as_array()
                    .context("Impact values")?
                    .iter()
                    .enumerate()
                {
                    let token = value.as_str().context("Impact operand")?;
                    let actual = *public_in
                        .get(operands)
                        .context("missing actual public operand")?;
                    operands += 1;
                    let expected = if token.starts_with('%') {
                        if let Some(value) = memory.get(token) {
                            *value
                        } else {
                            // These two words are actual PUBLIC hash operands;
                            // no private hash or before-state is regenerated.
                            ensure!(
                                profile == Profile::TinySet
                                    && position == 13
                                    && ((index == 4 && token == "%apk.7")
                                        || (index == 5 && token == "%apk.6")),
                                "undefined public SSA operand"
                            );
                            authority += 1;
                            memory.insert(token.to_owned(), actual);
                            actual
                        }
                    } else {
                        constant(token)?
                    };
                    ensure!(expected == actual, "complete public operand mismatch");
                    fields.push(expected);
                }
                skips.push(None);
            }
            "output" => {
                ensure!(returned.is_empty(), "single return");
                for name in instruction["vals"].as_array().context("return values")? {
                    returned.push(
                        *memory
                            .get(name.as_str().context("return SSA")?)
                            .context("defined return")?,
                    );
                }
            }
            "private_input"
            | "constrain_bits"
            | "persistent_hash"
            | "bytes32_into_low_high"
            | "test_eq"
            | "assert"
            | "cond_select" => {}
            _ => bail!("unadmitted IR opcode"),
        }
    }
    ensure!(
        operands == public_in.len() && reads == public_out.len() && returned == typed_out,
        "complete operand/read/output consumption"
    );
    let (dimension, impacts) = match profile {
        Profile::MapMember => {
            ensure!(
                public_out == typed_out && authority == 0,
                "actual Boolean result"
            );
            (17, 5)
        }
        Profile::TinyClear => {
            ensure!(
                memory.get("%t.0") == Some(&Fr::from(1u64)) && authority == 0,
                "clear requires initialized state"
            );
            (55, 15)
        }
        Profile::TinySet => {
            ensure!(
                memory.get("%t.1") == Some(&Fr::from(0u64)) && authority == 2,
                "set requires unset state and two public authority words"
            );
            (45, 12)
        }
    };
    ensure!(
        fields.len() == dimension && skips == vec![None; impacts],
        "complete independent public dimensions"
    );
    Ok(Statement { fields, skips })
}
