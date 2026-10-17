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

use super::super::SlotId;
use super::super::ledger8::{CellBinding, CounterBinding, Ledger8Bindings};
use crate::CompactError;

#[test]
fn equal_and_ancestor_paths_refuse_within_and_across_slot_kinds() {
    for (first, second) in [
        (&[0][..], &[0][..]),
        (&[0][..], &[0, 1][..]),
        (&[0, 1][..], &[0][..]),
        (&[][..], &[0][..]),
    ] {
        let cells = [
            CellBinding::new(SlotId::new(1), first),
            CellBinding::new(SlotId::new(2), second),
        ];
        assert!(matches!(
            Ledger8Bindings::new(&cells),
            Err(CompactError::InvalidLedgerCell(_))
        ));
        let counters = [
            CounterBinding::new(SlotId::new(1), first),
            CounterBinding::new(SlotId::new(2), second),
        ];
        assert!(matches!(
            Ledger8Bindings::with_counters(&[], &counters),
            Err(CompactError::InvalidLedgerCell(_))
        ));
        assert!(matches!(
            Ledger8Bindings::with_counters(&cells[..1], &counters[1..]),
            Err(CompactError::InvalidLedgerCell(_))
        ));
    }
}

#[test]
fn chunked_layout_siblings_and_a_single_root_binding_remain_valid() {
    // Shared array/chunk ancestors are fine: neither complete leaf path is an
    // ancestor of another. Only a complete prefix binding is rejected.
    let cells = [
        CellBinding::new(SlotId::new(1), &[0, 0]),
        CellBinding::new(SlotId::new(2), &[0, 1]),
    ];
    let counters = [
        CounterBinding::new(SlotId::new(3), &[0, 2]),
        CounterBinding::new(SlotId::new(4), &[1, 0]),
    ];
    assert!(Ledger8Bindings::with_counters(&cells, &counters).is_ok());
    assert!(Ledger8Bindings::new(&[CellBinding::new(SlotId::new(0), &[])]).is_ok());
}
