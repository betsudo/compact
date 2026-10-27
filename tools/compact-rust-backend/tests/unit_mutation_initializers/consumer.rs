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

use fixture::{ledger_contract as c, ledger_slots as slots, runtime as r};
use r::{
    CompactError, Field,
    context::{CircuitResult, ConstructorContext, WitnessContext},
    recording::PublicTrace,
};
use serde_json::{Value, json};
use std::cell::RefCell;

fn field(value: u64) -> Field {
    Field::from(value)
}
fn label_number(value: Field) -> u64 {
    (0..=31)
        .find(|&label| field(label) == value)
        .expect("fixture witness label")
}
fn encoded(value: impl Into<r::fab::AlignedValue>) -> Value {
    let value = value.into();
    json!({"valueAtoms":value.value.0.iter().map(|atom| &atom.0).collect::<Vec<_>>(),"alignment":value.alignment})
}
struct Witness<'a> {
    spec: &'a Value,
    events: RefCell<Vec<Value>>,
}
impl Witness<'_> {
    fn record(
        &self,
        label: Field,
        context: &WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(), CompactError> {
        let label = label_number(label);
        self.events.borrow_mut().push(json!([
            label,
            *context.private_state,
            context.ledger.values()?.size()?.value().to_string()
        ]));
        if self.spec["fail"].as_u64() == Some(label) {
            return Err(CompactError::AssertionFailed(format!(
                "witness {label} refused"
            )));
        }
        Ok(())
    }
}
impl c::TryWitnesses<u64> for Witness<'_> {
    fn point(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
        label: Field,
        value: Field,
    ) -> Result<(u64, Field), CompactError> {
        self.record(label, &context)?;
        Ok((*context.private_state + 1, value))
    }
    fn gate(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
        label: Field,
        accepted: bool,
    ) -> Result<(u64, bool), CompactError> {
        self.record(label, &context)?;
        Ok((*context.private_state + 1, accepted))
    }
    fn unitGuard(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
        label: Field,
        accepted: bool,
    ) -> Result<(u64, ()), CompactError> {
        self.record(label, &context)?;
        if !accepted {
            return Err(CompactError::AssertionFailed(format!(
                "unit guard {} refused",
                label_number(label)
            )));
        }
        Ok((*context.private_state + 1, ()))
    }
}
fn check<T: Into<r::fab::AlignedValue>>(
    result: Result<(CircuitResult<u64, T>, Option<PublicTrace>), CompactError>,
    spec: &Value,
    row: &Value,
    route: usize,
    journals: &mut Vec<Value>,
) -> usize {
    if let Some(error) = spec["error"].as_str() {
        assert!(
            result
                .err()
                .expect("expected first failure")
                .to_string()
                .contains(error),
            "{} route{route}",
            spec["id"]
        );
        return 0;
    }
    let (out, public) = result.unwrap();
    assert_eq!(
        encoded(out.result),
        row["output"],
        "{} route{route}",
        spec["id"]
    );
    assert_eq!(
        out.context.private_state,
        spec["privateState"].as_u64().unwrap()
    );
    assert_eq!(json!(out.context.private_state), row["privateState"]);
    assert_eq!(
        json!(
            out.private_transcript_outputs
                .iter()
                .cloned()
                .map(encoded)
                .collect::<Vec<_>>()
        ),
        row["privateOutputs"]
    );
    let view = slots::values
        .inspect(out.context.query.state.get_ref())
        .unwrap();
    let keys = (0..=7)
        .filter(|&n| view.member(field(n)))
        .collect::<Vec<_>>();
    assert_eq!(json!(keys), spec["keys"]);
    assert_eq!(json!(keys), row["keys"]);
    assert_eq!(view.size().unwrap().value() as usize, keys.len());
    assert_eq!(out.context.query.effects, Default::default());
    for (name, actual) in [
        (
            "readTime",
            serde_json::to_value(out.gas_cost.read_time)
                .unwrap()
                .as_u64()
                .unwrap()
                .to_string(),
        ),
        (
            "computeTime",
            serde_json::to_value(out.gas_cost.compute_time)
                .unwrap()
                .as_u64()
                .unwrap()
                .to_string(),
        ),
        ("bytesWritten", out.gas_cost.bytes_written.to_string()),
        ("bytesDeleted", out.gas_cost.bytes_deleted.to_string()),
    ] {
        assert_eq!(
            actual,
            row["gasCost"][name].as_str().unwrap(),
            "{} route{route} {name}",
            spec["id"]
        );
    }
    if let Some(public) = public {
        let replay = public
            .initial()
            .query(public.verify_ops(), None, &out.context.cost_model)
            .unwrap();
        assert_eq!(replay.context.state, out.context.query.state);
        assert_eq!(replay.context.effects, out.context.query.effects);
        journals.push(json!({"operations":public.verify_ops(),"replayGas":replay.gas_cost}));
        1
    } else {
        0
    }
}
#[test]
fn live_typescript_four_routes_and_pure_error_order() {
    let specs: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_UNIT_INITIALIZERS_CASES").unwrap()).unwrap(),
    )
    .unwrap();
    let reference: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_UNIT_INITIALIZERS_TS_REFERENCE").unwrap())
            .unwrap(),
    )
    .unwrap();
    let mut routes = 0;
    let mut replays = 0;
    for spec in specs.as_array().unwrap() {
        let row = reference["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == spec["id"])
            .unwrap();
        let boolean = |i: usize| spec["args"][i].as_bool().unwrap();
        let arg = |i: usize| field(spec["args"][i].as_u64().unwrap());
        let name = spec["name"].as_str().unwrap();
        if ["unitEq", "unitNe", "unitCondition"].contains(&name) {
            routes += 1;
            let actual = match name {
                "unitEq" => fixture::pure_circuits::unitEq(boolean(0), boolean(1)).map(encoded),
                "unitNe" => fixture::pure_circuits::unitNe(boolean(0), boolean(1)).map(encoded),
                _ => fixture::pure_circuits::unitCondition(boolean(0), boolean(1)).map(encoded),
            };
            if let Some(error) = spec["error"].as_str() {
                assert!(actual.unwrap_err().to_string().contains(error));
            } else {
                assert_eq!(actual.unwrap(), row["output"]);
            }
            continue;
        }
        let mut journals = Vec::new();
        for route in 0..4 {
            routes += 1;
            let witnesses = Witness {
                spec,
                events: RefCell::new(vec![]),
            };
            let mut context = c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(Default::default());
            for seed in spec["seed"].as_array().unwrap() {
                context = c::conditional(context, true, field(seed.as_u64().unwrap()))
                    .unwrap()
                    .context;
            }
            macro_rules! invoke {($entry:ident,[$($arg:expr),*]$(,$witness:expr)?)=>{{
                let result=match route {
                    0=>c::$entry(context,$($witness,)?$($arg),*).map(|out|(out,None)),
                    1=>c::recorded::$entry(context,$($witness,)?$($arg),*).map(|out|(out.execution,Some(out.public))),
                    2=>c::functional::$entry::Program::new($($arg),*).execute_native(context$(,$witness)?).map(|out|(out,None)),
                    _=>c::functional::$entry::Program::new($($arg),*).execute_recorded(context$(,$witness)?).map(|out|(out.execution,Some(out.public))),
                };
                check(result,spec,row,route,&mut journals)
            }}}
            replays += match name {
                "conditional" => invoke!(conditional, [boolean(0), arg(1)]),
                "observed" => invoke!(observed, [boolean(0), arg(1)], &witnesses),
                "sequential" => invoke!(sequential, [boolean(0), boolean(1), arg(2)], &witnesses),
                "nested" => invoke!(nested, [boolean(0), boolean(1), arg(2)]),
                "unit_result" => invoke!(unit_result, [boolean(0), arg(1)]),
                "unused" => invoke!(unused, [boolean(0), arg(1)], &witnesses),
                "unit_witness_comparison" => invoke!(
                    unit_witness_comparison,
                    [boolean(0), boolean(1)],
                    &witnesses
                ),
                "unit_witness_inequality" => invoke!(
                    unit_witness_inequality,
                    [boolean(0), boolean(1)],
                    &witnesses
                ),
                "shadowed" => invoke!(shadowed, [boolean(0), boolean(1), arg(2)]),
                _ => panic!("unknown case {name}"),
            };
            assert_eq!(
                json!(*witnesses.events.borrow()),
                spec["events"],
                "{} route{route}",
                spec["id"]
            );
            assert_eq!(json!(*witnesses.events.borrow()), row["events"]);
        }
        if !spec["error"].is_string() {
            assert_eq!(journals.len(), 2);
            assert_eq!(
                journals[0], journals[1],
                "{} journals/replay gas",
                spec["id"]
            );
        }
    }
    println!(
        "COMPACT_UNIT_INITIALIZERS {} {routes} {replays}",
        specs.as_array().unwrap().len()
    );
}
