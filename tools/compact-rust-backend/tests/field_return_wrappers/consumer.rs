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

use r::{
    BoundedUint, CompactError, Field,
    context::{ConstructorContext, WitnessContext},
};
use serde_json::{Value, json};
use std::cell::RefCell;
use wrapper_fixture::{ledger_contract as c, ledger_slots as slots, runtime as r};
fn field(x: u64) -> Field {
    Field::from(x)
}
fn encoded(value: impl Into<r::fab::AlignedValue>) -> Value {
    let v = value.into();
    json!({"valueAtoms":v.value.0.iter().map(|a|&a.0).collect::<Vec<_>>(),"alignment":v.alignment})
}
struct Witness<'a> {
    spec: &'a Value,
    events: RefCell<Vec<Value>>,
}
impl Witness<'_> {
    fn record(
        &self,
        name: &str,
        ctx: &WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(), CompactError> {
        self.events.borrow_mut().push(json!([
            name,
            *ctx.private_state,
            ctx.ledger.count()?.value().to_string(),
            ctx.ledger.values()?.size()?.value().to_string()
        ]));
        if self.spec["fail"] == name {
            return Err(CompactError::AssertionFailed(format!("{name} refused")));
        }
        Ok(())
    }
}
impl c::TryWitnesses<u64> for Witness<'_> {
    fn choose(
        &self,
        ctx: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, bool), CompactError> {
        self.record("choose", &ctx)?;
        Ok((
            *ctx.private_state + 1,
            self.spec["choice"].as_bool().unwrap(),
        ))
    }
    fn observe(
        &self,
        ctx: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, Field), CompactError> {
        self.record("observe", &ctx)?;
        Ok((*ctx.private_state + 1, field(9)))
    }
}
#[test]
fn all_cases_match_live_ts_across_four_routes() {
    let specs: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_FIELD_WRAPPERS_CASES").unwrap()).unwrap(),
    )
    .unwrap();
    let reference: Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("COMPACT_FIELD_WRAPPERS_TS_REFERENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let mut routes = 0;
    let mut replays = 0;
    for spec in specs.as_array().unwrap() {
        let row = reference["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == spec["id"])
            .unwrap();
        if spec["name"] == "expression_only" {
            routes += 1;
            let out = wrapper_fixture::pure_circuits::expression_only(
                BoundedUint::<65535>::new(spec["args"][0].as_u64().unwrap() as u128).unwrap(),
            )
            .unwrap();
            assert_eq!(out, field(spec["result"].as_u64().unwrap()));
            assert_eq!(encoded(out), row["output"]);
            continue;
        }
        for route in 0..4 {
            routes += 1;
            let witness = Witness {
                spec,
                events: RefCell::new(vec![]),
            };
            let context = c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(Default::default());
            let arg = |i: usize| field(spec["args"][i].as_u64().unwrap());
            let boolean = |i: usize| spec["args"][i].as_bool().unwrap();
            macro_rules! invoke{($name:ident,[$($arg:expr),*],$($w:expr)?)=>{{
    match route{
     0=>c::$name(context,$($w,)?$($arg),*).map(|v|(v,None)),
     1=>c::recorded::$name(context,$($w,)?$($arg),*).map(|v|(v.execution,Some(v.public))),
     2=>c::functional::$name::Program::new($($arg),*).execute_native(context$(,$w)?).map(|v|(v,None)),
     _=>c::functional::$name::Program::new($($arg),*).execute_recorded(context$(,$w)?).map(|v|(v.execution,Some(v.public))),
    }
   }}}
            let result = match spec["name"].as_str().unwrap() {
                "inferred" => invoke!(inferred, [boolean(0), arg(1)],),
                "explicit" => invoke!(explicit, [boolean(0), arg(1)],),
                "nested" => invoke!(nested, [boolean(0), boolean(1), arg(2)],),
                "ordered" => invoke!(ordered, [arg(0)], &witness),
                "checked" => invoke!(
                    checked,
                    [
                        boolean(0),
                        BoundedUint::<65535>::new(spec["args"][1].as_u64().unwrap() as u128)
                            .unwrap()
                    ],
                    &witness
                ),
                _ => panic!("unknown source case"),
            };
            assert_eq!(
                *witness.events.borrow(),
                row["events"].as_array().unwrap().clone(),
                "{} route{route}",
                spec["id"]
            );
            if let Some(error) = spec["error"].as_str() {
                let e = result.err().expect("expected refusal");
                if error != "subtract" {
                    assert!(e.to_string().contains(error));
                } else {
                    assert!(matches!(e, CompactError::UnsignedUnderflow), "{e:?}");
                }
                continue;
            }
            let (out, public) = result.unwrap();
            assert_eq!(out.result, field(spec["result"].as_u64().unwrap()));
            assert_eq!(encoded(out.result), row["output"]);
            assert_eq!(
                out.context.private_state,
                row["privateState"].as_u64().unwrap()
            );
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
            let state = out.context.query.state.get_ref();
            let view = slots::values.inspect(state).unwrap();
            let keys = (0..=8)
                .filter(|&n| view.member(field(n)))
                .collect::<Vec<_>>();
            assert_eq!(json!(keys), spec["keys"]);
            assert_eq!(view.size().unwrap().value() as usize, keys.len());
            assert_eq!(
                slots::count.inspect(state).unwrap(),
                spec["count"].as_u64().unwrap()
            );
            assert_eq!(slots::stored.inspect(state).unwrap(), field(2));
            assert_eq!(out.context.query.effects, Default::default());
            for (key, actual) in [
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
                    row["gasCost"][key].as_str().unwrap(),
                    "{} route{route} {key}",
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
                replays += 1;
            }
        }
    }
    println!(
        "COMPACT_FIELD_WRAPPERS {} {} {}",
        specs.as_array().unwrap().len(),
        routes,
        replays
    );
}
