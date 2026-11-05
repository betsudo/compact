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

#[cfg(test)]
use candidate::ledger_contract as api;
use candidate::runtime as r;
use r::{BoundedUint, FixedVector};
pub type U = BoundedUint<255>;
pub type Input = FixedVector<(FixedVector<(FixedVector<U, 2>, U), 2>, U), 2>;
fn u(value: u128) -> U {
    U::new(value).unwrap()
}
pub fn input() -> Input {
    FixedVector::new([
        (
            FixedVector::new([
                (FixedVector::new([u(0), u(255)]), u(17)),
                (FixedVector::new([u(13), u(128)]), u(9)),
            ]),
            u(33),
        ),
        (
            FixedVector::new([
                (FixedVector::new([u(254), u(1)]), u(23)),
                (FixedVector::new([u(5), u(219)]), u(42)),
            ]),
            u(67),
        ),
    ])
}
#[cfg(test)]
fn replay(
    out: r::recording::RecordedCircuitResult<u64, Input>,
) -> r::context::CircuitResult<u64, Input> {
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
    out.execution
}
#[test]
fn complete_typed_carrier_executes_all_four_routes() {
    let mut expected = None;
    for route in 0..4 {
        let context = api::initial_state(r::context::ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(Default::default());
        let original_state = context.query.state.clone();
        let value = input();
        let result: r::context::CircuitResult<u64, Input> = match route {
            0 => api::run(context, value.clone()).unwrap(),
            1 => replay(api::recorded::run(context, value.clone()).unwrap()),
            2 => api::functional::run::Program::new(value.clone())
                .execute_native(context)
                .unwrap(),
            3 => replay(
                api::functional::run::Program::new(value.clone())
                    .execute_recorded(context)
                    .unwrap(),
            ),
            _ => unreachable!(),
        };
        assert_eq!(result.result, value);
        assert_eq!(result.context.private_state, 7);
        assert!(result.private_transcript_outputs.is_empty());
        assert_eq!(
            candidate::ledger_slots::seen
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            r::Field::from(1_u64)
        );
        assert_eq!(
            candidate::ledger_slots::seen
                .inspect(original_state.get_ref())
                .unwrap(),
            r::Field::from(0_u64)
        );
        let observed = (
            result.context.query.state,
            result.context.query.effects,
            result.gas_cost,
        );
        if let Some(expected) = &expected {
            assert_eq!(&observed, expected);
        } else {
            expected = Some(observed);
        }
    }
}
