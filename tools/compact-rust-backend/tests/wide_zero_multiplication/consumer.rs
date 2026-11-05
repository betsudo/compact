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

use std::cell::Cell;
use wide_zero_fixture::{
    ledger_contract as c, ledger_slots as slots, pure_circuits as p, runtime as r,
};
type U = r::WideUint<4722366482869645213695, { u128::MAX }>;
type Z = r::BoundedUint<0>;
fn wide(high: u128, low: u128) -> U {
    let mut bytes = low.to_le_bytes().to_vec();
    bytes.extend(high.to_le_bytes());
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    U::from_le_bytes(&bytes).unwrap()
}
fn values() -> [U; 4] {
    [
        wide(0, 0),
        wide(0, 1),
        wide(1, 0),
        wide(4722366482869645213695, u128::MAX),
    ]
}
fn context() -> r::context::CircuitContext<u64> {
    c::initial_state(r::context::ConstructorContext::new(42))
        .unwrap()
        .into_circuit_context(Default::default())
}
#[test]
fn pure_boundary_values_and_aliases_keep_the_exact_zero_carrier() {
    for x in values() {
        let outputs: [Z; 4] = [
            p::rightZero(x).unwrap(),
            p::leftZero(x).unwrap(),
            p::rightParameter(x, Z::new(0).unwrap()).unwrap(),
            p::leftParameter(Z::new(0).unwrap(), x).unwrap(),
        ];
        assert!(outputs.iter().all(|z| z.value() == 0));
        for active in [false, true] {
            let z: Z = p::nested(x, active).unwrap();
            assert_eq!(z.value(), 0);
        }
    }
}
#[test]
fn checked_casts_and_distinct_first_errors_are_preserved_in_both_operands() {
    let big = wide(1, 0);
    let one = r::BoundedUint::<255>::new(1).unwrap();
    let zero = r::BoundedUint::<255>::new(0).unwrap();
    assert_eq!(
        p::checkedLeft(big, one).err(),
        Some(r::CompactError::InvalidUnsignedValue)
    );
    for result in [p::checkedRight(one, big), p::checkedLeft(wide(0, 1), one)] {
        assert_eq!(
            result.err(),
            Some(r::CompactError::UnsignedOutOfRange { value: 1, max: 0 })
        );
    }
    assert_eq!(
        p::checkedRight(zero, big).err(),
        Some(r::CompactError::InvalidUnsignedValue)
    );
    assert_eq!(
        p::firstErrorLeft(big, one).err(),
        Some(r::CompactError::InvalidUnsignedValue)
    );
    assert_eq!(
        p::firstErrorRight(one, big).err(),
        Some(r::CompactError::AssertionFailed(
            "zero operand guard".into()
        ))
    );
    assert_eq!(
        p::firstErrorLeft(wide(0, 1), one).err(),
        Some(r::CompactError::AssertionFailed(
            "zero operand guard".into()
        ))
    );
    assert_eq!(
        p::firstErrorRight(zero, big).err(),
        Some(r::CompactError::InvalidUnsignedValue)
    );
    assert_eq!(p::checkedLeft(wide(0, u128::MAX), zero).unwrap().value(), 0);
    assert_eq!(
        p::checkedRight(zero, wide(0, u128::MAX)).unwrap().value(),
        0
    );
}
fn compare<const MAX: u128>(
    a: &r::context::CircuitResult<u64, r::BoundedUint<MAX>>,
    b: &r::context::CircuitResult<u64, r::BoundedUint<MAX>>,
) {
    assert_eq!(a.result, b.result);
    assert_eq!(a.context.private_state, b.context.private_state);
    assert_eq!(a.context.query.state, b.context.query.state);
    assert_eq!(a.context.query.effects, b.context.query.effects);
    assert_eq!(a.gas_cost, b.gas_cost);
    assert_eq!(a.private_transcript_outputs, b.private_transcript_outputs);
}
fn replay<const MAX: u128>(out: &r::recording::RecordedCircuitResult<u64, r::BoundedUint<MAX>>) {
    let replay = out
        .public
        .initial()
        .query(
            out.public.verify_ops(),
            None,
            &out.execution.context.cost_model,
        )
        .unwrap();
    assert_eq!(replay.context.state, out.execution.context.query.state);
    assert_eq!(replay.context.effects, out.execution.context.query.effects);
}
macro_rules! ordinary {
    ($name:ident) => {
        for x in values() {
            let manual = slots::calls
                .record_increment(r::recording::RecordingFrame::new(context()), 1)
                .unwrap()
                .finish(r::BoundedUint::<255>::new(0).unwrap());
            let native = c::$name(context(), x).unwrap();
            compare(&native, &manual.execution);
            let recorded = c::recorded::$name(context(), x).unwrap();
            compare(&recorded.execution, &native);
            assert_eq!(recorded.public.verify_ops(), manual.public.verify_ops());
            replay(&recorded);
            let native = c::functional::$name::Program::new(x)
                .execute_native(context())
                .unwrap();
            compare(&native, &manual.execution);
            let recorded = c::functional::$name::Program::new(x)
                .execute_recorded(context())
                .unwrap();
            compare(&recorded.execution, &native);
            assert_eq!(recorded.public.verify_ops(), manual.public.verify_ops());
            replay(&recorded);
        }
    };
}
#[test]
fn both_stateful_operand_orders_match_manual_counter_and_both_functional_routes() {
    ordinary!(multiplyRight);
    ordinary!(multiplyLeft);
}
struct W {
    count: Cell<usize>,
    value: U,
    fail: bool,
}
impl c::TryWitnesses<u64> for W {
    fn wide(
        &self,
        ctx: r::context::WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, U), r::CompactError> {
        self.count.set(self.count.get() + 1);
        assert_eq!(ctx.ledger.calls()?.value(), 1);
        if self.fail {
            return Err(r::CompactError::AssertionFailed(
                "wide witness failure".into(),
            ));
        }
        Ok((*ctx.private_state + 1, self.value))
    }
}
macro_rules! witnessed {
    ($name:ident) => {
        for x in values() {
            for route in 0..4 {
                for fail in [false, true] {
                    let w = W {
                        count: Cell::new(0),
                        value: x,
                        fail,
                    };
                    let result = match route {
                        0 => c::$name(context(), &w),
                        1 => c::recorded::$name(context(), &w).map(|out| {
                            replay(&out);
                            out.execution
                        }),
                        2 => c::functional::$name::Program::new().execute_native(context(), &w),
                        _ => c::functional::$name::Program::new()
                            .execute_recorded(context(), &w)
                            .map(|out| {
                                replay(&out);
                                out.execution
                            }),
                    };
                    assert_eq!(w.count.get(), 1);
                    if fail {
                        assert_eq!(
                            result.err(),
                            Some(r::CompactError::AssertionFailed(
                                "wide witness failure".into()
                            ))
                        );
                    } else {
                        let out = result.unwrap();
                        assert_eq!(out.result.value(), 0);
                        assert_eq!(out.context.private_state, 43);
                        assert_eq!(
                            out.private_transcript_outputs,
                            vec![r::fab::AlignedValue::from(x)]
                        );
                    }
                }
            }
        }
    };
}
#[test]
fn wide_witness_is_observed_once_after_counter_even_when_literal_zero_is_left() {
    witnessed!(witnessedRight);
    witnessed!(witnessedLeft);
}
#[test]
fn untrusted_nonzero_encoding_cannot_become_the_zero_typed_operand() {
    let encoded = r::fab::AlignedValue::from(r::BoundedUint::<1>::new(1).unwrap());
    assert_eq!(
        Z::try_from(&*encoded.as_slice()),
        Err(r::CompactError::UnsignedOutOfRange { value: 1, max: 0 })
    );
}

