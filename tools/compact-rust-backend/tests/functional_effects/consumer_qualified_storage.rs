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

#[path = "qualified_cell_support.rs"]
mod qualified_cell_support;
#[path = "qualified_set_support.rs"]
mod qualified_set_support;
fn qualified_routes<T: Debug + PartialEq>(
    native: Result<CircuitResult<u64, T>, CompactError>,
    recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    functional: Result<CircuitResult<u64, T>, CompactError>,
    functional_recorded: Result<RecordedCircuitResult<u64, T>, CompactError>,
    manual: Result<RecordedCircuitResult<u64, T>, CompactError>,
) -> Option<CircuitResult<u64, T>> {
    if let Err(error) = manual {
        match &error {
            CompactError::InvalidLedgerCell(message) => {
                assert!(message.contains("Coin commitment not found"))
            }
            CompactError::AssertionFailed(message) => assert!(matches!(
                message.as_str(),
                "coin refused" | "recipient refused"
            )),
            _ => panic!("unexpected official qualified reference error: {error}"),
        }
        return outcomes(
            native,
            recorded,
            functional,
            functional_recorded,
            Some(&error.to_string()),
        );
    }
    let manual = manual.unwrap();
    for actual in [
        native.as_ref().unwrap(),
        &recorded.as_ref().unwrap().execution,
        functional.as_ref().unwrap(),
        &functional_recorded.as_ref().unwrap().execution,
    ] {
        assert_eq!(actual.result, manual.execution.result);
        assert_eq!(
            actual.context.query.state,
            manual.execution.context.query.state
        );
        assert_eq!(
            actual.context.query.effects,
            manual.execution.context.query.effects
        );
        assert_eq!(
            actual.context.query.call_context.com_indices,
            manual.execution.context.query.call_context.com_indices
        );
        assert_eq!(
            actual.context.circuit_zswap(),
            manual.execution.context.circuit_zswap()
        );
        assert_eq!(
            actual.private_transcript_outputs,
            manual.execution.private_transcript_outputs
        );
        assert_eq!(
            actual.context.private_state,
            manual.execution.context.private_state
        );
        assert_eq!(actual.gas_cost, manual.execution.gas_cost);
    }
    assert_eq!(
        serde_json::to_value(recorded.as_ref().unwrap().public.verify_ops()).unwrap(),
        serde_json::to_value(manual.public.verify_ops()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(functional_recorded.as_ref().unwrap().public.verify_ops()).unwrap(),
        serde_json::to_value(manual.public.verify_ops()).unwrap()
    );
    outcomes(native, recorded, functional, functional_recorded, None)
}
#[test]
fn qualified_cell_uses_independent_committed_index_and_both_recipient_arms() {
    use qualified_cell_fixture::ledger_contract as c;
    use qualified_cell_support as s;
    for user in [false, true] {
        for index in [0, 13] {
            let coin = s::coin(42);
            let recipient = s::recipient(user);
            let context = || s::allocate(s::context(), &coin, &recipient, index);
            let result = qualified_routes(
                c::write_read(context(), coin.clone(), recipient.clone()),
                c::recorded::write_read(context(), coin.clone(), recipient.clone()),
                c::functional::write_read::Program::new(coin.clone(), recipient.clone())
                    .execute_native(context()),
                c::functional::write_read::Program::new(coin.clone(), recipient.clone())
                    .execute_recorded(context()),
                s::write_read(context(), &coin, &recipient),
            )
            .unwrap();
            assert_eq!(result.result.nonce, coin.nonce);
            assert_eq!(result.result.color, coin.color);
            assert_eq!(result.result.value.value(), 42);
            assert_eq!(result.result.mt_index.value(), index as u128);
            assert_eq!(result.context.private_state, 7);
            assert!(result.private_transcript_outputs.is_empty());
            assert!(result.context.circuit_zswap().outputs().is_empty());
        }
    }
}
#[test]
fn qualified_cell_witness_inputs_keep_exact_order_and_outputs() {
    use qualified_cell_fixture::ledger_contract as c;
    use qualified_cell_support as s;
    for user in [false, true] {
        let w: [s::Witness; 5] = std::array::from_fn(|_| s::Witness {
            coin: s::coin(42),
            recipient: s::recipient(user),
            ..Default::default()
        });
        let context = || s::allocate(s::context(), &w[0].coin, &w[0].recipient, 13);
        let result = qualified_routes(
            c::write_once(context(), &w[0]),
            c::recorded::write_once(context(), &w[1]),
            c::functional::write_once::Program::new().execute_native(context(), &w[2]),
            c::functional::write_once::Program::new().execute_recorded(context(), &w[3]),
            s::write_once(context(), &w[4]),
        )
        .unwrap();
        assert_eq!(result.context.private_state, 9);
        assert_eq!(
            result.private_transcript_outputs,
            [
                runtime::fab::AlignedValue::from(w[0].coin.clone()),
                runtime::fab::AlignedValue::from(w[0].recipient.clone())
            ]
        );
        assert_eq!(
            qualified_cell_fixture::ledger_slots::pot
                .inspect(result.context.query.state.get_ref())
                .unwrap()
                .mt_index
                .value(),
            13
        );
        for witness in w {
            assert_eq!(*witness.calls.borrow(), [("coin", 7), ("recipient", 8)]);
        }
    }
}
#[test]
fn qualified_cell_missing_and_wrong_recipient_allocations_stay_refused() {
    use qualified_cell_fixture::ledger_contract as c;
    use qualified_cell_support as s;
    for wrong in [false, true] {
        let coin = s::coin(42);
        let recipient = s::recipient(true);
        let context = || {
            if wrong {
                s::allocate(s::context(), &coin, &s::recipient(false), 13)
            } else {
                s::context()
            }
        };
        assert!(
            qualified_routes(
                c::write_read(context(), coin.clone(), recipient.clone()),
                c::recorded::write_read(context(), coin.clone(), recipient.clone()),
                c::functional::write_read::Program::new(coin.clone(), recipient.clone())
                    .execute_native(context()),
                c::functional::write_read::Program::new(coin.clone(), recipient.clone())
                    .execute_recorded(context()),
                s::write_read(context(), &coin, &recipient)
            )
            .is_none()
        );
    }
}
#[test]
fn qualified_set_insert_uses_official_index_without_creating_output_intents() {
    use qualified_set_fixture::ledger_contract as c;
    use qualified_set_support as s;
    for user in [false, true] {
        for index in [0, 13] {
            let coin = s::coin(42);
            let recipient = s::recipient(user);
            let context = || s::allocate(s::context(), &coin, &recipient, index);
            let result = qualified_routes(
                c::insert_once(context(), coin.clone(), recipient.clone()),
                c::recorded::insert_once(context(), coin.clone(), recipient.clone()),
                c::functional::insert_once::Program::new(coin.clone(), recipient.clone())
                    .execute_native(context()),
                c::functional::insert_once::Program::new(coin.clone(), recipient.clone())
                    .execute_recorded(context()),
                s::insert_once(context(), &coin, &recipient),
            )
            .unwrap();
            assert!(
                qualified_set_fixture::ledger_slots::coins
                    .inspect(result.context.query.state.get_ref())
                    .unwrap()
                    .member(s::qualified(&coin, index))
            );
            assert_eq!(
                qualified_set_fixture::ledger_slots::coins
                    .inspect(result.context.query.state.get_ref())
                    .unwrap()
                    .size()
                    .unwrap()
                    .value(),
                1
            );
            assert!(result.context.circuit_zswap().inputs().is_empty());
            assert!(result.context.circuit_zswap().outputs().is_empty());
            assert!(result.private_transcript_outputs.is_empty());
            assert_eq!(result.context.private_state, 7);
        }
    }
}
#[test]
fn qualified_set_branch_reuses_operands_and_preserves_metered_size_observations() {
    use qualified_set_fixture::ledger_contract as c;
    use qualified_set_support as s;
    for user in [false, true] {
        for selected in [false, true] {
            let coin = s::coin(42);
            let recipient = s::recipient(user);
            let w: [s::Witness; 5] = std::array::from_fn(|_| s::Witness {
                coin: s::witness_coin(),
                recipient: s::recipient(!user),
                ..Default::default()
            });
            let context = || {
                s::allocate(
                    s::allocate(s::context(), &coin, &recipient, 11),
                    &w[0].coin,
                    &w[0].recipient,
                    13,
                )
            };
            let result = qualified_routes(
                c::insert_selected(context(), &w[0], coin.clone(), recipient.clone(), selected),
                c::recorded::insert_selected(
                    context(),
                    &w[1],
                    coin.clone(),
                    recipient.clone(),
                    selected,
                ),
                c::functional::insert_selected::Program::new(
                    coin.clone(),
                    recipient.clone(),
                    selected,
                )
                .execute_native(context(), &w[2]),
                c::functional::insert_selected::Program::new(
                    coin.clone(),
                    recipient.clone(),
                    selected,
                )
                .execute_recorded(context(), &w[3]),
                s::insert_selected(context(), &coin, &recipient, selected, &w[4]),
            )
            .unwrap();
            let view = qualified_set_fixture::ledger_slots::coins
                .inspect(result.context.query.state.get_ref())
                .unwrap();
            assert_eq!(view.size().unwrap().value(), if selected { 2 } else { 1 });
            assert!(view.member(s::qualified(&coin, 11)));
            if selected {
                assert!(view.member(s::qualified(&w[0].coin, 13)));
            }
            assert_eq!(result.context.private_state, if selected { 9 } else { 7 });
            assert_eq!(
                result.private_transcript_outputs,
                if selected {
                    vec![
                        runtime::fab::AlignedValue::from(w[0].coin.clone()),
                        runtime::fab::AlignedValue::from(w[0].recipient.clone()),
                    ]
                } else {
                    vec![]
                }
            );
            for witness in w {
                assert_eq!(
                    *witness.calls.borrow(),
                    if selected {
                        vec![("coin", 7, 1), ("recipient", 8, 1)]
                    } else {
                        vec![]
                    }
                );
            }
        }
    }
}
#[test]
fn qualified_set_missing_or_wrong_recipient_allocation_is_not_inferred() {
    use qualified_set_fixture::ledger_contract as c;
    use qualified_set_support as s;
    for wrong in [false, true] {
        let coin = s::coin(42);
        let recipient = s::recipient(true);
        let context = || {
            if wrong {
                s::allocate(s::context(), &coin, &s::recipient(false), 13)
            } else {
                s::context()
            }
        };
        assert!(
            qualified_routes(
                c::insert_once(context(), coin.clone(), recipient.clone()),
                c::recorded::insert_once(context(), coin.clone(), recipient.clone()),
                c::functional::insert_once::Program::new(coin.clone(), recipient.clone())
                    .execute_native(context()),
                c::functional::insert_once::Program::new(coin.clone(), recipient.clone())
                    .execute_recorded(context()),
                s::insert_once(context(), &coin, &recipient)
            )
            .is_none()
        );
    }
}
#[test]
fn qualified_set_callback_failure_precedes_absent_second_commitment_allocation() {
    use qualified_set_fixture::ledger_contract as c;
    use qualified_set_support as s;
    for refuse in ["coin", "recipient"] {
        let coin = s::coin(42);
        let recipient = s::recipient(true);
        let w: [s::Witness; 5] = std::array::from_fn(|_| s::Witness {
            coin: s::witness_coin(),
            recipient: s::recipient(false),
            refuse: Some(refuse),
            ..Default::default()
        });
        let context = || s::allocate(s::context(), &coin, &recipient, 11);
        assert!(
            qualified_routes(
                c::insert_selected(context(), &w[0], coin.clone(), recipient.clone(), true),
                c::recorded::insert_selected(
                    context(),
                    &w[1],
                    coin.clone(),
                    recipient.clone(),
                    true
                ),
                c::functional::insert_selected::Program::new(coin.clone(), recipient.clone(), true)
                    .execute_native(context(), &w[2]),
                c::functional::insert_selected::Program::new(coin.clone(), recipient.clone(), true)
                    .execute_recorded(context(), &w[3]),
                s::insert_selected(context(), &coin, &recipient, true, &w[4])
            )
            .is_none()
        );
        let expected = if refuse == "coin" {
            vec![("coin", 7, 1)]
        } else {
            vec![("coin", 7, 1), ("recipient", 8, 1)]
        };
        for witness in w {
            assert_eq!(*witness.calls.borrow(), expected);
        }
    }
}
