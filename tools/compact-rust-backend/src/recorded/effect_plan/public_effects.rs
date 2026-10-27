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

//! Static public-program evidence from validated effects, independent of IR shapes.
use super::model::{CheckedPlan, Operation};

/// At least one branch can contain a replayable public operation. This does not
/// promise every execution observes one: preparation still owns EmptyTranscript.
pub(in crate::recorded) fn has_public_program(plan: &CheckedPlan) -> bool {
    let mut blocks = vec![&plan.plan().body];
    while let Some(block) = blocks.pop() {
        for binding in &block.bindings {
            match &binding.operation {
                Operation::Call { body, .. } | Operation::VectorMap { body, .. } => blocks.push(body),
                Operation::VectorFold { sources, body, .. } => {
                    // Operand producers are in the enclosing block. A statically
                    // empty reduction cannot execute its public body.
                    if matches!(sources.first().map(|value| &value.ty), Some(crate::ir::Type::Vector { length, .. }) if *length > 0) {
                        blocks.push(body);
                    }
                }
                Operation::If { then, otherwise, .. } => {
                    blocks.push(otherwise);
                    blocks.push(then);
                }
                Operation::KernelSelf | Operation::KernelClaim { .. } | Operation::KernelMintShielded { .. }
                | Operation::List { .. } | Operation::Merkle { .. } | Operation::CellRead(_) | Operation::CellWrite { .. }
                | Operation::CellWriteCoin { .. } | Operation::MapMember { .. } | Operation::MapLookup { .. }
                | Operation::MapInsert { .. } | Operation::MapRemove { .. }
                | Operation::MapInsertDefault { .. } | Operation::MapReset { .. } | Operation::MapSize { .. } | Operation::MapIsEmpty { .. } | Operation::SetMember { .. }
                | Operation::SetInsert { .. } | Operation::SetInsertCoin { .. } | Operation::SetRemove { .. }
                | Operation::SetReset { .. } | Operation::SetSize { .. } | Operation::SetIsEmpty { .. }
                | Operation::CounterRead { .. } | Operation::CounterIncrement { .. }
                | Operation::CounterDecrement { .. } | Operation::CounterReset { .. } | Operation::CounterLessThan { .. } => return true,
                // Private transcript entries and Zswap intents do not emit VM ops.
                Operation::Witness { .. } | Operation::OwnPublicKey
                | Operation::CreateZswapInput { .. } | Operation::CreateZswapOutput { .. }
                // The complete body of a PureCall has already been certified context-free.
                | Operation::PureCall { .. } | Operation::Assert { .. }
                | Operation::Literal(_) | Operation::EnumVariant { .. } | Operation::Default
                | Operation::StructConstruct { .. } | Operation::StructProject { .. }
                | Operation::TupleProject { .. }
                | Operation::VectorProject { .. } | Operation::Tuple { .. } | Operation::Vector { .. }
                | Operation::PersistentCommit { .. } | Operation::FieldToBytes32(_)
                | Operation::PersistentHash(_) | Operation::Keccak256(_) | Operation::HashToCurve(_)
                | Operation::DegradeToTransient(_) | Operation::UpgradeFromTransient(_) | Operation::TransientHash(_)
                | Operation::TransientCommit { .. } | Operation::Add(_, _) | Operation::Subtract(_, _) | Operation::Multiply(_, _)
                | Operation::UnsignedAdd(_, _) | Operation::UnsignedSubtract(_, _) | Operation::UnsignedMultiply(_, _)
                | Operation::Compare { .. } | Operation::Equal(_, _) | Operation::NotEqual(_, _)
                | Operation::JubjubPointX(_) | Operation::JubjubPointY(_) | Operation::JubjubScalarFromNative(_)
                | Operation::EcMulGenerator(_) | Operation::EcNeg(_) | Operation::ConstructJubjubPoint { .. }
                | Operation::EcMul { .. } | Operation::EcAdd { .. } | Operation::Convert { .. } => {}
            }
        }
    }
    false
}
