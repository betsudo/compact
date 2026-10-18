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

//! Scalar Map projection and syntax reuse the checked declaration/codec authority.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    invalid,
    model::{Binding, LedgerId, MapSlot, Operation, Value},
};
use crate::ir::Type;

pub(super) struct MapUse<'a> {
    pub slot: MapSlot,
    pub key: &'a Type,
    pub value: &'a Type,
    pub write: bool,
}
pub(super) struct FunctionalMapSlot {
    pub id: u32,
    pub declaration: LedgerId,
}
pub(super) struct MapStep {
    pub expression: syn::Expr,
    pub slot: FunctionalMapSlot,
    pub bound: syn::TypeParamBound,
}

/// Even membership/size/removal require a scalar value codec on this adapter.
/// Ordinary checked Maps support additional nested operations; no such support
/// is inferred from a shared operation name or from its lack of value operands.
pub(super) fn declaration<'a>(
    binding: &Binding,
    declarations: &'a Declarations,
) -> Result<Option<MapUse<'a>>, RecordingGap> {
    let (slot, write) = match &binding.operation {
        Operation::MapMember { slot, .. }
        | Operation::MapLookup { slot, .. }
        | Operation::MapSize { slot }
        | Operation::MapIsEmpty { slot } => (*slot, false),
        Operation::MapInsert { slot, .. }
        | Operation::MapInsertDefault { slot, .. }
        | Operation::MapRemove { slot, .. }
        | Operation::MapReset { slot } => (*slot, true),
        _ => return Ok(None),
    };
    let (key, value) = super::validate::maps::types(declarations, slot, true, &binding.path)?;
    Ok(Some(MapUse {
        slot,
        key,
        value,
        write,
    }))
}

pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    execution: syn::Expr,
    operand: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<MapStep>, RecordingGap> {
    let Some(map) = declaration(binding, declarations)? else {
        return Ok(None);
    };
    let id = u32::try_from(map.slot.declaration.0)
        .map_err(|_| invalid(&binding.path, "functional Map identity exceeds u32"))?;
    let key = rust_type(map.key).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let value = rust_type(map.value).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Map::<#key,#value>::new(
        crate::runtime::functional::SlotId::new(#id)));
    let bound = if map.write {
        syn::parse_quote!(crate::runtime::functional::MapWrite<#key,#value>)
    } else {
        syn::parse_quote!(crate::runtime::functional::MapRead<#key,#value>)
    };
    let expression = match &binding.operation {
        Operation::MapMember { key, .. } => {
            let key = operand(key);
            syn::parse_quote!(#execution.map_member(#token,#key))
        }
        Operation::MapLookup { key, .. } => {
            let key = operand(key);
            syn::parse_quote!(#execution.map_lookup(#token,#key))
        }
        Operation::MapInsert { key, value, .. } => {
            let key = operand(key);
            let value = operand(value);
            syn::parse_quote!(#execution.map_insert(#token,#key,#value))
        }
        Operation::MapRemove { key, .. } => {
            let key = operand(key);
            syn::parse_quote!(#execution.map_remove(#token,#key))
        }
        Operation::MapInsertDefault { key, .. } => {
            let key = operand(key);
            syn::parse_quote!(#execution.map_insert_default(#token,#key))
        }
        Operation::MapReset { .. } => syn::parse_quote!(#execution.map_reset(#token)),
        Operation::MapIsEmpty { .. } => syn::parse_quote!(#execution.map_is_empty(#token)),
        Operation::MapSize { .. } => {
            let ty = rust_type(&binding.value.ty)
                .map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(#execution.map_size(#token).bind(|__compact_map_size,__compact_execution| {
                __compact_execution.try_returning(|| <#ty>::new(__compact_map_size as u128))
            }))
        }
        _ => unreachable!("checked scalar Map dispatch"),
    };
    Ok(Some(MapStep {
        expression,
        slot: FunctionalMapSlot {
            id,
            declaration: map.slot.declaration,
        },
        bound,
    }))
}
