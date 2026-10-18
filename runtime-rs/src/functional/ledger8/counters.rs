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

//! Remaining Counter capabilities delegate existing slots and frames.
use super::super::{Counter, CounterCompare, CounterDecrement, CounterReset};
use super::{NativeLedger8, RecordedLedger8};
use crate::{CompactError, ledger::DB};
impl<Private, D: DB, W> CounterCompare for NativeLedger8<'_, Private, D, W> {
    fn counter_less_than(
        self,
        counter: Counter,
        threshold: u64,
    ) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.counter(counter)?;
        let (frame, value) = self
            .frame
            .apply(|context| slot.less_than(context, threshold))?;
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
impl<Private, D: DB, W> CounterDecrement for NativeLedger8<'_, Private, D, W> {
    fn decrement(self, counter: Counter, amount: u16) -> Result<Self, CompactError> {
        let slot = self.bindings.counter(counter)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.decrement(context, amount))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> CounterReset for NativeLedger8<'_, Private, D, W> {
    fn reset_counter(self, counter: Counter) -> Result<Self, CompactError> {
        let slot = self.bindings.counter(counter)?;
        let (frame, ()) = self.frame.apply(|context| slot.reset(context))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> CounterCompare for RecordedLedger8<'_, Private, D, W> {
    fn counter_less_than(
        self,
        counter: Counter,
        threshold: u64,
    ) -> Result<(Self, bool), CompactError> {
        let slot = self.bindings.counter(counter)?;
        let (frame, value) = slot.record_less_than(self.frame, threshold)?;
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
impl<Private, D: DB, W> CounterDecrement for RecordedLedger8<'_, Private, D, W> {
    fn decrement(self, counter: Counter, amount: u16) -> Result<Self, CompactError> {
        let slot = self.bindings.counter(counter)?;
        let frame = slot.record_decrement(self.frame, amount)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> CounterReset for RecordedLedger8<'_, Private, D, W> {
    fn reset_counter(self, counter: Counter) -> Result<Self, CompactError> {
        let slot = self.bindings.counter(counter)?;
        let frame = slot.record_reset(self.frame)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
