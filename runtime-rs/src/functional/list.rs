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

//! Typed logical Lists; the interpreter owns physical state and Maybe codecs.
use super::{Executor, SlotId};
use crate::CompactError;
use std::marker::PhantomData;
/// A checked declaration identity; handwritten construction confers no authority.
pub struct List<T> {
    id: SlotId,
    element: PhantomData<fn() -> T>,
}
impl<T> List<T> {
    pub const fn new(id: SlotId) -> Self {
        Self {
            id,
            element: PhantomData,
        }
    }
    pub const fn id(self) -> SlotId {
        self.id
    }
}
impl<T> Copy for List<T> {}
impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Inspection does not require an element default or a Maybe result codec.
pub trait ListRead<T>: Executor {
    fn list_length(self, list: List<T>) -> Result<(Self, u64), CompactError>;
    fn list_is_empty(self, list: List<T>) -> Result<(Self, bool), CompactError>;
}
/// M is the existing checked generated Maybe carrier, never an inferred Option.
pub trait ListHead<T, M>: Executor {
    fn list_head(self, list: List<T>) -> Result<(Self, M), CompactError>;
}
pub trait ListWrite<T>: Executor {
    fn list_push_front(self, list: List<T>, value: T) -> Result<Self, CompactError>;
    fn list_pop_front(self, list: List<T>) -> Result<Self, CompactError>;
    fn list_reset(self, list: List<T>) -> Result<Self, CompactError>;
}
