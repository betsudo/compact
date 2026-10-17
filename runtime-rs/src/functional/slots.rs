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

//! Logical slot identities and leaf capabilities, without ledger or VM types.
use super::Executor;
use crate::CompactError;
use std::marker::PhantomData;

/// Contract-local identity assigned by the checked compiler, not a ledger path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotId(u32);
impl SlotId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A typed logical Cell. Its interpreter supplies the physical ledger binding.
/// Creating one in handwritten Rust does not establish declaration authority.
pub struct Cell<T> {
    id: SlotId,
    value: PhantomData<fn() -> T>,
}
impl<T> Cell<T> {
    pub const fn new(id: SlotId) -> Self {
        Self {
            id,
            value: PhantomData,
        }
    }
    pub const fn id(self) -> SlotId {
        self.id
    }
}
impl<T> Copy for Cell<T> {}
impl<T> Clone for Cell<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait CellRead<T>: Executor {
    fn read(self, cell: Cell<T>) -> Result<(Self, T), CompactError>;
}
pub trait CellWrite<T>: Executor {
    fn write(self, cell: Cell<T>, value: T) -> Result<Self, CompactError>;
}

/// A logical Counter declaration; its physical path belongs to the interpreter.
#[derive(Clone, Copy, Debug)]
pub struct Counter {
    id: SlotId,
}
impl Counter {
    pub const fn new(id: SlotId) -> Self {
        Self { id }
    }
    pub const fn id(self) -> SlotId {
        self.id
    }
}
pub trait CounterRead: Executor {
    fn read_counter(self, counter: Counter) -> Result<(Self, u64), CompactError>;
}
pub trait CounterIncrement: Executor {
    fn increment(self, counter: Counter, amount: u16) -> Result<Self, CompactError>;
}
