// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


fn expression_with_calls(
    expr: &Expr,
    parameters: &HashMap<&str, (&Type, syn::Ident)>,
    circuits: &HashMap<&str, &PureCircuit>,
) -> Result<(syn::Expr, Type), RenderError> {
    match expr {
        Expr::Unit => pure_leaf_expression(expr, parameters, circuits),
        Expr::Default { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::Boolean { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::FieldLiteral { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::BytesLiteral { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::UnsignedLiteral { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::Parameter { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::EnumVariant { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::StructField { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::TupleIndex { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::StructLiteral { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::Assert { .. } => pure_control_expression(expr, parameters, circuits),
        Expr::Sequence { .. } => pure_control_expression(expr, parameters, circuits),
        Expr::Tuple { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::Vector { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::VectorMap { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::VectorFold { .. } => Err(RenderError::EffectfulExpression),
        Expr::VectorFoldCall { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::If { .. } => pure_control_expression(expr, parameters, circuits),
        Expr::Let { .. } => pure_control_expression(expr, parameters, circuits),
        Expr::Call { .. } => pure_aggregate_expression(expr, parameters, circuits),
        Expr::TransientHash { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::TransientCommit { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::PersistentHash { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::Keccak256 { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::PersistentCommit { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::DegradeToTransient { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::UpgradeFromTransient { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::HashToCurve { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::JubjubPointX { .. } | Expr::JubjubPointY { .. } => {
            pure_crypto_expression(expr, parameters, circuits)
        }
        Expr::EcAdd { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::ConstructJubjubPoint { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::EcNeg { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::EcMul { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::EcMulGenerator { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::JubjubScalarFromNative { .. } => pure_crypto_expression(expr, parameters, circuits),
        Expr::WitnessCall { .. }
        | Expr::KernelClaim { .. }
        | Expr::KernelMintShielded { .. }
        | Expr::CreateZswapInput { .. }
        | Expr::CreateZswapOutput { .. }
        | Expr::NativeWitnessCall { .. }
        | Expr::SetMember { .. }
        | Expr::MapMember { .. }
        | Expr::MapLookup { .. }
        | Expr::MerkleCheckRoot { .. }
        | Expr::HistoricMerkleCheckRoot { .. }
        | Expr::SetIsEmpty { .. }
        | Expr::SetSize { .. }
        | Expr::MapSize { .. }
        | Expr::MapIsEmpty { .. }
        | Expr::ListLength { .. }
        | Expr::ListIsEmpty { .. }
        | Expr::ListHead { .. }
        | Expr::CellRead { .. }
        | Expr::CounterRead { .. }
        | Expr::CounterLessThan { .. }
        | Expr::KernelSelf { .. } => pure_leaf_expression(expr, parameters, circuits),
        Expr::FieldCast { .. } => pure_conversion_expression(expr, parameters, circuits),
        Expr::FieldToBytes32 { .. } => pure_conversion_expression(expr, parameters, circuits),
        Expr::Coerce { .. } => pure_conversion_expression(expr, parameters, circuits),
        Expr::UnsignedCast { .. } => pure_conversion_expression(expr, parameters, circuits),
        Expr::UnsignedAdd { .. }
        | Expr::UnsignedSubtract { .. }
        | Expr::UnsignedMultiply { .. } => pure_arithmetic_expression(expr, parameters, circuits),
        Expr::Add { .. } | Expr::Subtract { .. } | Expr::Multiply { .. } => {
            pure_arithmetic_expression(expr, parameters, circuits)
        }
        Expr::Equal { .. } | Expr::NotEqual { .. } => {
            pure_arithmetic_expression(expr, parameters, circuits)
        }
        Expr::Compare { .. } => pure_arithmetic_expression(expr, parameters, circuits),
    }
}
