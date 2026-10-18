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

//! Render actual checked List bindings and retain their declaration refusals.
use super::*;
use crate::ir::{
    Expr, LedgerField, LedgerFieldKind, Parameter, StateAction, StateReturn, StatefulCircuit, Type,
};
use quote::ToTokens;
use std::collections::HashMap;
fn field() -> LedgerField {
    LedgerField {
        id: "items".into(),
        index: 0,
        path: vec![1, 3],
        source: None,
        declaration: LedgerFieldKind::List { ty: Type::Field },
    }
}
fn root(kind: usize) -> StatefulCircuit {
    let maybe = crate::list_head_result_type(
        &Type::Field,
        &Type::Struct {
            name: "MaybeCompact7".into(),
            fields: vec![],
        },
    );
    let actions = match kind {
        0 => vec![StateAction::ListPushFront {
            field: "items".into(),
            index: 0,
            value: Expr::Parameter {
                name: "input".into(),
            },
        }],
        1 => vec![StateAction::ListPopFront {
            field: "items".into(),
            index: 0,
        }],
        2 => vec![StateAction::ListReset {
            field: "items".into(),
            index: 0,
        }],
        _ => vec![],
    };
    let (result, return_value) = match kind {
        3 => (
            Type::Unsigned {
                max: u64::MAX.to_string(),
            },
            StateReturn::ListLength {
                field: "items".into(),
                index: 0,
            },
        ),
        4 => (
            Type::Boolean,
            StateReturn::ListIsEmpty {
                field: "items".into(),
                index: 0,
            },
        ),
        5 => (
            maybe,
            StateReturn::ListHead {
                field: "items".into(),
                index: 0,
            },
        ),
        _ => (Type::Unit, StateReturn::Unit),
    };
    StatefulCircuit {
        name: "root".into(),
        source: None,
        internal: false,
        parameters: vec![Parameter {
            name: "input".into(),
            ty: Type::Field,
        }],
        actions,
        result,
        return_value,
    }
}
fn checked(root: &StatefulCircuit, field: &LedgerField) -> super::super::model::CheckedPlan {
    super::super::check_with_calls_observed(
        root,
        &HashMap::from([("items", field)]),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", root)]),
        None,
    )
    .unwrap()
}
fn list_binding(checked: &super::super::model::CheckedPlan) -> &Binding {
    checked
        .plan()
        .body
        .bindings
        .iter()
        .find(|b| matches!(b.operation, Operation::List { .. }))
        .unwrap()
}
#[test]
fn six_checked_operations_use_separate_read_head_write_bounds() {
    for (kind, method, capability) in [
        (0, "list_push_front", "ListWrite"),
        (1, "list_pop_front", "ListWrite"),
        (2, "list_reset", "ListWrite"),
        (3, "list_length", "ListRead"),
        (4, "list_is_empty", "ListRead"),
        (5, "list_head", "ListHead"),
    ] {
        let root = root(kind);
        let field = field();
        let checked = checked(&root, &field);
        let binding = list_binding(&checked);
        let step = render(
            binding,
            checked.declarations(),
            syn::parse_quote!(execution),
            |value| {
                assert_eq!(value.ty, Type::Field);
                syn::parse_quote!(retained_input)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(step.slot.id, 0);
        assert_eq!(step.slot.declaration, LedgerId(0));
        let source = step.expression.to_token_stream().to_string();
        assert!(source.contains(method), "{source}");
        let bound = step.bound.to_token_stream().to_string();
        assert!(bound.contains(capability), "{bound}");
        if kind == 3 {
            assert!(source.contains("__compact_list_length as u128"));
            assert!(source.contains("18446744073709551615"));
        }
        if kind == 5 {
            assert!(bound.contains("crate :: types :: MaybeCompact7"), "{bound}");
            assert!(!bound.contains("Option"));
            assert!(!source.contains("Option"));
        }
    }
}
#[test]
fn malformed_list_identity_keeps_existing_validator_cause() {
    let root = root(3);
    let field = field();
    let checked = checked(&root, &field);
    for slot in [
        ListSlot {
            declaration: LedgerId(8),
            index: 0,
        },
        ListSlot {
            declaration: LedgerId(0),
            index: 8,
        },
    ] {
        let mut binding = list_binding(&checked).clone();
        binding.operation = Operation::List {
            slot,
            operation: ListOperation::Length,
        };
        let expected =
            super::super::validate::lists::element(checked.declarations(), slot, &binding.path)
                .unwrap_err();
        let actual = match declaration(&binding, checked.declarations()) {
            Err(e) => e,
            Ok(_) => panic!("invalid List identity accepted"),
        };
        assert_eq!(actual, expected);
    }
}
#[test]
fn head_rechecks_native_maybe_carrier_instead_of_inventing_option() {
    let root = root(5);
    let field = field();
    let checked = checked(&root, &field);
    let mut binding = list_binding(&checked).clone();
    binding.value.ty = Type::Boolean;
    let expected = super::super::validate::lists::head_type(
        checked.declarations(),
        &Type::Field,
        &binding.value.ty,
        &binding.path,
    )
    .unwrap_err();
    assert_eq!(
        expected.detail,
        "List head result differs from native Maybe type"
    );
    let actual = match declaration(&binding, checked.declarations()) {
        Err(e) => e,
        Ok(_) => panic!("non-Maybe head accepted"),
    };
    assert_eq!(actual, expected);
}
#[test]
fn unrelated_operations_do_not_evaluate_list_operands() {
    let root = root(0);
    let field = field();
    let checked = checked(&root, &field);
    let mut binding = list_binding(&checked).clone();
    binding.operation = Operation::Literal(super::super::model::Literal::Unit);
    assert!(
        render(
            &binding,
            checked.declarations(),
            syn::parse_quote!(execution),
            |_| panic!("unrelated operand evaluated")
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn canonical_but_malformed_maybe_shapes_keep_native_layout_refusals() {
    let valid_root = root(5);
    let field = field();
    let checked = checked(&valid_root, &field);
    for mutation in ["element", "name", "order"] {
        let mut malformed = valid_root.result.clone();
        let Type::Struct { fields, .. } = &mut malformed else {
            unreachable!()
        };
        match mutation {
            "element" => fields[1].ty = Type::Boolean,
            "name" => fields[0].name = "present".into(),
            "order" => fields.swap(0, 1),
            _ => unreachable!(),
        }
        // Register this actual source shape independently, so the assertion
        // exercises native Maybe layout validation rather than canonical-name
        // conflict rejection against the good source's definition.
        let malformed_root = StatefulCircuit {
            result: malformed.clone(),
            ..valid_root.clone()
        };
        let fields = HashMap::from([("items", &field)]);
        let calls = HashMap::from([("root", &malformed_root)]);
        let types = super::super::type_environment::TypeEnvironment::from_sources(
            &malformed_root,
            &fields,
            &HashMap::new(),
            &HashMap::new(),
            &calls,
        )
        .unwrap();
        let declarations = Declarations::new(&fields, &HashMap::new()).with_type_environment(types);
        let mut binding = list_binding(&checked).clone();
        binding.value.ty = malformed;
        let expected = super::super::validate::lists::head_type(
            &declarations,
            &Type::Field,
            &binding.value.ty,
            &binding.path,
        )
        .unwrap_err();
        assert_eq!(
            expected.detail, "List head result differs from native Maybe type",
            "{mutation}"
        );
        let actual = match declaration(&binding, &declarations) {
            Err(error) => error,
            Ok(_) => panic!("invalid {mutation} Maybe accepted"),
        };
        assert_eq!(actual, expected, "{mutation}");
    }
}

#[test]
fn currently_eligible_list_codecs_also_have_default_capability() {
    // stored_value requires value_type + derives_supported: tuples recursively
    // stop at arity8. Default uses the same value domain and tuple bound12.
    // Therefore no canonical, resource-valid List element currently reaches
    // the nondefaultable-head refusal. Keep the production guard for future
    // domain changes; do not manufacture an impossible checked declaration.
    let mut examples = vec![
        Type::Field,
        Type::Boolean,
        Type::Bytes { length: 0 },
        Type::OpaqueString,
        Type::OpaqueBytes,
        Type::Unsigned {
            max: u128::MAX.to_string(),
        },
        Type::Tuple { elements: vec![] },
        Type::Tuple {
            elements: vec![Type::Field; 8],
        },
    ];
    let nested = Type::Tuple {
        elements: vec![Type::OpaqueString, Type::Field],
    };
    examples.push(Type::Vector {
        element: Box::new(nested),
        length: 0,
    });
    for element in examples {
        assert!(
            super::super::validate::list_element_type(&element),
            "{element:?}"
        );
        assert!(
            super::super::validate::default_value_type(&element),
            "{element:?}"
        );
    }
}
