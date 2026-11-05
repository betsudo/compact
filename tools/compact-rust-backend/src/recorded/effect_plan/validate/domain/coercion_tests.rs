// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Shared conversion facts retain the stricter recorded value domain.
use super::{Type, coercible, unsigned_bound, value_type};
use crate::ir::StructField;
mod old_reference;

fn uint(max: &str) -> Type {
    Type::Unsigned { max: max.into() }
}
fn tuple(elements: Vec<Type>) -> Type {
    Type::Tuple { elements }
}
fn vector(element: Type, length: usize) -> Type {
    Type::Vector {
        element: Box::new(element),
        length,
    }
}
fn named(field: Type) -> Type {
    Type::Struct {
        name: "Record".into(),
        fields: vec![StructField {
            name: "value".into(),
            ty: field,
        }],
    }
}

#[test]
fn nested_component_products_keep_checked_shape_bounds_and_empty_validation() {
    let nested = |leaf: Type| {
        vector(
            tuple(vec![
                vector(
                    tuple(vec![tuple(vec![leaf.clone(), uint("65535")]), leaf]),
                    1,
                ),
                uint("65535"),
            ]),
            2,
        )
    };
    let narrow = nested(uint("255"));
    let wide = nested(uint("65535"));
    assert!(coercible(&narrow, &wide));
    assert!(coercible(&narrow, &nested(Type::Field)));
    assert!(coercible(&nested(Type::Boolean), &nested(Type::Boolean)));
    assert!(!coercible(&wide, &narrow));
    assert!(!coercible(&nested(Type::Boolean), &wide));
    assert!(!coercible(&narrow, &nested(Type::OpaqueBytes)));

    let mut wrong_length = wide.clone();
    let Type::Vector { length, .. } = &mut wrong_length else {
        unreachable!()
    };
    *length = 3;
    assert!(!coercible(&narrow, &wrong_length));

    let mut wrong_arity = wide.clone();
    let Type::Vector { element, .. } = &mut wrong_arity else {
        unreachable!()
    };
    let Type::Tuple { elements } = element.as_mut() else {
        unreachable!()
    };
    elements.pop();
    assert!(!coercible(&narrow, &wrong_arity));

    fn component(ty: &mut Type) -> &mut Type {
        let Type::Vector { element, .. } = ty else {
            unreachable!()
        };
        let Type::Tuple { elements } = element.as_mut() else {
            unreachable!()
        };
        &mut elements[0]
    }
    fn nested_child(ty: &mut Type) -> &mut Type {
        let Type::Vector { element, .. } = component(ty) else {
            unreachable!()
        };
        let Type::Tuple { elements } = element.as_mut() else {
            unreachable!()
        };
        &mut elements[0]
    }
    let mut inner_length = wide.clone();
    let Type::Vector { length, .. } = component(&mut inner_length) else {
        unreachable!()
    };
    *length = 2;
    assert!(!coercible(&narrow, &inner_length));

    let mut inner_arity = wide.clone();
    let Type::Tuple { elements } = nested_child(&mut inner_arity) else {
        unreachable!()
    };
    elements.pop();
    assert!(!coercible(&narrow, &inner_arity));

    let mut scalar_child = wide.clone();
    *nested_child(&mut scalar_child) = uint("65535");
    assert!(!coercible(&narrow, &scalar_child));

    // Static empty values still validate the entire conversion, not only
    // iterations reached at runtime. Invalid identity leaves stay invalid too.
    assert!(!coercible(&vector(wide, 0), &vector(narrow, 0)));
    let invalid = nested(uint("00"));
    assert!(!coercible(&invalid, &invalid));
}

