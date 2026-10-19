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

//! Typed logical Maps; physical paths and codecs belong to interpreters.
use super::{Executor, SlotId};
use crate::CompactError;
use std::marker::PhantomData;

/// Contract-local declaration identity; handwritten construction certifies no type.
/// Structural child Maps have no scalar lookup/insert implementation:
/// ```compile_fail,E0599
/// use midnight_compact_runtime::{Field, context::CircuitContext, slots::MapNode,
///     functional::{Map, MapLookup, SlotId, ledger8::{NativeLedger8,Ledger8Bindings}}};
/// fn lookup(context:CircuitContext<()>) {
///     NativeLedger8::new(context,Ledger8Bindings::new(&[]).unwrap())
///         .map_lookup(Map::<bool,MapNode<bool,Field>>::new(SlotId::new(0)),true);
/// }
/// ```
/// ```compile_fail,E0599
/// use midnight_compact_runtime::{Field, context::CircuitContext, slots::MapNode,
///     functional::{Map, MapWrite, SlotId, ledger8::{NativeLedger8,Ledger8Bindings}}};
/// fn reset(context:CircuitContext<()>) {
///     NativeLedger8::new(context,Ledger8Bindings::new(&[]).unwrap())
///         .map_reset(Map::<bool,MapNode<bool,Field>>::new(SlotId::new(0)));
/// }
/// ```
pub struct Map<K, V> {
    id: SlotId,
    types: PhantomData<fn(K) -> V>,
}
impl<K, V> Map<K, V> {
    pub const fn new(id: SlotId) -> Self {
        Self {
            id,
            types: PhantomData,
        }
    }
    pub const fn id(self) -> SlotId {
        self.id
    }
}
impl<K, V> Copy for Map<K, V> {}
impl<K, V> Clone for Map<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait MapRead<K, V>: Executor {
    fn map_member(self, map: Map<K, V>, key: K) -> Result<(Self, bool), CompactError>;
    fn map_size(self, map: Map<K, V>) -> Result<(Self, u64), CompactError>;
    fn map_is_empty(self, map: Map<K, V>) -> Result<(Self, bool), CompactError>;
}
pub trait MapWrite<K, V>: Executor {
    fn map_insert(self, map: Map<K, V>, key: K, value: V) -> Result<Self, CompactError>;
    fn map_reset(self, map: Map<K, V>) -> Result<Self, CompactError>;
    fn map_insert_default(self, map: Map<K, V>, key: K) -> Result<Self, CompactError>
    where
        V: Default;
}

/// Scalar lookup needs a value codec; structural inspection does not.
pub trait MapLookup<K, V>: Executor {
    fn map_lookup(self, map: Map<K, V>, key: K) -> Result<(Self, V), CompactError>;
}
/// Removing a nested child needs its key, never a scalar value codec.
pub trait MapRemove<K, V>: Executor {
    fn map_remove(self, map: Map<K, V>, key: K) -> Result<Self, CompactError>;
}
