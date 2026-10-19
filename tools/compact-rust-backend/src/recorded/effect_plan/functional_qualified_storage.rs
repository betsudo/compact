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

//! Qualified storage uses existing shape, declaration and official conversion owners.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    functional_render::FunctionalCellSlot,
    functional_sets::FunctionalSetSlot,
    invalid,
    model::{Binding, Operation},
};
use crate::ir::Type;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QualifiedAccess {
    CellWrite,
    SetInsert,
}
pub(super) enum QualifiedSlot {
    Cell(FunctionalCellSlot),
    Set(FunctionalSetSlot),
}
pub(super) struct QualifiedStep {
    pub expression: syn::Expr,
    pub slot: QualifiedSlot,
    pub bound: syn::TypeParamBound,
}
pub(super) fn access(binding: &Binding) -> Option<QualifiedAccess> {
    match &binding.operation {
        Operation::CellWriteCoin { .. } => Some(QualifiedAccess::CellWrite),
        Operation::SetInsertCoin { .. } => Some(QualifiedAccess::SetInsert),
        _ => None,
    }
}
pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    execution: syn::Expr,
) -> Result<Option<QualifiedStep>, RecordingGap> {
    let (declaration, ty, coin, recipient, cell) = match &binding.operation {
        Operation::CellWriteCoin {
            slot,
            coin,
            recipient,
        } => (
            slot.declaration,
            super::validate::cell(declarations, *slot, &binding.path)?,
            coin,
            recipient,
            true,
        ),
        Operation::SetInsertCoin {
            slot,
            coin,
            recipient,
        } => (
            slot.declaration,
            super::validate::sets::element(declarations, *slot, &binding.path)?,
            coin,
            recipient,
            false,
        ),
        _ => return Ok(None),
    };
    // These exact shapes are already independently enforced by CheckedPlan. Keep
    // canonical ownership here too; never infer coin authority from field names.
    if *ty != crate::coin_shapes::qualified_coin_type()
        || coin.ty != crate::coin_shapes::shielded_coin_type()
        || recipient.ty != crate::coin_shapes::shielded_recipient_type()
        || binding.value.ty != Type::Unit
    {
        return Err(invalid(
            &binding.path,
            "qualified storage differs from checked canonical coin shapes",
        ));
    }
    let id = u32::try_from(declaration.0)
        .map_err(|_| invalid(&binding.path, "qualified storage identity exceeds u32"))?;
    let ty = rust_type(ty).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let coin = super::render::effect_values::coin_info(coin);
    let recipient = super::render::effect_values::recipient(recipient);
    let coin = super::functional_paths::expression(coin);
    let recipient = super::functional_paths::expression(recipient);
    let (expression, slot, bound) = if cell {
        let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Cell::<#ty>::new(crate::runtime::functional::SlotId::new(#id)));
        (
            syn::parse_quote!(#execution.write_qualified_coin(#token,#coin,#recipient)),
            QualifiedSlot::Cell(FunctionalCellSlot { id, declaration }),
            syn::parse_quote!(crate::runtime::functional::ledger8::QualifiedCellWrite<#ty>),
        )
    } else {
        let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Set::<#ty>::new(crate::runtime::functional::SlotId::new(#id)));
        (
            syn::parse_quote!(#execution.insert_qualified_coin(#token,#coin,#recipient)),
            QualifiedSlot::Set(FunctionalSetSlot { id, declaration }),
            syn::parse_quote!(crate::runtime::functional::ledger8::QualifiedSetInsert<#ty>),
        )
    };
    Ok(Some(QualifiedStep {
        expression,
        slot,
        bound,
    }))
}
#[cfg(test)]
#[path = "functional_qualified_storage_tests.rs"]
pub(super) mod tests;
