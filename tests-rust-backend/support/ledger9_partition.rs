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

use midnight_compact_runtime::{ledger::QueryContext, recording::RecordedCircuitResult};
use midnight_ledger_v9::{
    construct::{PreTranscript, partition_transcripts},
    structure::INITIAL_PARAMETERS,
};
use serde_json::{Value, json};

pub fn normalized_effects(mut effects: Value) -> Value {
    for key in [
        "claimedNullifiers",
        "claimedShieldedReceives",
        "claimedShieldedSpends",
    ] {
        effects[key]
            .as_array_mut()
            .expect("shielded claim set")
            .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
    // WASM exposes bigint mint amounts as decimal strings. Rust serde
    // exposes the same exact u64 amounts as JSON numbers.
    for amount in effects["shieldedMints"]
        .as_object_mut()
        .expect("shielded mint map")
        .values_mut()
    {
        if let Some(number) = amount.as_u64() {
            *amount = json!(number.to_string());
        }
    }
    effects
}

pub fn assert_partition<P, O>(recorded: &RecordedCircuitResult<P, O>, row: &Value) {
    let initial = recorded.public.initial();
    let mut context = QueryContext::new(initial.state.clone(), initial.address);
    context.call_context.com_indices = recorded
        .execution
        .context
        .query
        .call_context
        .com_indices
        .clone();
    let partition = partition_transcripts(
        &[PreTranscript {
            context,
            program: recorded.public.verify_ops().to_vec(),
            comm_comm: None,
        }],
        &INITIAL_PARAMETERS,
    )
    .expect("the published Ledger 9 Rust partitioner accepts the recorded call");
    let [(guaranteed, fallible)] = partition.as_slice() else {
        panic!("expected one transcript pair");
    };
    let expected = row["partition"]
        .as_array()
        .expect("TypeScript transcript pair");
    assert_eq!(expected.len(), 2);
    for (actual, expected) in [guaranteed, fallible].into_iter().zip(expected) {
        let Some(actual) = actual else {
            assert!(expected.is_null(), "{}: absent transcript", row["name"]);
            continue;
        };
        let actual_program = actual.program.iter_deref().collect::<Vec<_>>();
        if expected.get("program").is_some() {
            assert_eq!(
                json!(actual_program),
                expected["program"],
                "{}",
                row["name"]
            );
        } else {
            assert_eq!(
                json!(actual_program.len()),
                expected["programLength"],
                "{}",
                row["name"]
            );
        }
        assert_eq!(
            normalized_effects(json!(actual.effects)),
            normalized_effects(expected["effects"].clone()),
            "{}: partition effects",
            row["name"]
        );
        let gas = json!(actual.gas);
        for field in ["readTime", "computeTime", "bytesWritten", "bytesDeleted"] {
            assert_eq!(
                gas[field].as_u64().unwrap().to_string(),
                expected["gas"][field],
                "{}: partition gas {field}",
                row["name"]
            );
        }
    }
}
