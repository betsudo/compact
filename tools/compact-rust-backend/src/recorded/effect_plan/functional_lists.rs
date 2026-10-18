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

//! List syntax shares declaration and native Maybe result authority.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    invalid,
    model::{Binding, LedgerId, ListOperation, ListSlot, Operation, Value},
};
use crate::ir::Type;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ListAccess {
    Read,
    Head,
    Write,
}
pub(super) struct ListUse<'a> {
    pub slot: ListSlot,
    pub element: &'a Type,
    pub access: ListAccess,
}
pub(super) struct FunctionalListSlot {
    pub id: u32,
    pub declaration: LedgerId,
}
pub(super) struct ListStep {
    pub expression: syn::Expr,
    pub slot: FunctionalListSlot,
    pub bound: syn::TypeParamBound,
}
pub(super) fn declaration<'a>(
    binding: &Binding,
    declarations: &'a Declarations,
) -> Result<Option<ListUse<'a>>, RecordingGap> {
    let Operation::List { slot, operation } = &binding.operation else {
        return Ok(None);
    };
    let element = super::validate::lists::element(declarations, *slot, &binding.path)?;
    let access = match operation {
        ListOperation::Length | ListOperation::IsEmpty => ListAccess::Read,
        ListOperation::Head => {
            super::validate::lists::head_type(
                declarations,
                element,
                &binding.value.ty,
                &binding.path,
            )?;
            ListAccess::Head
        }
        ListOperation::PushFront(_) | ListOperation::PopFront | ListOperation::Reset => {
            ListAccess::Write
        }
    };
    Ok(Some(ListUse {
        slot: *slot,
        element,
        access,
    }))
}
pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    execution: syn::Expr,
    operand: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<ListStep>, RecordingGap> {
    let Some(list) = declaration(binding, declarations)? else {
        return Ok(None);
    };
    let id = u32::try_from(list.slot.declaration.0)
        .map_err(|_| invalid(&binding.path, "functional List identity exceeds u32"))?;
    let element =
        rust_type(list.element).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::List::<#element>::new(
        crate::runtime::functional::SlotId::new(#id)));
    let bound = match list.access {
        ListAccess::Read => syn::parse_quote!(crate::runtime::functional::ListRead<#element>),
        ListAccess::Head => {
            let result = rust_type(&binding.value.ty)
                .map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(crate::runtime::functional::ListHead<#element,#result>)
        }
        ListAccess::Write => syn::parse_quote!(crate::runtime::functional::ListWrite<#element>),
    };
    let Operation::List { operation, .. } = &binding.operation else {
        unreachable!("checked List dispatch")
    };
    let expression = match operation {
        ListOperation::PushFront(value) => {
            let value = operand(value);
            syn::parse_quote!(#execution.list_push_front(#token,#value))
        }
        ListOperation::PopFront => syn::parse_quote!(#execution.list_pop_front(#token)),
        ListOperation::Reset => syn::parse_quote!(#execution.list_reset(#token)),
        ListOperation::IsEmpty => syn::parse_quote!(#execution.list_is_empty(#token)),
        ListOperation::Head => syn::parse_quote!(#execution.list_head(#token)),
        ListOperation::Length => {
            let result = rust_type(&binding.value.ty)
                .map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(#execution.list_length(#token).bind(|__compact_list_length,__compact_execution| {
                __compact_execution.try_returning(|| <#result>::new(__compact_list_length as u128))
            }))
        }
    };
    Ok(Some(ListStep {
        expression,
        slot: FunctionalListSlot {
            id,
            declaration: list.slot.declaration,
        },
        bound,
    }))
}
#[cfg(test)]
#[path = "functional_lists_tests.rs"]
mod tests;
