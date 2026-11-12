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

//! Binary operation temporaries stay off recursive dispatch for unrelated families.
use super::super::validate;
use super::*;
use crate::pure_semantics::arithmetic::{
    Secp256k1Unary, check_field_arithmetic_pair, check_secp256k1_field,
};

impl Lowerer<'_> {
    pub(super) fn secp256k1_unary(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let (kind, value) = match expression {
            Expr::Secp256k1Neg { value } => (Secp256k1Unary::Negate, value),
            Expr::Secp256k1Inv { value } => (Secp256k1Unary::Invert, value),
            _ => return Err(self.unsupported_expression(expression, path)),
        };
        let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
        check_secp256k1_field(&value.ty).map_err(|error| invalid(path, error.to_string()))?;
        let ty = value.ty.clone();
        Ok((Operation::Secp256k1Unary { kind, value }, ty))
    }

    pub(super) fn binary(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let (left, right) = match expression {
            Expr::Add { left, right }
            | Expr::Subtract { left, right }
            | Expr::Multiply { left, right }
            | Expr::Compare { left, right, .. }
            | Expr::Equal { left, right }
            | Expr::NotEqual { left, right }
            | Expr::UnsignedAdd { left, right, .. }
            | Expr::UnsignedSubtract { left, right, .. }
            | Expr::UnsignedMultiply { left, right, .. } => (left, right),
            _ => return Err(self.unsupported_expression(expression, path)),
        };
        let a = self.expression(left, scope, bindings, &format!("{path}.left"))?;
        let b = self.expression(right, scope, bindings, &format!("{path}.right"))?;
        Ok(match expression {
            Expr::Add { .. } | Expr::Subtract { .. } | Expr::Multiply { .. } => {
                check_field_arithmetic_pair(&a.ty, &b.ty)
                    .map_err(|_| invalid(path, "invalid arithmetic operand types"))?;
                let ty = a.ty.clone();
                let operation = if matches!(expression, Expr::Multiply { .. }) {
                    Operation::Multiply(a, b)
                } else if matches!(expression, Expr::Subtract { .. }) {
                    Operation::Subtract(a, b)
                } else {
                    Operation::Add(a, b)
                };
                (operation, ty)
            }
            Expr::Compare { operator, .. }
                if validate::bound(&a.ty).is_some() && validate::bound(&b.ty).is_some() =>
            {
                (
                    Operation::Compare {
                        operator: *operator,
                        left: a,
                        right: b,
                    },
                    Type::Boolean,
                )
            }
            Expr::Equal { .. } | Expr::NotEqual { .. }
                if validate::equality_type(&a.ty) && a.ty == b.ty =>
            {
                let operation = if matches!(expression, Expr::NotEqual { .. }) {
                    Operation::NotEqual(a, b)
                } else {
                    Operation::Equal(a, b)
                };
                (operation, Type::Boolean)
            }
            Expr::UnsignedAdd { max, .. }
                if validate::unsigned_add_type(&a.ty, &b.ty, &uint(max)) =>
            {
                (Operation::UnsignedAdd(a, b), uint(max))
            }
            Expr::UnsignedMultiply { max, .. }
                if validate::unsigned_multiply_type(&a.ty, &b.ty, &uint(max)) =>
            {
                (Operation::UnsignedMultiply(a, b), uint(max))
            }
            Expr::UnsignedSubtract { max, .. }
                if validate::bound(&a.ty).is_some()
                    && validate::bound(&b.ty).is_some()
                    && validate::bound(&uint(max)).is_some() =>
            {
                (Operation::UnsignedSubtract(a, b), uint(max))
            }
            _ => return Err(invalid(path, "invalid arithmetic operand types")),
        })
    }
}
