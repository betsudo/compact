// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

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
