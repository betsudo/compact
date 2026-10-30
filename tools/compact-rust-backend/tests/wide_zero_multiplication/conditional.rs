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

use conditional_fixture::{ledger_contract as c, ledger_slots as slots, runtime as r};

#[test]
fn conditional_return_keeps_the_singleton_in_both_arms_and_all_routes() {
    type U = r::WideUint<4722366482869645213695, { u128::MAX }>;
    let reference: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("COMPACT_WIDE_ZERO_TS_REFERENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rows = reference["rows"]["conditional"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    for (index, bytes) in [
        vec![],
        vec![1],
        {
            let mut b = vec![0; 17];
            b[16] = 1;
            b
        },
        vec![255; 25],
    ]
    .into_iter()
    .enumerate()
    {
        let x = U::from_le_bytes(&bytes).unwrap();
        for active in [false, true] {
            let context = || {
                c::initial_state(r::context::ConstructorContext::new(42_u64))
                    .unwrap()
                    .into_circuit_context(Default::default())
            };
            let manual = slots::calls
                .record_increment(r::recording::RecordingFrame::new(context()), 1)
                .unwrap()
                .finish(r::BoundedUint::<0>::new(0).unwrap());
            let native = c::conditional(context(), x, active).unwrap();
            let decimal = [
                "0",
                "1",
                "340282366920938463463374607431768211456",
                "1606938044258990275541962092341162602522202993782792835301375",
            ][index];
            let found: Vec<_> = rows
                .iter()
                .filter(|row| row["input"] == decimal && row["active"] == active)
                .collect();
            assert_eq!(found.len(), 1);
            assert_eq!(found[0]["result"], native.result.value().to_string());
            assert_eq!(
                found[0]["calls"],
                slots::calls
                    .inspect(native.context.query.state.get_ref())
                    .unwrap()
                    .value()
                    .to_string()
            );
            assert_eq!(found[0]["privateState"], native.context.private_state);
            let recorded = c::recorded::conditional(context(), x, active).unwrap();
            let functional = c::functional::conditional::Program::new(x, active)
                .execute_native(context())
                .unwrap();
            let functional_recorded = c::functional::conditional::Program::new(x, active)
                .execute_recorded(context())
                .unwrap();
            for actual in [
                &native,
                &recorded.execution,
                &functional,
                &functional_recorded.execution,
            ] {
                let zero: r::BoundedUint<0> = actual.result;
                assert_eq!(zero.value(), 0);
                assert_eq!(
                    actual.context.query.state,
                    manual.execution.context.query.state
                );
                assert_eq!(
                    actual.context.query.effects,
                    manual.execution.context.query.effects
                );
                assert_eq!(actual.context.private_state, 42);
                assert_eq!(
                    actual.private_transcript_outputs,
                    manual.execution.private_transcript_outputs
                );
                assert_eq!(actual.gas_cost, manual.execution.gas_cost);
            }
            for actual in [&recorded, &functional_recorded] {
                assert_eq!(actual.public.verify_ops(), manual.public.verify_ops());
                let replay = actual
                    .public
                    .initial()
                    .query(
                        actual.public.verify_ops(),
                        None,
                        &actual.execution.context.cost_model,
                    )
                    .unwrap();
                assert_eq!(replay.context.state, actual.execution.context.query.state);
                assert_eq!(
                    replay.context.effects,
                    actual.execution.context.query.effects
                );
            }
        }
    }
}
