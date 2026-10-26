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

//! Typed ordered Fold operands and a lexical, effect-capable reducer body.
use super::*;

pub(super) fn render(
    source: &Expr,
    parameters: &HashMap<&str, (&Type, syn::Ident)>,
    statements: &mut Vec<syn::Stmt>,
    next_temp: &mut usize,
    query_effect: &mut bool,
    environment: &StateExpressionContext<'_, '_>,
) -> Result<(syn::Expr, Type, bool), RenderError> {
    let Expr::VectorFold {
        accumulator,
        items,
        initial,
        sources,
        body,
        length,
    } = source
    else {
        unreachable!("Fold dispatcher");
    };
    crate::pure_semantics::aggregate::fold_formals(accumulator, items, sources.len())
        .inspect_err(|_| environment.refused(source))?;
    let (initial, actual, mut witness_effect) = render_state_expression_observed(
        initial,
        parameters,
        statements,
        next_temp,
        query_effect,
        environment,
    )?;
    let initial = coerce_expression(initial, &actual, &accumulator.ty, 0)?;
    let initial = materialize(initial, &accumulator.ty, statements, next_temp);
    let mut inputs = Vec::new();
    for (source, item) in sources.iter().zip(items) {
        let target = Type::Vector {
            element: Box::new(item.ty.clone()),
            length: *length,
        };
        let (value, actual, effect) = render_state_expression_observed(
            source,
            parameters,
            statements,
            next_temp,
            query_effect,
            environment,
        )?;
        let value = coerce_expression(value, &actual, &target, 0)?;
        let value = materialize(value, &target, statements, next_temp);
        let identity = crate::pure_semantics::coercion::plan_coercion(&target, &target)?;
        inputs.push((value, identity));
        witness_effect |= effect;
    }
    let mut locals: HashMap<&str, (&Type, syn::Ident)> = parameters.clone();
    let mut formals = Vec::new();
    for formal in std::iter::once(accumulator).chain(items) {
        let symbol = temporary(next_temp);
        locals.insert(formal.name.as_str(), (&formal.ty, symbol.clone()));
        formals.push(symbol);
    }
    let (body_value, actual, effect) =
        super::fold_body::render(body, &locals, next_temp, environment)?;
    if actual != accumulator.ty {
        return Err(RenderError::TypeMismatch {
            expected: accumulator.ty.clone(),
            actual,
        });
    }
    witness_effect |= effect;
    // The native loop explicitly carries the owned context, including after a
    // lexical action shadows it. This syntax assignment requires a mutable
    // outer context even when the reducer has no public query.
    *query_effect = true;
    let mut pairs = inputs
        .into_iter()
        .map(|(source, _)| source)
        .zip(formals.iter().skip(1));
    let (first, first_item) = pairs.next().expect("checked Fold iterable");
    let mut iterator = crate::pure_native::aggregate::fold_iterator(first);
    let mut pattern: syn::Pat = syn::parse_quote!(#first_item);
    for (source, item) in pairs {
        let mut zip: syn::ExprMethodCall = syn::parse_quote!(__iter.zip(__source));
        zip.receiver = Box::new(iterator);
        *zip.args.first_mut().expect("Fold zip source") =
            crate::pure_native::aggregate::fold_iterator(source);
        iterator = syn::Expr::MethodCall(zip);
        pattern = syn::parse_quote!((#pattern, #item));
    }
    let accumulator_name = &formals[0];
    let loop_context = temporary(next_temp);
    let iterator_name = temporary(next_temp);
    let body_context = temporary(next_temp);
    let body_result = temporary(next_temp);
    let mut value: syn::ExprBlock = syn::parse_quote!({
        let (mut #loop_context, mut #accumulator_name, #iterator_name) = (context, __initial, __iterator);
        for #pattern in #iterator_name {
            let context = #loop_context;
            let (#body_context, #body_result) = __body;
            #loop_context = #body_context;
            #accumulator_name = #body_result;
        }
        context = #loop_context;
        #accumulator_name
    });
    let syn::Stmt::Local(binding) = &mut value.block.stmts[0] else {
        unreachable!("Fold state tuple");
    };
    let syn::Expr::Tuple(tuple) = binding
        .init
        .as_mut()
        .expect("Fold state initializer")
        .expr
        .as_mut()
    else {
        unreachable!("Fold state tuple");
    };
    tuple.elems[1] = initial;
    tuple.elems[2] = iterator;
    let syn::Stmt::Expr(syn::Expr::ForLoop(loop_), _) = &mut value.block.stmts[1] else {
        unreachable!("Fold loop");
    };
    let syn::Stmt::Local(binding) = &mut loop_.body.stmts[1] else {
        unreachable!("Fold body result tuple");
    };
    *binding.init.as_mut().expect("Fold body initializer").expr = body_value;
    Ok((
        syn::Expr::Block(value),
        accumulator.ty.clone(),
        witness_effect,
    ))
}

pub(super) fn temporary(next: &mut usize) -> syn::Ident {
    let symbol = syn::Ident::new(&format!("__compact_fold_value_{next}"), Span::call_site());
    *next += 1;
    symbol
}
fn materialize(
    value: syn::Expr,
    ty: &Type,
    statements: &mut Vec<syn::Stmt>,
    next: &mut usize,
) -> syn::Expr {
    if *ty == Type::Unit {
        statements.extend(discard_expression(value, ty));
        syn::parse_quote!(())
    } else {
        let symbol = temporary(next);
        let syn::Stmt::Local(mut local) = syn::parse_quote!(let #symbol = __value;) else {
            unreachable!("Fold operand local");
        };
        *local.init.as_mut().expect("Fold operand initializer").expr = value;
        statements.push(syn::Stmt::Local(local));
        syn::parse_quote!(#symbol)
    }
}
