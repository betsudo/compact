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

//! Optional proof-data admission for a guarded three-Cell set(Field).
//! The fixed typed shape captures one declared private Bytes32 witness, hashes
//! it through an inspected pure helper, and writes authority/value/state in order.
//! The existing recorded execution owns all effects, hashing and encoding.

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
    let [input] = circuit.parameters.as_slice() else {
        return None;
    };
    if input.ty != Type::Field
        || input.name.is_empty()
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
    let (state, state_index, state_ty) = state_guard(guard, circuits, fields, 0)?;
    let (secret, with_key) = binding(with_secret)?;
    let (key, writes) = binding(with_key)?;
    // Local shadowing could replace the disclosed public parameter. Each
    // binding has its own type/domain and is used exactly once by this shape.
    if input.name == secret.name
        || input.name == key.name
        || !secret_witness(secret, witnesses)
        || !key_hash(key, secret, pure)
    {
        return None;
    }
    let StateAction::Sequence { actions } = writes else {
        return None;
    };
    let [authority_write, value_write, state_write] = actions.as_slice() else {
        return None;
    };
    let StateAction::CellWrite {
        field: authority,
        index: authority_index,
        value,
    } = authority_write
    else {
        return None;
    };
    if cell(fields, authority, *authority_index)? != &KEY || !parameter(value, &key.name, &KEY) {
        return None;
    }
    let StateAction::CellWrite {
        field,
        index,
        value,
    } = value_write
    else {
        return None;
    };
    if cell(fields, field, *index)? != &Type::Field
        || !parameter(value, &input.name, &Type::Field)
        || field == authority
        || *field == state
        || *authority == state
    {
        return None;
    }
    let Type::Enum { variants, .. } = state_ty else {
        return None;
    };
    matches!(state_write, StateAction::CellWrite { field, index, value: Expr::EnumVariant { ty, variant } }
        if *field == state && *index == state_index && ty == state_ty && *variant == variants[1]).then_some(())
}

#[cfg(test)]
mod tests;
