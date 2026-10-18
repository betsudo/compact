// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An immutable view of the checked graph, not another semantic IR.
//!
//! Projection records capability requirements, effective branch guards and
//! independently supplied original-occurrence evidence. Operations and types remain borrowed
//! from CheckedPlan; execution/rendering must consume that same graph.

use super::model::{Binding, Block, CheckedPlan, Operation, Value};
use crate::ir::SourceLocation;
use std::collections::BTreeSet;

/// Capabilities of this first functional projection, not all ledger8 support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Capability {
    Pure,
    CellRead,
    CellWrite,
    CounterRead,
    CounterIncrement,
    BooleanWitness,
    TypedWitness,
    Require,
    Select,
}

/// Only the pinned ledger8 adapter exists. Other target revisions are not
/// implicitly accepted because they happen to recognize the same node names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ledger8Target {
    capabilities: &'static [Capability],
}
impl Ledger8Target {
    pub(crate) const PINNED: Self = Self {
        capabilities: &[
            Capability::Pure,
            Capability::CellRead,
            Capability::CellWrite,
            Capability::CounterRead,
            Capability::CounterIncrement,
            Capability::BooleanWitness,
            Capability::TypedWitness,
            Capability::Require,
            Capability::Select,
        ],
    };
    pub(crate) fn ledger_version(self) -> &'static str {
        "8.1.3"
    }
    pub(crate) fn supports(self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Diagnostic paths are labels, never source identities or source-file paths.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SourceSite<'a> {
    Root { name: &'a str },
    Value { id: usize, diagnostic_path: &'a str },
}

/// Evidence supplied only by the frontend issuer, after binding original node
/// identity to its invocation. Snapshot correspondence is a separate fact.
/// A coarse root fallback must be labeled as such; it is not a node span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginEvidence<'source> {
    Unknown,
    // Exact-node issuer is exercised by model controls. Production currently
    // retains only the explicitly coarse original-root fallback.
    #[allow(dead_code)]
    BoundOriginalNode(&'source SourceLocation),
    BoundOriginalRootFallback(&'source SourceLocation),
}
/// Original-node binding and a later disk snapshot cannot prove which bytes
/// the frontend consumed. This experiment does not upgrade that guarantee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConsumedSourceBytes {
    Unknown,
}

/// The minimal view has not retained a source-range correspondence assertion.
/// Keep this separate from both original-node binding and consumed bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SnapshotCorrespondence {
    Unknown,
}

impl OriginEvidence<'_> {
    pub(crate) fn consumed_source_bytes(self) -> ConsumedSourceBytes {
        ConsumedSourceBytes::Unknown
    }
    pub(crate) fn snapshot_correspondence(self) -> SnapshotCorrespondence {
        SnapshotCorrespondence::Unknown
    }
    fn is_bound_original(self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Trusted internal frontend boundary. Implementations must retain the exact
/// original-node/checked-plan association and invocation snapshot. Neither an SSA
/// number by itself nor diagnostic_path/name text establishes an original
/// occurrence association.
/// In particular, do not implement this by parsing a path, trusting candidate
/// annotations, or treating a plain IR SourceLocation as an issuance certificate.
/// Originless ingress supplies Unknown and cannot publish this experiment.
pub(crate) trait SourceAuthority {
    fn origin<'source>(&'source self, site: SourceSite<'_>) -> OriginEvidence<'source>;
}

/// The current checked graph does not retain disclosure/privacy facts. Never
/// infer Public or Secret merely from Field, argument, storage or witness types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Privacy {
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GuardId(pub(crate) usize);

/// Guards are derived from checked If structure; they do not evaluate conditions
/// or authorize executing an inactive branch. Both branches remain checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectiveGuard {
    Root,
    Branch {
        parent: GuardId,
        condition: usize,
        taken: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ProjectionRefusal {
    MissingRootDeclaration,
    MissingSourceAuthority {
        path: String,
    },
    MissingTargetCapability {
        path: String,
        capability: Capability,
    },
    OutsideFunctionalSubset {
        path: String,
    },
    UnsupportedWitnessSignature {
        path: String,
        declaration: usize,
    },
}

/// Root inputs retain their checked type and independent origin resolution.
#[derive(Debug)]
pub(super) struct Input<'plan, 'source> {
    value: &'plan Value,
    origin: OriginEvidence<'source>,
}
impl<'plan, 'source> Input<'plan, 'source> {
    pub(super) fn value(&self) -> &'plan Value {
        self.value
    }
    pub(super) fn origin(&self) -> OriginEvidence<'source> {
        self.origin
    }
    pub(super) fn privacy(&self) -> Privacy {
        Privacy::Unknown
    }
}

/// One borrowed occurrence, in source execution order within its checked block.
/// The flattened inventory is for review/validation, not an executable schedule:
/// branch effects must still execute through the original checked If blocks.
#[derive(Debug)]
pub(super) struct Effect<'plan, 'source> {
    binding: &'plan Binding,
    guard: GuardId,
    capability: Capability,
    origin: OriginEvidence<'source>,
}
impl<'plan, 'source> Effect<'plan, 'source> {
    pub(super) fn binding(&self) -> &'plan Binding {
        self.binding
    }
    pub(super) fn guard(&self) -> GuardId {
        self.guard
    }
    pub(super) fn capability(&self) -> Capability {
        self.capability
    }
    pub(super) fn origin(&self) -> OriginEvidence<'source> {
        self.origin
    }
    pub(super) fn privacy(&self) -> Privacy {
        Privacy::Unknown
    }
}

