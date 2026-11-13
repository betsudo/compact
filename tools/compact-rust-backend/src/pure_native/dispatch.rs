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

//! One exhaustive source variant map, instantiated for two native output views.
//! The family owner checks children; projection runs only after it returns.
//! No recording profile or syntax-free checker is selected here.

macro_rules! dispatch {
    ($expr:expr, $parameters:expr, $circuits:expr; $control:ident, $value:ident) => {{
        // Keep one post-match Result slot for all ordinary value families.
        // Per-arm adapter arguments inflate unoptimized dispatcher frames.
        let result = match $expr {
            $crate::ir::Expr::Assert { .. }
            | $crate::ir::Expr::Sequence { .. }
            | $crate::ir::Expr::If { .. }
            | $crate::ir::Expr::Let { .. } => {
                return $control(
                    $expr,
                    $crate::pure_control_fragment($expr, $parameters, $circuits),
                );
            }
            $crate::ir::Expr::Unit => $crate::pure_leaf_expression($expr, $parameters, $circuits),
            $crate::ir::Expr::Default { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Boolean { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::FieldLiteral { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::BytesLiteral { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::UnsignedLiteral { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Parameter { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::EnumVariant { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::StructField { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::TupleIndex { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::StructLiteral { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Tuple { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Vector { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::VectorMap { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::VectorFold { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::VectorFoldCall { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Call { .. } => {
                $crate::pure_aggregate_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::TransientHash { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::TransientCommit { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::PersistentHash { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Keccak256 { .. } | $crate::ir::Expr::Sha512 { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::PersistentCommit { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::DegradeToTransient { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::UpgradeFromTransient { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::HashToCurve { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::JubjubPointX { .. } | $crate::ir::Expr::JubjubPointY { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::EcAdd { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::ConstructJubjubPoint { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::EcNeg { .. }
            | $crate::ir::Expr::Secp256k1Neg { .. }
            | $crate::ir::Expr::Secp256k1Inv { .. }
            | $crate::ir::Expr::Secp256k1PointX { .. }
            | $crate::ir::Expr::Secp256k1PointY { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::EcMul { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::EcMulGenerator { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::JubjubScalarFromNative { .. } => {
                $crate::pure_crypto_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::WitnessCall { .. }
            | $crate::ir::Expr::KernelClaim { .. }
            | $crate::ir::Expr::EmitShieldedSpend { .. }
            | $crate::ir::Expr::EmitShieldedBurn { .. }
            | $crate::ir::Expr::EmitShieldedMint { .. }
            | $crate::ir::Expr::EmitShieldedReceive { .. }
            | $crate::ir::Expr::EmitUnshieldedSpend { .. }
            | $crate::ir::Expr::EmitUnshieldedReceive { .. }
            | $crate::ir::Expr::EmitUnshieldedMint { .. }
            | $crate::ir::Expr::EmitUnshieldedBurn { .. }
            | $crate::ir::Expr::KernelMintShielded { .. }
            | $crate::ir::Expr::CreateZswapInput { .. }
            | $crate::ir::Expr::CreateZswapOutput { .. }
            | $crate::ir::Expr::NativeWitnessCall { .. }
            | $crate::ir::Expr::SetMember { .. }
            | $crate::ir::Expr::MapMember { .. }
            | $crate::ir::Expr::MapLookup { .. }
            | $crate::ir::Expr::NestedMapLookup { .. }
            | $crate::ir::Expr::MerkleCheckRoot { .. }
            | $crate::ir::Expr::HistoricMerkleCheckRoot { .. }
            | $crate::ir::Expr::SetIsEmpty { .. }
            | $crate::ir::Expr::SetSize { .. }
            | $crate::ir::Expr::MapSize { .. }
            | $crate::ir::Expr::MapIsEmpty { .. }
            | $crate::ir::Expr::ListLength { .. }
            | $crate::ir::Expr::ListIsEmpty { .. }
            | $crate::ir::Expr::ListHead { .. }
            | $crate::ir::Expr::CellRead { .. }
            | $crate::ir::Expr::CounterRead { .. }
            | $crate::ir::Expr::CounterLessThan { .. }
            | $crate::ir::Expr::KernelSelf { .. } => {
                $crate::pure_leaf_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::FieldCast { .. } => {
                $crate::pure_conversion_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::FieldToBytes32 { .. } => {
                $crate::pure_conversion_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::VectorToBytes { .. }
            | $crate::ir::Expr::BytesToVector { .. }
            | $crate::ir::Expr::UintToBytes { .. }
            | $crate::ir::Expr::WideUintToBytes { .. }
            | $crate::ir::Expr::BytesToField { .. }
            | $crate::ir::Expr::BytesToUint { .. }
            | $crate::ir::Expr::BytesZeroPad { .. }
            | $crate::ir::Expr::BytesFromChunks { .. }
            | $crate::ir::Expr::BytesSlice { .. }
            | $crate::ir::Expr::JubjubCast { .. }
            | $crate::ir::Expr::Secp256k1BytesCast { .. }
            | $crate::ir::Expr::EnumCast { .. } => {
                $crate::pure_conversion_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Coerce { .. } => {
                $crate::pure_conversion_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::UnsignedCast { .. } => {
                $crate::pure_conversion_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::UnsignedAdd { .. }
            | $crate::ir::Expr::UnsignedSubtract { .. }
            | $crate::ir::Expr::UnsignedMultiply { .. } => {
                $crate::pure_arithmetic_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Add { .. }
            | $crate::ir::Expr::Subtract { .. }
            | $crate::ir::Expr::Multiply { .. } => {
                $crate::pure_arithmetic_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Equal { .. } | $crate::ir::Expr::NotEqual { .. } => {
                $crate::pure_arithmetic_expression($expr, $parameters, $circuits)
            }
            $crate::ir::Expr::Compare { .. } => {
                $crate::pure_arithmetic_expression($expr, $parameters, $circuits)
            }
        };
        $value($expr, result)
    }};
}

pub(crate) use dispatch;
