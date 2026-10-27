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

//! Set interpretation delegates the existing scalar slots and frames.
use super::super::{Set, SetRead, SetWrite};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    ledger::{CellValue, DB},
};
impl<Private, D: DB, W, T: CellValue + Clone> SetRead<T> for NativeLedger8<'_, Private, D, W> {
    fn set_member(self, set: Set<T>, value: T) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.set(set)?;
        let (frame, value) = self.frame.apply(|context| slot.member(context, value))?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn set_size(self, set: Set<T>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.set(set)?;
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
    fn set_is_empty(self, set: Set<T>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.set(set)?;
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
impl<Private, D: DB, W, T: CellValue + Clone> SetWrite<T> for NativeLedger8<'_, Private, D, W> {
    fn set_insert(self, set: Set<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let (frame, ()) = self.frame.apply(|context| slot.insert(context, value))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn set_remove(self, set: Set<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let (frame, ()) = self.frame.apply(|context| slot.remove(context, value))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn set_reset(self, set: Set<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let (frame, ()) = self.frame.apply(|context| slot.reset(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, T: CellValue + Clone> SetRead<T> for RecordedLedger8<'_, Private, D, W> {
    fn set_member(self, set: Set<T>, value: T) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.set(set)?;
        let (frame, value) = slot.record_member(self.frame, value)?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn set_size(self, set: Set<T>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.set(set)?;
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
    fn set_is_empty(self, set: Set<T>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.set(set)?;
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
impl<Private, D: DB, W, T: CellValue + Clone> SetWrite<T> for RecordedLedger8<'_, Private, D, W> {
    fn set_insert(self, set: Set<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let frame = slot.record_insert(self.frame, value)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn set_remove(self, set: Set<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let frame = slot.record_remove(self.frame, value)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn set_reset(self, set: Set<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.set(set)?;
        let frame = slot.record_reset(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
