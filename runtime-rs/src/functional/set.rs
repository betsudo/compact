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

//! Typed logical Sets; coin allocation is a separate capability.
use super::{Executor, SlotId};
use crate::CompactError;
use std::marker::PhantomData;
/// Compiler-local declaration identity, not certification of handwritten values.
pub struct Set<T> {
    id: SlotId,
    value: PhantomData<fn() -> T>,
}
impl<T> Set<T> {
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
impl<T> Copy for Set<T> {}
impl<T> Clone for Set<T> {
    fn clone(&self) -> Self {
        *self
    }
}
pub trait SetRead<T>: Executor {
    fn set_member(self, set: Set<T>, value: T) -> Result<(Self, bool), CompactError>;
    fn set_size(self, set: Set<T>) -> Result<(Self, u64), CompactError>;
    fn set_is_empty(self, set: Set<T>) -> Result<(Self, bool), CompactError>;
}
pub trait SetWrite<T>: Executor {
    fn set_insert(self, set: Set<T>, value: T) -> Result<Self, CompactError>;
    fn set_remove(self, set: Set<T>, value: T) -> Result<Self, CompactError>;
    fn set_reset(self, set: Set<T>) -> Result<Self, CompactError>;
}
