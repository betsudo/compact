// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
//! New vector semantics are checked independently of the frozen tuple oracle.
use super::*;
use crate::pure_semantics::aggregate::tests::protocol::{Consumer, Visitor};

fn vector(element: Type, length: usize) -> Type {
    Type::Vector {
        element: Box::new(element),
        length,
    }
}
fn project(value: Expr, index: usize) -> Expr {
    Expr::TupleIndex {
        value: Box::new(value),
        index,
    }
}

#[test]
fn vector_projection_checks_first_last_singleton_and_retains_only_selected_owned_leaf() {
    for (element, length, index, expected) in [
        (Type::Field, 2, 0, quote::quote!((input).0[0])),
        (Type::Field, 2, 1, quote::quote!((input).0[1])),
        (Type::Boolean, 1, 0, quote::quote!((input).0[0])),
        (
            Type::OpaqueString,
            2,
            1,
            quote::quote!(((input).0[1]).clone()),
        ),
        (
            vector(Type::Field, 2),
            1,
            0,
            quote::quote!(((input).0[0]).clone()),
        ),
    ] {
        let ty = vector(element.clone(), length);
        let bindings = HashMap::from([("input", (&ty, syn::parse_quote!(input)))]);
        let source = project(reference("input"), index);
        let ((expression, statements, result), visits) =
            visitation::observe(|| observe(&source, &bindings, &HashMap::new()).unwrap());
        assert_eq!(result, element);
        assert_eq!(expression, expected.to_string());
        assert!(statements.is_empty());
        assert!(visits.values().all(|count| *count == 1));
        let mut sink = Consumer::default();
        let checked = Visitor::new(HashMap::from([("input", (&ty, ()))]), &HashMap::new())
            .check(&source, &mut sink)
            .unwrap();
        assert_eq!(checked.ty, result);
        assert!(
            matches!(sink.outputs.last(), Some(crate::pure_semantics::aggregate::CheckedAggregate::VectorIndex { index: actual, .. }) if *actual == index)
        );
    }
}

#[test]
fn vector_projection_refuses_empty_bounds_and_child_failure_before_index() {
    for (length, index) in [(0, 0), (2, 2), (2, usize::MAX)] {
        let ty = vector(Type::Field, length);
        let bindings = HashMap::from([("input", (&ty, syn::parse_quote!(input)))]);
        assert_eq!(
            observe(
                &project(reference("input"), index),
                &bindings,
                &HashMap::new()
            ),
            Err(RenderError::InvalidVectorIndex { index, length })
        );
    }
    assert_eq!(
        observe(
            &project(reference("missing"), usize::MAX),
            &HashMap::new(),
            &HashMap::new()
        ),
        Err(RenderError::UnknownParameter("missing".into()))
    );
}

#[test]
fn vector_projection_temporary_call_is_rendered_once() {
    let mut helper = pure("values", vector(Type::OpaqueString, 2));
    helper.body = Expr::Vector {
        element: Type::OpaqueString,
        elements: vec![
            Expr::Default {
                ty: Type::OpaqueString
            };
            2
        ],
    };
    let source = project(call("values"), 1);
    let (text, _, ty) = observe(
        &source,
        &HashMap::new(),
        &HashMap::from([("values", &helper)]),
    )
    .unwrap();
    assert_eq!(ty, Type::OpaqueString);
    assert_eq!(text.matches("crate :: pure_circuits :: values").count(), 1);
    assert_eq!(text.matches("clone").count(), 1);
}
