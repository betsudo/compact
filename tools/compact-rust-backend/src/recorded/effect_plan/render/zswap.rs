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
use super::{Value, name};

pub(super) fn input(coin: &Value) -> syn::Stmt {
    let coin = name(coin);
    // Canonical coin fields are Copy; the aggregate remains available for reuse.
    // Independent validation proves value is Uint128 and mt_index is Uint64.
    syn::parse_quote! {
        let frame = frame.create_zswap_input(
            runtime::ledger::coin_info_from_compact(#coin.nonce, #coin.color, #coin.value.value())
                .qualify(#coin.mt_index.value() as u64)
        );
    }
}

pub(super) fn output(coin: &Value, recipient: &Value) -> syn::Stmt {
    let (coin, recipient) = (name(coin), name(recipient));
    // The official frame owns allocation errors and the single private Unit entry.
    syn::parse_quote! {
        let frame = frame.create_zswap_output(
            runtime::ledger::coin_info_from_compact(#coin.nonce, #coin.color, #coin.value.value()),
            runtime::ledger::coin_recipient_from_compact(
                #recipient.is_left, #recipient.left.bytes, #recipient.right.bytes
            )
        )?;
    }
}
