// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Optional proof-data admission for an authenticated three-Cell clear.
//!
//! This checks a fixed, bounded graph: state guard, one Bytes<32> witness,
//! one pure persistent hash, authority guard, and three default writes.
//! It neither executes nor reconstructs witnesses, hashes or transcripts.
//! The existing recorded method and runtime carrier own those operations.

use crate::ir::{
    Expr, LedgerField, LedgerFieldKind, LocalBinding, PureCircuit, StateAction, StateReturn,
    StatefulCircuit, Type, WitnessDeclaration,
};
use std::collections::HashMap;

const KEY: Type = Type::Bytes { length: 32 };

// Identity coercions are the only wrappers accepted here. Fixed helper/action
// arities below prevent recursion; cap coercions independently of IR validity.
fn uncoerce<'a>(mut value: &'a Expr, expected: &Type) -> Option<&'a Expr> {
    for _ in 0..8 {
        match value {
            Expr::Coerce { value: inner, ty } if ty == expected => value = inner,
            Expr::Coerce { .. } => return None,
            _ => return Some(value),
        }
    }
    None
}

fn parameter(value: &Expr, name: &str, ty: &Type) -> bool {
    matches!(uncoerce(value, ty), Some(Expr::Parameter { name: actual }) if actual == name)
}

fn cell<'a>(fields: &'a HashMap<&str, &LedgerField>, field: &str, index: u8) -> Option<&'a Type> {
    let declaration = fields.get(field)?;
    if declaration.index != index {
        return None;
    }
    // The caller has validated complete physical paths and disjoint slots.
    match &declaration.declaration {
        LedgerFieldKind::Cell { ty } => Some(ty),
        _ => None,
    }
}

fn binding(action: &StateAction) -> Option<(&LocalBinding, &StateAction)> {
    let StateAction::Let { bindings, action } = action else {
        return None;
    };
    let [binding] = bindings.as_slice() else {
        return None;
    };
    (!binding.name.is_empty()).then_some((binding, action))
}

/// Return the guarded state Cell and its exact two-variant type. The helper
/// is inspected in full, not accepted by its spelling or its Boolean result.
fn state_guard<'a>(
    action: &StateAction,
    circuits: &[StatefulCircuit],
    fields: &'a HashMap<&str, &LedgerField>,
) -> Option<(String, u8, &'a Type)> {
    let StateAction::Assert {
        condition: Expr::Call { name, arguments },
        ..
    } = action
    else {
        return None;
    };
    let [argument] = arguments.as_slice() else {
        return None;
    };
    let helper = circuits.iter().find(|c| c.name == *name)?;
    let [input] = helper.parameters.as_slice() else {
        return None;
    };
    if helper.result != Type::Boolean || !helper.actions.is_empty() {
        return None;
    }
    let StateReturn::Expression {
        value: Expr::Equal { left, right },
    } = &helper.return_value
    else {
        return None;
    };
    let Expr::CellRead { field, index } = left.as_ref() else {
        return None;
    };
    let ty = cell(fields, field, *index)?;
    let Type::Enum { variants, .. } = ty else {
        return None;
    };
    if variants.len() != 2 || input.ty != *ty || !parameter(right, &input.name, ty) {
        return None;
    }
    if !matches!(uncoerce(argument, ty), Some(Expr::EnumVariant { ty: actual, variant })
        if actual == ty && variant == &variants[1])
    {
        return None;
    }
    Some((field.clone(), *index, ty))
}

fn secret_witness(binding: &LocalBinding, witnesses: &[WitnessDeclaration]) -> bool {
    if binding.ty != KEY {
        return false;
    }
    let Expr::WitnessCall { name, arguments } = &binding.value else {
        return false;
    };
    arguments.is_empty()
        && witnesses
            .iter()
            .any(|w| w.name == *name && w.parameters.is_empty() && w.result == KEY)
}

fn key_hash(binding: &LocalBinding, secret: &LocalBinding, pure: &[PureCircuit]) -> bool {
    if binding.ty != KEY || binding.name == secret.name {
        return false;
    }
    let Expr::Call { name, arguments } = &binding.value else {
        return false;
    };
    let [argument] = arguments.as_slice() else {
        return false;
    };
    let Some(helper) = pure.iter().find(|c| c.name == *name) else {
        return false;
    };
    let [input] = helper.parameters.as_slice() else {
        return false;
    };
    if helper.result != KEY || input.ty != KEY || !parameter(argument, &secret.name, &KEY) {
        return false;
    }
    matches!(&helper.body, Expr::PersistentHash { value }
        if matches!(value.as_ref(), Expr::Tuple { elements }
            if matches!(elements.as_slice(), [Expr::BytesLiteral { bytes }, key]
                if bytes.len() == 32 && parameter(key, &input.name, &KEY))))
}

pub(super) fn supported(
    circuit: &StatefulCircuit,
    circuits: &[StatefulCircuit],
    pure: &[PureCircuit],
    fields: &HashMap<&str, &LedgerField>,
    witnesses: &[WitnessDeclaration],
) -> bool {
    checked(circuit, circuits, pure, fields, witnesses).is_some()
}

fn checked(
    circuit: &StatefulCircuit,
    circuits: &[StatefulCircuit],
    pure: &[PureCircuit],
    fields: &HashMap<&str, &LedgerField>,
    witnesses: &[WitnessDeclaration],
) -> Option<()> {
    if !circuit.parameters.is_empty()
        || circuit.result != Type::Unit
        || circuit.return_value != StateReturn::Unit
    {
        return None;
    }
    let [StateAction::Sequence { actions }] = circuit.actions.as_slice() else {
        return None;
    };
    let [guard, with_secret] = actions.as_slice() else {
        return None;
    };
    let (state, state_index, state_ty) = state_guard(guard, circuits, fields)?;
    let (secret, with_key) = binding(with_secret)?;
    let (key, writes) = binding(with_key)?;
    if !secret_witness(secret, witnesses) || !key_hash(key, secret, pure) {
        return None;
    }
    let StateAction::Sequence { actions } = writes else {
        return None;
    };
    let [authority_guard, zero_authority, zero_value, zero_state] = actions.as_slice() else {
        return None;
    };
    let StateAction::Assert {
        condition: Expr::Equal { left, right },
        ..
    } = authority_guard
    else {
        return None;
    };
    let Expr::CellRead {
        field: authority,
        index: authority_index,
    } = right.as_ref()
    else {
        return None;
    };
    if !parameter(left, &key.name, &KEY) || cell(fields, authority, *authority_index)? != &KEY {
        return None;
    }
    let (zero, write_authority) = binding(zero_authority)?;
    if zero.ty != KEY
        || zero.name == key.name
        || zero.name == secret.name
        || !matches!(&zero.value, Expr::Default { ty } if *ty == KEY)
        || !matches!(write_authority, StateAction::CellWrite { field, index, value }
            if field == authority && index == authority_index && parameter(value, &zero.name, &KEY))
    {
        return None;
    }
    let StateAction::CellWrite {
        field,
        index,
        value,
    } = zero_value
    else {
        return None;
    };
    if cell(fields, field, *index)? != &Type::Field
        || !matches!(value, Expr::FieldLiteral { value } if value == "0")
        || field == authority
        || *field == state
        || *authority == state
    {
        return None;
    }
    let Type::Enum { variants, .. } = state_ty else {
        return None;
    };
    matches!(zero_state, StateAction::CellWrite { field, index, value: Expr::EnumVariant { ty, variant } }
        if *field == state && *index == state_index && ty == state_ty && *variant == variants[0])
    .then_some(())
}

#[cfg(test)]
mod tests;
