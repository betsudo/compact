// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Checked SSA operands map directly to official frame intents.
use super::Value;

pub(super) fn input(coin: &Value) -> syn::Stmt {
    let coin = super::effect_values::qualified_coin(coin);
    syn::parse_quote! { let frame = frame.create_zswap_input(#coin); }
}
pub(super) fn output(coin: &Value, recipient: &Value) -> syn::Stmt {
    let coin = super::effect_values::coin_info(coin);
    let recipient = super::effect_values::recipient(recipient);
    syn::parse_quote! { let frame = frame.create_zswap_output(#coin,#recipient)?; }
}
