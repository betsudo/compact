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

//! Recorded leaf interpretation delegates VM trace and FAB handling to the
//! existing `RecordingFrame`; this adapter does not decode or re-encode read FABs.
use super::super::{Cell, CellRead, CellWrite, Counter, CounterIncrement, CounterRead, Executor};
use super::{Ledger8Bindings, NoWitnesses};
use crate::{
    CompactError,
    context::CircuitContext,
    ledger::{CellValue, DB, DefaultDB},
    recording::{RecordedCircuitResult, RecordingFrame},
};

pub struct RecordedLedger8<'a, Private, D: DB = DefaultDB, W = NoWitnesses> {
    pub(super) frame: RecordingFrame<Private, D>,
    pub(super) bindings: Ledger8Bindings<'a>,
    pub(super) witnesses: W,
}
impl<'a, Private, D: DB> RecordedLedger8<'a, Private, D> {
    pub fn new(context: CircuitContext<Private, D>, bindings: Ledger8Bindings<'a>) -> Self {
        Self {
            frame: RecordingFrame::new(context),
            bindings,
            witnesses: NoWitnesses,
        }
    }
}
impl<'a, Private, D: DB, W> RecordedLedger8<'a, Private, D, W> {
    pub fn with_witnesses<H>(self, witnesses: H) -> RecordedLedger8<'a, Private, D, H> {
        RecordedLedger8 {
            frame: self.frame,
            bindings: self.bindings,
            witnesses,
        }
    }
}
impl<Private, D: DB, W> Executor for RecordedLedger8<'_, Private, D, W> {
    type Outcome<T> = RecordedCircuitResult<Private, T, D>;
    fn finish<T>(self, value: T) -> Self::Outcome<T> {
        self.frame.finish(value)
    }
}
impl<Private, D: DB, W, T: CellValue> CellRead<T> for RecordedLedger8<'_, Private, D, W> {
    fn read(self, cell: Cell<T>) -> Result<(Self, T), CompactError> {
        let slot = self.bindings.slot(cell)?;
        let (frame, value) = slot.record_read(self.frame)?;
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
impl<Private, D: DB, W, T: CellValue> CellWrite<T> for RecordedLedger8<'_, Private, D, W> {
    fn write(self, cell: Cell<T>, value: T) -> Result<Self, CompactError> {
        let slot = self.bindings.slot(cell)?;
        let frame = slot.record_write(self.frame, value)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> CounterRead for RecordedLedger8<'_, Private, D, W> {
    fn read_counter(self, counter: Counter) -> Result<(Self, u64), CompactError> {
        let slot = self.bindings.counter(counter)?;
        let (frame, value) = slot.record_read(self.frame)?;
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
impl<Private, D: DB, W> CounterIncrement for RecordedLedger8<'_, Private, D, W> {
    fn increment(self, counter: Counter, amount: u16) -> Result<Self, CompactError> {
        let slot = self.bindings.counter(counter)?;
        let frame = slot.record_increment(self.frame, amount)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
