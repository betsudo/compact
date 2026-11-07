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

#![cfg(test)]
use generated::{ledger_contract as c, runtime as r};
use r::Field;
use r::context::{ConstructorContext, ConstructorResult};
use serde_json::{Value, json};
use std::cell::RefCell;

// Test-library helpers are public so each included constructor can use exactly
// the helpers it needs without weakening dead-code checks in generated SDKs.
pub fn inputs() -> r::FixedVector<Field, 2> {
    r::FixedVector([Field::from(2u64), Field::from(3u64)])
}
fn encode(value: Field) -> Value {
    let value = r::fab::AlignedValue::from(value);
    json!({"valueAtoms":value.value.0.iter().map(|x| &x.0).collect::<Vec<_>>(), "alignment":value.alignment})
}
pub struct Log {
    trace: RefCell<Vec<Value>>,
    failure: Option<String>,
}
impl Log {
    pub fn observe(&self, name: &str, private: u64, seen: Field) -> u64 {
        self.trace
            .borrow_mut()
            .push(json!({"name":name,"privateState":private,"seen":encode(seen)}));
        private + 1
    }
    pub fn accept(&self, name: &str) -> Result<(), r::CompactError> {
        if self.failure.as_deref() == Some(name) {
            Err(r::CompactError::AssertionFailed(
                "constructor-witness-stop".into(),
            ))
        } else {
            Ok(())
        }
    }
}
include!("constructor.rs");

fn check_program_bindings() {
    let root =
        std::path::PathBuf::from(std::env::var_os("COMPACT_CONSTRUCTOR_PROGRAM_ROOT").unwrap());
    let mut assets = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        assert!(path.is_file() && path.extension().unwrap() == "json");
        let text = std::fs::read_to_string(path).unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        let name = value["root"].as_str().unwrap().to_owned();
        assert!(
            assets.insert(name, text).is_none(),
            "duplicate Program root"
        );
    }
    assert_eq!(assets.len(), 3);
    for (name, exported) in [
        ("read", c::functional::read::CHECKED_PROGRAM),
        ("advance", c::functional::advance::CHECKED_PROGRAM),
        (
            "advance_other",
            c::functional::advance_other::CHECKED_PROGRAM,
        ),
    ] {
        let value: Value = serde_json::from_str(exported).unwrap();
        assert_eq!(value["root"], name, "exported Program root");
        assert_eq!(value["purpose"], "review_only_not_execution_authority");
        assert_eq!(exported, assets[name], "exported Program asset bytes");
    }
}

fn replay(
    out: r::recording::RecordedCircuitResult<u64, Field>,
    row: &Value,
    expected: u64,
) -> r::context::CircuitResult<u64, Field> {
    assert_eq!(
        serde_json::to_value(out.public.verify_ops()).unwrap(),
        row["publicTranscript"]
    );
    let replay = out
        .public
        .initial()
        .query(
            out.public.verify_ops(),
            None,
            &out.execution.context.cost_model,
        )
        .unwrap();
    assert_eq!(replay.context.effects, out.execution.context.query.effects);
    assert_eq!(replay.context.state, out.execution.context.query.state);
    assert_eq!(
        generated::ledger_slots::stored
            .inspect(replay.context.state.get_ref())
            .unwrap(),
        Field::from(expected), // replay-state-expectation
    );
    out.execution
}
#[test]
fn constructor_cases() {
    check_program_bindings();
    let role = std::env::var("COMPACT_CONSTRUCTOR_ROLE").unwrap();
    let reference: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_CONSTRUCTOR_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(reference["status"], "passed");
    assert_eq!(reference["format"], "constructor-vector-operands-ts/v1");
    let cases: Vec<Value> = serde_json::from_str(include_str!("cases.json")).unwrap();
    let mut executed = 0;
    for case in cases.iter().filter(|case| case["role"] == role) {
        executed += 1;
        let row = reference["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == case["id"])
            .unwrap();
        let mut baseline = None;
        for lane in 0..if case["error"].is_null() { 4 } else { 1 } {
            let log = Log {
                trace: RefCell::new(vec![]),
                failure: case["failWitness"].as_str().map(str::to_owned),
            };
            let result = initialize(case, &log);
            let expected_trace: Vec<Value> = case["trace"].as_array().unwrap().iter().map(|item| json!({"name":item["name"],"privateState":item["privateState"],"seen":encode(Field::from(item["seen"].as_u64().unwrap()))})).collect();
            assert_eq!(
                *log.trace.borrow(),
                expected_trace,
                "constructor witness prefix: {}",
                case["id"]
            );
            assert_eq!(
                *log.trace.borrow(),
                row["observedWitnesses"].as_array().unwrap().clone()
            );
            if let Some(message) = case["error"].as_str() {
                assert!(row["observedError"].as_str().unwrap().contains(message));
                assert_eq!(
                    result.err().expect("expected actual constructor refusal"),
                    r::CompactError::AssertionFailed(message.into())
                );
                continue;
            }
            let state = result.unwrap();
            check_extra(&state);
            let expected = case["expected"].as_u64().unwrap();
            let private = case["privateState"].as_u64().unwrap();
            assert_eq!(state.private_state, private);
            assert_eq!(
                generated::ledger_slots::stored
                    .inspect(state.ledger_state.get_ref())
                    .unwrap(),
                Field::from(expected)
            );
            assert_eq!(row["stored"], expected.to_string());
            assert_eq!(row["privateState"], private);
            let read = c::read(state.into_circuit_context(Default::default())).unwrap();
            assert_eq!(read.result, Field::from(expected));
            let delta = Field::from(4u64);
            let out = match lane {
                0 => c::advance(read.context, delta).unwrap(),
                1 => replay(
                    c::recorded::advance(read.context, delta).unwrap(),
                    row,
                    expected + 4,
                ),
                2 => c::functional::advance::Program::new(delta)
                    .execute_native(read.context)
                    .unwrap(),
                3 => replay(
                    c::functional::advance::Program::new(delta)
                        .execute_recorded(read.context)
                        .unwrap(),
                    row,
                    expected + 4,
                ),
                _ => unreachable!(),
            };
            assert_eq!(out.result, Field::from(expected + 4));
            assert_eq!(row["afterAdvance"], (expected + 4).to_string());
            assert_eq!(out.context.private_state, private);
            assert_eq!(
                generated::ledger_slots::stored
                    .inspect(out.context.query.state.get_ref())
                    .unwrap(),
                Field::from(expected + 4)
            );
            assert!(out.private_transcript_outputs.is_empty());
            let observed = (
                out.context.query.state.clone(),
                out.context.query.effects.clone(),
                out.gas_cost,
            );
            if let Some(before) = &baseline {
                assert_eq!(&observed, before);
            } else {
                baseline = Some(observed);
            }
        }
    }
    assert!(
        executed > 0,
        "every complete source must have an executing case"
    );
    println!("constructor-cases:{executed}");
}