/// Construction has no ledger/witness effects and stores no Rust closures.
/// No public constructor accepts a replacement operation graph or guard labels.
#[derive(Debug)]
pub(super) struct FunctionalProgram<'plan, 'source> {
    checked: &'plan CheckedPlan,
    target: Ledger8Target,
    root_origin: OriginEvidence<'source>,
    guards: Vec<EffectiveGuard>,
    inputs: Vec<Input<'plan, 'source>>,
    effects: Vec<Effect<'plan, 'source>>,
    requirements: BTreeSet<Capability>,
}
impl<'plan, 'source> FunctionalProgram<'plan, 'source> {
    pub(super) fn project(
        checked: &'plan CheckedPlan,
        authority: &'source dyn SourceAuthority,
        target: Ledger8Target,
    ) -> Result<Self, ProjectionRefusal> {
        let root = checked
            .declarations()
            .root()
            .ok_or(ProjectionRefusal::MissingRootDeclaration)?;
        let root_origin = authority.origin(SourceSite::Root { name: &root.name });
        if !root_origin.is_bound_original() {
            return Err(ProjectionRefusal::MissingSourceAuthority {
                path: "root".into(),
            });
        }
        let mut inputs = Vec::with_capacity(checked.plan().parameters.len());
        for (index, value) in checked.plan().parameters.iter().enumerate() {
            let path = format!("parameters[{index}]");
            let origin = authority.origin(SourceSite::Value {
                id: value.id,
                diagnostic_path: &path,
            });
            if !origin.is_bound_original() {
                return Err(ProjectionRefusal::MissingSourceAuthority { path });
            }
            inputs.push(Input { value, origin });
        }
        let mut program = Self {
            checked,
            target,
            root_origin,
            guards: vec![EffectiveGuard::Root],
            inputs,
            effects: Vec::new(),
            requirements: BTreeSet::new(),
        };
        program.block(&checked.plan().body, GuardId(0), authority)?;
        Ok(program)
    }

