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

//! Original queries bodies, without new checks or traversal.
use super::super::borrowed_operations::LeafQuery;
use super::*;

impl Lowerer<'_> {
    pub(super) fn query_operation(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let value = match expression {
            Expr::NativeWitnessCall {
                builtin: NativeWitnessBuiltin::OwnPublicKey,
            } => {
                let ty = NativeWitnessBuiltin::OwnPublicKey.result_type();
                self.declarations.types().validate_at(&ty, path)?;
                (Operation::OwnPublicKey, ty)
            }
            Expr::KernelSelf { ty } => {
                self.declarations.types().validate_at(ty, path)?;
                if *ty != crate::ir::contract_address_type() {
                    return Err(invalid(
                        path,
                        "kernel self requires canonical ContractAddress",
                    ));
                }
                (Operation::KernelSelf, ty.clone())
            }
            Expr::KernelBalance { token }
            | Expr::KernelBalanceLessThan { token, .. }
            | Expr::KernelBalanceGreaterThan { token, .. } => {
                let token_path = format!("{path}.token");
                let token = self.expression(token, scope, bindings, &token_path)?;
                if !crate::coin_shapes::same_either_instantiation(
                    &token.ty,
                    &crate::coin_shapes::token_type(),
                ) {
                    return Err(invalid(
                        &token_path,
                        "Kernel balance requires canonical TokenType",
                    ));
                }
                match expression {
                    Expr::KernelBalance { .. } => (
                        Operation::KernelBalance { token },
                        uint(&u128::MAX.to_string()),
                    ),
                    Expr::KernelBalanceLessThan { amount, .. }
                    | Expr::KernelBalanceGreaterThan { amount, .. } => {
                        let amount_path = format!("{path}.amount");
                        let amount = self.expression(amount, scope, bindings, &amount_path)?;
                        if amount.ty != uint(&u128::MAX.to_string()) {
                            return Err(invalid(
                                &amount_path,
                                "Kernel balance bound must be Uint<128>",
                            ));
                        }
                        if matches!(expression, Expr::KernelBalanceLessThan { .. }) {
                            (
                                Operation::KernelBalanceLessThan { token, amount },
                                Type::Boolean,
                            )
                        } else {
                            (
                                Operation::KernelBalanceGreaterThan { token, amount },
                                Type::Boolean,
                            )
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::MerkleCheckRoot { field, index, root }
            | Expr::HistoricMerkleCheckRoot { field, index, root } => self.merkle_query(
                field,
                *index,
                matches!(expression, Expr::HistoricMerkleCheckRoot { .. }),
                Some(root),
                scope,
                bindings,
                path,
            )?,
            Expr::SetMember {
                field,
                index,
                value,
            } => self.set_member(field, *index, value, scope, bindings, path)?,
            Expr::SetSize { field, index } => self.leaf_query_operation(
                LeafQuery::SetSize {
                    field,
                    index: *index,
                },
                path,
            )?,
            Expr::SetIsEmpty { field, index } => self.leaf_query_operation(
                LeafQuery::SetIsEmpty {
                    field,
                    index: *index,
                },
                path,
            )?,
            Expr::ListLength { .. } | Expr::ListIsEmpty { .. } | Expr::ListHead { .. } => {
                self.list_expression(expression, path)?
            }
            Expr::CellRead { field, index } => self.leaf_query_operation(
                LeafQuery::CellRead {
                    field,
                    index: *index,
                },
                path,
            )?,
            Expr::CounterLessThan {
                field,
                index,
                threshold,
            } => {
                let slot = self.counter(field, *index, path)?;
                let threshold_path = format!("{path}.threshold");
                let threshold = self.expression(threshold, scope, bindings, &threshold_path)?;
                if threshold.ty != uint("18446744073709551615") {
                    return Err(invalid(
                        &threshold_path,
                        "counter threshold must be Uint<64>",
                    ));
                }
                (
                    Operation::CounterLessThan { slot, threshold },
                    Type::Boolean,
                )
            }
            Expr::CounterRead { field, index } => self.leaf_query_operation(
                LeafQuery::CounterRead {
                    field,
                    index: *index,
                },
                path,
            )?,
            _ => unreachable!("expression dispatcher selects queries"),
        };
        Ok(value)
    }

    /// The existing four no-operand checks, shared without synthetic Expr nodes.
    pub(in crate::recorded::effect_plan::lower) fn leaf_query_operation(
        &mut self,
        query: LeafQuery<'_>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        Ok(match query {
            LeafQuery::CellRead { field, index } => {
                let (slot, ty) = self.cell(field, index, path)?;
                (Operation::CellRead(slot), ty)
            }
            LeafQuery::CounterRead { field, index } => {
                let slot = self.counter(field, index, path)?;
                (
                    Operation::CounterRead { slot },
                    uint("18446744073709551615"),
                )
            }
            LeafQuery::SetSize { field, index } => {
                let (slot, _) = self.set(field, index, path)?;
                (Operation::SetSize { slot }, uint("18446744073709551615"))
            }
            LeafQuery::SetIsEmpty { field, index } => {
                let (slot, _) = self.set(field, index, path)?;
                (Operation::SetIsEmpty { slot }, Type::Boolean)
            }
        })
    }
}
