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

//! A stable consuming execution carrier, not a stored effect graph.
use super::{
    Cell, CellRead, CellWrite, Counter, CounterIncrement, CounterRead, Witness, WitnessCall,
};
use crate::CompactError;
mod counters;
mod lists;
mod maps;
mod sets;

/// A selected interpreter owns its execution state and final result policy.
/// Custom implementations are not certified compiler or ledger authorities.
pub trait Executor: Sized {
    type Outcome<T>;
    fn finish<T>(self, value: T) -> Self::Outcome<T>;
}

/// One checked assertion. Messages are source literals in this prototype.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    condition: bool,
    message: &'static str,
}
impl Rule {
    pub const fn new(condition: bool, message: &'static str) -> Self {
        Self { condition, message }
    }
}

/// Owns exactly one success state or a terminal error. It is neither Clone nor
/// recoverable. Continuations execute immediately during a program's run only.
///
/// Method operands still follow ordinary eager Rust evaluation. All step and
/// result methods share the [module-level operand contract](super): use `bind`
/// or `try_returning` to prevent operand evaluation after a previous failure.
#[must_use = "finish the execution or propagate its failure"]
pub struct Circuit<E: Executor, T> {
    state: Result<(E, T), CompactError>,
}
impl<E: Executor> Circuit<E, ()> {
    pub fn begin(executor: E) -> Self {
        Self {
            state: Ok((executor, ())),
        }
    }

    pub fn read<T>(self, cell: Cell<T>) -> Circuit<E, T>
    where
        E: CellRead<T>,
    {
        Circuit {
            state: self.state.and_then(|(executor, ())| executor.read(cell)),
        }
    }

    /// Arguments are ordinary Rust expressions. Generated code evaluates
    /// fallible/effectful operands inside a successful bind, never eagerly here.
    pub fn write<T>(self, cell: Cell<T>, value: T) -> Self
    where
        E: CellWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.write(cell, value).map(|next| (next, ()))),
        }
    }

    pub fn read_counter(self, counter: Counter) -> Circuit<E, u64>
    where
        E: CounterRead,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.read_counter(counter)),
        }
    }

    pub fn increment(self, counter: Counter, amount: u16) -> Self
    where
        E: CounterIncrement,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.increment(counter, amount).map(|next| (next, ()))
            }),
        }
    }

    pub fn witness<Args, T>(self, witness: Witness<Args, T>, arguments: Args) -> Circuit<E, T>
    where
        E: WitnessCall<Args, T>,
    {
        self.witness_in(witness, arguments)
    }

    /// Execute a witness using an explicitly selected interpreter strategy.
    /// Like `witness`, ordinary arguments are eager; defer fallible producers
    /// inside a successful bind. Strategy selection does not certify callbacks.
    pub fn witness_in<Args, T, Mode>(
        self,
        witness: Witness<Args, T, Mode>,
        arguments: Args,
    ) -> Circuit<E, T>
    where
        E: WitnessCall<Args, T, Mode>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.witness(witness, arguments)),
        }
    }

    pub fn require(self, rule: Rule) -> Self {
        Self {
            state: self.state.and_then(|(executor, ())| {
                if rule.condition {
                    Ok((executor, ()))
                } else {
                    Err(CompactError::AssertionFailed(rule.message.to_owned()))
                }
            }),
        }
    }

    /// Evaluate a checked fallible pure producer only after all previous steps
    /// succeed. The callback is invoked here and never stored in the carrier.
    pub fn try_returning<T, F>(self, producer: F) -> Circuit<E, T>
    where
        F: FnOnce() -> Result<T, CompactError>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| producer().map(|value| (executor, value))),
        }
    }

    pub fn returning<T>(self, value: T) -> Circuit<E, T> {
        Circuit {
            state: self.state.map(|(executor, ())| (executor, value)),
        }
    }
}
impl<E: Executor, T> Circuit<E, T> {
    pub fn bind<U, F>(self, continuation: F) -> Circuit<E, U>
    where
        F: FnOnce(T, Circuit<E, ()>) -> Circuit<E, U>,
    {
        match self.state {
            Ok((executor, value)) => continuation(value, Circuit::begin(executor)),
            Err(error) => Circuit { state: Err(error) },
        }
    }

    pub fn finish(self) -> Result<E::Outcome<T>, CompactError> {
        self.state.map(|(executor, value)| executor.finish(value))
    }
}
