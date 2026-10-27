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

use super::super::{Cell, CellRead, CellWrite, Circuit, Executor, Rule, SlotId};
use crate::CompactError;
use std::{cell::RefCell, rc::Rc};

struct Probe {
    events: Rc<RefCell<Vec<&'static str>>>,
    fail_read: bool,
}
impl Drop for Probe {
    fn drop(&mut self) {
        self.events.borrow_mut().push("drop-executor");
    }
}
impl Executor for Probe {
    type Outcome<T> = T;
    fn finish<T>(self, value: T) -> T {
        self.events.borrow_mut().push("finish");
        value
    }
}
struct Owned(Rc<RefCell<Vec<&'static str>>>);
impl Drop for Owned {
    fn drop(&mut self) {
        self.0.borrow_mut().push("drop-value");
    }
}
impl CellRead<Owned> for Probe {
    fn read(self, _: Cell<Owned>) -> Result<(Self, Owned), CompactError> {
        self.events.borrow_mut().push("read");
        if self.fail_read {
            return Err(CompactError::InvalidLedgerCell("injected read".into()));
        }
        let value = Owned(self.events.clone());
        Ok((self, value))
    }
}
impl CellWrite<bool> for Probe {
    fn write(self, _: Cell<bool>, _: bool) -> Result<Self, CompactError> {
        self.events.borrow_mut().push("write");
        Ok(self)
    }
}
fn probe(events: &Rc<RefCell<Vec<&'static str>>>, fail_read: bool) -> Probe {
    Probe {
        events: events.clone(),
        fail_read,
    }
}

#[test]
fn terminal_read_failure_never_executes_continuation_or_later_effect() {
    let events = Rc::new(RefCell::new(vec![]));
    let result = Circuit::begin(probe(&events, true))
        .read(Cell::<Owned>::new(SlotId::new(0)))
        .bind(|_value, circuit| {
            events.borrow_mut().push("bind");
            circuit.write(Cell::new(SlotId::new(1)), true)
        })
        .finish();
    assert!(matches!(result, Err(CompactError::InvalidLedgerCell(_))));
    assert_eq!(*events.borrow(), ["read", "drop-executor"]);
}

#[test]
fn failed_rule_drops_value_and_executor_once_without_write_or_finish() {
    let events = Rc::new(RefCell::new(vec![]));
    let result = Circuit::begin(probe(&events, false))
        .read(Cell::<Owned>::new(SlotId::new(0)))
        .bind(|value, circuit| {
            events.borrow_mut().push("bind");
            drop(value);
            circuit
                .require(Rule::new(false, "stop"))
                .write(Cell::new(SlotId::new(1)), true)
        })
        .finish();
    assert_eq!(
        result.err().unwrap(),
        CompactError::AssertionFailed("stop".into())
    );
    assert_eq!(
        *events.borrow(),
        ["read", "bind", "drop-value", "drop-executor"]
    );
}

#[test]
fn successful_owned_values_are_not_cloned_and_drop_in_source_order() {
    let events = Rc::new(RefCell::new(vec![]));
    let result = Circuit::begin(probe(&events, false))
        .read(Cell::<Owned>::new(SlotId::new(0)))
        .bind(|value, circuit| {
            events.borrow_mut().push("bind");
            drop(value);
            circuit
                .write(Cell::new(SlotId::new(1)), true)
                .returning(7_u8)
        })
        .finish()
        .unwrap();
    assert_eq!(result, 7);
    assert_eq!(
        *events.borrow(),
        [
            "read",
            "bind",
            "drop-value",
            "write",
            "finish",
            "drop-executor"
        ]
    );
}

#[test]
fn pure_fallible_producers_execute_only_after_success_and_never_recover() {
    let events = Rc::new(RefCell::new(vec![]));
    let skipped = Circuit::begin(probe(&events, false))
        .require(Rule::new(false, "stop"))
        .try_returning(|| {
            events.borrow_mut().push("pure");
            Ok(1_u8)
        })
        .finish();
    assert_eq!(
        skipped.err().unwrap(),
        CompactError::AssertionFailed("stop".into())
    );
    assert_eq!(*events.borrow(), ["drop-executor"]);
    events.borrow_mut().clear();
    let failed = Circuit::begin(probe(&events, false))
        .try_returning(|| -> Result<u8, CompactError> {
            events.borrow_mut().push("pure");
            Err(CompactError::UnsignedOverflow)
        })
        .bind(|_, circuit| {
            events.borrow_mut().push("bind");
            circuit.returning(())
        })
        .finish();
    assert_eq!(failed.err().unwrap(), CompactError::UnsignedOverflow);
    assert_eq!(*events.borrow(), ["pure", "drop-executor"]);
}
