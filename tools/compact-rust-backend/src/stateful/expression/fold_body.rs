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

//! Fold-local ReturnPlan traversal. Every branch returns its owned successful
//! context with the accumulator; original actions keep their native leaf owners.
use super::*;
use crate::ir::{ReturnPlan, StateAction};

pub(super) fn render(
    plan: &ReturnPlan,
    parameters: &HashMap<&str, (&Type, syn::Ident)>,
    next: &mut usize,
    environment: &StateExpressionContext<'_, '_>,
) -> Result<(syn::Expr, Type, bool), RenderError> {
    let mut statements = vec![syn::parse_quote!(let mut context = context;)];
    let mut witness = false;
    let (tail, ty) = match plan {
        ReturnPlan::Value { value } => {
            let (value, ty) = expression(
                value,
                parameters,
                &mut statements,
                next,
                environment,
                &mut witness,
            )?;
            let result = bind(value, &mut statements, next);
            (syn::parse_quote!((context, #result)), ty)
        }
        ReturnPlan::Let { bindings, result } => {
            let local = locals(
                bindings,
                parameters,
                &mut statements,
                next,
                environment,
                &mut witness,
            )?;
            let (tail, ty, effect) = render(result, &local, next, environment)?;
            witness |= effect;
            (tail, ty)
        }
        ReturnPlan::Sequence { actions, result } => {
            for source in actions {
                action(
                    source,
                    parameters,
                    &mut statements,
                    next,
                    environment,
                    &mut witness,
                )?;
            }
            let (tail, ty, effect) = render(result, parameters, next, environment)?;
            witness |= effect;
            (tail, ty)
        }
        ReturnPlan::Conditional {
            condition,
            then,
            otherwise,
        } => {
            let (condition_value, condition_ty) = expression(
                condition,
                parameters,
                &mut statements,
                next,
                environment,
                &mut witness,
            )?;
            if condition_ty != Type::Boolean {
                environment.refused(condition);
                return Err(RenderError::TypeMismatch {
                    expected: Type::Boolean,
                    actual: condition_ty,
                });
            }
            let (then_value, ty, then_effect) = render(then, parameters, next, environment)?;
            let (else_value, else_ty, else_effect) =
                render(otherwise, parameters, next, environment)?;
            if ty != else_ty {
                return Err(RenderError::TypeMismatch {
                    expected: ty,
                    actual: else_ty,
                });
            }
            witness |= then_effect || else_effect;
            (conditional(condition_value, then_value, else_value), ty)
        }
    };
    Ok((block(statements, tail), ty, witness))
}
fn expression(
    value: &Expr,
    parameters: &HashMap<&str, (&Type, syn::Ident)>,
    statements: &mut Vec<syn::Stmt>,
    next: &mut usize,
    environment: &StateExpressionContext<'_, '_>,
    witness: &mut bool,
) -> Result<(syn::Expr, Type), RenderError> {
    let mut steps = Vec::new();
    let mut query = false;
    let (value, ty, effect) = render_state_expression_observed(
        value,
        parameters,
        &mut steps,
        next,
        &mut query,
        environment,
    )?;
    if effect || query {
        statements.push(syn::parse_quote!(let mut context = context;));
    }
    statements.extend(steps);
    *witness |= effect;
    Ok((value, ty))
}
fn locals<'a>(
    bindings: &'a [crate::ir::LocalBinding],
    parameters: &HashMap<&'a str, (&'a Type, syn::Ident)>,
    statements: &mut Vec<syn::Stmt>,
    next: &mut usize,
    environment: &StateExpressionContext<'_, '_>,
    witness: &mut bool,
) -> Result<HashMap<&'a str, (&'a Type, syn::Ident)>, RenderError> {
    let mut local = parameters.clone();
    for binding in bindings {
        ident(&binding.name)?;
        let (value, actual) = expression(
            &binding.value,
            &local,
            statements,
            next,
            environment,
            witness,
        )?;
        if actual != binding.ty {
            environment.refused(&binding.value);
            return Err(RenderError::TypeMismatch {
                expected: binding.ty.clone(),
                actual,
            });
        }
        let name = bind(value, statements, next);
        local.insert(binding.name.as_str(), (&binding.ty, name));
    }
    Ok(local)
}
fn action(
    source: &StateAction,
    parameters: &HashMap<&str, (&Type, syn::Ident)>,
    statements: &mut Vec<syn::Stmt>,
    next: &mut usize,
    environment: &StateExpressionContext<'_, '_>,
    witness: &mut bool,
) -> Result<(), RenderError> {
    match source {
        StateAction::Sequence { actions } => {
            for source in actions {
                action(source, parameters, statements, next, environment, witness)?;
            }
        }
        StateAction::Let {
            bindings,
            action: inner,
        } => {
            let local = locals(bindings, parameters, statements, next, environment, witness)?;
            action(inner, &local, statements, next, environment, witness)?;
        }
        StateAction::If {
            condition,
            then,
            otherwise,
        } => {
            let (value, actual) = expression(
                condition,
                parameters,
                statements,
                next,
                environment,
                witness,
            )?;
            if actual != Type::Boolean {
                environment.refused(condition);
                return Err(RenderError::TypeMismatch {
                    expected: Type::Boolean,
                    actual,
                });
            }
            let mut then_steps = vec![syn::parse_quote!(let mut context = context;)];
            let mut else_steps = vec![syn::parse_quote!(let mut context = context;)];
            action(
                then,
                parameters,
                &mut then_steps,
                next,
                environment,
                witness,
            )?;
            action(
                otherwise,
                parameters,
                &mut else_steps,
                next,
                environment,
                witness,
            )?;
            let tail = conditional(
                value,
                block(then_steps, syn::parse_quote!(context)),
                block(else_steps, syn::parse_quote!(context)),
            );
            let mut local: syn::Stmt = syn::parse_quote!(let context = __branch;);
            let syn::Stmt::Local(binding) = &mut local else {
                unreachable!("Fold action context");
            };
            *binding
                .init
                .as_mut()
                .expect("Fold action context initializer")
                .expr = tail;
            statements.push(local);
        }
        _ => crate::stateful::actions::lower_observed(
            source,
            parameters,
            environment,
            crate::stateful::actions::ActionOutput {
                statements,
                next_temp: next,
                uses_witness: witness,
            },
        )?,
    }
    Ok(())
}
fn bind(value: syn::Expr, statements: &mut Vec<syn::Stmt>, next: &mut usize) -> syn::Ident {
    let name = super::fold::temporary(next);
    let mut local: syn::Stmt = syn::parse_quote!(let (#name,) = (__value,););
    let syn::Stmt::Local(binding) = &mut local else {
        unreachable!("Fold result binding");
    };
    let syn::Expr::Tuple(tuple) = binding
        .init
        .as_mut()
        .expect("Fold result initializer")
        .expr
        .as_mut()
    else {
        unreachable!("Fold tuple initializer");
    };
    tuple.elems[0] = value;
    statements.push(local);
    name
}
fn block(mut statements: Vec<syn::Stmt>, tail: syn::Expr) -> syn::Expr {
    let mut value: syn::ExprBlock = syn::parse_quote!({});
    statements.push(syn::Stmt::Expr(tail, None));
    value.block.stmts = statements;
    syn::Expr::Block(value)
}
fn conditional(condition: syn::Expr, then: syn::Expr, otherwise: syn::Expr) -> syn::Expr {
    let mut value: syn::ExprIf = syn::parse_quote!(if __condition {
    } else {
    });
    value.cond = Box::new(condition);
    let syn::Expr::Block(then) = then else {
        unreachable!("Fold then block");
    };
    value.then_branch = then.block;
    *value.else_branch.as_mut().expect("Fold else block").1 = otherwise;
    syn::Expr::If(value)
}
