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

//! Checked SSA values become official coin inputs to the declared storage slot.
use super::Value;

pub(super) fn write(slot: &syn::Ident, coin: &Value, recipient: &Value) -> syn::Stmt {
    storage_operation(slot, &syn::parse_quote!(record_write_coin), coin, recipient)
}

pub(super) fn insert(slot: &syn::Ident, coin: &Value, recipient: &Value) -> syn::Stmt {
    storage_operation(
        slot,
        &syn::parse_quote!(record_insert_coin),
        coin,
        recipient,
    )
}

fn storage_operation(
    slot: &syn::Ident,
    method: &syn::Ident,
    coin: &Value,
    recipient: &Value,
) -> syn::Stmt {
    let coin = super::effect_values::coin_info(coin);
    let recipient = super::effect_values::recipient(recipient);
    // Existing slot/frame retains recipient-bound commitment/index authority.
    syn::parse_quote! { let frame = crate::ledger_slots::#slot.#method(frame,#coin,#recipient)?; }
}