#[test]
fn genuine_typescript_capture_agrees_with_wide_zero_source_results() {
    let path = std::env::var("COMPACT_WIDE_ZERO_TS_REFERENCE").expect("TS capture required");
    let reference: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(reference["format"], "wide-zero-ts/v1");
    let pure = reference["rows"]["pure"].as_array().unwrap();
    assert_eq!(pure.len(), 24);
    for x in values() {
        let decimal = num_decimal(x);
        for name in [
            "rightZero",
            "leftZero",
            "rightParameter",
            "leftParameter",
            "nested",
        ] {
            let found: Vec<_> = pure
                .iter()
                .filter(|row| row["name"] == name && row["input"] == decimal)
                .collect();
            assert_eq!(found.len(), if name == "nested" { 2 } else { 1 });
            assert!(
                found
                    .iter()
                    .all(|row| row["result"] == p::rightZero(x).unwrap().value().to_string())
            );
        }
    }
    let stateful = reference["rows"]["stateful"].as_array().unwrap();
    assert_eq!(stateful.len(), 16);
    for x in values() {
        for name in [
            "multiplyRight",
            "multiplyLeft",
            "witnessedRight",
            "witnessedLeft",
        ] {
            let found: Vec<_> = stateful
                .iter()
                .filter(|row| row["name"] == name && row["input"] == num_decimal(x))
                .collect();
            assert_eq!(found.len(), 1);
            let row = found[0];
            let w = W {
                count: Cell::new(0),
                value: x,
                fail: false,
            };
            // Compare actual wrapper observations with the actual Rust route.
            let (result, out) = match name {
                "multiplyRight" => {
                    let o = c::multiplyRight(context(), x).unwrap();
                    (o.result.value(), o.context)
                }
                "multiplyLeft" => {
                    let o = c::multiplyLeft(context(), x).unwrap();
                    (o.result.value(), o.context)
                }
                "witnessedRight" => {
                    let o = c::witnessedRight(context(), &w).unwrap();
                    (o.result.value(), o.context)
                }
                _ => {
                    let o = c::witnessedLeft(context(), &w).unwrap();
                    (o.result.value(), o.context)
                }
            };
            assert_eq!(row["result"], result.to_string());
            assert_eq!(
                row["calls"],
                slots::calls
                    .inspect(out.query.state.get_ref())
                    .unwrap()
                    .to_string()
            );
            assert_eq!(row["privateState"], out.private_state);
            assert_eq!(row["witnesses"], w.count.get());
        }
    }
    let errors = reference["rows"]["errors"].as_array().unwrap();
    let one = r::BoundedUint::<255>::new(1).unwrap();
    let zero = r::BoundedUint::<255>::new(0).unwrap();
    let big = wide(1, 0);
    let actual = [
        ("checkedLeft", p::checkedLeft(big, one)),
        ("checkedRight", p::checkedRight(one, big)),
        ("checkedLeft", p::checkedLeft(wide(0, 1), one)),
        ("checkedRight", p::checkedRight(zero, big)),
        ("firstErrorLeft", p::firstErrorLeft(big, one)),
        ("firstErrorRight", p::firstErrorRight(one, big)),
        ("firstErrorLeft", p::firstErrorLeft(wide(0, 1), one)),
        ("firstErrorRight", p::firstErrorRight(zero, big)),
    ];
    assert_eq!(errors.len(), actual.len());
    for (row, (name, result)) in errors.iter().zip(actual) {
        assert_eq!(row["name"], name);
        match result.unwrap_err() {
            r::CompactError::InvalidUnsignedValue | r::CompactError::UnsignedOutOfRange { .. } => {
                assert_eq!(row["kind"], "cast")
            }
            r::CompactError::AssertionFailed(message) => {
                assert_eq!(row["kind"], "assert");
                assert_eq!(row["message"], format!("failed assert: {message}"));
            }
            error => panic!("unexpected Rust error: {error:?}"),
        }
    }
}
fn num_decimal(value: U) -> String {
    // Independent fixture roster, with no bigint runtime dependency.
    let boundary = values().iter().position(|v| *v == value).unwrap();
    [
        "0",
        "1",
        "340282366920938463463374607431768211456",
        "1606938044258990275541962092341162602522202993782792835301375",
    ][boundary]
        .into()
}
