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

#[derive(Default)]
struct IntentStamps(RefCell<Vec<(u8, u64, Field)>>);
impl intents_fixture::ledger_contract::Witnesses<u64> for IntentStamps {
    fn stamp(
        &self,
        ctx: WitnessContext<'_, u64, intents_fixture::ledger_contract::LedgerView<'_>>,
        step: runtime::BoundedUint<255>,
    ) -> (u64, ()) {
        self.0.borrow_mut().push((
            step.value() as u8,
            *ctx.private_state,
            ctx.ledger.marker().unwrap(),
        ));
        (*ctx.private_state + 1, ())
    }
}
#[test]
fn raw_input_output_intents_preserve_official_coin_domains_cursor_and_branches() {
    use intents_fixture::{ledger_contract as c, types};
    for user in [false, true] {
        for selected in [false, true] {
            let coin = types::ShieldedCoinInfo {
                nonce: runtime::FixedBytes::new([1; 32]),
                color: runtime::FixedBytes::new([2; 32]),
                value: runtime::BoundedUint::new(42).unwrap(),
            };
            let input = types::QualifiedShieldedCoinInfo {
                nonce: coin.nonce,
                color: coin.color,
                value: coin.value,
                mt_index: runtime::BoundedUint::new(17).unwrap(),
            };
            let recipient = types::Either {
                is_left: user,
                left: types::ZswapCoinPublicKey {
                    bytes: runtime::FixedBytes::new([3; 32]),
                },
                right: types::ContractAddress {
                    bytes: runtime::FixedBytes::new([4; 32]),
                },
            };
            let context = || {
                let mut c = c::initial_state(ConstructorContext::new(7_u64))
                    .unwrap()
                    .into_circuit_context(ContractAddress::default());
                c.set_zswap_output_start(9).unwrap();
                c
            };
            let w: [IntentStamps; 4] = std::array::from_fn(|_| IntentStamps::default());
            let native = c::exercise(
                context(),
                &w[0],
                input.clone(),
                coin.clone(),
                recipient.clone(),
                selected,
                true,
            );
            let recorded = c::recorded::exercise(
                context(),
                &w[1],
                input.clone(),
                coin.clone(),
                recipient.clone(),
                selected,
                true,
            );
            let functional = c::functional::exercise::Program::new(
                input.clone(),
                coin.clone(),
                recipient.clone(),
                selected,
                true,
            )
            .execute_native(context(), &w[2]);
            let functional_recorded = c::functional::exercise::Program::new(
                input.clone(),
                coin.clone(),
                recipient.clone(),
                selected,
                true,
            )
            .execute_recorded(context(), &w[3]);
            // Generic outcomes additionally checks state/effects/gas/public replay.
            for candidate in [
                &recorded.as_ref().unwrap().execution,
                functional.as_ref().unwrap(),
                &functional_recorded.as_ref().unwrap().execution,
            ] {
                assert_eq!(
                    candidate.context.circuit_zswap(),
                    native.as_ref().unwrap().context.circuit_zswap()
                );
                assert_eq!(
                    candidate.context.query.call_context.com_indices,
                    native
                        .as_ref()
                        .unwrap()
                        .context
                        .query
                        .call_context
                        .com_indices
                );
            }
            let out = outcomes(native, recorded, functional, functional_recorded, None).unwrap();
            let plan = out.context.circuit_zswap();
            let official = runtime::ledger::coin_info_from_compact(coin.nonce, coin.color, 42);
            assert_eq!(plan.inputs(), &[official.qualify(17)]);
            assert_eq!(plan.outputs().len(), usize::from(selected));
            assert_eq!(plan.next_index(), if selected { 10 } else { 9 });
            if selected {
                let recipient = runtime::ledger::coin_recipient_from_compact(
                    user,
                    recipient.left.bytes,
                    recipient.right.bytes,
                );
                let output = &plan.outputs()[0];
                assert_eq!(output.coin, official);
                assert_eq!(output.recipient, recipient);
                assert_eq!(output.provisional_index, 9);
                assert_eq!(
                    *out.context
                        .query
                        .call_context
                        .com_indices
                        .get(&official.commitment(&recipient))
                        .unwrap(),
                    9
                );
            } else {
                assert!(out.context.query.call_context.com_indices.is_empty());
            }
            assert_eq!(out.context.private_state, 9);
            let mut private = vec![
                runtime::fab::AlignedValue::from(()),
                runtime::fab::AlignedValue::from(()),
            ];
            if selected {
                private.push(runtime::fab::AlignedValue::from(()));
            }
            private.push(runtime::fab::AlignedValue::from(()));
            assert_eq!(out.private_transcript_outputs, private);
            assert_eq!(
                intents_fixture::ledger_slots::marker
                    .inspect(out.context.query.state.get_ref())
                    .unwrap(),
                Field::from(9_u64)
            );
            for witness in w {
                assert_eq!(
                    *witness.0.borrow(),
                    [(1, 7, Field::from(1_u64)), (2, 8, Field::from(1_u64))]
                );
            }
        }
    }
}
#[test]
fn raw_intent_guard_runs_before_witnesses_or_cursor_overflow() {
    use intents_fixture::{ledger_contract as c, types};
    let coin = types::ShieldedCoinInfo {
        nonce: runtime::FixedBytes::new([1; 32]),
        color: runtime::FixedBytes::new([2; 32]),
        value: runtime::BoundedUint::new(42).unwrap(),
    };
    let input = types::QualifiedShieldedCoinInfo {
        nonce: coin.nonce,
        color: coin.color,
        value: coin.value,
        mt_index: runtime::BoundedUint::new(17).unwrap(),
    };
    let recipient = types::Either {
        is_left: true,
        left: types::ZswapCoinPublicKey {
            bytes: runtime::FixedBytes::new([3; 32]),
        },
        right: types::ContractAddress {
            bytes: runtime::FixedBytes::new([4; 32]),
        },
    };
    let context = || {
        let mut c = c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default());
        c.set_zswap_output_start(u64::MAX).unwrap();
        c
    };
    let w: [IntentStamps; 4] = std::array::from_fn(|_| IntentStamps::default());
    outcomes(
        c::exercise(
            context(),
            &w[0],
            input.clone(),
            coin.clone(),
            recipient.clone(),
            true,
            false,
        ),
        c::recorded::exercise(
            context(),
            &w[1],
            input.clone(),
            coin.clone(),
            recipient.clone(),
            true,
            false,
        ),
        c::functional::exercise::Program::new(
            input.clone(),
            coin.clone(),
            recipient.clone(),
            true,
            false,
        )
        .execute_native(context(), &w[2]),
        c::functional::exercise::Program::new(input, coin, recipient, true, false)
            .execute_recorded(context(), &w[3]),
        Some("failed assert: intents denied"),
    );
    for witness in w {
        assert!(witness.0.borrow().is_empty());
    }
}
