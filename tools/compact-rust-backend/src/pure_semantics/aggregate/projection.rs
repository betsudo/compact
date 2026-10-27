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

//! Projection validation and ownership facts, distinct from operation tuple caps.
use super::{CheckedAggregate, Output};
use crate::{
    RenderError,
    ir::{Expr, Type},
    pure_semantics::{
        children::Children,
        identifier::check_name,
        ownership::{Ownership, copy_type},
        sink::{AggregateSink, Typed},
    },
};

pub(super) fn field<S: AggregateSink>(
    source: &Expr,
    field: &str,
    index: usize,
    children: &mut impl Children<S>,
    sink: &mut S,
) -> Result<Typed<Output<S>>, RenderError> {
    let value = children.visit(source, sink)?;
    let Type::Struct { fields, .. } = value.ty else {
        return Err(RenderError::InvalidStructField(field.to_owned()));
    };
    let declaration = fields
        .get(index)
        .filter(|declaration| declaration.name == field)
        .ok_or_else(|| RenderError::InvalidStructField(field.to_owned()))?;
    let field = check_name(field)?;
    let ownership = if copy_type(&declaration.ty) {
        Ownership::Copy
    } else {
        Ownership::Owned
    };
    Ok(Typed {
        output: CheckedAggregate::StructField {
            value: value.output,
            field,
            ownership,
        },
        ty: declaration.ty.clone(),
    })
}
pub(super) fn tuple<S: AggregateSink>(
    source: &Expr,
    index: usize,
    children: &mut impl Children<S>,
    sink: &mut S,
) -> Result<Typed<Output<S>>, RenderError> {
    let value = children.visit(source, sink)?;
    let (facts, ty) = check_index(value.ty, index)?;
    let output = match facts {
        IndexFacts::Tuple => CheckedAggregate::TupleIndex {
            value: value.output,
            index,
        },
        IndexFacts::Vector(ownership) => CheckedAggregate::VectorIndex {
            value: value.output,
            index,
            ownership,
        },
    };
    Ok(Typed { output, ty })
}

/// The frontend's private `tuple_index` transport is type-directed. Tuple
/// arity and vector bounds are checked before any syntax or selected value.
pub(crate) enum IndexFacts {
    Tuple,
    Vector(Ownership),
}

pub(crate) fn check_index(ty: Type, index: usize) -> Result<(IndexFacts, Type), RenderError> {
    match ty {
        Type::Tuple { elements } => {
            let ty = elements
                .get(index)
                .ok_or(RenderError::InvalidTupleIndex(index))?
                .clone();
            Ok((IndexFacts::Tuple, ty))
        }
        Type::Vector { element, length } => {
            if index >= length {
                return Err(RenderError::InvalidVectorIndex { index, length });
            }
            let ownership = if copy_type(&element) {
                Ownership::Copy
            } else {
                Ownership::Owned
            };
            Ok((IndexFacts::Vector(ownership), *element))
        }
        ty => Err(RenderError::ExpectedTuple(ty)),
    }
}
