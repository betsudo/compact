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

//! circuit-only Ledger8 capabilities over existing official domains.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::{expression, rust_type},
    invalid,
    model::{Binding, Operation},
};
pub(super) struct KernelStep {
    pub expression: syn::Expr,
    pub bound: syn::TypeParamBound,
}
pub(super) fn render(
    binding: &Binding,
    _declarations: &Declarations,
    execution: syn::Expr,
) -> Result<Option<KernelStep>, RecordingGap> {
    let (step, bound) = match &binding.operation {
        Operation::OwnPublicKey => {
            let ty =
                rust_type(&binding.value.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
            let wrapped = expression(super::render::effect_values::identity(
                &ty,
                syn::parse_quote!(__compact_identity),
            ));
            (
                syn::parse_quote!(#execution.own_public_key().bind(|__compact_identity,__compact_execution| __compact_execution.returning(#wrapped))),
                syn::parse_quote!(crate::runtime::functional::ledger8::ExecutionIdentity),
            )
        }
        Operation::KernelSelf => {
            let ty =
                rust_type(&binding.value.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
            let wrapped = expression(super::render::effect_values::address(
                &ty,
                syn::parse_quote!(__compact_address),
            ));
            (
                syn::parse_quote!(#execution.kernel_address().bind(|__compact_address,__compact_execution| __compact_execution.returning(#wrapped))),
                syn::parse_quote!(crate::runtime::functional::ledger8::KernelAddress),
            )
        }
        Operation::KernelClaim { kind, value } => {
            let claim = expression(super::render::effect_values::claim(*kind, value));
            (
                syn::parse_quote!(#execution.kernel_claim(#claim)),
                syn::parse_quote!(crate::runtime::functional::ledger8::KernelClaims),
            )
        }
        Operation::KernelMintShielded { domain, amount } => {
            let domain = super::functional_render::value(domain);
            let amount = super::functional_render::value(amount);
            (
                syn::parse_quote!(#execution.mint_shielded(crate::runtime::ledger::HashOutput((#domain).into_array()),(#amount).value() as u64)),
                syn::parse_quote!(crate::runtime::functional::ledger8::ShieldedMint),
            )
        }
        Operation::CreateZswapInput { coin } => {
            let coin = expression(super::render::effect_values::qualified_coin(coin));
            (
                syn::parse_quote!(#execution.zswap_input(#coin)),
                syn::parse_quote!(crate::runtime::functional::ledger8::ZswapInput),
            )
        }
        Operation::CreateZswapOutput { coin, recipient } => {
            let coin = expression(super::render::effect_values::coin_info(coin));
            let recipient = expression(super::render::effect_values::recipient(recipient));
            (
                syn::parse_quote!(#execution.zswap_output(#coin,#recipient)),
                syn::parse_quote!(crate::runtime::functional::ledger8::ZswapOutput),
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(KernelStep {
        expression: step,
        bound,
    }))
}
