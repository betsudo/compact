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

//! Map identity and active operand types come from immutable declarations.
use super::*;

/// Scalar Map values are ledger Cells, not necessarily scalar Rust primitives.
/// Existing CellValue implementations cover owned products, fixed vectors and
/// canonical wide integers; tuple codecs stop at arity8 (FAB alone reaches11).
/// Canonical definitions and resource preflight remain the caller's authority.
/// Keep the established top-level Unit refusal; Unit members and empty named
/// products still have recursive CellValue representations.
/// Storage roles share codec capability; operation and declaration checks remain separate.
pub(in super::super) fn scalar_value_type(ty: &Type) -> bool {
    super::storage::map_value_type(ty)
}

pub(in super::super) fn types<'a>(
    declarations: &'a Declarations,
    slot: MapSlot,
    scalar: bool,
    path: &str,
) -> Result<(&'a Type, &'a Type), RecordingGap> {
    let field = declarations
        .field(slot.declaration)
        .ok_or_else(|| invalid(path, "unknown Map declaration"))?;
    if field.index != slot.index {
        return Err(invalid(path, "Map declaration index mismatch"));
    }
    let LedgerFieldKind::Map { key, value } = &field.declaration else {
        return Err(invalid(path, "expected Map declaration"));
    };
    declarations.types().validate_at(key, path)?;
    if !map_key_type(key) {
        return Err(invalid(path, "unsupported Map key type"));
    }
    if scalar {
        declarations.types().validate_at(value, path)?;
        if !scalar_value_type(value) {
            return Err(invalid(path, "unsupported scalar Map value type"));
        }
    } else {
        // Use the native slot authority, not a second approximation of its
        // representation rules. The bounded result is discarded; this grants
        // neither a scalar value codec nor a checked SSA value for nested Maps.
        crate::resource_limits::recording_type_nodes(value)
            .map_err(|e| invalid(path, e.to_string()))?;
        if crate::map_slot_types(key, value)
            .map_err(|e| invalid(path, e.to_string()))?
            .is_none()
        {
            return Err(invalid(path, "Map value has no native slot representation"));
        }
    }
    Ok((key, value))
}

/// Each plan is checked independently of lowering and its inferred annotations.
pub(in super::super) fn child_types(
    declarations: &Declarations,
    slot: MapSlot,
    path: &str,
) -> Result<(Type, Type, Type), RecordingGap> {
    let (outer, value) = types(declarations, slot, false, path)?;
    let Type::LedgerMap { key: inner, value } = value else {
        return Err(invalid(path, "expected nested Map declaration"));
    };
    declarations.types().validate_at(inner, path)?;
    declarations.types().validate_at(value, path)?;
    if !map_key_type(inner) || !scalar_value_type(value) {
        return Err(invalid(path, "unsupported nested Map key or value type"));
    }
    Ok((
        outer.clone(),
        inner.as_ref().clone(),
        value.as_ref().clone(),
    ))
}

pub(super) fn validate(
    operation: &Operation,
    result: &Type,
    declarations: &Declarations,
    scope: &HashMap<usize, Type>,
    path: &str,
) -> Result<bool, RecordingGap> {
    Ok(match operation {
        Operation::NestedMapLookup { slot, outer, inner } => {
            used(inner, scope, declarations, path)?;
            used(outer, scope, declarations, path)?;
            let (outer_ty, inner_ty, value) = child_types(declarations, *slot, path)?;
            outer.ty == outer_ty && inner.ty == inner_ty && *result == value
        }
        Operation::NestedMapInsert {
            slot,
            outer,
            inner,
            value,
        } => {
            used(inner, scope, declarations, path)?;
            used(value, scope, declarations, path)?;
            used(outer, scope, declarations, path)?;
            let (outer_ty, inner_ty, value_ty) = child_types(declarations, *slot, path)?;
            outer.ty == outer_ty
                && inner.ty == inner_ty
                && value.ty == value_ty
                && *result == Type::Unit
        }
        Operation::MapInitChild { slot, key } => {
            used(key, scope, declarations, path)?;
            let (outer, value) = types(declarations, *slot, false, path)?;
            matches!(value, Type::LedgerMap { .. }) && key.ty == *outer && *result == Type::Unit
        }
        Operation::MapMember { slot, key }
        | Operation::MapLookup { slot, key }
        | Operation::MapRemove { slot, key }
        | Operation::MapInsertDefault { slot, key } => {
            used(key, scope, declarations, path)?;
            let scalar = matches!(
                operation,
                Operation::MapLookup { .. } | Operation::MapInsertDefault { .. }
            );
            let (declared_key, declared_value) = types(declarations, *slot, scalar, path)?;
            let default_valid = !matches!(operation, Operation::MapInsertDefault { .. })
                || default_value_type(declared_value);
            key.ty == *declared_key
                && default_valid
                && *result
                    == match operation {
                        Operation::MapMember { .. } => Type::Boolean,
                        Operation::MapLookup { .. } => declared_value.clone(),
                        _ => Type::Unit,
                    }
        }
        Operation::MapInsert { slot, key, value } => {
            used(key, scope, declarations, path)?;
            used(value, scope, declarations, path)?;
            let (declared_key, declared_value) = types(declarations, *slot, true, path)?;
            key.ty == *declared_key && value.ty == *declared_value && *result == Type::Unit
        }
        Operation::MapReset { slot }
        | Operation::MapSize { slot }
        | Operation::MapIsEmpty { slot } => {
            types(
                declarations,
                *slot,
                matches!(operation, Operation::MapReset { .. }),
                path,
            )?;
            *result
                == match operation {
                    Operation::MapSize { .. } => uint("18446744073709551615"),
                    Operation::MapIsEmpty { .. } => Type::Boolean,
                    _ => Type::Unit,
                }
        }
        _ => unreachable!("Map validation dispatch"),
    })
}
