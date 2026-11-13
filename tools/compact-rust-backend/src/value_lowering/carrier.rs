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

//! Native formatting of carrier facts already validated by pure_semantics.

use crate::{
    naming::checked_ident,
    pure_semantics::{
        carrier::{CarrierSink, Leaf},
        numeric::UnsignedMaximum,
    },
};
use proc_macro2::Span;

pub(crate) struct NativeCarrierSink;

impl CarrierSink for NativeCarrierSink {
    type Value = syn::Type;
    type Tuple = syn::TypeTuple;

    fn leaf(&mut self, leaf: Leaf<'_>) -> syn::Type {
        match leaf {
            Leaf::Unit => syn::parse_quote!(()),
            Leaf::Boolean => syn::parse_quote!(bool),
            Leaf::Field => syn::parse_quote!(runtime::Field),
            Leaf::JubjubScalar => syn::parse_quote!(runtime::JubjubScalar),
            Leaf::JubjubPoint => syn::parse_quote!(runtime::JubjubPoint),
            Leaf::Secp256k1Base => syn::parse_quote!(runtime::Secp256k1Base),
            Leaf::Secp256k1Scalar => syn::parse_quote!(runtime::Secp256k1Scalar),
            Leaf::Secp256r1Base => syn::parse_quote!(runtime::Secp256r1Base),
            Leaf::Curve25519Base => syn::parse_quote!(runtime::Curve25519Base),
            Leaf::Secp256r1Scalar => syn::parse_quote!(runtime::Secp256r1Scalar),
            Leaf::Curve25519Scalar => syn::parse_quote!(runtime::Curve25519Scalar),
            Leaf::Secp256k1Point => syn::parse_quote!(runtime::Secp256k1Point),
            Leaf::Secp256r1Point => syn::parse_quote!(runtime::Secp256r1Point),
            Leaf::OpaqueString => syn::parse_quote!(runtime::OpaqueString),
            Leaf::OpaqueBytes => syn::parse_quote!(runtime::OpaqueBytes),
            Leaf::Bytes(length) => {
                let length = syn::LitInt::new(&length.to_string(), Span::call_site());
                syn::parse_quote!(runtime::FixedBytes<#length>)
            }
            Leaf::Named(name) => {
                let name = checked_ident(&name);
                syn::parse_quote!(crate::types::#name)
            }
            Leaf::Unsigned { spelling, maximum } => match maximum {
                UnsignedMaximum::Small(_) => {
                    let max = syn::LitInt::new(spelling, Span::call_site());
                    syn::parse_quote!(runtime::BoundedUint<#max>)
                }
                UnsignedMaximum::Wide { high, low } => {
                    let high = syn::LitInt::new(&format!("{high}u128"), Span::call_site());
                    let low = syn::LitInt::new(&format!("{low}u128"), Span::call_site());
                    syn::parse_quote!(runtime::WideUint<#high, #low>)
                }
            },
        }
    }

    fn tuple(&mut self) -> syn::TypeTuple {
        syn::TypeTuple {
            paren_token: syn::token::Paren::default(),
            elems: syn::punctuated::Punctuated::new(),
        }
    }

    fn element(&mut self, tuple: &mut syn::TypeTuple, element: syn::Type) {
        tuple.elems.push_value(element);
        tuple.elems.push_punct(syn::token::Comma::default());
    }

    fn finish_tuple(&mut self, mut tuple: syn::TypeTuple) -> syn::Type {
        if tuple.elems.len() > 1 {
            tuple.elems.pop_punct();
        }
        syn::Type::Tuple(tuple)
    }

    fn vector(&mut self, element: syn::Type, length: usize) -> syn::Type {
        let length = syn::LitInt::new(&length.to_string(), Span::call_site());
        syn::parse_quote!(runtime::FixedVector<#element, #length>)
    }
}
