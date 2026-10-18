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
