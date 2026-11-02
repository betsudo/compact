// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use quote::ToTokens;

#[test]
fn vector_component_signature_metadata_preserves_types_bodies_and_item_scope() {
    let mut file: syn::File = syn::parse_quote! {
        mod api {
            fn ordinary(
                input: runtime::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2>
            ) -> runtime::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2> { input }
            struct Program;
            impl Program {
                fn execute(self) -> Result<
                    crate::runtime::FixedVector<(u16, crate::runtime::FixedVector<u8, 2>), 2>,
                    Error
                > { run_once() }
            }
            trait Witnesses {
                fn vector_value(&self) -> runtime::FixedVector<
                    (runtime::FixedVector<u8, 2>, u16), 2
                >;
            }
            fn simple(input: runtime::FixedVector<(u8, u16), 2>) { consume(input); }
        }
    };
    let before = file.to_token_stream().to_string();
    CompactSignatureLint.visit_file_mut(&mut file);
    let printed = prettyplease::unparse(&file);
    assert_eq!(printed.matches("clippy::type_complexity").count(), 3);
    assert_eq!(
        printed
            .matches("preserves Compact Vector/product carriers")
            .count(),
        3
    );
    assert!(!printed.contains("#![allow"));
    // Remove only the three expected function-level attributes. Everything else,
    // including the complete public carriers and the once-only body, is identical.
    let syn::Item::Mod(module) = &mut file.items[0] else {
        panic!("module")
    };
    assert!(module.attrs.is_empty());
    let items = &mut module.content.as_mut().unwrap().1;
    let syn::Item::Fn(function) = &mut items[0] else {
        panic!("function")
    };
    assert_eq!(function.attrs.len(), 1);
    function.attrs.clear();
    let syn::Item::Impl(implementation) = &mut items[2] else {
        panic!("impl")
    };
    assert!(implementation.attrs.is_empty());
    let syn::ImplItem::Fn(method) = &mut implementation.items[0] else {
        panic!("method")
    };
    assert_eq!(method.attrs.len(), 1);
    method.attrs.clear();
    let syn::Item::Trait(witnesses) = &mut items[3] else {
        panic!("trait")
    };
    assert!(witnesses.attrs.is_empty());
    let syn::TraitItem::Fn(method) = &mut witnesses.items[0] else {
        panic!("witness")
    };
    assert_eq!(method.attrs.len(), 1);
    method.attrs.clear();
    let syn::Item::Fn(simple) = &items[4] else {
        panic!("simple")
    };
    assert!(simple.attrs.is_empty());
    assert_eq!(file.to_token_stream().to_string(), before);
}

#[test]
fn signature_metadata_retains_previous_shapes_and_excludes_unrelated_types() {
    let marked: Vec<syn::Type> = vec![
        syn::parse_quote!(runtime::FixedVector<(u8, u16, u32), 2>),
        syn::parse_quote!(runtime::FixedVector<runtime::FixedVector<(u8, u16), 2>, 2>),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2>),
        syn::parse_quote!(
            crate::runtime::FixedVector<(u16, crate::runtime::FixedVector<u8, 2>), 2>
        ),
    ];
    let unmarked: Vec<syn::Type> = vec![
        syn::parse_quote!(runtime::FixedVector<(u8, u16), 2>),
        syn::parse_quote!(runtime::FixedVector<runtime::FixedVector<u8, 2>, 2>),
        syn::parse_quote!(runtime::FixedVector<(other::FixedVector<u8, 2>, u16), 2>),
        syn::parse_quote!(other::FixedVector<(runtime::FixedVector<u8, 2>, u16), 2>),
        syn::parse_quote!(runtime::FixedVector<(runtime::FixedVector<u8, 2>,), 2>),
        syn::parse_quote!((runtime::FixedVector<u8, 2>, u16)),
    ];
    for (types, expected) in [(marked, true), (unmarked, false)] {
        for ty in types {
            let signature: syn::Signature = syn::parse_quote!(fn example(input: #ty));
            assert_eq!(
                CompactSignatureLint::has_complex_product_vector(&signature),
                expected,
                "{}",
                ty.to_token_stream()
            );
        }
    }
}
