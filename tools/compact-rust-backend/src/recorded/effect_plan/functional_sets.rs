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

//! Scalar Set syntax reuses checked declaration eligibility and runtime capabilities.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    invalid,
    model::{Binding, LedgerId, Operation, SetSlot, Value},
};
use crate::ir::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SetAccess {
    Read,
    Write,
}
pub(super) struct SetUse<'a> {
    pub slot: SetSlot,
    pub element: &'a Type,
    pub access: SetAccess,
}
pub(super) struct FunctionalSetSlot {
    pub id: u32,
    pub declaration: LedgerId,
}
pub(super) struct SetStep {
    pub expression: syn::Expr,
    pub slot: FunctionalSetSlot,
    pub bound: syn::TypeParamBound,
}

/// Coin allocation is not an ordinary Set insertion capability.
pub(super) fn declaration<'a>(
    binding: &Binding,
    declarations: &'a Declarations,
) -> Result<Option<SetUse<'a>>, RecordingGap> {
    let (slot, access) = match &binding.operation {
        Operation::SetMember { slot, .. }
        | Operation::SetSize { slot }
        | Operation::SetIsEmpty { slot } => (*slot, SetAccess::Read),
        Operation::SetInsert { slot, .. }
        | Operation::SetRemove { slot, .. }
        | Operation::SetReset { slot } => (*slot, SetAccess::Write),
        _ => return Ok(None),
    };
    let element = super::validate::sets::element(declarations, slot, &binding.path)?;
    Ok(Some(SetUse {
        slot,
        element,
        access,
    }))
}

pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    execution: syn::Expr,
    operand: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<SetStep>, RecordingGap> {
    let Some(set) = declaration(binding, declarations)? else {
        return Ok(None);
    };
    let id = u32::try_from(set.slot.declaration.0)
        .map_err(|_| invalid(&binding.path, "functional Set identity exceeds u32"))?;
    let element =
        rust_type(set.element).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Set::<#element>::new(
        crate::runtime::functional::SlotId::new(#id)));
    let bound = match set.access {
        SetAccess::Read => syn::parse_quote!(crate::runtime::functional::SetRead<#element>),
        SetAccess::Write => syn::parse_quote!(crate::runtime::functional::SetWrite<#element>),
    };
    let expression = match &binding.operation {
        Operation::SetMember { value, .. } => {
            let value = operand(value);
            syn::parse_quote!(#execution.set_member(#token,#value))
        }
        Operation::SetInsert { value, .. } => {
            let value = operand(value);
            syn::parse_quote!(#execution.set_insert(#token,#value))
        }
        Operation::SetRemove { value, .. } => {
            let value = operand(value);
            syn::parse_quote!(#execution.set_remove(#token,#value))
        }
        Operation::SetReset { .. } => syn::parse_quote!(#execution.set_reset(#token)),
        Operation::SetIsEmpty { .. } => syn::parse_quote!(#execution.set_is_empty(#token)),
        Operation::SetSize { .. } => {
            let ty = rust_type(&binding.value.ty)
                .map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(#execution.set_size(#token).bind(|__compact_set_size,__compact_execution| {
                __compact_execution.try_returning(|| <#ty>::new(__compact_set_size as u128))
            }))
        }
        _ => unreachable!("checked Set dispatch"),
    };
    Ok(Some(SetStep {
        expression,
        slot: FunctionalSetSlot {
            id,
            declaration: set.slot.declaration,
        },
        bound,
    }))
}

#[cfg(test)]
#[path = "functional_sets_tests.rs"]
mod tests;
