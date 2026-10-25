// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared checked value syntax. Frame ownership and result bindings stay in callers.
use super::*;
use crate::ir::Type;

/// Keep the finite dispatch list in leaf.rs aligned with these supported arms.
/// Operands may retain non-Copy values; projections instead need their raw
/// addressable place so only the selected result is retained, not its aggregate.
pub(in super::super) fn expression(
    binding: &Binding,
    ty: &syn::Type,
    value: impl Fn(&Value) -> syn::Expr,
    place: impl Fn(&Value) -> syn::Expr,
) -> Result<Option<syn::Expr>, RecordingGap> {
    let path = &binding.path;
    let expression = match &binding.operation {
        Operation::StructConstruct { fields: inputs } => {
            let crate::ir::Type::Struct { fields, .. } = &binding.value.ty else {
                unreachable!("checked struct construction")
            };
            let members = fields
                .iter()
                .map(|f| ident(&f.name))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| invalid(path, e.to_string()))?;
            let inputs: Vec<_> = inputs.iter().map(value).collect();
            syn::parse_quote!(#ty { #(#members: #inputs),* })
        }
        Operation::StructProject {
            value: input,
            field,
            ..
        } => {
            let member = ident(field).map_err(|e| invalid(path, e.to_string()))?;
            let source = place(input);
            crate::retained_value(syn::parse_quote!(#source.#member), &binding.value.ty)
        }
        Operation::TupleProject {
            value: input,
            index,
        } => {
            let source = place(input);
            let member = syn::Index::from(*index);
            crate::retained_value(syn::parse_quote!(#source.#member), &binding.value.ty)
        }
        Operation::VectorProject {
            value: input,
            index,
        } => {
            let source = place(input);
            let index = syn::Index::from(*index);
            crate::retained_value(syn::parse_quote!((#source).0[#index]), &binding.value.ty)
        }
        Operation::Vector { elements } => {
            let Type::Vector { element, .. } = &binding.value.ty else {
                unreachable!("checked vector output");
            };
            if elements.is_empty() {
                let element =
                    crate::rust_type(element).map_err(|error| invalid(path, error.to_string()))?;
                syn::parse_quote!(runtime::FixedVector::<#element, 0>::new([]))
            } else {
                let elements: Vec<_> = elements.iter().map(value).collect();
                syn::parse_quote!(runtime::FixedVector::new([#(#elements),*]))
            }
        }
        Operation::Tuple { elements } => {
            let elements: Vec<_> = elements.iter().map(value).collect();
            syn::parse_quote!((#(#elements,)*))
        }
        Operation::Add(a, b) | Operation::Subtract(a, b) | Operation::Multiply(a, b) => {
            let (a, b) = (value(a), value(b));
            if matches!(binding.operation, Operation::Multiply(..)) {
                syn::parse_quote!(#a * #b)
            } else if matches!(binding.operation, Operation::Subtract(..)) {
                syn::parse_quote!(#a - #b)
            } else {
                syn::parse_quote!(#a + #b)
            }
        }
        Operation::Compare {
            operator,
            left,
            right,
        } => {
            use crate::ir::ComparisonOperator;
            let (left, right) = (value(left), value(right));
            match operator {
                ComparisonOperator::Less => syn::parse_quote!(#left.value() < #right.value()),
                ComparisonOperator::LessEqual => syn::parse_quote!(#left.value() <= #right.value()),
                ComparisonOperator::Greater => syn::parse_quote!(#left.value() > #right.value()),
                ComparisonOperator::GreaterEqual => {
                    syn::parse_quote!(#left.value() >= #right.value())
                }
            }
        }
        Operation::Equal(a, b) | Operation::NotEqual(a, b) => {
            let (a, b) = (value(a), value(b));
            if matches!(binding.operation, Operation::NotEqual(..)) {
                syn::parse_quote!(#a != #b)
            } else {
                syn::parse_quote!(#a == #b)
            }
        }
        Operation::UnsignedAdd(a, b)
        | Operation::UnsignedSubtract(a, b)
        | Operation::UnsignedMultiply(a, b) => {
            // The native syntax helper only uses the variant as an operation tag;
            // operands have already been evaluated once in SSA source order.
            let max = unsigned_max(&binding.value.ty).into();
            let left = Box::new(crate::ir::Expr::Unit);
            let right = Box::new(crate::ir::Expr::Unit);
            let op = match &binding.operation {
                Operation::UnsignedAdd(..) => crate::ir::Expr::UnsignedAdd { max, left, right },
                Operation::UnsignedSubtract(..) => {
                    crate::ir::Expr::UnsignedSubtract { max, left, right }
                }
                Operation::UnsignedMultiply(..) => {
                    crate::ir::Expr::UnsignedMultiply { max, left, right }
                }
                _ => unreachable!("unsigned arithmetic arm"),
            };
            crate::unsigned_arithmetic_syntax(
                &op,
                value(a),
                value(b),
                unsigned_max(&a.ty),
                unsigned_max(&b.ty),
                unsigned_max(&binding.value.ty),
            )
            .map_err(|e| invalid(path, e.to_string()))?
        }
        Operation::Convert { value: input, kind } => match kind {
            Conversion::Coerce => {
                crate::coerce_expression(value(input), &input.ty, &binding.value.ty, 0)
            }
            Conversion::UnsignedCast => crate::unsigned_cast_syntax(
                value(input),
                unsigned_max(&input.ty),
                unsigned_max(&binding.value.ty),
            ),
        }
        .map_err(|e| invalid(path, e.to_string()))?,
        _ => return Ok(None),
    };
    Ok(Some(expression))
}

/// Invoke only the exact source declaration certified by the checked environment.
/// Arguments are already evaluated SSA operands; this helper adds no admission.
pub(in super::super) fn certified_call(
    binding: &Binding,
    declarations: &Declarations,
    operand: impl Fn(&Value) -> syn::Expr,
) -> Result<syn::Expr, RecordingGap> {
    let Operation::PureCall {
        declaration,
        arguments,
    } = &binding.operation
    else {
        return Err(invalid(&binding.path, "expected checked pure call"));
    };
    let certified = declarations
        .certified_pure(*declaration)
        .ok_or_else(|| invalid(&binding.path, "missing checked pure declaration"))?;
    let method = ident(&certified.source().name)
        .map_err(|error| invalid(&binding.path, error.to_string()))?;
    let arguments: Vec<_> = arguments.iter().map(operand).collect();
    Ok(syn::parse_quote!(crate::pure_circuits::#method(#(#arguments),*)?))
}

#[cfg(test)]
mod tests;
