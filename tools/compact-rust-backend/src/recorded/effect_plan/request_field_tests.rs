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

use super::request_field;
use quote::{ToTokens, format_ident};
use syn::parse::Parser;

#[test]
fn nested_request_field_metadata_preserves_the_complete_private_carrier() {
    let ty: syn::Type = syn::parse_quote! {
        crate::runtime::FixedVector<crate::runtime::FixedVector<(
            crate::runtime::FixedVector<(u8, u8), 2>, u16
        ), 2>, 2>
    };
    let name = format_ident!("r#type");
    let before = syn::Field::parse_named
        .parse2(quote::quote!(#name: #ty))
        .unwrap();
    let mut actual = request_field(&name, &ty);
    assert_eq!(actual.attrs.len(), 1);
    let attr = actual.attrs.pop().unwrap();
    assert_eq!(attr.path().to_token_stream().to_string(), "allow");
    assert_eq!(
        attr.meta.to_token_stream().to_string(),
        quote::quote!(allow(
            clippy::type_complexity,
            reason = "preserves the Compact nested Vector/product carrier in this field"
        ))
        .to_string()
    );
    assert_eq!(
        actual.to_token_stream().to_string(),
        before.to_token_stream().to_string()
    );
    assert!(matches!(actual.vis, syn::Visibility::Inherited));
}

#[test]
fn nested_component_product_field_keeps_its_exact_publicly_usable_carrier() {
    // The same-type source compiles before widening is supported, but strict
    // generated-crate Clippy rejects this private input without local metadata.
    let types: Vec<syn::Type> = vec![
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<
                        (
                            (runtime::BoundedUint<255>, runtime::BoundedUint<255>),
                            runtime::BoundedUint<255>
                        ),
                        2,
                    >,
                    runtime::BoundedUint<255>
                ),
                2,
            >
        ),
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<
                        (
                            runtime::BoundedUint<255>,
                            (runtime::BoundedUint<255>, runtime::BoundedUint<255>)
                        ),
                        2,
                    >,
                    runtime::BoundedUint<255>
                ),
                2,
            >
        ),
    ];
    for ty in types {
        let name = format_ident!("input");
        let before = syn::Field::parse_named
            .parse2(quote::quote!(#name: #ty))
            .unwrap();
        let mut actual = request_field(&name, &ty);
        assert_eq!(actual.attrs.len(), 1);
        let attribute = actual.attrs.pop().unwrap();
        assert_eq!(
            attribute.meta.to_token_stream().to_string(),
            quote::quote!(allow(
                clippy::type_complexity,
                reason = "preserves the Compact nested Vector/product carrier in this field"
            ))
            .to_string()
        );
        assert_eq!(
            actual.to_token_stream().to_string(),
            before.to_token_stream().to_string()
        );
        assert!(matches!(actual.vis, syn::Visibility::Inherited));
    }
}

#[test]
fn request_field_metadata_keeps_prior_and_unrelated_carriers_unmarked() {
    let types: Vec<syn::Type> = vec![
        syn::parse_quote!(u8),
        syn::parse_quote!((u8, u16)),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2>),
        syn::parse_quote!(runtime::FixedVector<runtime::FixedVector<(u8, u16), 2>, 2>),
        syn::parse_quote!(
            runtime::FixedVector<runtime::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2>, 2>
        ),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<(u8, u8), 2>, u16), 2>),
        syn::parse_quote!(
            other::FixedVector<
                runtime::FixedVector<(runtime::FixedVector<(u8, u8), 2>, u16), 2>,
                2,
            >
        ),
        syn::parse_quote!(
            runtime::FixedVector<
                runtime::FixedVector<(other::FixedVector<(u8, u8), 2>, u16), 2>,
                2,
            >
        ),
    ];
    for ty in types {
        let actual = request_field(&format_ident!("input"), &ty);
        assert!(actual.attrs.is_empty(), "{}", ty.to_token_stream());
        assert_eq!(
            actual.ty.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }
}

#[test]
fn component_dimensions_and_mixed_product_fields_preserve_private_carriers() {
    let types: Vec<syn::Type> = vec![
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<runtime::FixedVector<(u8, u8), 2>, 2>,
                    u16
                ),
                2,
            >
        ),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<(u8, bool, u16), 2>, u8), 2>),
    ];
    for ty in types {
        let name = format_ident!("input");
        let before = syn::Field::parse_named
            .parse2(quote::quote!(#name:#ty))
            .unwrap();
        let mut actual = request_field(&name, &ty);
        assert_eq!(actual.attrs.len(), 1);
        actual.attrs.clear();
        assert_eq!(
            actual.to_token_stream().to_string(),
            before.to_token_stream().to_string()
        );
    }
}

#[test]
fn vector_leaf_component_fields_preserve_the_exact_private_carrier() {
    let types: Vec<syn::Type> = vec![
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<(runtime::FixedVector<u8, 2>, u8), 2>,
                    u8
                ),
                2,
            >
        ),
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<(u8, runtime::FixedVector<u8, 2>), 2>,
                    u8
                ),
                2,
            >
        ),
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<((runtime::FixedVector<u8, 1>,), u8), 1>,
                    u8
                ),
                1,
            >
        ),
        syn::parse_quote!(
            crate::runtime::FixedVector<
                (
                    crate::runtime::FixedVector<(u8, (crate::runtime::FixedVector<u8, 1>,)), 1>,
                    u8
                ),
                1,
            >
        ),
    ];
    for ty in types {
        let name = format_ident!("input");
        let before = syn::Field::parse_named
            .parse2(quote::quote!(#name: #ty))
            .unwrap();
        let mut actual = request_field(&name, &ty);
        assert_eq!(actual.attrs.len(), 1);
        let attribute = actual.attrs.pop().unwrap();
        assert_eq!(
            attribute.meta.to_token_stream().to_string(),
            quote::quote!(allow(
                clippy::type_complexity,
                reason = "preserves the Compact nested Vector/product carrier in this field"
            ))
            .to_string()
        );
        assert_eq!(
            actual.to_token_stream().to_string(),
            before.to_token_stream().to_string()
        );
        assert!(matches!(actual.vis, syn::Visibility::Inherited));
    }
}

#[test]
fn vector_leaf_metadata_does_not_mark_foreign_or_scalar_singleton_children() {
    let types: Vec<syn::Type> = vec![
        syn::parse_quote!(
            runtime::FixedVector<(runtime::FixedVector<(other::FixedVector<u8, 2>, u8), 2>, u8), 2>
        ),
        syn::parse_quote!(
            runtime::FixedVector<
                (
                    runtime::FixedVector<((other::FixedVector<u8, 1>,), u8), 1>,
                    u8
                ),
                1,
            >
        ),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<((u8,), u8), 1>, u8), 1>),
    ];
    for ty in types {
        let actual = request_field(&format_ident!("input"), &ty);
        assert!(actual.attrs.is_empty(), "{}", ty.to_token_stream());
        assert_eq!(
            actual.ty.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }
}
