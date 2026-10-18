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

//! Merkle carrier operations share the existing eager-operand contract.
use super::super::{
    Merkle, MerkleCheckRoot, MerkleInsert, MerkleInsertHash, MerkleInspect, MerkleReset,
    MerkleResetHistory,
};
use super::{Circuit, Executor};
use crate::{BoundedUint, FixedBytes};
impl<E: Executor> Circuit<E, ()> {
    pub fn merkle_is_full<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
    ) -> Circuit<E, bool>
    where
        E: MerkleInspect<T, DEPTH, H>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.merkle_is_full(tree)),
        }
    }
    pub fn merkle_check_root<T, R, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        root: R,
    ) -> Circuit<E, bool>
    where
        E: MerkleCheckRoot<T, R, DEPTH, H>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.merkle_check_root(tree, root)),
        }
    }
    pub fn merkle_insert<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        value: T,
    ) -> Self
    where
        E: MerkleInsert<T, DEPTH, H>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.merkle_insert(tree, value).map(|next| (next, ()))
            }),
        }
    }
    pub fn merkle_insert_index<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        value: T,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Self
    where
        E: MerkleInsert<T, DEPTH, H>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .merkle_insert_index(tree, value, index)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn merkle_insert_default<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Self
    where
        E: MerkleInsert<T, DEPTH, H>,
        T: Default,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .merkle_insert_default(tree, index)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn merkle_insert_hash<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        hash: FixedBytes<32>,
    ) -> Self
    where
        E: MerkleInsertHash<T, DEPTH, H>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .merkle_insert_hash(tree, hash)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn merkle_insert_hash_index<T, const DEPTH: u8, const H: bool>(
        self,
        tree: Merkle<T, DEPTH, H>,
        hash: FixedBytes<32>,
        index: BoundedUint<{ u64::MAX as u128 }>,
    ) -> Self
    where
        E: MerkleInsertHash<T, DEPTH, H>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .merkle_insert_hash_index(tree, hash, index)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn merkle_reset<T, const DEPTH: u8, const H: bool>(self, tree: Merkle<T, DEPTH, H>) -> Self
    where
        E: MerkleReset<T, DEPTH, H>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.merkle_reset(tree).map(|next| (next, ()))),
        }
    }
    pub fn merkle_reset_history<T, const DEPTH: u8>(self, tree: Merkle<T, DEPTH, true>) -> Self
    where
        E: MerkleResetHistory<T, DEPTH>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.merkle_reset_history(tree).map(|next| (next, ()))
            }),
        }
    }
}
