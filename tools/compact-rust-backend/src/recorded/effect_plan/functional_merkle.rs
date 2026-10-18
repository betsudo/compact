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

//! Merkle projection delegates checked declaration authority and existing slots.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    invalid,
    model::{Binding, LedgerId, MerkleOperation, MerkleSlot, Operation, Value},
};
use crate::ir::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MerkleAccess {
    Inspect,
    CheckRoot,
    Insert,
    InsertHash,
    Reset,
    ResetHistory,
}
pub(super) struct MerkleUse<'a> {
    pub slot: MerkleSlot,
    pub leaf: &'a Type,
    pub access: MerkleAccess,
}
pub(super) struct FunctionalMerkleSlot {
    pub id: u32,
    pub declaration: LedgerId,
    pub depth: u8,
    pub historic: bool,
}
pub(super) struct MerkleStep {
    pub expression: syn::Expr,
    pub slot: FunctionalMerkleSlot,
    pub bound: syn::TypeParamBound,
}

/// The checked graph owns operation operands/results. Recheck the declaration's
/// identity, depth, history and leaf codec through the original validator owner.
/// Integration requires only narrow effect_plan visibility on validate::merkle::leaf.
pub(super) fn declaration<'a>(
    binding: &Binding,
    declarations: &'a Declarations,
) -> Result<Option<MerkleUse<'a>>, RecordingGap> {
    let Operation::Merkle { slot, operation } = &binding.operation else {
        return Ok(None);
    };
    let leaf = super::validate::merkle::leaf(declarations, *slot, &binding.path)?;
    let access = match operation {
        MerkleOperation::IsFull => MerkleAccess::Inspect,
        MerkleOperation::CheckRoot { .. } => MerkleAccess::CheckRoot,
        MerkleOperation::Insert { .. } | MerkleOperation::InsertDefault { .. } => {
            MerkleAccess::Insert
        }
        MerkleOperation::InsertHash { .. } => MerkleAccess::InsertHash,
        MerkleOperation::ResetDefault => MerkleAccess::Reset,
        MerkleOperation::ResetHistory => MerkleAccess::ResetHistory,
    };
    Ok(Some(MerkleUse {
        slot: *slot,
        leaf,
        access,
    }))
}

pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
    execution: syn::Expr,
    operand: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<MerkleStep>, RecordingGap> {
    let Some(tree) = declaration(binding, declarations)? else {
        return Ok(None);
    };
    let id = u32::try_from(tree.slot.declaration.0)
        .map_err(|_| invalid(&binding.path, "functional Merkle identity exceeds u32"))?;
    let leaf = rust_type(tree.leaf).map_err(|error| invalid(&binding.path, error.to_string()))?;
    let depth = tree.slot.depth;
    let historic = tree.slot.historic;
    let token: syn::Expr = syn::parse_quote!(crate::runtime::functional::Merkle::<#leaf,#depth,#historic>::new(crate::runtime::functional::SlotId::new(#id)));
    let Operation::Merkle { operation, .. } = &binding.operation else {
        unreachable!("checked Merkle dispatch")
    };
    let bound = match tree.access {
        MerkleAccess::Inspect => {
            syn::parse_quote!(crate::runtime::functional::MerkleInspect<#leaf,#depth,#historic>)
        }
        MerkleAccess::CheckRoot => {
            let MerkleOperation::CheckRoot { root } = operation else {
                unreachable!("checked root operation")
            };
            let root =
                rust_type(&root.ty).map_err(|error| invalid(&binding.path, error.to_string()))?;
            syn::parse_quote!(crate::runtime::functional::MerkleCheckRoot<#leaf,#root,#depth,#historic>)
        }
        MerkleAccess::Insert => {
            syn::parse_quote!(crate::runtime::functional::MerkleInsert<#leaf,#depth,#historic>)
        }
        MerkleAccess::InsertHash => {
            syn::parse_quote!(crate::runtime::functional::MerkleInsertHash<#leaf,#depth,#historic>)
        }
        MerkleAccess::Reset => {
            syn::parse_quote!(crate::runtime::functional::MerkleReset<#leaf,#depth,#historic>)
        }
        MerkleAccess::ResetHistory => {
            syn::parse_quote!(crate::runtime::functional::MerkleResetHistory<#leaf,#depth>)
        }
    };
    let expression = match operation {
        MerkleOperation::Insert {
            value,
            position: None,
        } => {
            let value = operand(value);
            syn::parse_quote!(#execution.merkle_insert(#token,#value))
        }
        MerkleOperation::Insert {
            value,
            position: Some(position),
        } => {
            let value = operand(value);
            let position = operand(position);
            syn::parse_quote!(#execution.merkle_insert_index(#token,#value,#position))
        }
        MerkleOperation::InsertDefault { position } => {
            let position = operand(position);
            syn::parse_quote!(#execution.merkle_insert_default(#token,#position))
        }
        MerkleOperation::InsertHash {
            hash,
            position: None,
        } => {
            let hash = operand(hash);
            syn::parse_quote!(#execution.merkle_insert_hash(#token,#hash))
        }
        MerkleOperation::InsertHash {
            hash,
            position: Some(position),
        } => {
            let hash = operand(hash);
            let position = operand(position);
            syn::parse_quote!(#execution.merkle_insert_hash_index(#token,#hash,#position))
        }
        MerkleOperation::ResetDefault => syn::parse_quote!(#execution.merkle_reset(#token)),
        MerkleOperation::ResetHistory => syn::parse_quote!(#execution.merkle_reset_history(#token)),
        MerkleOperation::CheckRoot { root } => {
            let root = operand(root);
            syn::parse_quote!(#execution.merkle_check_root(#token,#root))
        }
        MerkleOperation::IsFull => syn::parse_quote!(#execution.merkle_is_full(#token)),
    };
    Ok(Some(MerkleStep {
        expression,
        slot: FunctionalMerkleSlot {
            id,
            declaration: tree.slot.declaration,
            depth,
            historic,
        },
        bound,
    }))
}
#[cfg(test)]
#[path = "functional_merkle_tests.rs"]
pub(super) mod tests;
