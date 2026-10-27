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

//! Shared checked crypto syntax; admission and proof capability remain separate.
use super::*;

pub(in super::super) fn expression(
    binding: &Binding,
    value: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<syn::Expr>, RecordingGap> {
    let path = &binding.path;
    let operand = |input: &Value| (value(input), input.ty.clone());
    let expression = match &binding.operation {
        Operation::PersistentCommit {
            value: input,
            opening,
        } => {
            let (input, opening) = (value(input), value(opening));
            syn::parse_quote!(runtime::persistent_commit(#input, #opening))
        }
        Operation::FieldToBytes32(input) => crate::field_to_bytes_32_syntax(value(input)),
        Operation::PersistentHash(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::persistent_hash(#input))
        }
        Operation::HashToCurve(input) => {
            let input = crate::hash_input::syntax(value(input), &input.ty);
            syn::parse_quote!(runtime::hash_to_curve(#input))
        }
        Operation::Keccak256(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::keccak256(#input))
        }
        Operation::JubjubPointX(input) | Operation::JubjubPointY(input) => {
            let coordinate = if matches!(binding.operation, Operation::JubjubPointX(_)) {
                crate::jubjub_value::Coordinate::X
            } else {
                crate::jubjub_value::Coordinate::Y
            };
            coordinate
                .lower(operand(input))
                .map(|(syntax, _)| syntax)
                .map_err(|e| invalid(path, e.to_string()))?
        }
        Operation::JubjubScalarFromNative(input) => {
            jubjub(crate::jubjub_value::Operation::Reduce(operand(input)), path)?
        }
        Operation::EcMulGenerator(input) => jubjub(
            crate::jubjub_value::Operation::Generator(operand(input)),
            path,
        )?,
        Operation::EcNeg(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::ec_neg(#input))
        }
        Operation::ConstructJubjubPoint { x, y } => {
            let (x, y) = (value(x), value(y));
            syn::parse_quote!(runtime::construct_jubjub_point(#x, #y)?)
        }
        Operation::EcMul { point, scalar } => jubjub(
            crate::jubjub_value::Operation::Multiply(operand(point), operand(scalar)),
            path,
        )?,
        Operation::EcAdd { left, right } => jubjub(
            crate::jubjub_value::Operation::Add(operand(left), operand(right)),
            path,
        )?,
        Operation::DegradeToTransient(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::degrade_to_transient(#input))
        }
        Operation::UpgradeFromTransient(input) => {
            let input = value(input);
            syn::parse_quote!(runtime::upgrade_from_transient(#input))
        }
        Operation::TransientHash(input) => {
            let input = crate::hash_input::syntax(value(input), &input.ty);
            syn::parse_quote!(runtime::transient_hash(#input))
        }
        Operation::TransientCommit {
            value: input,
            opening,
        } => {
            let (input, opening) = (
                crate::hash_input::syntax(value(input), &input.ty),
                value(opening),
            );
            syn::parse_quote!(runtime::transient_commit(#input, #opening))
        }
        _ => return Ok(None),
    };
    Ok(Some(expression))
}
