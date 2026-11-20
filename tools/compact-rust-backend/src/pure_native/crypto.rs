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

//! Exact runtime call syntax from arity-safe checked crypto facts.
//! No Type/source inspection, eligibility decision or operand checking occurs.
use super::leaf::NativeLeafSink;
use crate::{
    hash_input::render_input,
    jubjub_value,
    pure_semantics::{crypto::CheckedCrypto, sink::CryptoSink},
    value_lowering::secp256k1_unary_syntax,
};

impl CryptoSink for NativeLeafSink {
    fn crypto(&mut self, checked: CheckedCrypto<syn::Expr>) -> syn::Expr {
        match checked {
            CheckedCrypto::TransientHash { value, encoding } => {
                let value = render_input(value, encoding);
                syn::parse_quote!(runtime::transient_hash(#value))
            }
            CheckedCrypto::TransientCommit {
                value,
                opening,
                encoding,
            } => {
                let value = render_input(value, encoding);
                syn::parse_quote!(runtime::transient_commit(#value, #opening))
            }
            CheckedCrypto::PersistentHash(value) => {
                syn::parse_quote!(runtime::persistent_hash(#value))
            }
            CheckedCrypto::PersistentCommit { value, opening } => {
                syn::parse_quote!(runtime::persistent_commit(#value, #opening))
            }
            CheckedCrypto::Keccak256(value, encoding) => render_keccak(value, encoding),
            CheckedCrypto::Sha512(value) => syn::parse_quote!(runtime::sha512(#value)?),
            CheckedCrypto::DegradeToTransient(value) => {
                syn::parse_quote!(runtime::degrade_to_transient(#value))
            }
            CheckedCrypto::UpgradeFromTransient(value) => {
                syn::parse_quote!(runtime::upgrade_from_transient(#value))
            }
            CheckedCrypto::HashToCurve { value, encoding } => {
                let value = render_input(value, encoding);
                syn::parse_quote!(runtime::hash_to_curve(#value))
            }
            CheckedCrypto::Curve(checked) => jubjub_value::render(checked),
            CheckedCrypto::Secp256k1Unary(kind, value) => secp256k1_unary_syntax(kind, value),
        }
    }
}

pub(crate) fn render_keccak(value: syn::Expr, encoding: crate::ir::KeccakEncoding) -> syn::Expr {
    match encoding {
        crate::ir::KeccakEncoding::NormalizedAtoms => syn::parse_quote!(runtime::keccak256(#value)),
        crate::ir::KeccakEncoding::Aligned => {
            syn::parse_quote!(runtime::keccak256_aligned(#value)?)
        }
    }
}
