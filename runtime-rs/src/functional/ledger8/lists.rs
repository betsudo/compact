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

//! List interpreters delegate existing slots; no codec or VM implementation lives here.
use super::super::{List, ListHead, ListRead, ListWrite};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    ledger::{CellValue, DB},
};
impl<Private, D: DB, W, T: CellValue> ListRead<T> for NativeLedger8<'_, Private, D, W> {
    fn list_length(self, list: List<T>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, value) = self.frame.apply(|context| slot.length(context))?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn list_is_empty(self, list: List<T>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.list(list)?;
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
impl<Private, D: DB, W, T: CellValue + Default, M: CellValue> ListHead<T, M>
    for NativeLedger8<'_, Private, D, W>
{
    fn list_head(self, list: List<T>) -> Result<(Self, M), CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, value) = self.frame.apply(|context| slot.head::<M, _, _>(context))?;
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
impl<Private, D: DB, W, T: CellValue> ListWrite<T> for NativeLedger8<'_, Private, D, W> {
    fn list_push_front(self, list: List<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.push_front(context, value))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn list_pop_front(self, list: List<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, ()) = self.frame.apply(|context| slot.pop_front(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn list_reset(self, list: List<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, ()) = self.frame.apply(|context| slot.reset(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, T: CellValue> ListRead<T> for RecordedLedger8<'_, Private, D, W> {
    fn list_length(self, list: List<T>) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, value) = slot.record_length(self.frame)?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
    fn list_is_empty(self, list: List<T>) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.list(list)?;
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
impl<Private, D: DB, W, T: CellValue + Default, M: CellValue> ListHead<T, M>
    for RecordedLedger8<'_, Private, D, W>
{
    fn list_head(self, list: List<T>) -> Result<(Self, M), CompactError> {
        let slot = self.bindings.list(list)?;
        let (frame, value) = slot.record_head::<M, _, _>(self.frame)?;
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
impl<Private, D: DB, W, T: CellValue> ListWrite<T> for RecordedLedger8<'_, Private, D, W> {
    fn list_push_front(self, list: List<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let frame = slot.record_push_front(self.frame, value)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn list_pop_front(self, list: List<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let frame = slot.record_pop_front(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
    fn list_reset(self, list: List<T>) -> Result<Self, CompactError> {
        let slot = self.bindings.list(list)?;
        let frame = slot.record_reset(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
