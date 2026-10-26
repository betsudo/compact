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

//! External ordinary consumers of the complete unedited generated crate.
use map_fixture::{ledger_contract as c, ledger_slots as slots, runtime as r, types};
use r::context::{
    CircuitContext, CircuitResult, ConstructorContext, ConstructorResult, WitnessContext,
};
use r::recording::RecordedCircuitResult;
use r::{BoundedUint, CompactError, Field};
use std::{cell::RefCell, fmt::Debug};

type Count = BoundedUint<{ u64::MAX as u128 }>;
fn count(value: u64) -> Count {
    Count::new(value as u128).unwrap()
}
fn field(value: u64) -> Field {
    Field::from(value)
}
fn context(mode: &str) -> CircuitContext<u64> {
    use r::fab::AlignedValue;
    use r::ledger::{LedgerHashMap, StateValue, constructor_cell, contract_state};
    if mode == "seed" {
        return c::initial_state(ConstructorContext::new(7))
            .unwrap()
            .into_circuit_context(Default::default());
    }
    let table = if mode == "missing" {
        return ConstructorResult::new(ConstructorContext::new(7), contract_state(vec![]))
            .into_circuit_context(Default::default());
    } else if mode == "wrong-kind" {
        constructor_cell(field(9))
    } else if mode == "wrong-array" {
        StateValue::Array(vec![].into())
    } else if mode == "empty" {
        StateValue::Map(LedgerHashMap::new())
    } else {
        assert_eq!(mode, "nested");
        StateValue::Map(
            LedgerHashMap::new().insert(AlignedValue::from(field(1)), constructor_cell(field(11))),
        )
    };
    let outer = if mode == "nested" {
        StateValue::Map(
            LedgerHashMap::new()
                .insert(
                    AlignedValue::from(field(3)),
                    StateValue::Map(LedgerHashMap::new()),
                )
                .insert(
                    AlignedValue::from(field(4)),
                    StateValue::Map(LedgerHashMap::new()),
                ),
        )
    } else {
        StateValue::Map(LedgerHashMap::new())
    };
    ConstructorResult::new(
        ConstructorContext::new(7),
        contract_state(vec![table, outer]),
    )
    .into_circuit_context(Default::default())
}
fn encoded(value: impl Into<r::fab::AlignedValue>) -> serde_json::Value {
    let value = value.into();
    serde_json::json!({"valueAtoms":value.value.0.iter().map(|atom|&atom.0).collect::<Vec<_>>(),"alignment":value.alignment})
}
fn ts_row(group: &str, id: &str) -> Option<serde_json::Value> {
    let path = std::env::var_os("COMPACT_MAP_SIZE_TS_REFERENCE")?;
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(value["format"], "compact-map-size-expression-ts/v1");
    Some(
        value[group]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()
            .clone(),
    )
}
fn table_values(state: &r::ledger::StateValue<r::ledger::DefaultDB>) -> serde_json::Value {
    let view = slots::table.inspect(state).unwrap();
    let pairs = (1..=2)
        .filter(|&key| view.member(field(key)))
        .map(|key| serde_json::json!({"key":encoded(field(key)),"value":encoded(view.lookup(field(key)).unwrap())}))
        .collect::<Vec<_>>();
    assert_eq!(
        view.size().unwrap().value(),
        pairs.len() as u128,
        "all known source keys accounted for"
    );
    serde_json::json!(pairs)
}
fn compare<T: Debug + PartialEq + Clone + Into<r::fab::AlignedValue>>(
    id: &str,
    native: Result<CircuitResult<u64, T>, CompactError>,
    recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    functional_native: Result<CircuitResult<u64, T>, CompactError>,
    functional_recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    expected: &T,
) -> CircuitResult<u64, T> {
    let (native, recorded, functional_native, functional_recorded) = (
        native.unwrap(),
        recorded.unwrap(),
        functional_native.unwrap(),
        functional_recorded.unwrap(),
    );
    assert_eq!(&native.result, expected, "independent source expectation");
    for actual in [
        &recorded.execution,
        &functional_native,
        &functional_recorded.execution,
    ] {
        assert_eq!(&actual.result, expected);
        assert_eq!(actual.context.query.state, native.context.query.state);
        assert_eq!(actual.context.query.effects, native.context.query.effects);
        assert_eq!(actual.context.private_state, native.context.private_state);
        assert_eq!(
            actual.private_transcript_outputs,
            native.private_transcript_outputs
        );
        assert_eq!(actual.gas_cost, native.gas_cost);
    }
    assert_eq!(
        serde_json::to_value(recorded.public.verify_ops()).unwrap(),
        serde_json::to_value(functional_recorded.public.verify_ops()).unwrap()
    );
    for actual in [&recorded, &functional_recorded] {
        let replay = actual
            .public
            .initial()
            .query(actual.public.verify_ops(), None, &native.context.cost_model)
            .unwrap();
        assert_eq!(replay.context.state, native.context.query.state);
        assert_eq!(replay.context.effects, native.context.query.effects);
    }
    if let Some(row) = ts_row("cases", id) {
        assert_eq!(encoded(native.result.clone()), row["output"]);
        assert_eq!(
            table_values(native.context.query.state.get_ref()),
            row["table"]
        );
        // These source circuits only mutate storage. Rust serializes empty
        // effect maps as objects; TS exposes entry arrays. Require the actual
        // typed/default and exact TS neutral effects, without a general JSON
        // normalization that could conceal nonempty effects.
        assert_eq!(native.context.query.effects, Default::default());
        assert_eq!(
            row["effects"],
            serde_json::json!({
            "claimedNullifiers":[],"claimedShieldedReceives":[],"claimedShieldedSpends":[],
            "claimedContractCalls":[],"shieldedMints":[],"unshieldedMints":[],
            "unshieldedInputs":[],"unshieldedOutputs":[],"claimedUnshieldedSpends":[]})
        );
        let r::ledger::StateValue::Array(fields) = native.context.query.state.get_ref() else {
            panic!("source ledger array")
        };
        let r::ledger::StateValue::Map(outer) = fields.get(1).unwrap() else {
            panic!("source nested Map")
        };
        assert_eq!(outer.size().to_string(), row["outerSize"]);
        assert_eq!(
            serde_json::json!(native.context.private_state),
            row["privateState"]
        );
        assert_eq!(
            serde_json::json!(
                native
                    .private_transcript_outputs
                    .iter()
                    .cloned()
                    .map(encoded)
                    .collect::<Vec<_>>()
            ),
            row["privateOutputs"]
        );
        assert_eq!(
            serde_json::to_value(native.gas_cost.read_time).unwrap(),
            serde_json::json!(
                row["gasCost"]["readTime"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            )
        );
        assert_eq!(
            serde_json::to_value(native.gas_cost.compute_time).unwrap(),
            serde_json::json!(
                row["gasCost"]["computeTime"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            )
        );
        assert_eq!(
            native.gas_cost.bytes_written.to_string(),
            row["gasCost"]["bytesWritten"]
        );
        assert_eq!(
            native.gas_cost.bytes_deleted.to_string(),
            row["gasCost"]["bytesDeleted"]
        );
    }
    native
}
#[derive(Default)]
struct Witness {
    events: RefCell<Vec<(u64, Count)>>,
    fail: bool,
}
impl c::TryWitnesses<u64> for Witness {
    fn offset(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, Count), CompactError> {
        self.events
            .borrow_mut()
            .push((*context.private_state, context.ledger.table()?.size()?));
        if self.fail {
            return Err(CompactError::AssertionFailed(
                "map size witness refused".into(),
            ));
        }
        Ok((*context.private_state + 1, count(2)))
    }
}

#[test]
fn metadata_counts_empty_populated_and_whole_nested_maps_across_four_routes() {
    for (mode, size, outer) in [("seed", 1, 0), ("empty", 0, 0), ("nested", 1, 2)] {
        macro_rules! scalar {
            ($name:ident,$expected:expr) => {
                compare(
                    &format!("{}-{mode}", stringify!($name)),
                    c::$name(context(mode)),
                    c::recorded::$name(context(mode)),
                    c::functional::$name::Program::new().execute_native(context(mode)),
                    c::functional::$name::Program::new().execute_recorded(context(mode)),
                    &$expected,
                );
            };
        }
        scalar!(direct, count(size));
        scalar!(empty, size == 0);
        scalar!(plus_one, count(size + 1));
        scalar!(
            snapshot,
            types::Counts {
                before: count(size),
                after: count(size),
                outer: count(outer)
            }
        );
    }
}
#[test]
fn lexical_count_is_retained_across_replacement_and_new_insertion() {
    for key in [1, 2] {
        let actual = compare(
            &format!("put-{key}"),
            c::put(context("seed"), field(key), field(99)),
            c::recorded::put(context("seed"), field(key), field(99)),
            c::functional::put::Program::new(field(key), field(99)).execute_native(context("seed")),
            c::functional::put::Program::new(field(key), field(99))
                .execute_recorded(context("seed")),
            &types::Counts {
                before: count(1),
                after: count(key),
                outer: count(0),
            },
        );
        assert_eq!(
            slots::table
                .inspect(actual.context.query.state.get_ref())
                .unwrap()
                .lookup(field(key))
                .unwrap(),
            field(99)
        );
    }
}
#[test]
fn conditional_query_executes_only_the_selected_arm() {
    for active in [false, true] {
        compare(
            &format!("choose-{active}"),
            c::choose(context("seed"), active),
            c::recorded::choose(context("seed"), active),
            c::functional::choose::Program::new(active).execute_native(context("seed")),
            c::functional::choose::Program::new(active).execute_recorded(context("seed")),
            &count(if active { 1 } else { 7 }),
        );
    }
}
#[test]
fn ordered_query_precedes_one_metered_witness_and_matches_real_cost() {
    let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness::default());
    let actual = compare(
        "ordered",
        c::ordered(context("seed"), &witnesses[0]),
        c::recorded::ordered(context("seed"), &witnesses[1]),
        c::functional::ordered::Program::new().execute_native(context("seed"), &witnesses[2]),
        c::functional::ordered::Program::new().execute_recorded(context("seed"), &witnesses[3]),
        &count(3),
    );
    assert_eq!(actual.context.private_state, 8);
    for witness in witnesses {
        assert_eq!(*witness.events.borrow(), [(7, count(1))]);
    }
}
#[test]
fn inactive_witness_is_skipped_and_active_insertion_changes_size() {
    for active in [false, true] {
        let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness {
            fail: !active,
            ..Default::default()
        });
        let actual = compare(
            &format!("witnessed-{active}"),
            c::witnessed(context("seed"), &witnesses[0], active),
            c::recorded::witnessed(context("seed"), &witnesses[1], active),
            c::functional::witnessed::Program::new(active)
                .execute_native(context("seed"), &witnesses[2]),
            c::functional::witnessed::Program::new(active)
                .execute_recorded(context("seed"), &witnesses[3]),
            &count(if active { 2 } else { 1 }),
        );
        assert_eq!(actual.context.private_state, 7 + u64::from(active));
        for witness in witnesses {
            assert_eq!(
                *witness.events.borrow(),
                if active { vec![(7, count(1))] } else { vec![] }
            );
        }
    }
}
#[test]
fn first_query_failure_prevents_the_witness_and_inactive_query_never_runs() {
    for mode in ["missing", "wrong-kind"] {
        if let Some(row) = ts_row("refusals", &format!("ordered-{mode}")) {
            assert_eq!(row["events"], serde_json::json!([]));
            assert_eq!(row["queries"].as_array().unwrap().len(), 1);
            assert!(row["queries"][0]["failure"].is_string());
        }
        let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness {
            fail: true,
            ..Default::default()
        });
        for error in [
            c::ordered(context(mode), &witnesses[0]).err(),
            c::recorded::ordered(context(mode), &witnesses[1]).err(),
            c::functional::ordered::Program::new()
                .execute_native(context(mode), &witnesses[2])
                .err(),
            c::functional::ordered::Program::new()
                .execute_recorded(context(mode), &witnesses[3])
                .err(),
        ] {
            assert!(
                matches!(error, Some(CompactError::LedgerQueryRejected(_))),
                "{mode}: {error:?}"
            );
        }
        for witness in witnesses {
            assert!(witness.events.borrow().is_empty());
        }
        let a = c::choose(context(mode), false).unwrap();
        let b = c::recorded::choose(context(mode), false).unwrap();
        let d = c::functional::choose::Program::new(false)
            .execute_native(context(mode))
            .unwrap();
        let e = c::functional::choose::Program::new(false)
            .execute_recorded(context(mode))
            .unwrap();
        for result in [&a, &b.execution, &d, &e.execution] {
            assert_eq!(result.result, count(7));
            assert_eq!(result.gas_cost, r::context::RunningCost::default());
        }
        if let Some(row) = ts_row("boundaries", &format!("choose-inactive-{mode}")) {
            assert_eq!(row["output"], encoded(count(7)));
            assert!(row["queries"].as_array().unwrap().is_empty());
        }
    }
}
#[test]
fn first_witness_error_is_preserved_before_any_insertion() {
    for ordered in [false, true] {
        let name = if ordered { "ordered" } else { "witnessed" };
        if let Some(row) = ts_row("refusals", &format!("{name}-failure")) {
            assert_eq!(row["error"], "map size witness refused");
            assert_eq!(row["events"], serde_json::json!([[7, "1"]]));
        }
        let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness {
            fail: true,
            ..Default::default()
        });
        let errors = if ordered {
            [
                c::ordered(context("seed"), &witnesses[0]).err(),
                c::recorded::ordered(context("seed"), &witnesses[1]).err(),
                c::functional::ordered::Program::new()
                    .execute_native(context("seed"), &witnesses[2])
                    .err(),
                c::functional::ordered::Program::new()
                    .execute_recorded(context("seed"), &witnesses[3])
                    .err(),
            ]
        } else {
            [
                c::witnessed(context("seed"), &witnesses[0], true).err(),
                c::recorded::witnessed(context("seed"), &witnesses[1], true).err(),
                c::functional::witnessed::Program::new(true)
                    .execute_native(context("seed"), &witnesses[2])
                    .err(),
                c::functional::witnessed::Program::new(true)
                    .execute_recorded(context("seed"), &witnesses[3])
                    .err(),
            ]
        };
        for error in errors {
            assert_eq!(
                error,
                Some(CompactError::AssertionFailed(
                    "map size witness refused".into()
                ))
            );
        }
        for witness in witnesses {
            assert_eq!(*witness.events.borrow(), [(7, count(1))]);
        }
    }
}

