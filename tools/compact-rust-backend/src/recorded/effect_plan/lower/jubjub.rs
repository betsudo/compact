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

//! Point operands are evaluated once, left to right, before creating an operation.
use super::*;
use crate::pure_semantics::jubjub::Curve;

impl Lowerer<'_> {
    pub(super) fn jubjub(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let mut operand = |expression, name, expected: Option<Type>| {
            let path = format!("{path}.{name}");
            let value = self.expression(expression, scope, bindings, &path)?;
            if let Some(expected) = expected {
                check_operand(value, expected, &path)
            } else {
                Ok(value)
            }
        };
        Ok(match expression {
            Expr::JubjubScalarFromNative { value } => (
                Operation::JubjubScalarFromNative(operand(value, "value", Some(Type::Field))?),
                Type::Field,
            ),
            Expr::EcMulGenerator { scalar } => {
                let scalar = operand(scalar, "scalar", None)?;
                let curve = Curve::for_scalar(&scalar.ty);
                let scalar = check_operand(scalar, curve.scalar(), &format!("{path}.scalar"))?;
                (Operation::EcMulGenerator(scalar), curve.point())
            }
            Expr::EcNeg { value } => (
                Operation::EcNeg(operand(value, "value", Some(Type::JubjubPoint))?),
                Type::JubjubPoint,
            ),
            Expr::EcMul { point, scalar } => {
                let point = operand(point, "point", None)?;
                let curve = Curve::for_point(&point.ty);
                let point = check_operand(point, curve.point(), &format!("{path}.point"))?;
                let scalar = operand(scalar, "scalar", Some(curve.scalar()))?;
                (Operation::EcMul { point, scalar }, curve.point())
            }
            Expr::EcAdd { left, right } => {
                let left = operand(left, "left", None)?;
                let curve = Curve::for_point(&left.ty);
                let left = check_operand(left, curve.point(), &format!("{path}.left"))?;
                let right = operand(right, "right", Some(curve.point()))?;
                (Operation::EcAdd { left, right }, curve.point())
            }
            _ => return Err(self.unsupported_expression(expression, path)),
        })
    }
}

fn check_operand(value: Value, expected: Type, path: &str) -> Result<Value, RecordingGap> {
    if value.ty == expected {
        Ok(value)
    } else {
        let curve = match expected {
            Type::Secp256k1Point | Type::Secp256k1Scalar => "secp256k1",
            _ => "Jubjub",
        };
        Err(invalid(
            path,
            format!("expected {expected:?} {curve} operand"),
        ))
    }
}

// Keep coordinate-only temporaries off each recursive arithmetic lowering frame.
impl Lowerer<'_> {
    pub(super) fn jubjub_coordinate(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let (input, x_coordinate) = match expression {
            Expr::JubjubPointX { value } => (value, true),
            Expr::JubjubPointY { value } => (value, false),
            _ => return Err(self.unsupported_expression(expression, path)),
        };
        let operand_path = format!("{path}.value");
        let value = self.expression(input, scope, bindings, &operand_path)?;
        if value.ty != Type::JubjubPoint {
            return Err(invalid(
                &operand_path,
                "expected JubjubPoint Jubjub operand",
            ));
        }
        let operation = if x_coordinate {
            Operation::JubjubPointX(value)
        } else {
            Operation::JubjubPointY(value)
        };
        Ok((operation, Type::Field))
    }
}

// Constructor temporaries must not enlarge every recursive EcAdd/EcMul frame.
// Recursive dispatch selects this helper using its shared result slot.
impl Lowerer<'_> {
    pub(super) fn jubjub_constructor(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let Expr::ConstructJubjubPoint { x, y } = expression else {
            return Err(self.unsupported_expression(expression, path));
        };
        let mut operand = |expression, name| {
            let path = format!("{path}.{name}");
            let value = self.expression(expression, scope, bindings, &path)?;
            if value.ty != Type::Field {
                return Err(invalid(&path, "expected Field Jubjub operand"));
            }
            Ok(value)
        };
        let x = operand(x, "x")?;
        let y = operand(y, "y")?;
        Ok((Operation::ConstructJubjubPoint { x, y }, Type::JubjubPoint))
    }
}
