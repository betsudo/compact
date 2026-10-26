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

//! Map lexical overlays and Fold signature/coercion sequencing.
//! Static Fold checks initial before source; its output keeps source-first runtime
//! initialization. Explicit borrowed annotations retain their existing bounds.
use super::{CheckedAggregate, Output};
use crate::{
    RenderError,
    ir::{Expr, Parameter, Type},
    pure_semantics::{
        children::ScopedChildren,
        coercion::plan_coercion,
        declarations::PureDeclarations,
        identifier::check_name,
        sink::{AggregateSink, Typed},
    },
};

pub(super) struct MapInput<'a> {
    pub(super) parameter: &'a Parameter,
    pub(super) source: &'a Expr,
    pub(super) body: &'a Expr,
    pub(super) result: &'a Type,
    pub(super) length: usize,
}
pub(super) struct FoldInput<'a> {
    pub(super) name: &'a str,
    pub(super) initial: &'a Expr,
    pub(super) source: &'a Expr,
    pub(super) accumulator: &'a Type,
    pub(super) element: &'a Type,
    pub(super) length: usize,
}

pub(super) fn map<S: AggregateSink>(
    input: MapInput<'_>,
    children: &mut impl ScopedChildren<S>,
    sink: &mut S,
) -> Result<Typed<Output<S>>, RenderError> {
    let MapInput {
        parameter,
        source,
        body,
        result,
        length,
    } = input;
    let source = children.visit(source, sink)?;
    let target = Type::Vector {
        element: Box::new(parameter.ty.clone()),
        length,
    };
    let source_plan = plan_coercion(&source.ty, &target)?;
    let name = check_name(&parameter.name)?;
    let binding = sink.binding(&name);
    let body = children.visit_bound(body, &parameter.name, &parameter.ty, &binding, sink)?;
    if body.ty != *result {
        return Err(RenderError::TypeMismatch {
            expected: result.clone(),
            actual: body.ty,
        });
    }
    Ok(Typed {
        output: CheckedAggregate::Map {
            source: source.output,
            source_plan,
            binding,
            body: body.output,
            length,
        },
        ty: Type::Vector {
            element: Box::new(result.clone()),
            length,
        },
    })
}

pub(super) fn fold<S: AggregateSink>(
    input: FoldInput<'_>,
    children: &mut impl ScopedChildren<S>,
    declarations: &impl PureDeclarations,
    sink: &mut S,
) -> Result<Typed<Output<S>>, RenderError> {
    let FoldInput {
        name,
        initial,
        source,
        accumulator,
        element,
        length,
    } = input;
    let circuit = declarations
        .signature(name)
        .ok_or_else(|| RenderError::UnknownCircuit(name.to_owned()))?;
    if circuit.parameters.len() != 2
        || circuit.parameters[0].ty != *accumulator
        || circuit.parameters[1].ty != *element
        || circuit.result != accumulator
    {
        return Err(RenderError::TypeMismatch {
            expected: accumulator.clone(),
            actual: circuit.result.clone(),
        });
    }
    let initial = children.visit(initial, sink)?;
    let initial_plan = plan_coercion(&initial.ty, accumulator)?;
    let source = children.visit(source, sink)?;
    let target = Type::Vector {
        element: Box::new(element.clone()),
        length,
    };
    let source_plan = plan_coercion(&source.ty, &target)?;
    let function = check_name(name)?;
    Ok(Typed {
        output: CheckedAggregate::Fold {
            function,
            initial: initial.output,
            initial_plan,
            source: source.output,
            source_plan,
        },
        ty: accumulator.clone(),
    })
}

/// All formals are distinct under the actual Rust spelling, before body scope overlay.
pub(crate) fn fold_formals(
    accumulator: &Parameter,
    items: &[Parameter],
    sources: usize,
) -> Result<Vec<crate::pure_semantics::identifier::CheckedName>, RenderError> {
    if sources == 0 || sources != items.len() {
        return Err(RenderError::MalformedVectorFold(
            "one item formal is required per iterable",
        ));
    }
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for formal in std::iter::once(accumulator).chain(items) {
        let name = check_name(&formal.name)?;
        if !seen.insert(name.text().to_owned()) {
            return Err(RenderError::MalformedVectorFold(
                "duplicate accumulator/item formal",
            ));
        }
        names.push(name);
    }
    Ok(names)
}
pub(super) struct ReductionInput<'a> {
    pub(super) accumulator: &'a Parameter,
    pub(super) items: &'a [Parameter],
    pub(super) initial: &'a Expr,
    pub(super) sources: &'a [Expr],
    pub(super) body: &'a Expr,
    pub(super) length: usize,
}
pub(super) fn reduction<S: AggregateSink>(
    input: ReductionInput<'_>,
    children: &mut impl ScopedChildren<S>,
    sink: &mut S,
) -> Result<Typed<Output<S>>, RenderError> {
    let ReductionInput {
        accumulator,
        items,
        initial,
        sources,
        body,
        length,
    } = input;
    let names = fold_formals(accumulator, items, sources.len())?;
    let initial = children.visit(initial, sink)?;
    let initial_plan = plan_coercion(&initial.ty, &accumulator.ty)?;
    let mut checked_sources = Vec::new();
    for (source, item) in sources.iter().zip(items) {
        let source = children.visit(source, sink)?;
        let plan = plan_coercion(
            &source.ty,
            &Type::Vector {
                element: Box::new(item.ty.clone()),
                length,
            },
        )?;
        checked_sources.push((source.output, plan));
    }
    let mut bindings = names
        .iter()
        .map(|name| sink.binding(name))
        .collect::<Vec<_>>();
    let bound = std::iter::once(accumulator)
        .chain(items)
        .zip(&bindings)
        .map(|(formal, binding)| (formal.name.as_str(), &formal.ty, binding))
        .collect::<Vec<_>>();
    let body = children.visit_bound_many(body, &bound, sink)?;
    if body.ty != accumulator.ty {
        return Err(RenderError::TypeMismatch {
            expected: accumulator.ty.clone(),
            actual: body.ty,
        });
    }
    Ok(Typed {
        output: CheckedAggregate::Reduction {
            initial: initial.output,
            initial_plan,
            sources: checked_sources,
            accumulator: bindings.remove(0),
            items: bindings,
            body: body.output,
        },
        ty: accumulator.ty.clone(),
    })
}
