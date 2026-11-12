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

//! Pure conversion rules, independent of output and child traversal.
//! This family is complete; the exhaustive syntax-free body fold is pending.
use super::{
    children::Children,
    coercion::{CoercionPlan, plan_coercion},
    numeric::{UnsignedCastStrategy, UnsignedMaximum, cast_strategy_from_bounds, unsigned_maximum},
    sink::{ConversionSink, Typed},
};
use crate::{
    RenderError,
    ir::{Expr, Type},
};

pub(crate) enum ConversionSource<'a> {
    FieldCast { value: &'a Expr },
    FieldToBytes32 { value: &'a Expr },
    Secp256k1BytesCast { value: &'a Expr, ty: &'a Type },
    Coerce { value: &'a Expr, ty: &'a Type },
    UnsignedCast { max: &'a str, value: &'a Expr },
}

pub(crate) enum ConversionFacts {
    UintToField { wide: bool },
    FieldToBytes32,
    Secp256k1BytesCast(Secp256k1BytesCast),
    Coercion(CoercionPlan),
    ExplicitUnsignedCast(UnsignedCastStrategy),
}

pub(crate) enum Secp256k1BytesCast {
    ToBase,
    ToScalar,
    ToBytes,
}

impl Secp256k1BytesCast {
    pub(crate) fn check(source: &Type, target: &Type) -> Result<Self, RenderError> {
        match (source, target) {
            (Type::Bytes { length: 32 }, Type::Secp256k1Base) => Ok(Self::ToBase),
            (Type::Bytes { length: 32 }, Type::Secp256k1Scalar) => Ok(Self::ToScalar),
            (Type::Secp256k1Base | Type::Secp256k1Scalar, Type::Bytes { length: 32 }) => {
                Ok(Self::ToBytes)
            }
            _ => Err(RenderError::InvalidSecp256k1BytesCast {
                source: source.clone(),
                target: target.clone(),
            }),
        }
    }
}

pub(crate) fn fold_conversion<S: ConversionSink, C: Children<S>>(
    source: ConversionSource<'_>,
    children: &mut C,
    sink: &mut S,
) -> Result<Typed<S::Value>, RenderError> {
    // One common output projection after the shared semantic match. Do not
    // allocate per-arm native-result temporaries on recursive debug paths.
    let (facts, value, ty) = match source {
        ConversionSource::FieldCast { value } => {
            let child = children.visit(value, sink)?;
            let Type::Unsigned { max } = &child.ty else {
                return Err(RenderError::ExpectedUnsigned(child.ty));
            };
            let wide = matches!(unsigned_maximum(max)?, UnsignedMaximum::Wide { .. });
            (
                ConversionFacts::UintToField { wide },
                child.output,
                Type::Field,
            )
        }
        ConversionSource::FieldToBytes32 { value } => {
            let child = children.visit(value, sink)?;
            if child.ty != Type::Field {
                return Err(RenderError::TypeMismatch {
                    expected: Type::Field,
                    actual: child.ty,
                });
            }
            (
                ConversionFacts::FieldToBytes32,
                child.output,
                Type::Bytes { length: 32 },
            )
        }
        ConversionSource::Secp256k1BytesCast { value, ty } => {
            let child = children.visit(value, sink)?;
            let cast = Secp256k1BytesCast::check(&child.ty, ty)?;
            (
                ConversionFacts::Secp256k1BytesCast(cast),
                child.output,
                ty.clone(),
            )
        }
        ConversionSource::Coerce { value, ty } => {
            let child = children.visit(value, sink)?;
            let plan = plan_coercion(&child.ty, ty)?;
            (ConversionFacts::Coercion(plan), child.output, ty.clone())
        }
        ConversionSource::UnsignedCast { max, value } => {
            // Target validity precedes the child. The immutable parsed fact is
            // reused after source validation, without a second target parse.
            let target = unsigned_maximum(max)?;
            let child = children.visit(value, sink)?;
            let Type::Unsigned { max: source_max } = child.ty else {
                return Err(RenderError::TypeMismatch {
                    expected: Type::Unsigned {
                        max: max.to_owned(),
                    },
                    actual: child.ty,
                });
            };
            let source = unsigned_maximum(&source_max)?;
            let cast = cast_strategy_from_bounds(source, target);
            (
                ConversionFacts::ExplicitUnsignedCast(cast),
                child.output,
                Type::Unsigned {
                    max: max.to_owned(),
                },
            )
        }
    };
    Ok(Typed {
        output: sink.conversion(facts, value),
        ty,
    })
}

#[cfg(test)]
mod tests;
