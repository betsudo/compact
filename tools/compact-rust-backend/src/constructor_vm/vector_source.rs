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

//! Checked constructor Vector operands, prepared once before borrowed iteration.
use crate::{
    RenderError,
    ir::{Expr, Parameter, Type},
    native_header,
    source_diagnostics::{Owner, RefusalSite},
    stateful,
};
use proc_macro2::Span;
use std::collections::HashMap;

pub(crate) fn vector_source<'a>(
    source: &'a Expr,
    binding: &'a Parameter,
    length: usize,
    parameters: &HashMap<&'a str, (&'a Type, syn::Ident)>,
    environment: &stateful::NativeExpressionContext<'_, '_>,
    next_temp: &mut usize,
    loop_index: usize,
) -> Result<(syn::Ident, Vec<syn::Stmt>), RenderError> {
    let mut actions = Vec::new();
    let expected = Type::Vector {
        element: Box::new(binding.ty.clone()),
        length,
    };
    let iterable = if let Expr::Parameter { name } = source {
        // Preserve the existing borrowed parameter/local route and
        // its emitted source; do not clone a whole input Vector.
        let (actual, iterable) = parameters.get(name.as_str()).ok_or_else(|| {
            super::refused(RefusalSite::node(Owner::Expr(source)), environment.observer);
            RenderError::UnknownParameter(name.clone())
        })?;
        if **actual != expected {
            super::refused(RefusalSite::node(Owner::Expr(source)), environment.observer);
            return Err(RenderError::TypeMismatch {
                expected,
                actual: (*actual).clone(),
            });
        }
        iterable.clone()
    } else {
        // Literal/constructor forms with a known non-Vector kind
        // retain the original outer-kind refusal before parsing
        // malformed payloads or checking their children.
        if matches!(
            source,
            Expr::Unit
                | Expr::Boolean { .. }
                | Expr::FieldLiteral { .. }
                | Expr::BytesLiteral { .. }
                | Expr::UnsignedLiteral { .. }
                | Expr::EnumVariant { .. }
                | Expr::Tuple { .. }
                | Expr::StructLiteral { .. }
        ) {
            super::refused(RefusalSite::node(Owner::Expr(source)), environment.observer);
            return Err(RenderError::InvalidConstructorInitializer(
                binding.name.clone(),
            ));
        }
        let mut expression_steps = Vec::new();
        let mut query_effect = false;
        let (value, actual, _) = stateful::render_state_expression_observed(
            source,
            parameters,
            &mut expression_steps,
            next_temp,
            &mut query_effect,
            environment,
        )?;
        if !matches!(actual, Type::Vector { .. }) {
            super::refused(RefusalSite::node(Owner::Expr(source)), environment.observer);
            return Err(RenderError::InvalidConstructorInitializer(
                binding.name.clone(),
            ));
        }
        if actual != expected {
            super::refused(RefusalSite::node(Owner::Expr(source)), environment.observer);
            return Err(RenderError::TypeMismatch { expected, actual });
        }
        let iterable = syn::Ident::new(
            &format!("__compact_constructor_iterable_{loop_index}"),
            Span::call_site(),
        );
        let ty = native_header::carrier(&expected, environment.observer)?;
        // The source and any fallible context steps run once before
        // iteration, including when the checked length is zero.
        actions.extend(expression_steps);
        actions.push(syn::parse_quote!(let #iterable: #ty = #value;));
        iterable
    };
    Ok((iterable, actions))
}