#[test]
fn vector_leaves_in_component_products_keep_static_conversion_boundaries() {
    // The frontend slice selects this relation; both the native checked plan
    // and the independent recorded-domain validator already compose it.
    let shape = |leaf: Type, wrapped: bool, outer: usize, inner: usize, leaves: usize| {
        let child = vector(leaf, leaves);
        let child = if wrapped { tuple(vec![child]) } else { child };
        vector(
            tuple(vec![
                vector(tuple(vec![child, uint("255")]), inner),
                uint("65535"),
            ]),
            outer,
        )
    };
    for wrapped in [false, true] {
        let narrow = shape(uint("255"), wrapped, 2, 1, 2);
        let wide = shape(uint("65535"), wrapped, 2, 1, 2);
        let field = shape(Type::Field, wrapped, 2, 1, 2);
        for target in [&wide, &field] {
            assert!(crate::pure_semantics::coercion::plan_coercion(&narrow, target).is_ok());
            assert!(coercible(&narrow, target));
        }
        for ty in [Type::Boolean, Type::Field] {
            let identity = shape(ty, wrapped, 2, 1, 2);
            assert!(crate::pure_semantics::coercion::plan_coercion(&identity, &identity).is_ok());
            assert!(coercible(&identity, &identity));
        }
        for target in [
            shape(uint("15"), wrapped, 2, 1, 2),
            shape(uint("65535"), wrapped, 3, 1, 2),
            shape(uint("65535"), wrapped, 2, 2, 2),
            shape(uint("65535"), wrapped, 2, 1, 3),
            shape(Type::Boolean, wrapped, 2, 1, 2),
            shape(Type::OpaqueBytes, wrapped, 2, 1, 2),
        ] {
            assert!(crate::pure_semantics::coercion::plan_coercion(&narrow, &target).is_err());
            assert!(!coercible(&narrow, &target));
        }
        let wrong_arity = vector(tuple(vec![uint("65535")]), 2);
        assert!(!coercible(&narrow, &wrong_arity));
        let wrong_result = vector(Type::Field, 2);
        assert!(!coercible(&narrow, &wrong_result));

        // No runtime element is needed to validate the original leaf types.
        for (outer, leaves) in [(0, 2), (2, 0), (0, 0)] {
            let from = shape(uint("255"), wrapped, outer, 1, leaves);
            let to = shape(uint("65535"), wrapped, outer, 1, leaves);
            assert!(coercible(&from, &to));
            assert!(!coercible(&to, &from));
            let invalid = shape(uint("00"), wrapped, outer, 1, leaves);
            assert!(!coercible(&invalid, &invalid));
        }
    }
}

#[test]
fn recorded_conversion_matches_the_frozen_relation_on_bounded_domains() {
    let mut cases = vec![
        Type::Unit,
        Type::Boolean,
        Type::Field,
        Type::JubjubPoint,
        Type::OpaqueString,
        Type::OpaqueBytes,
        Type::Bytes { length: 0 },
        Type::Bytes { length: 32 },
        Type::Bytes { length: 65537 },
        uint("0"),
        uint("15"),
        uint("255"),
        uint("340282366920938463463374607431768211456"),
        uint(""),
        uint("00"),
        uint("-1"),
        tuple(vec![]),
        tuple(vec![Type::Boolean; 13]),
        Type::Struct {
            name: "Empty".into(),
            fields: vec![],
        },
        named(Type::OpaqueString),
        named(tuple(vec![Type::Boolean; 9])),
        Type::Enum {
            name: "Tag".into(),
            variants: vec!["A".into(), "B".into()],
        },
        Type::Enum {
            name: "EmptyTag".into(),
            variants: vec![],
        },
        Type::LedgerMap {
            key: Box::new(Type::Boolean),
            value: Box::new(Type::Field),
        },
    ];
    for ty in [
        Type::Boolean,
        Type::Field,
        uint("15"),
        uint("255"),
        uint("00"),
        Type::OpaqueBytes,
    ] {
        cases.extend([
            vector(ty.clone(), 0),
            vector(ty.clone(), 2),
            tuple(vec![ty.clone(), ty.clone()]),
            vector(tuple(vec![ty.clone(), Type::Boolean]), 2),
            tuple(vec![vector(ty.clone(), 2), vector(ty, 2)]),
        ]);
    }
    for source in &cases {
        for target in &cases {
            assert_eq!(
                coercible(source, target),
                old_reference::coercible(source, target),
                "source={source:?}, target={target:?}"
            );
        }
    }
}

#[test]
fn native_identity_does_not_bypass_recorded_domain_requirements() {
    for ty in [
        uint("00"),
        Type::LedgerMap {
            key: Box::new(Type::Boolean),
            value: Box::new(Type::Boolean),
        },
        named(tuple(vec![Type::Boolean; 9])),
        Type::Enum {
            name: "Empty".into(),
            variants: vec![],
        },
    ] {
        assert!(crate::pure_semantics::coercion::plan_coercion(&ty, &ty).is_ok());
        assert!(!coercible(&ty, &ty));
    }
    let owned = tuple(vec![Type::OpaqueString, Type::OpaqueBytes]);
    assert!(coercible(&owned, &owned));
}

#[test]
fn root_preflight_refuses_oversized_private_recorded_conversions() {
    let large = Type::Enum {
        name: "TooLarge".into(),
        variants: vec!["x".repeat(262145)],
    };
    assert!(!coercible(&large, &large));
    let near = vector(uint("15"), usize::MAX);
    assert!(
        coercible(&near, &vector(uint("255"), usize::MAX)),
        "declared vector length does not unroll conversion planning"
    );
}

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
