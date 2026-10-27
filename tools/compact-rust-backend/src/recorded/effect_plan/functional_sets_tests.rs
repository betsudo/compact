// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Isolated syntax/declaration tests; checked-plan integration is tested by the parent module.
use super::super::model::Literal;
use super::*;
use crate::ir::{LedgerField, LedgerFieldKind};
use quote::ToTokens;
use std::collections::HashMap;
fn field(ty: Type) -> LedgerField {
    LedgerField {
        source: None,
        id: "entries".into(),
        index: 2,
        path: vec![1, 2],
        declaration: LedgerFieldKind::Set { ty },
    }
}
fn declarations(field: &LedgerField) -> Declarations {
    Declarations::new(&HashMap::from([("entries", field)]), &HashMap::new())
}
fn slot() -> SetSlot {
    SetSlot {
        declaration: LedgerId(0),
        index: 2,
    }
}
fn value() -> Value {
    Value {
        id: 7,
        ty: Type::Field,
    }
}
fn binding(operation: Operation, ty: Type) -> Binding {
    Binding {
        operation,
        value: Value { id: 8, ty },
        path: "actions[3]".into(),
    }
}
#[test]
fn all_six_set_operations_keep_typed_capability_and_operand_mapping() {
    let field = field(Type::Field);
    let declarations = declarations(&field);
    let size = Type::Unsigned {
        max: u64::MAX.to_string(),
    };
    let cases = [
        (
            Operation::SetMember {
                slot: slot(),
                value: value(),
            },
            Type::Boolean,
            "set_member",
            SetAccess::Read,
        ),
        (
            Operation::SetInsert {
                slot: slot(),
                value: value(),
            },
            Type::Unit,
            "set_insert",
            SetAccess::Write,
        ),
        (
            Operation::SetRemove {
                slot: slot(),
                value: value(),
            },
            Type::Unit,
            "set_remove",
            SetAccess::Write,
        ),
        (
            Operation::SetReset { slot: slot() },
            Type::Unit,
            "set_reset",
            SetAccess::Write,
        ),
        (
            Operation::SetSize { slot: slot() },
            size,
            "set_size",
            SetAccess::Read,
        ),
        (
            Operation::SetIsEmpty { slot: slot() },
            Type::Boolean,
            "set_is_empty",
            SetAccess::Read,
        ),
    ];
    for (operation, ty, method, access) in cases {
        let binding = binding(operation, ty);
        assert_eq!(
            declaration(&binding, &declarations)
                .unwrap()
                .unwrap()
                .access,
            access
        );
        let step = render(&binding, &declarations, syn::parse_quote!(execution), |v| {
            assert_eq!(v.id, 7);
            syn::parse_quote!(retained_value)
        })
        .unwrap()
        .unwrap();
        assert_eq!(step.slot.id, 0);
        assert_eq!(step.slot.declaration, LedgerId(0));
        let source = step.expression.to_token_stream().to_string();
        assert!(source.contains(method), "{source}");
        assert!(source.contains("SlotId :: new (0u32)"), "{source}");
        let bound = step.bound.to_token_stream().to_string();
        assert!(bound.contains(match access {
            SetAccess::Read => "SetRead",
            SetAccess::Write => "SetWrite",
        }));
        if method == "set_size" {
            assert!(source.contains("__compact_set_size as u128"));
            assert!(source.contains("18446744073709551615"));
            assert!(source.contains("try_returning"));
        }
    }
}
#[test]
fn malformed_set_declarations_retain_original_checked_refusal() {
    let check = |field: &LedgerField, slot: SetSlot, expected: &str| {
        let declarations = declarations(field);
        let binding = binding(
            Operation::SetSize { slot },
            Type::Unsigned {
                max: u64::MAX.to_string(),
            },
        );
        let expected_cause =
            super::super::validate::sets::element(&declarations, slot, &binding.path).unwrap_err();
        let cause = match declaration(&binding, &declarations) {
            Err(e) => e,
            Ok(_) => panic!("invalid Set accepted"),
        };
        assert_eq!(cause, expected_cause);
        assert_eq!(cause.path, "actions[3]");
        assert_eq!(cause.detail, expected);
    };
    let field = field(Type::Field);
    check(
        &field,
        SetSlot {
            declaration: LedgerId(0),
            index: 9,
        },
        "Set declaration index mismatch",
    );
    check(
        &field,
        SetSlot {
            declaration: LedgerId(9),
            index: 2,
        },
        "unknown Set declaration",
    );
    let wrong = LedgerField {
        declaration: LedgerFieldKind::Cell { ty: Type::Field },
        ..field.clone()
    };
    check(&wrong, slot(), "expected Set declaration");
    let nested = field_with_nested_type();
    let declarations = declarations(&nested);
    let binding = binding(
        Operation::SetSize { slot: slot() },
        Type::Unsigned {
            max: u64::MAX.to_string(),
        },
    );
    let direct =
        super::super::validate::sets::element(&declarations, slot(), &binding.path).unwrap_err();
    let actual = match declaration(&binding, &declarations) {
        Err(e) => e,
        Ok(_) => panic!("structural Set accepted"),
    };
    assert_eq!(actual, direct);
}
fn field_with_nested_type() -> LedgerField {
    field(Type::LedgerMap {
        key: Box::new(Type::Field),
        value: Box::new(Type::Field),
    })
}
#[test]
fn unrelated_and_coin_allocating_operations_do_not_become_scalar_set_steps() {
    let field = field(Type::Field);
    let declarations = declarations(&field);
    for operation in [
        Operation::Literal(Literal::Unit),
        Operation::SetInsertCoin {
            slot: slot(),
            coin: value(),
            recipient: value(),
        },
    ] {
        let binding = binding(operation, Type::Unit);
        assert!(declaration(&binding, &declarations).unwrap().is_none());
        assert!(
            render(
                &binding,
                &declarations,
                syn::parse_quote!(execution),
                |_| panic!("unrelated operand evaluated")
            )
            .unwrap()
            .is_none()
        );
    }
}
