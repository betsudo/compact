// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

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
