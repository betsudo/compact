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

//! Scalar Map interpretation delegates existing slots and owned frames.
use super::super::{Map, MapLookup, MapRead, MapRemove, MapWrite};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    ledger::{CellValue, DB},
};

impl<Private, D: DB, W, K: CellValue + Clone, V> MapRead<K, V>
    for NativeLedger8<'_, Private, D, W>
{
    fn map_member(self, map: Map<K, V>, key: K) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = self.frame.apply(|context| slot.member(context, key))?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn map_size(self, map: Map<K, V>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = self.frame.apply(|context| slot.size(context))?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn map_is_empty(self, map: Map<K, V>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = self.frame.apply(|context| slot.is_empty(context))?;
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
impl<Private, D: DB, W, K: CellValue + Clone, V: CellValue> MapLookup<K, V>
    for NativeLedger8<'_, Private, D, W>
{
    fn map_lookup(self, map: Map<K, V>, key: K) -> Result<(Self, V), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = self.frame.apply(|context| slot.lookup(context, key))?;
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
impl<Private, D: DB, W, K: CellValue + Clone, V> MapRemove<K, V>
    for NativeLedger8<'_, Private, D, W>
{
    fn map_remove(self, map: Map<K, V>, key: K) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, ()) = self.frame.apply(|context| slot.remove(context, key))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, K: CellValue + Clone, V: CellValue> MapWrite<K, V>
    for NativeLedger8<'_, Private, D, W>
{
    fn map_insert(self, map: Map<K, V>, key: K, value: V) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.insert(context, key, value))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn map_reset(self, map: Map<K, V>) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, ()) = self.frame.apply(|context| slot.reset(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn map_insert_default(self, map: Map<K, V>, key: K) -> Result<Self, CompactError>
    where
        V: Default,
    {
        let slot = self.bindings.map(map)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.insert_default(context, key))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, K: CellValue + Clone, V> MapRead<K, V>
    for RecordedLedger8<'_, Private, D, W>
{
    fn map_member(self, map: Map<K, V>, key: K) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = slot.record_member(self.frame, key)?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn map_size(self, map: Map<K, V>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = slot.record_size(self.frame)?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn map_is_empty(self, map: Map<K, V>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = slot.record_is_empty(self.frame)?;
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
impl<Private, D: DB, W, K: CellValue + Clone, V: CellValue> MapLookup<K, V>
    for RecordedLedger8<'_, Private, D, W>
{
    fn map_lookup(self, map: Map<K, V>, key: K) -> Result<(Self, V), CompactError> {
        let slot = self.bindings.map(map)?;
        let (frame, value) = slot.record_lookup(self.frame, key)?;
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
impl<Private, D: DB, W, K: CellValue + Clone, V> MapRemove<K, V>
    for RecordedLedger8<'_, Private, D, W>
{
    fn map_remove(self, map: Map<K, V>, key: K) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let frame = slot.record_remove(self.frame, key)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, K: CellValue + Clone, V: CellValue> MapWrite<K, V>
    for RecordedLedger8<'_, Private, D, W>
{
    fn map_insert(self, map: Map<K, V>, key: K, value: V) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let frame = slot.record_insert(self.frame, key, value)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn map_reset(self, map: Map<K, V>) -> Result<Self, CompactError> {
        let slot = self.bindings.map(map)?;
        let frame = slot.record_reset(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn map_insert_default(self, map: Map<K, V>, key: K) -> Result<Self, CompactError>
    where
        V: Default,
    {
        let slot = self.bindings.map(map)?;
        let frame = slot.record_insert_default(self.frame, key)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
