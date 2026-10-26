// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Token fixtures and ownership boundaries for the shared non-frame renderer.
use super::*;
use quote::ToTokens;
use std::cell::RefCell;

fn input(id: usize, ty: Type) -> Value {
    Value { id, ty }
}
fn binding(operation: Operation, ty: Type) -> Binding {
    Binding {
        value: input(3, ty),
        operation,
        path: "shared.pure".into(),
    }
}
fn ordinary(binding: &Binding) -> syn::Expr {
    let ty = rust_type(&binding.value.ty).unwrap();
    expression(binding, &ty, super::super::value, |input| {
        let source = super::super::name(input);
        syn::parse_quote!(#source)
    })
    .unwrap()
    .unwrap()
}
fn same(actual: &syn::Expr, expected: syn::Expr) {
    assert_eq!(
        actual.to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
}
#[test]
fn scalar_arithmetic_equality_and_comparisons_keep_original_tokens() {
    let a = input(1, Type::Field);
    let b = input(2, Type::Field);
    for (op, expected) in [
        (
            Operation::Add(a.clone(), b.clone()),
            syn::parse_quote!(__compact_effect_1 + __compact_effect_2),
        ),
        (
            Operation::Subtract(a.clone(), b.clone()),
            syn::parse_quote!(__compact_effect_1 - __compact_effect_2),
        ),
        (
            Operation::Multiply(a.clone(), b.clone()),
            syn::parse_quote!(__compact_effect_1 * __compact_effect_2),
        ),
        (
            Operation::Equal(a.clone(), b.clone()),
            syn::parse_quote!(__compact_effect_1 == __compact_effect_2),
        ),
        (
            Operation::NotEqual(a, b),
            syn::parse_quote!(__compact_effect_1 != __compact_effect_2),
        ),
    ] {
        let output = if matches!(&op, Operation::Equal(..) | Operation::NotEqual(..)) {
            Type::Boolean
        } else {
            Type::Field
        };
        same(&ordinary(&binding(op, output)), expected);
    }
    same(
        &ordinary(&binding(
            Operation::Equal(input(1, Type::OpaqueString), input(2, Type::OpaqueString)),
            Type::Boolean,
        )),
        syn::parse_quote!((__compact_effect_1).clone() == (__compact_effect_2).clone()),
    );
    let uint = Type::Unsigned { max: "255".into() };
    use crate::ir::ComparisonOperator::*;
    for (operator, expected) in [
        (
            Less,
            syn::parse_quote!(__compact_effect_1.value() < __compact_effect_2.value()),
        ),
        (
            LessEqual,
            syn::parse_quote!(__compact_effect_1.value() <= __compact_effect_2.value()),
        ),
        (
            Greater,
            syn::parse_quote!(__compact_effect_1.value() > __compact_effect_2.value()),
        ),
        (
            GreaterEqual,
            syn::parse_quote!(__compact_effect_1.value() >= __compact_effect_2.value()),
        ),
    ] {
        same(
            &ordinary(&binding(
                Operation::Compare {
                    operator,
                    left: input(1, uint.clone()),
                    right: input(2, uint.clone()),
                },
                Type::Boolean,
            )),
            expected,
        );
    }
}
#[test]
fn projection_retains_only_the_selected_noncopy_result_and_uses_raw_place() {
    let record = Type::Struct {
        name: "Packet".into(),
        fields: vec![crate::ir::StructField {
            name: "text".into(),
            ty: Type::OpaqueString,
        }],
    };
    for (op, expected) in [
        (
            Operation::VectorProject {
                value: input(
                    1,
                    Type::Vector {
                        element: Box::new(Type::OpaqueString),
                        length: 2,
                    },
                ),
                index: 1,
            },
            syn::parse_quote!(((raw_source).0[1]).clone()),
        ),
        (
            Operation::StructProject {
                value: input(1, record),
                field: "text".into(),
                index: 0,
            },
            syn::parse_quote!((raw_source.text).clone()),
        ),
        (
            Operation::TupleProject {
                value: input(
                    1,
                    Type::Tuple {
                        elements: vec![Type::OpaqueString],
                    },
                ),
                index: 0,
            },
            syn::parse_quote!((raw_source.0).clone()),
        ),
    ] {
        let binding = binding(op, Type::OpaqueString);
        let operand_calls = RefCell::new(0);
        let places = RefCell::new(vec![]);
        let actual = expression(
            &binding,
            &syn::parse_quote!(runtime::OpaqueString),
            |_| {
                *operand_calls.borrow_mut() += 1;
                syn::parse_quote!(incorrect_aggregate_clone())
            },
            |value| {
                places.borrow_mut().push(value.id);
                syn::parse_quote!(raw_source)
            },
        )
        .unwrap()
        .unwrap();
        same(&actual, expected);
        assert_eq!(*operand_calls.borrow(), 0);
        assert_eq!(*places.borrow(), [1]);
    }
}
#[test]
fn aggregate_construction_preserves_field_order_tuple_arity_and_empty_vector_type() {
    let packet = Type::Struct {
        name: "Packet".into(),
        fields: vec![
            crate::ir::StructField {
                name: "left".into(),
                ty: Type::Field,
            },
            crate::ir::StructField {
                name: "right".into(),
                ty: Type::Field,
            },
        ],
    };
    let construct = binding(
        Operation::StructConstruct {
            fields: vec![input(1, Type::Field), input(2, Type::Field)],
        },
        packet,
    );
    let actual = expression(
        &construct,
        &syn::parse_quote!(Packet),
        super::super::value,
        |_| unreachable!(),
    )
    .unwrap()
    .unwrap();
    same(
        &actual,
        syn::parse_quote!(Packet {
            left: __compact_effect_1,
            right: __compact_effect_2
        }),
    );
    same(
        &ordinary(&binding(
            Operation::Tuple {
                elements: vec![input(1, Type::Field)],
            },
            Type::Tuple {
                elements: vec![Type::Field],
            },
        )),
        syn::parse_quote!((__compact_effect_1,)),
    );
    same(
        &ordinary(&binding(
            Operation::Vector { elements: vec![] },
            Type::Vector {
                length: 0,
                element: Box::new(Type::Field),
            },
        )),
        syn::parse_quote!(runtime::FixedVector::<runtime::Field, 0>::new([])),
    );
    same(
        &ordinary(&binding(
            Operation::Vector {
                elements: vec![input(2, Type::Field), input(1, Type::Field)],
            },
            Type::Vector {
                length: 2,
                element: Box::new(Type::Field),
            },
        )),
        syn::parse_quote!(runtime::FixedVector::new([
            __compact_effect_2,
            __compact_effect_1
        ])),
    );
}
#[test]
fn injected_operands_are_used_once_in_the_original_order_and_ordinary_binding_is_unchanged() {
    let binding = binding(
        Operation::Equal(input(1, Type::Field), input(2, Type::Field)),
        Type::Boolean,
    );
    let seen = RefCell::new(vec![]);
    let actual = expression(
        &binding,
        &syn::parse_quote!(bool),
        |value| {
            seen.borrow_mut().push(value.id);
            let id = proc_macro2::Literal::usize_unsuffixed(value.id);
            syn::parse_quote!(operand(#id))
        },
        |_| unreachable!(),
    )
    .unwrap()
    .unwrap();
    same(&actual, syn::parse_quote!(operand(1) == operand(2)));
    assert_eq!(*seen.borrow(), [1, 2]);
    let mut statements = vec![];
    super::super::leaf::render(&binding, &Declarations::default(), &mut statements).unwrap();
    let expected: syn::Stmt =
        syn::parse_quote!(let __compact_effect_3: bool = __compact_effect_1 == __compact_effect_2;);
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
}
#[test]
fn checked_casts_keep_the_native_helper_result_and_nonvalue_ops_are_not_claimed() {
    let source = Type::Unsigned { max: "255".into() };
    let target = Type::Unsigned {
        max: "65535".into(),
    };
    let cast_binding = binding(
        Operation::Convert {
            value: input(1, source),
            kind: Conversion::UnsignedCast,
        },
        target,
    );
    let expected =
        crate::unsigned_cast_syntax(syn::parse_quote!(__compact_effect_1), "255", "65535").unwrap();
    same(&ordinary(&cast_binding), expected);
    for operation in [
        Operation::Literal(Literal::Unit),
        Operation::Default,
        Operation::Assert {
            condition: input(1, Type::Boolean),
            message: "stop".into(),
        },
    ] {
        assert!(
            expression(
                &binding(operation, Type::Unit),
                &syn::parse_quote!(()),
                |_| unreachable!(),
                |_| unreachable!()
            )
            .unwrap()
            .is_none()
        );
    }
}
