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

//! shared conversion syntax over checked SSA places, not semantic codecs.
use super::{Value, name, value};
use crate::ir::KernelClaimKind;
/// Canonical fields are Copy; retain aggregate reuse without cloning it per field.
pub(in super::super) fn coin_info(coin: &Value) -> syn::Expr {
    let coin = name(coin);
    syn::parse_quote!(runtime::ledger::coin_info_from_compact(#coin.nonce,#coin.color,#coin.value.value()))
}
pub(in super::super) fn qualified_coin(coin: &Value) -> syn::Expr {
    let info = coin_info(coin);
    let coin = name(coin);
    syn::parse_quote!(#info.qualify(#coin.mt_index.value() as u64))
}
pub(in super::super) fn recipient(recipient: &Value) -> syn::Expr {
    let recipient = name(recipient);
    syn::parse_quote!(runtime::ledger::coin_recipient_from_compact(#recipient.is_left,#recipient.left.bytes,#recipient.right.bytes))
}
pub(in super::super) fn claim(kind: KernelClaimKind, input: &Value) -> syn::Expr {
    let input = value(input);
    match kind {
        KernelClaimKind::Nullifier => {
            syn::parse_quote!(runtime::ledger::KernelClaim::Nullifier(runtime::ledger::CoinNullifier(runtime::ledger::HashOutput((#input).into_array()))))
        }
        KernelClaimKind::CoinSpend => {
            syn::parse_quote!(runtime::ledger::KernelClaim::CoinSpend(runtime::ledger::CoinCommitment(runtime::ledger::HashOutput((#input).into_array()))))
        }
        KernelClaimKind::CoinReceive => {
            syn::parse_quote!(runtime::ledger::KernelClaim::CoinReceive(runtime::ledger::CoinCommitment(runtime::ledger::HashOutput((#input).into_array()))))
        }
    }
}
pub(in super::super) fn identity(ty: &syn::Type, observed: syn::Expr) -> syn::Expr {
    syn::parse_quote!(#ty {bytes:runtime::FixedBytes::new(#observed)})
}
pub(in super::super) fn address(ty: &syn::Type, observed: syn::Expr) -> syn::Expr {
    syn::parse_quote!(#ty {bytes:runtime::ledger::contract_address_bytes(&#observed)})
}