#[test]
fn array_in_map_slot_refuses_before_metadata_is_observed() {
    let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness::default());
    for error in [
        c::ordered(context("wrong-array"), &witnesses[0]).err(),
        c::recorded::ordered(context("wrong-array"), &witnesses[1]).err(),
        c::functional::ordered::Program::new()
            .execute_native(context("wrong-array"), &witnesses[2])
            .err(),
        c::functional::ordered::Program::new()
            .execute_recorded(context("wrong-array"), &witnesses[3])
            .err(),
    ] {
        assert_eq!(
            error,
            Some(CompactError::LedgerQueryRejected(
                "expected Map at metadata query path".into()
            ))
        );
    }
    for witness in witnesses {
        assert!(witness.events.borrow().is_empty());
    }
    for result in [
        c::choose(context("wrong-array"), false).unwrap(),
        c::recorded::choose(context("wrong-array"), false)
            .unwrap()
            .execution,
        c::functional::choose::Program::new(false)
            .execute_native(context("wrong-array"))
            .unwrap(),
        c::functional::choose::Program::new(false)
            .execute_recorded(context("wrong-array"))
            .unwrap()
            .execution,
    ] {
        assert_eq!(result.result, count(7));
        assert_eq!(result.gas_cost, r::context::RunningCost::default());
    }
}
