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

//! Logical Merkle descriptors retain checked depth and history kind.
use super::{Executor, SlotId};
use crate::{BoundedUint, CompactError, FixedBytes};
use std::marker::PhantomData;
pub struct Merkle<T, const DEPTH: u8, const HISTORIC: bool> {
    id: SlotId,
    leaf: PhantomData<fn() -> T>,
}
impl<T, const DEPTH: u8, const HISTORIC: bool> Merkle<T, DEPTH, HISTORIC> {
    pub const fn new(id: SlotId) -> Self {
        Self {
            id,
            leaf: PhantomData,
        }
    }
    pub const fn id(self) -> SlotId {
        self.id
    }
}
impl<T, const D: u8, const H: bool> Copy for Merkle<T, D, H> {}
impl<T, const D: u8, const H: bool> Clone for Merkle<T, D, H> {
    fn clone(&self) -> Self {
        *self
    }
}
pub trait MerkleInspect<T, const D: u8, const H: bool>: Executor {
    fn merkle_is_full(self, tree: Merkle<T, D, H>) -> Result<(Self, bool), CompactError>;
}
pub trait MerkleCheckRoot<T, R, const D: u8, const H: bool>: Executor {
    fn merkle_check_root(
        self,
        tree: Merkle<T, D, H>,
        root: R,
    ) -> Result<(Self, bool), CompactError>;
}
pub trait MerkleInsert<T, const D: u8, const H: bool>: Executor {
    fn merkle_insert(self, tree: Merkle<T, D, H>, value: T) -> Result<Self, CompactError>;
    fn merkle_insert_index(
        self,
        tree: Merkle<T, D, H>,
        value: T,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Result<Self, CompactError>;
    fn merkle_insert_default(
        self,
        tree: Merkle<T, D, H>,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Result<Self, CompactError>
    where
        T: Default;
}
pub trait MerkleInsertHash<T, const D: u8, const H: bool>: Executor {
    fn merkle_insert_hash(
        self,
        tree: Merkle<T, D, H>,
        hash: FixedBytes<32>,
    ) -> Result<Self, CompactError>;
    fn merkle_insert_hash_index(
        self,
        tree: Merkle<T, D, H>,
        hash: FixedBytes<32>,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Result<Self, CompactError>;
}
pub trait MerkleReset<T, const D: u8, const H: bool>: Executor {
    fn merkle_reset(self, tree: Merkle<T, D, H>) -> Result<Self, CompactError>;
}
/// Only historic descriptors can select history reset.
pub trait MerkleResetHistory<T, const D: u8>: Executor {
    fn merkle_reset_history(self, tree: Merkle<T, D, true>) -> Result<Self, CompactError>;
}
