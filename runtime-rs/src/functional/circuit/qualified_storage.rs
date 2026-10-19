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

//! Explicit Ledger8 storage operations preserve the carrier's private state.
use super::super::{
    Cell, Set,
    ledger8::{QualifiedCellWrite, QualifiedSetInsert},
};
use super::{Circuit, Executor};
use crate::ledger;
impl<E: Executor> Circuit<E, ()> {
    pub fn write_qualified_coin<T>(
        self,
        cell: Cell<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Self
    where
        E: QualifiedCellWrite<T>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .write_qualified_coin(cell, coin, recipient)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn insert_qualified_coin<T>(
        self,
        set: Set<T>,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Self
    where
        E: QualifiedSetInsert<T>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .insert_qualified_coin(set, coin, recipient)
                    .map(|next| (next, ()))
            }),
        }
    }
}
