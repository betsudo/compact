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

//! Additional Counter steps retain private consuming carrier ownership.
use super::super::{Counter, CounterCompare, CounterDecrement, CounterReset};
use super::{Circuit, Executor};
impl<E: Executor> Circuit<E, ()> {
    pub fn counter_less_than(self, counter: Counter, threshold: u64) -> Circuit<E, bool>
    where
        E: CounterCompare,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.counter_less_than(counter, threshold)),
        }
    }
    pub fn decrement(self, counter: Counter, amount: u16) -> Self
    where
        E: CounterDecrement,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.decrement(counter, amount).map(|next| (next, ()))
            }),
        }
    }
    pub fn reset_counter(self, counter: Counter) -> Self
    where
        E: CounterReset,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.reset_counter(counter).map(|next| (next, ()))),
        }
    }
}
