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

//! Ledger8 qualified storage delegates recipient-bound allocation authority.
//! No allocator, coin model or constructor interpreter is introduced here.
use super::super::{Cell, Executor, Set};
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    ledger::{self, CellValue, DB},
};
pub trait QualifiedCellWrite<T>: Executor {
    fn write_qualified_coin(
        self,
        cell: Cell<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError>;
}
pub trait QualifiedSetInsert<T>: Executor {
    fn insert_qualified_coin(
        self,
        set: Set<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError>;
}
impl<Private, D: DB, W, T: CellValue> QualifiedCellWrite<T> for NativeLedger8<'_, Private, D, W> {
    fn write_qualified_coin(
        self,
        logical: Cell<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let slot = self.bindings.slot(logical)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.write_coin(context, coin, recipient))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, T: CellValue> QualifiedSetInsert<T> for NativeLedger8<'_, Private, D, W> {
    fn insert_qualified_coin(
        self,
        logical: Set<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let slot = self.bindings.set(logical)?;
        let (frame, ()) = self
            .frame
            .apply(|context| slot.insert_coin(context, coin, recipient))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, T: CellValue> QualifiedCellWrite<T> for RecordedLedger8<'_, Private, D, W> {
    fn write_qualified_coin(
        self,
        logical: Cell<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let slot = self.bindings.slot(logical)?;
        let frame = slot.record_write_coin(self.frame, coin, recipient)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W, T: CellValue> QualifiedSetInsert<T> for RecordedLedger8<'_, Private, D, W> {
    fn insert_qualified_coin(
        self,
        logical: Set<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let slot = self.bindings.set(logical)?;
        let frame = slot.record_insert_coin(self.frame, coin, recipient)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
