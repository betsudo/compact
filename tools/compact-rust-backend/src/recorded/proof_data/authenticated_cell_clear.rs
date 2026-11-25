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

use super::authenticated_cell::{
    KEY, binding, cell, key_hash, parameter, secret_witness, state_guard,
};
use crate::ir::{
    Expr, LedgerField, PureCircuit, StateAction, StateReturn, StatefulCircuit, Type,
    WitnessDeclaration,
};
use std::collections::HashMap;

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
    let (state, state_index, state_ty) = state_guard(guard, circuits, fields, 1)?;
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
