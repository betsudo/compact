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

//! External consumers of unedited, freshly generated source APIs.
use r::context::{CircuitContext, CircuitResult, ConstructorContext, WitnessContext};
use r::recording::RecordedCircuitResult;
use r::{CompactError, Field, FixedVector, OpaqueString};
use std::{cell::RefCell, fmt::Debug};
use vector_fixture::{
    ledger_contract as c, ledger_slots as slots, pure_circuits as pure, runtime as r, types,
};

fn context() -> CircuitContext<u64> {
    c::initial_state(ConstructorContext::new(7_u64))
        .unwrap()
        .into_circuit_context(Default::default())
}
fn fields() -> FixedVector<Field, 2> {
    FixedVector([Field::from(17_u64), Field::from(41_u64)])
}

#[test]
fn independent_programs_with_fresh_contexts_execute_successfully() {
    let first = c::functional::choose::Program::new(fields(), true)
        .execute_native(context())
        .unwrap();
    let last = c::functional::choose::Program::new(fields(), false)
        .execute_native(context())
        .unwrap();
    assert_eq!(first.result, Field::from(17_u64));
    assert_eq!(last.result, Field::from(41_u64));
    assert_eq!(
        slots::stored
            .inspect(first.context.query.state.get_ref())
            .unwrap(),
        first.result
    );
    assert_eq!(
        slots::stored
            .inspect(last.context.query.state.get_ref())
            .unwrap(),
        last.result
    );
}
fn texts() -> FixedVector<OpaqueString, 2> {
    FixedVector([
        OpaqueString::from("α\0first"),
        OpaqueString::from("β\0last"),
    ])
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
    assert_eq!(&native.result, expected, "independent element expectation");
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
    let replay = |value: &RecordedCircuitResult<u64, T>| {
        value
            .public
            .initial()
            .query(value.public.verify_ops(), None, &native.context.cost_model)
            .unwrap()
    };
    let ordinary = replay(&recorded);
    let functional = replay(&functional_recorded);
    assert_eq!(ordinary.context.state, native.context.query.state);
    assert_eq!(ordinary.context.effects, native.context.query.effects);
    assert_eq!(functional.context.state, ordinary.context.state);
    assert_eq!(functional.context.effects, ordinary.context.effects);
    assert_eq!(functional.gas_cost, ordinary.gas_cost);
    if let Some(row) = ts_row("cases", id) {
        assert_eq!(encoded(native.result.clone()), row["output"]);
        assert_eq!(
            encoded(
                slots::stored
                    .inspect(native.context.query.state.get_ref())
                    .unwrap()
            ),
            row["storedFab"]
        );
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
        for (actual, key) in [
            (
                serde_json::to_value(native.gas_cost.read_time).unwrap(),
                "readTime",
            ),
            (
                serde_json::to_value(native.gas_cost.compute_time).unwrap(),
                "computeTime",
            ),
        ] {
            assert_eq!(
                actual,
                serde_json::json!(
                    row["gasCost"][key]
                        .as_str()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap()
                )
            );
        }
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
// This environment variable is set only by the maintained live TS gate. A bad or
// incomplete reference fails; the standalone Rust consumer makes no TS claim.
fn ts_row(group: &str, id: &str) -> Option<serde_json::Value> {
    let path = std::env::var_os("COMPACT_VECTOR_TS_REFERENCE")?;
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(value["format"], "compact-vector-projection-ts/v1");
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
fn encoded(value: impl Into<r::fab::AlignedValue>) -> serde_json::Value {
    let value = value.into();
    serde_json::json!({"valueAtoms": value.value.0.iter().map(|atom| &atom.0).collect::<Vec<_>>(),
        "alignment": value.alignment})
}
fn pure_parity(id: &str, value: impl Into<r::fab::AlignedValue>) {
    if let Some(row) = ts_row("pure", id) {
        assert_eq!(encoded(value), row["output"]);
    }
}
#[derive(Default)]
struct Witness {
    events: RefCell<Vec<(u64, Field)>>,
    fail: bool,
}
impl c::TryWitnesses<u64> for Witness {
    fn values(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, FixedVector<Field, 2>), CompactError> {
        self.events
            .borrow_mut()
            .push((*context.private_state, context.ledger.stored()?));
        if self.fail {
            return Err(CompactError::AssertionFailed(
                "vector witness refused".into(),
            ));
        }
        Ok((*context.private_state + 1, fields()))
    }
}
#[test]
fn pure_first_last_singleton_and_owned_temporary_match_independent_values() {
    pure_parity("first", pure::first(fields()).unwrap());
    pure_parity("last", pure::last(fields()).unwrap());
    pure_parity(
        "singleton",
        pure::singleton(FixedVector([Field::from(99_u64)])).unwrap(),
    );
    pure_parity("first_text", pure::first_text(texts()).unwrap());
    pure_parity("temporary_text", pure::temporary_text(texts()).unwrap());
    assert_eq!(pure::first(fields()).unwrap(), Field::from(17_u64));
    assert_eq!(pure::last(fields()).unwrap(), Field::from(41_u64));
    assert_eq!(
        pure::singleton(FixedVector([Field::from(99_u64)])).unwrap(),
        Field::from(99_u64)
    );
    assert_eq!(
        pure::first_text(texts()).unwrap(),
        OpaqueString::from("α\0first")
    );
    assert_eq!(
        pure::temporary_text(texts()).unwrap(),
        OpaqueString::from("β\0last")
    );
}
#[test]
fn source_selection_updates_the_cell_and_replays_across_four_routes() {
    for select_first in [false, true] {
        let expected = Field::from(if select_first { 17_u64 } else { 41_u64 });
        let result = compare(
            if select_first {
                "choose-first"
            } else {
                "choose-last"
            },
            c::choose(context(), fields(), select_first),
            c::recorded::choose(context(), fields(), select_first),
            c::functional::choose::Program::new(fields(), select_first).execute_native(context()),
            c::functional::choose::Program::new(fields(), select_first).execute_recorded(context()),
            &expected,
        );
        assert_eq!(
            slots::stored
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            expected
        );
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
    }
}
#[test]
fn owned_projection_repeats_without_consuming_the_original_vector() {
    let expected = types::OwnedSelection {
        first: OpaqueString::from("α\0first"),
        repeated: OpaqueString::from("α\0first"),
        original: texts(),
    };
    let result = compare(
        "owned",
        c::owned(context(), texts()),
        c::recorded::owned(context(), texts()),
        c::functional::owned::Program::new(texts()).execute_native(context()),
        c::functional::owned::Program::new(texts()).execute_recorded(context()),
        &expected,
    );
    assert_eq!(
        slots::stored
            .inspect(result.context.query.state.get_ref())
            .unwrap(),
        Field::from(1_u64)
    );
}
#[test]
fn witnessed_selection_evaluates_once_before_the_cell_write() {
    for select_first in [false, true] {
        let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness::default());
        let expected = Field::from(if select_first { 17_u64 } else { 41_u64 });
        let result = compare(
            if select_first {
                "witnessed-first"
            } else {
                "witnessed-last"
            },
            c::witnessed(context(), &witnesses[0], select_first),
            c::recorded::witnessed(context(), &witnesses[1], select_first),
            c::functional::witnessed::Program::new(select_first)
                .execute_native(context(), &witnesses[2]),
            c::functional::witnessed::Program::new(select_first)
                .execute_recorded(context(), &witnesses[3]),
            &expected,
        );
        assert_eq!(result.context.private_state, 8);
        assert_eq!(
            result.private_transcript_outputs,
            vec![r::fab::AlignedValue::from(fields())]
        );
        for witness in witnesses {
            assert_eq!(*witness.events.borrow(), [(7, Field::from(0_u64))]);
        }
    }
}
#[test]
fn temporary_witness_vector_is_selected_once_on_every_route() {
    let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness::default());
    let result = compare(
        "temporary",
        c::temporary(context(), &witnesses[0]),
        c::recorded::temporary(context(), &witnesses[1]),
        c::functional::temporary::Program::new().execute_native(context(), &witnesses[2]),
        c::functional::temporary::Program::new().execute_recorded(context(), &witnesses[3]),
        &Field::from(41_u64),
    );
    assert_eq!(result.context.private_state, 8);
    for witness in witnesses {
        assert_eq!(*witness.events.borrow(), [(7, Field::from(0_u64))]);
    }
}
#[test]
fn inactive_branch_skips_even_a_failing_witness() {
    let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness {
        fail: true,
        ..Default::default()
    });
    let result = compare(
        "conditional-inactive",
        c::conditional(context(), &witnesses[0], false),
        c::recorded::conditional(context(), &witnesses[1], false),
        c::functional::conditional::Program::new(false).execute_native(context(), &witnesses[2]),
        c::functional::conditional::Program::new(false).execute_recorded(context(), &witnesses[3]),
        &Field::from(0_u64),
    );
    assert_eq!(result.context.private_state, 7);
    assert!(result.private_transcript_outputs.is_empty());
    for witness in witnesses {
        assert!(witness.events.borrow().is_empty());
    }
}
#[test]
fn active_branch_propagates_the_original_first_witness_error() {
    if let Some(row) = ts_row("refusals", "conditional-failure") {
        assert_eq!(row["error"], "vector witness refused");
        assert_eq!(row["events"], serde_json::json!([[7, "0"]]));
    }
    let witnesses: [Witness; 4] = std::array::from_fn(|_| Witness {
        fail: true,
        ..Default::default()
    });
    for error in [
        c::conditional(context(), &witnesses[0], true).err(),
        c::recorded::conditional(context(), &witnesses[1], true).err(),
        c::functional::conditional::Program::new(true)
            .execute_native(context(), &witnesses[2])
            .err(),
        c::functional::conditional::Program::new(true)
            .execute_recorded(context(), &witnesses[3])
            .err(),
    ] {
        assert_eq!(
            error,
            Some(CompactError::AssertionFailed(
                "vector witness refused".into()
            ))
        );
    }
    for witness in witnesses {
        assert_eq!(*witness.events.borrow(), [(7, Field::from(0_u64))]);
    }
}
