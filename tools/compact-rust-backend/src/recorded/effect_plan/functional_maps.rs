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

//! Typed Map projection distinguishes structural inspection from scalar access.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    invalid,
    model::{Binding, LedgerId, MapSlot, Operation, Value},
};
use crate::ir::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MapAccess {
    Read,
    Lookup,
    Remove,
    Write,
}
pub(super) struct MapUse<'a> {
    pub slot: MapSlot,
    pub key: &'a Type,
    pub value: &'a Type,
    pub access: MapAccess,
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

/// Use the same per-operation scalar distinction as checked Map validation.
/// Structural MapNode markers are type-only; inspection/removal never decode V.
pub(super) fn declaration<'a>(
    binding: &Binding,
    declarations: &'a Declarations,
) -> Result<Option<MapUse<'a>>, RecordingGap> {
    let (slot, access) = match &binding.operation {
        Operation::MapMember { slot, .. }
        | Operation::MapSize { slot }
        | Operation::MapIsEmpty { slot } => (*slot, MapAccess::Read),
        Operation::MapLookup { slot, .. } => (*slot, MapAccess::Lookup),
        Operation::MapRemove { slot, .. } => (*slot, MapAccess::Remove),
        Operation::MapInsert { slot, .. }
        | Operation::MapInsertDefault { slot, .. }
        | Operation::MapReset { slot } => (*slot, MapAccess::Write),
        _ => return Ok(None),
    };
    let scalar = matches!(access, MapAccess::Lookup | MapAccess::Write);
    let (key, value) = super::validate::maps::types(declarations, slot, scalar, &binding.path)?;
    Ok(Some(MapUse {
        slot,
        key,
        value,
        access,
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
    let (key, value) = crate::map_slot_types(map.key, map.value)
        .map_err(|error| invalid(&binding.path, error.to_string()))?
        .ok_or_else(|| {
            invalid(
                &binding.path,
                "Map declaration has no native slot representation",
            )
        })?;
    let key = super::functional_paths::type_syntax(key);
    let value = super::functional_paths::type_syntax(value);
    let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Map::<#key,#value>::new(
        crate::runtime::functional::SlotId::new(#id)));
    let bound = match map.access {
        MapAccess::Read => syn::parse_quote!(crate::runtime::functional::MapRead<#key,#value>),
        MapAccess::Lookup => syn::parse_quote!(crate::runtime::functional::MapLookup<#key,#value>),
        MapAccess::Remove => syn::parse_quote!(crate::runtime::functional::MapRemove<#key,#value>),
        MapAccess::Write => syn::parse_quote!(crate::runtime::functional::MapWrite<#key,#value>),
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
        _ => unreachable!("checked Map dispatch"),
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
