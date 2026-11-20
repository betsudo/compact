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

//! Original persistent bodies, without new checks or traversal.
use super::*;

impl Lowerer<'_> {
    pub(super) fn persistent_operation(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let value = match expression {
            Expr::PersistentHash { value } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                if !validate::fixed_aligned_fab_type(&value.ty) {
                    return Err(invalid(
                        path,
                        "persistent hash requires a supported aligned FAB value",
                    ));
                }
                (Operation::PersistentHash(value), Type::Bytes { length: 32 })
            }
            Expr::Keccak256 { value, encoding } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                if !validate::keccak_input_type(&value.ty, *encoding) {
                    return Err(invalid(path, "Keccak requires a supported FAB value"));
                }
                (
                    Operation::Keccak256(value, *encoding),
                    Type::Bytes { length: 32 },
                )
            }
            Expr::Sha512 { value } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                if !validate::fixed_aligned_fab_type(&value.ty) {
                    return Err(invalid(
                        path,
                        "SHA-512 requires a supported fixed aligned FAB value",
                    ));
                }
                (Operation::Sha512(value), Type::Bytes { length: 64 })
            }
            Expr::PersistentCommit { value, opening } => {
                let value = self.expression(value, scope, bindings, &format!("{path}.value"))?;
                let opening =
                    self.expression(opening, scope, bindings, &format!("{path}.opening"))?;
                if !validate::fixed_aligned_fab_type(&value.ty)
                    || opening.ty != (Type::Bytes { length: 32 })
                {
                    return Err(self.unsupported_expression_reason(
                        expression,
                        path,
                        "persistent commitment requires a fixed aligned FAB value and Bytes32 opening",
                    ));
                }
                (
                    Operation::PersistentCommit { value, opening },
                    Type::Bytes { length: 32 },
                )
            }
            _ => unreachable!("expression dispatcher selects persistent"),
        };
        Ok(value)
    }
}
