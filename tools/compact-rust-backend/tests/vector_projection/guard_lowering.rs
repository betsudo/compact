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

mod stateful_known {
    use guard_fixture::{ledger_contract as c, runtime as r};
    use std::cell::Cell;

    struct W(Cell<usize>);
    impl c::Witnesses<()> for W {
        fn values(
            &self,
            _: r::context::WitnessContext<'_, (), c::LedgerView<'_>>,
        ) -> ((), r::FixedVector<r::Field, 2>) {
            self.0.set(self.0.get() + 1);
            ((), r::FixedVector([17_u64, 29].map(r::Field::from)))
        }
    }
    fn context() -> r::context::CircuitContext<()> {
        c::initial_state(r::context::ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(Default::default())
    }
    #[test]
    fn source_failure_order_on_all_four_routes() {
        for active in [false, true] {
            for route in 0..4 {
                let w = W(Cell::new(0));
                let result = match route {
                    0 => c::guarded(context(), &w, active).map(|x| x.result),
                    1 => c::recorded::guarded(context(), &w, active).map(|x| x.execution.result),
                    2 => c::functional::guarded::Program::new(active)
                        .execute_native(context(), &w)
                        .map(|x| x.result),
                    _ => c::functional::guarded::Program::new(active)
                        .execute_recorded(context(), &w)
                        .map(|x| x.execution.result),
                };
                if active {
                    assert!(
                        matches!(result,Err(r::CompactError::AssertionFailed(message)) if message=="result of subtraction would be negative")
                    );
                } else {
                    assert_eq!(result.unwrap(), r::Field::from(0_u64));
                }
                assert_eq!(w.0.get(), if active { 1 } else { 0 });
            }
        }
    }
}

mod stateful_unknown {
    use guard_fixture::{ledger_contract as c, runtime as r};
    use std::cell::Cell;

    struct W(Cell<usize>);
    impl c::Witnesses<()> for W {
        fn values(
            &self,
            _: r::context::WitnessContext<'_, (), c::LedgerView<'_>>,
        ) -> ((), r::FixedVector<r::Field, 2>) {
            self.0.set(self.0.get() + 1);
            ((), r::FixedVector([17_u64, 29].map(r::Field::from)))
        }
    }
    fn context() -> r::context::CircuitContext<()> {
        c::initial_state(r::context::ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(Default::default())
    }
    #[test]
    fn source_failure_order_on_all_four_routes() {
        for active in [false, true] {
            for route in 0..4 {
                let w = W(Cell::new(0));
                let result = match route {
                    0 => c::guarded_unknown(context(), &w, active).map(|x| x.result),
                    1 => c::recorded::guarded_unknown(context(), &w, active)
                        .map(|x| x.execution.result),
                    2 => c::functional::guarded_unknown::Program::new(active)
                        .execute_native(context(), &w)
                        .map(|x| x.result),
                    _ => c::functional::guarded_unknown::Program::new(active)
                        .execute_recorded(context(), &w)
                        .map(|x| x.execution.result),
                };
                if active {
                    assert!(
                        matches!(result,Err(r::CompactError::AssertionFailed(message)) if message=="result of subtraction would be negative")
                    );
                } else {
                    assert_eq!(result.unwrap(), r::Field::from(0_u64));
                }
                assert_eq!(w.0.get(), if active { 0 } else { 0 });
            }
        }
    }
}

mod constructor_simple_known {
    use guard_constructor_known::{ledger_contract as c, runtime as r};

    #[test]
    fn constructor_keeps_guard_failure() {
        let result = c::initial_state(
            r::context::ConstructorContext::new(()),
            r::FixedVector([17_u64, 29].map(r::Field::from)),
        );
        assert!(
            matches!(result,Err(r::CompactError::AssertionFailed(message)) if message=="result of subtraction would be negative")
        );
    }
}

mod constructor_simple_unknown {
    use guard_constructor_unknown::{ledger_contract as c, runtime as r};

    #[test]
    fn constructor_keeps_guard_failure() {
        let result = c::initial_state(
            r::context::ConstructorContext::new(()),
            r::FixedVector([17_u64, 29].map(r::Field::from)),
        );
        assert!(
            matches!(result,Err(r::CompactError::AssertionFailed(message)) if message=="result of subtraction would be negative")
        );
    }
}
