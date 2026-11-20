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

//! Original conversions bodies, without new checks or traversal.
use super::*;
use crate::pure_semantics::conversion::{FixedBytesConversion, JubjubCast, Secp256k1BytesCast};

impl Lowerer<'_> {
    pub(super) fn conversion_operation(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let value = match expression {
            Expr::FieldToBytes32 { value } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                if value.ty != Type::Field {
                    return Err(invalid(path, "FieldToBytes32 requires a Field operand"));
                }
                (Operation::FieldToBytes32(value), Type::Bytes { length: 32 })
            }
            Expr::VectorToBytes { .. }
            | Expr::BytesToVector { .. }
            | Expr::UintToBytes { .. }
            | Expr::WideUintToBytes { .. }
            | Expr::BytesToField { .. }
            | Expr::BytesToUint { .. }
            | Expr::BytesZeroPad { .. }
            | Expr::BytesFromChunks { .. }
            | Expr::BytesSlice { .. } => {
                let (value, operation) = FixedBytesConversion::from_expression(expression);
                let operation = operation.map_err(|error| invalid(path, error.to_string()))?;
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                let ty = operation
                    .result_type(&value.ty)
                    .map_err(|error| invalid(path, error.to_string()))?;
                (
                    Operation::Convert {
                        value,
                        kind: Conversion::FixedBytes(operation),
                    },
                    ty,
                )
            }
            Expr::JubjubCast { value, ty } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                JubjubCast::check(&value.ty, ty)
                    .map_err(|error| invalid(path, error.to_string()))?;
                (
                    Operation::Convert {
                        value,
                        kind: Conversion::JubjubCast,
                    },
                    ty.clone(),
                )
            }
            Expr::EnumCast { value, ty } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                crate::pure_semantics::conversion::EnumCast::check(&value.ty, ty)
                    .map_err(|error| invalid(path, error.to_string()))?;
                (
                    Operation::Convert {
                        value,
                        kind: Conversion::EnumCast,
                    },
                    ty.clone(),
                )
            }
            Expr::Secp256k1BytesCast { value, ty } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                Secp256k1BytesCast::check(&value.ty, ty)
                    .map_err(|error| invalid(path, error.to_string()))?;
                (
                    Operation::Convert {
                        value,
                        kind: Conversion::Secp256k1BytesCast,
                    },
                    ty.clone(),
                )
            }
            Expr::UnsignedCast { value, max } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                if validate::unsigned_bound(&value.ty).is_none()
                    || validate::unsigned_bound(&uint(max)).is_none()
                {
                    return Err(invalid(path, "unsupported unsigned cast"));
                }
                (
                    Operation::Convert {
                        value,
                        kind: Conversion::UnsignedCast,
                    },
                    uint(max),
                )
            }
            _ => unreachable!("expression dispatcher selects conversions"),
        };
        Ok(value)
    }
}
