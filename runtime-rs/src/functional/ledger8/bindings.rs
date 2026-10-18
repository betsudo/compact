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

//! Physical Ledger8 slot bindings; logical identities remain target neutral.
use super::super::{Cell, Counter, Map, SlotId};
use crate::{CompactError, ledger::CellValue};
#[derive(Clone, Copy)]
pub struct CellBinding {
    id: SlotId,
    path: &'static [u8],
}
impl CellBinding {
    pub const fn new(id: SlotId, path: &'static [u8]) -> Self {
        Self { id, path }
    }
}
#[derive(Clone, Copy)]
pub struct CounterBinding {
    id: SlotId,
    path: &'static [u8],
}
impl CounterBinding {
    pub const fn new(id: SlotId, path: &'static [u8]) -> Self {
        Self { id, path }
    }
}

#[derive(Clone, Copy)]
pub struct MapBinding {
    id: SlotId,
    path: &'static [u8],
}
impl MapBinding {
    pub const fn new(id: SlotId, path: &'static [u8]) -> Self {
        Self { id, path }
    }
}

/// Immutable uniquely keyed declarations; not certification of handwritten types.
#[derive(Clone, Copy)]
pub struct Ledger8Bindings<'a> {
    cells: &'a [CellBinding],
    counters: &'a [CounterBinding],
    maps: &'a [MapBinding],
}
impl<'a> Ledger8Bindings<'a> {
    pub fn new(cells: &'a [CellBinding]) -> Result<Self, CompactError> {
        Self::with_counters(cells, &[])
    }
    pub fn with_counters(
        cells: &'a [CellBinding],
        counters: &'a [CounterBinding],
    ) -> Result<Self, CompactError> {
        Self::with_maps(cells, counters, &[])
    }
    pub fn with_maps(
        cells: &'a [CellBinding],
        counters: &'a [CounterBinding],
        maps: &'a [MapBinding],
    ) -> Result<Self, CompactError> {
        for (i, cell) in cells.iter().enumerate() {
            for other in &cells[..i] {
                distinct(cell.id, cell.path, other.id, other.path)?;
            }
        }
        for (i, counter) in counters.iter().enumerate() {
            for other in &counters[..i] {
                distinct(counter.id, counter.path, other.id, other.path)?;
            }
            for cell in cells {
                distinct(counter.id, counter.path, cell.id, cell.path)?;
            }
        }
        for (i, map) in maps.iter().enumerate() {
            for other in &maps[..i] {
                distinct(map.id, map.path, other.id, other.path)?;
            }
            for cell in cells {
                distinct(map.id, map.path, cell.id, cell.path)?;
            }
            for counter in counters {
                distinct(map.id, map.path, counter.id, counter.path)?;
            }
        }
        Ok(Self {
            cells,
            counters,
            maps,
        })
    }
    pub(super) fn slot<T: CellValue>(
        self,
        cell: Cell<T>,
    ) -> Result<crate::slots::CellSlot<T>, CompactError> {
        let binding = self
            .cells
            .iter()
            .find(|binding| binding.id == cell.id())
            .ok_or_else(|| {
                CompactError::InvalidLedgerCell(format!(
                    "unbound logical Cell {}",
                    cell.id().index()
                ))
            })?;
        Ok(crate::slots::CellSlot::new(binding.path))
    }
    pub(super) fn map<K: CellValue, V>(
        self,
        map: Map<K, V>,
    ) -> Result<crate::slots::MapSlot<K, V>, CompactError> {
        let binding = self
            .maps
            .iter()
            .find(|binding| binding.id == map.id())
            .ok_or_else(|| {
                CompactError::InvalidLedgerCell(format!("unbound logical Map {}", map.id().index()))
            })?;
        Ok(crate::slots::MapSlot::new(binding.path))
    }
    pub(super) fn counter(
        self,
        counter: Counter,
    ) -> Result<crate::slots::CounterSlot, CompactError> {
        let binding = self
            .counters
            .iter()
            .find(|binding| binding.id == counter.id())
            .ok_or_else(|| {
                CompactError::InvalidLedgerCell(format!(
                    "unbound logical Counter {}",
                    counter.id().index()
                ))
            })?;
        Ok(crate::slots::CounterSlot::new(binding.path))
    }
}
fn distinct(id: SlotId, path: &[u8], other: SlotId, other_path: &[u8]) -> Result<(), CompactError> {
    if id == other {
        return Err(CompactError::InvalidLedgerCell(format!(
            "duplicate logical slot {}",
            id.index()
        )));
    }
    // A Cell/Counter occupies one leaf. Equal paths or an ancestor binding
    // would allow one declaration to overwrite another; shared prefixes alone
    // remain valid for chunked arrays and other sibling layout paths.
    if path.starts_with(other_path) || other_path.starts_with(path) {
        return Err(CompactError::InvalidLedgerCell(format!(
            "overlapping physical paths for logical slots {} and {}",
            id.index(),
            other.index()
        )));
    }
    Ok(())
}
