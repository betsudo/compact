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

//! Mechanical Merkle bridges reuse existing typed slot and frame semantics.
use super::super::{
    Merkle, MerkleCheckRoot, MerkleInsert, MerkleInsertHash, MerkleInspect, MerkleReset,
    MerkleResetHistory,
};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    BoundedUint, CompactError, FixedBytes,
    ledger::{CellValue, DB},
};
macro_rules! native_merkle {
    ($historic:literal) => {
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInspect<T, DEPTH, $historic>
            for NativeLedger8<'_, Private, D, W>
        {
            fn merkle_is_full(
                self,
                tree: Merkle<T, DEPTH, $historic>,
            ) -> Result<(Self, bool), CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, value) = self.frame.apply(|context| slot.is_full(context))?;
                Ok((
                    Self {
                        frame,
                        bindings: self.bindings,
                        witnesses: self.witnesses,
                    },
                    value,
                ))
            }
        }
        impl<Private, D: DB, W, T: CellValue, R: CellValue, const DEPTH: u8>
            MerkleCheckRoot<T, R, DEPTH, $historic> for NativeLedger8<'_, Private, D, W>
        {
            fn merkle_check_root(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                root: R,
            ) -> Result<(Self, bool), CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, value) = self.frame.apply(|context| slot.check_root(context, root))?;
                Ok((
                    Self {
                        frame,
                        bindings: self.bindings,
                        witnesses: self.witnesses,
                    },
                    value,
                ))
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInsert<T, DEPTH, $historic>
            for NativeLedger8<'_, Private, D, W>
        {
            fn merkle_insert(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                value: T,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self.frame.apply(|context| slot.insert(context, value))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_index(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                value: T,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self
                    .frame
                    .apply(|context| slot.insert_index(context, value, index))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_default(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError>
            where
                T: Default,
            {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self
                    .frame
                    .apply(|context| slot.insert_index_default(context, index))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInsertHash<T, DEPTH, $historic>
            for NativeLedger8<'_, Private, D, W>
        {
            fn merkle_insert_hash(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                hash: FixedBytes<32>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self
                    .frame
                    .apply(|context| slot.insert_hash(context, hash))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_hash_index(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                hash: FixedBytes<32>,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self
                    .frame
                    .apply(|context| slot.insert_hash_index(context, hash, index))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleReset<T, DEPTH, $historic>
            for NativeLedger8<'_, Private, D, W>
        {
            fn merkle_reset(self, tree: Merkle<T, DEPTH, $historic>) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, ()) = self.frame.apply(|context| slot.reset_to_default(context))?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
    };
}
native_merkle!(false);
native_merkle!(true);
impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleResetHistory<T, DEPTH>
    for NativeLedger8<'_, Private, D, W>
{
    fn merkle_reset_history(self, tree: Merkle<T, DEPTH, true>) -> Result<Self, CompactError> {
        let slot = self.bindings.merkle(tree)?;
        let (frame, ()) = self.frame.apply(|context| slot.reset_history(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
macro_rules! recorded_merkle {
    ($historic:literal) => {
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInspect<T, DEPTH, $historic>
            for RecordedLedger8<'_, Private, D, W>
        {
            fn merkle_is_full(
                self,
                tree: Merkle<T, DEPTH, $historic>,
            ) -> Result<(Self, bool), CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, value) = slot.record_is_full(self.frame)?;
                Ok((
                    Self {
                        frame,
                        bindings: self.bindings,
                        witnesses: self.witnesses,
                    },
                    value,
                ))
            }
        }
        impl<Private, D: DB, W, T: CellValue, R: CellValue + Clone, const DEPTH: u8>
            MerkleCheckRoot<T, R, DEPTH, $historic> for RecordedLedger8<'_, Private, D, W>
        {
            fn merkle_check_root(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                root: R,
            ) -> Result<(Self, bool), CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let (frame, value) = slot.record_check_root(self.frame, root)?;
                Ok((
                    Self {
                        frame,
                        bindings: self.bindings,
                        witnesses: self.witnesses,
                    },
                    value,
                ))
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInsert<T, DEPTH, $historic>
            for RecordedLedger8<'_, Private, D, W>
        {
            fn merkle_insert(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                value: T,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_insert(self.frame, value)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_index(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                value: T,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_insert_index(self.frame, value, index)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_default(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError>
            where
                T: Default,
            {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_insert_index_default(self.frame, index)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleInsertHash<T, DEPTH, $historic>
            for RecordedLedger8<'_, Private, D, W>
        {
            fn merkle_insert_hash(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                hash: FixedBytes<32>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_insert_hash(self.frame, hash)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
            fn merkle_insert_hash_index(
                self,
                tree: Merkle<T, DEPTH, $historic>,
                hash: FixedBytes<32>,
                index: BoundedUint<{ u64::MAX as u128 }>,
            ) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_insert_hash_index(self.frame, hash, index)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
        impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleReset<T, DEPTH, $historic>
            for RecordedLedger8<'_, Private, D, W>
        {
            fn merkle_reset(self, tree: Merkle<T, DEPTH, $historic>) -> Result<Self, CompactError> {
                let slot = self.bindings.merkle(tree)?;
                let frame = slot.record_reset_to_default(self.frame)?;
                Ok(Self {
                    frame,
                    bindings: self.bindings,
                    witnesses: self.witnesses,
                })
            }
        }
    };
}
recorded_merkle!(false);
recorded_merkle!(true);
impl<Private, D: DB, W, T: CellValue, const DEPTH: u8> MerkleResetHistory<T, DEPTH>
    for RecordedLedger8<'_, Private, D, W>
{
    fn merkle_reset_history(self, tree: Merkle<T, DEPTH, true>) -> Result<Self, CompactError> {
        let slot = self.bindings.merkle(tree)?;
        let frame = slot.record_reset_history(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
