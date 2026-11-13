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
    FieldCast {
        value: &'a Expr,
    },
    FieldToBytes32 {
        value: &'a Expr,
    },
    JubjubCast {
        value: &'a Expr,
        ty: &'a Type,
    },
    Secp256k1BytesCast {
        value: &'a Expr,
        ty: &'a Type,
    },
    FixedBytes {
        value: &'a Expr,
        operation: FixedBytesConversion,
    },
    Coerce {
        value: &'a Expr,
        ty: &'a Type,
    },
    UnsignedCast {
        max: &'a str,
        value: &'a Expr,
    },
}

pub(crate) enum ConversionFacts {
    UintToField { wide: bool },
    FieldToBytes32,
    JubjubCast(JubjubCast),
    Secp256k1BytesCast(Secp256k1BytesCast),
    FixedBytes(FixedBytesConversion),
    Coercion(CoercionPlan),
    ExplicitUnsignedCast(UnsignedCastStrategy),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixedBytesConversion {
    ToVector { length: usize },
    FromVector { length: usize },
    Slice { start: usize, length: usize },
}

impl FixedBytesConversion {
    pub(crate) fn from_expression(expr: &Expr) -> Option<(&Expr, Self)> {
        match expr {
            Expr::VectorToBytes { value, length } => {
                Some((value, Self::FromVector { length: *length }))
            }
            Expr::BytesToVector { value, length } => {
                Some((value, Self::ToVector { length: *length }))
            }
            Expr::BytesSlice {
                value,
                start,
                length,
            } => Some((
                value,
                Self::Slice {
                    start: *start,
                    length: *length,
                },
            )),
            _ => None,
        }
    }

    fn byte_vector(length: usize) -> Type {
        Type::Vector {
            length,
            element: Box::new(Type::Unsigned { max: "255".into() }),
        }
    }

    pub(crate) fn result_type(self, source: &Type) -> Result<Type, RenderError> {
        match self {
            Self::ToVector { length } => {
                let expected = Type::Bytes { length };
                if *source != expected {
                    return Err(RenderError::TypeMismatch {
                        expected,
                        actual: source.clone(),
                    });
                }
                Ok(Self::byte_vector(length))
            }
            Self::FromVector { length } => {
                let expected = Self::byte_vector(length);
                if *source != expected {
                    return Err(RenderError::TypeMismatch {
                        expected,
                        actual: source.clone(),
                    });
                }
                Ok(Type::Bytes { length })
            }
            Self::Slice { start, length } => {
                let Type::Bytes {
                    length: input_length,
                } = *source
                else {
                    return Err(RenderError::ExpectedBytes(source.clone()));
                };
                if length > input_length || start > input_length - length {
                    return Err(RenderError::InvalidBytesSlice {
                        start,
                        length,
                        input_length,
                    });
                }
                Ok(Type::Bytes { length })
            }
        }
    }
}

pub(crate) enum JubjubCast {
    ToScalar,
    ToField,
    ToUnsigned(UnsignedMaximum),
}

impl JubjubCast {
    pub(crate) fn check(source: &Type, target: &Type) -> Result<Self, RenderError> {
        match (source, target) {
            (Type::Field, Type::JubjubScalar) => Ok(Self::ToScalar),
            (Type::JubjubScalar, Type::Field) => Ok(Self::ToField),
            (Type::JubjubScalar, Type::Unsigned { max }) => {
                Ok(Self::ToUnsigned(unsigned_maximum(max)?))
            }
            _ => Err(RenderError::TypeMismatch {
                expected: if *target == Type::JubjubScalar {
                    Type::Field
                } else {
                    Type::JubjubScalar
                },
                actual: source.clone(),
            }),
        }
    }
}

pub(crate) enum Secp256k1BytesCast {
    ToBase,
    ToScalar,
    ToSecp256r1Base,
    ToSecp256r1Scalar,
    ToCurve25519Base,
    ToCurve25519Scalar,
    WideToCurve25519Scalar,
    ToBytes,
}

impl Secp256k1BytesCast {
    pub(crate) fn check(source: &Type, target: &Type) -> Result<Self, RenderError> {
        match (source, target) {
            (Type::Bytes { length: 32 }, Type::Secp256k1Base) => Ok(Self::ToBase),
            (Type::Bytes { length: 32 }, Type::Secp256k1Scalar) => Ok(Self::ToScalar),
            (Type::Bytes { length: 32 }, Type::Secp256r1Base) => Ok(Self::ToSecp256r1Base),
            (Type::Bytes { length: 32 }, Type::Secp256r1Scalar) => Ok(Self::ToSecp256r1Scalar),
            (Type::Bytes { length: 32 }, Type::Curve25519Base) => Ok(Self::ToCurve25519Base),
            (Type::Bytes { length: 32 }, Type::Curve25519Scalar) => Ok(Self::ToCurve25519Scalar),
            (Type::Bytes { length: 64 }, Type::Curve25519Scalar) => Ok(Self::WideToCurve25519Scalar),
            (
                Type::Secp256k1Base
                | Type::Secp256k1Scalar
                | Type::Secp256r1Base
                | Type::Secp256r1Scalar
                | Type::Curve25519Base
                | Type::Curve25519Scalar,
                Type::Bytes { length: 32 },
            ) => Ok(Self::ToBytes),
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
        ConversionSource::JubjubCast { value, ty } => {
            let child = children.visit(value, sink)?;
            let cast = JubjubCast::check(&child.ty, ty)?;
            (ConversionFacts::JubjubCast(cast), child.output, ty.clone())
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
        ConversionSource::FixedBytes { value, operation } => {
            let child = children.visit(value, sink)?;
            let ty = operation.result_type(&child.ty)?;
            (ConversionFacts::FixedBytes(operation), child.output, ty)
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
