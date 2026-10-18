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

//! Fold admitted context-free blocks without frame or executor capture.
use super::{
    RecordingGap,
    declarations::Declarations,
    functional_paths::rust_type,
    functional_render::{name, pure, value},
    invalid,
    model::{Binding, Block, Operation},
};
use crate::ir::Type;

/// Produce an expression in the surrounding Result-returning pure producer.
/// The checked source is retained once before try_map consumes it. Captures
/// remain lexical borrows and are retained per use inside each iteration.
pub(super) fn vector_map(
    binding: &Binding,
    declarations: &Declarations,
) -> Result<syn::Expr, RecordingGap> {
    let Operation::VectorMap {
        source,
        parameter,
        body,
    } = &binding.operation
    else {
        return Err(invalid(&binding.path, "expected checked VectorMap"));
    };
    let input = value(source);
    let formal = name(parameter);
    let input_type = rust_type(&parameter.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
    let output_type =
        rust_type(&body.result.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
    let body = block_value(body, declarations)?;
    // No move closure and no frame/executor argument. FixedVector owns order,
    // zero-length behavior, short-circuiting and temporary-element cleanup.
    Ok(syn::parse_quote!(
        (#input).try_map(|#formal: #input_type|
            -> Result<#output_type, crate::runtime::CompactError> {
                Ok(#body)
            })?
    ))
}

fn block_value(block: &Block, declarations: &Declarations) -> Result<syn::Expr, RecordingGap> {
    let mut statements = Vec::<syn::Stmt>::new();
    for binding in &block.bindings {
        let expression = expression(binding, declarations)?;
        // Exact Unit has no name uses, but a Unit producer may still fail.
        // Empty tuple and empty named products are NOT elided as exact Unit.
        if binding.value.ty == Type::Unit {
            statements.push(syn::parse_quote!(#expression;));
        } else {
            let target = name(&binding.value);
            let ty =
                rust_type(&binding.value.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
            statements.push(syn::parse_quote!(let #target: #ty = #expression;));
        }
    }
    let result = value(&block.result);
    Ok(syn::parse_quote!({ #(#statements)* #result }))
}

fn expression(binding: &Binding, declarations: &Declarations) -> Result<syn::Expr, RecordingGap> {
    match &binding.operation {
        Operation::VectorMap { .. } => vector_map(binding, declarations),
        Operation::If {
            condition,
            then,
            otherwise,
        } => {
            let condition = value(condition);
            let then = block_value(then, declarations)?;
            let otherwise = block_value(otherwise, declarations)?;
            Ok(syn::parse_quote!(if #condition { #then } else { #otherwise }))
        }
        Operation::PureCall { .. } => {
            super::render::pure::certified_call(binding, declarations, value)
                .map(super::functional_paths::expression)
        }
        // Same checked assertion/error syntax as ordinary recorded emission.
        Operation::Assert { condition, message } => {
            let condition = value(condition);
            Ok(syn::parse_quote!({
                if !#condition {
                    return Err(crate::runtime::CompactError::AssertionFailed(#message.to_owned()));
                }
            }))
        }
        operation => pure(binding, operation),
    }
}