    fn block(
        &mut self,
        block: &'plan Block,
        guard: GuardId,
        authority: &'source dyn SourceAuthority,
    ) -> Result<(), ProjectionRefusal> {
        for binding in &block.bindings {
            let capability = match &binding.operation {
                Operation::Literal(_)
                | Operation::EnumVariant { .. }
                | Operation::Default
                | Operation::Equal(_, _)
                | Operation::NotEqual(_, _) => Capability::Pure,
                Operation::CellRead(_) => Capability::CellRead,
                Operation::CellWrite { .. } => Capability::CellWrite,
                Operation::CounterRead { .. } => Capability::CounterRead,
                Operation::CounterIncrement { .. } => Capability::CounterIncrement,
                Operation::Witness {
                    declaration,
                    arguments,
                } => {
                    let signature = self.checked.declarations().witness(*declaration);
                    if !super::functional_witnesses::supported(
                        signature,
                        arguments,
                        &binding.value.ty,
                    ) {
                        return Err(ProjectionRefusal::UnsupportedWitnessSignature {
                            path: binding.path.clone(),
                            declaration: declaration.0,
                        });
                    }
                    if signature.is_some_and(|signature| {
                        signature.parameters.len() == 1
                            && signature.parameters[0].ty == crate::ir::Type::Boolean
                            && signature.result == crate::ir::Type::Boolean
                    }) {
                        Capability::BooleanWitness
                    } else {
                        Capability::TypedWitness
                    }
                }
                Operation::Assert { .. } => Capability::Require,
                Operation::If { .. } => Capability::Select,
                // Deliberately exhaustive: every new checked operation needs
                // an explicit functional admission or refusal decision.
                Operation::StructConstruct { .. }
                | Operation::StructProject { .. }
                | Operation::TupleProject { .. }
                | Operation::Tuple { .. }
                | Operation::Vector { .. }
                | Operation::VectorMap { .. }
                | Operation::CreateZswapInput { .. }
                | Operation::CreateZswapOutput { .. }
                | Operation::OwnPublicKey
                | Operation::KernelSelf
                | Operation::KernelClaim { .. }
                | Operation::KernelMintShielded { .. }
                | Operation::PersistentCommit { .. }
                | Operation::FieldToBytes32(..)
                | Operation::PersistentHash(..)
                | Operation::Keccak256(..)
                | Operation::HashToCurve(..)
                | Operation::DegradeToTransient(..)
                | Operation::UpgradeFromTransient(..)
                | Operation::TransientHash(..)
                | Operation::TransientCommit { .. }
                | Operation::Add(..)
                | Operation::Subtract(..)
                | Operation::Multiply(..)
                | Operation::UnsignedAdd(..)
                | Operation::UnsignedSubtract(..)
                | Operation::UnsignedMultiply(..)
                | Operation::Compare { .. }
                | Operation::JubjubPointX(..)
                | Operation::JubjubPointY(..)
                | Operation::JubjubScalarFromNative(..)
                | Operation::EcMulGenerator(..)
                | Operation::EcNeg(..)
                | Operation::ConstructJubjubPoint { .. }
                | Operation::EcMul { .. }
                | Operation::EcAdd { .. }
                | Operation::Convert { .. }
                | Operation::Merkle { .. }
                | Operation::List { .. }
                | Operation::CellWriteCoin { .. }
                | Operation::MapMember { .. }
                | Operation::MapInsert { .. }
                | Operation::MapRemove { .. }
                | Operation::MapLookup { .. }
                | Operation::MapInsertDefault { .. }
                | Operation::MapReset { .. }
                | Operation::MapSize { .. }
                | Operation::MapIsEmpty { .. }
                | Operation::SetMember { .. }
                | Operation::SetInsert { .. }
                | Operation::SetInsertCoin { .. }
                | Operation::SetRemove { .. }
                | Operation::SetReset { .. }
                | Operation::SetSize { .. }
                | Operation::SetIsEmpty { .. }
                | Operation::CounterDecrement { .. }
                | Operation::CounterReset { .. }
                | Operation::CounterLessThan { .. }
                | Operation::Call { .. }
                | Operation::PureCall { .. } => {
                    return Err(ProjectionRefusal::OutsideFunctionalSubset {
                        path: binding.path.clone(),
                    });
                }
            };
            if !self.target.supports(capability) {
                return Err(ProjectionRefusal::MissingTargetCapability {
                    path: binding.path.clone(),
                    capability,
                });
            }
            let origin = authority.origin(SourceSite::Value {
                id: binding.value.id,
                diagnostic_path: &binding.path,
            });
            if !origin.is_bound_original() {
                return Err(ProjectionRefusal::MissingSourceAuthority {
                    path: binding.path.clone(),
                });
            }
            self.requirements.insert(capability);
            self.effects.push(Effect {
                binding,
                guard,
                capability,
                origin,
            });
            if let Operation::If {
                condition,
                then,
                otherwise,
            } = &binding.operation
            {
                let then_guard = self.branch(guard, condition, true);
                self.block(then, then_guard, authority)?;
                let else_guard = self.branch(guard, condition, false);
                self.block(otherwise, else_guard, authority)?;
            }
        }
        Ok(())
    }
    fn branch(&mut self, parent: GuardId, condition: &Value, taken: bool) -> GuardId {
        let id = GuardId(self.guards.len());
        self.guards.push(EffectiveGuard::Branch {
            parent,
            condition: condition.id,
            taken,
        });
        id
    }
    pub(super) fn checked(&self) -> &'plan CheckedPlan {
        self.checked
    }
    pub(super) fn target(&self) -> Ledger8Target {
        self.target
    }
    pub(super) fn root_origin(&self) -> OriginEvidence<'source> {
        self.root_origin
    }
    pub(super) fn guards(&self) -> &[EffectiveGuard] {
        &self.guards
    }
    pub(super) fn inputs(&self) -> &[Input<'plan, 'source>] {
        &self.inputs
    }
    pub(super) fn effects(&self) -> &[Effect<'plan, 'source>] {
        &self.effects
    }
    pub(super) fn requirements(&self) -> &BTreeSet<Capability> {
        &self.requirements
    }
}

#[cfg(test)]
#[path = "functional_tests.rs"]
mod tests;
