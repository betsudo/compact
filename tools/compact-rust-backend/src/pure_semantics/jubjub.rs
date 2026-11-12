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

//! Checked curve signatures shared by pure semantics and legacy adapters.
//! Callers finish both source visits before these checks. No clones or new
//! preflights are introduced here; output facts carry no unchecked source Type.
use super::sink::Typed;
use crate::{RenderError, ir::Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Curve {
    Jubjub,
    Secp256k1,
}

impl Curve {
    pub(crate) fn for_point(ty: &Type) -> Self {
        match ty {
            Type::Secp256k1Point => Self::Secp256k1,
            _ => Self::Jubjub,
        }
    }

    pub(crate) fn for_scalar(ty: &Type) -> Self {
        match ty {
            Type::Secp256k1Scalar => Self::Secp256k1,
            _ => Self::Jubjub,
        }
    }

    pub(crate) fn point(self) -> Type {
        match self {
            Self::Jubjub => Type::JubjubPoint,
            Self::Secp256k1 => Type::Secp256k1Point,
        }
    }

    pub(crate) fn scalar(self) -> Type {
        match self {
            Self::Jubjub => Type::Field,
            Self::Secp256k1 => Type::Secp256k1Scalar,
        }
    }
}

pub(crate) enum CurveOperands<V> {
    Reduce(Typed<V>),
    Generator(Typed<V>),
    Multiply(Typed<V>, Typed<V>),
    Add(Typed<V>, Typed<V>),
    Coordinate(Axis, Typed<V>),
    Construct(Typed<V>, Typed<V>),
    Negate(Typed<V>),
}

pub(crate) enum CheckedCurve<V> {
    Reduce(V),
    Generator(Curve, V),
    Multiply(Curve, V, V),
    Add(Curve, V, V),
    Coordinate(Axis, V),
    Construct(V, V),
    Negate(V),
}

fn expect<V>(value: Typed<V>, expected: Type) -> Result<V, RenderError> {
    if value.ty != expected {
        return Err(RenderError::TypeMismatch {
            expected,
            actual: value.ty,
        });
    }
    Ok(value.output)
}

pub(crate) fn check_curve<V>(
    input: CurveOperands<V>,
) -> Result<Typed<CheckedCurve<V>>, RenderError> {
    let (output, ty) = match input {
        CurveOperands::Reduce(value) => (
            CheckedCurve::Reduce(expect(value, Type::Field)?),
            Type::Field,
        ),
        CurveOperands::Generator(value) => {
            let curve = Curve::for_scalar(&value.ty);
            (
                CheckedCurve::Generator(curve, expect(value, curve.scalar())?),
                curve.point(),
            )
        }
        CurveOperands::Multiply(point, scalar) => {
            let curve = Curve::for_point(&point.ty);
            let point = expect(point, curve.point())?;
            let scalar = expect(scalar, curve.scalar())?;
            (CheckedCurve::Multiply(curve, point, scalar), curve.point())
        }
        CurveOperands::Add(left, right) => {
            let curve = Curve::for_point(&left.ty);
            let left = expect(left, curve.point())?;
            let right = expect(right, curve.point())?;
            (CheckedCurve::Add(curve, left, right), curve.point())
        }
        CurveOperands::Coordinate(axis, point) => (
            CheckedCurve::Coordinate(axis, expect(point, Type::JubjubPoint)?),
            Type::Field,
        ),
        CurveOperands::Construct(x, y) => {
            let x = expect(x, Type::Field)?;
            let y = expect(y, Type::Field)?;
            (CheckedCurve::Construct(x, y), Type::JubjubPoint)
        }
        CurveOperands::Negate(point) => (
            CheckedCurve::Negate(expect(point, Type::JubjubPoint)?),
            Type::JubjubPoint,
        ),
    };
    Ok(Typed { output, ty })
}
