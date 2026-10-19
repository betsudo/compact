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

fn counter_more_context() -> runtime::context::CircuitContext<u64> {
    counter_more_fixture::ledger_contract::initial_state(ConstructorContext::new(7_u64))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}
macro_rules! counter_more_routes {
    ($entry:ident, [$($arg:expr),*], $error:expr) => {{
        use counter_more_fixture::ledger_contract as c;
        outcomes(c::$entry(counter_more_context(), $($arg),*),
            c::recorded::$entry(counter_more_context(), $($arg),*),
            c::functional::$entry::Program::new($($arg),*).execute_native(counter_more_context()),
            c::functional::$entry::Program::new($($arg),*).execute_recorded(counter_more_context()), $error)
    }};
}
#[test]
fn remaining_counter_roundtrip_retains_all_u16_amount_boundaries() {
    for amount in [0_u128, 1, 65535] {
        let amount = runtime::BoundedUint::<65535>::new(amount).unwrap();
        let result = counter_more_routes!(roundtrip, [amount], None).unwrap();
        assert_eq!(result.result.value(), 7);
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
        assert!(counter_more_fixture::ledger_slots::total.path().len() > 1);
    }
}
#[test]
fn remaining_counter_comparison_keeps_strict_and_full_width_thresholds() {
    for threshold in [0_u64, 7, 8, u64::MAX] {
        let argument =
            runtime::BoundedUint::<18446744073709551615>::new(threshold as u128).unwrap();
        let result = counter_more_routes!(compare, [argument], None).unwrap();
        assert_eq!(result.result, 7 < threshold);
        assert_eq!(
            result.context.query.state,
            counter_more_context().query.state
        );
    }
}
#[test]
fn remaining_counter_reset_has_an_immediate_zero_read() {
    let result = counter_more_routes!(reset, [], None).unwrap();
    assert_eq!(result.result.value(), 0);
    assert_eq!(result.context.private_state, 7);
    assert!(result.private_transcript_outputs.is_empty());
}
#[test]
fn remaining_counter_guard_precedes_decrement() {
    for permit in [false, true] {
        let error = if permit {
            None
        } else {
            Some("failed assert: counter decrement denied")
        };
        let result = counter_more_routes!(guarded, [permit], error);
        if let Some(result) = result {
            assert_eq!(result.result.value(), 6);
        }
    }
}
