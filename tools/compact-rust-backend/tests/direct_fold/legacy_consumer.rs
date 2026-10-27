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

//! Legacy native compatibility, distinct from new source Fold's TS order.
use legacy_two::runtime as r;

macro_rules! legacy_order {
    ($test:ident, $module:ident, $length:literal) => {
        #[test]
        fn $test() {
            use $module::ledger_contract as c;
            let (mut cases, mut executions, mut replays) = (0, 0, 0);
            for seed_ok in [false, true] {
                for source_ok in [false, true] {
                    for reducer_ok in [false, true] {
                        cases += 1;
                        let expected_error = if !source_ok {
                            Some("source failed")
                        } else if !seed_ok {
                            Some("seed failed")
                        } else if $length > 0 && !reducer_ok {
                            Some("step failed")
                        } else {
                            None
                        };
                        for recorded in [false, true] {
                            executions += 1;
                            let context = c::initial_state(
                                r::context::ConstructorContext::new(7_u64),
                            )
                            .unwrap()
                            .into_circuit_context(Default::default());
                            let values = r::FixedVector::new(std::array::from_fn::<_, $length, _>(
                                |i| r::Field::from(if !reducer_ok { 7_u64 } else { i as u64 + 2 }),
                            ));
                            let result = if recorded {
                                c::recorded::run(context, r::Field::from(1_u64), values, seed_ok, source_ok)
                                    .map(|out| {
                                        let replay = out.public.initial().query(
                                            out.public.verify_ops(),
                                            None,
                                            &out.execution.context.cost_model,
                                        ).unwrap();
                                        assert_eq!(replay.context.state, out.execution.context.query.state);
                                        assert_eq!(replay.context.effects, out.execution.context.query.effects);
                                        replays += 1;
                                        out.execution
                                    })
                            } else {
                                c::run(context, r::Field::from(1_u64), values, seed_ok, source_ok)
                            };
                            match (result, expected_error) {
                                (Err(error), Some(message)) => {
                                    assert_eq!(error, r::CompactError::AssertionFailed(message.into()));
                                }
                                (Ok(out), None) => {
                                    let expected = r::Field::from(if $length == 0 { 1_u64 } else { 123 });
                                    assert_eq!(out.result, expected);
                                    assert_eq!(out.context.private_state, 7);
                                    assert!(out.private_transcript_outputs.is_empty());
                                    assert_eq!(r::slots::CellSlot::<r::Field>::new(&[0])
                                        .inspect(out.context.query.state.get_ref()).unwrap(), expected);
                                }
                                _ => panic!("legacy order mismatch: seed={seed_ok}, source={source_ok}, reducer={reducer_ok}, recorded={recorded}"),
                            }
                        }
                    }
                }
            }
            println!("COMPACT_LEGACY_FOLD {} {} {} {}", stringify!($test), cases, executions, replays);
        }
    };
}
legacy_order!(legacy_empty_operand_failures, legacy_empty, 0);
legacy_order!(legacy_two_operand_failures, legacy_two, 2);
