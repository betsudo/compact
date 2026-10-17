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

//! Projection tests use an explicit authority double, not frontend issuance.
use super::super::{
    declarations::Declarations,
    model::{CellSlot, Literal, Plan},
};
use super::*;
use crate::ir::{LedgerField, LedgerFieldKind, Parameter, StateReturn, StatefulCircuit, Type};
use std::collections::HashMap;

fn value(id: usize, ty: Type) -> Value {
    Value { id, ty }
}
fn unit(id: usize) -> Value {
    value(id, Type::Unit)
}
fn binding(value: Value, operation: Operation) -> Binding {
    Binding {
        path: format!("checked[{}]", value.id),
        value,
        operation,
    }
}
fn checked(branch_operation: Option<Operation>) -> CheckedPlan {
    let slot = CellSlot {
        declaration: super::super::model::LedgerId(0),
        index: 0,
    };
    let field = LedgerField {
        source: None,
        id: "value".into(),
        index: 0,
        path: vec![],
        declaration: LedgerFieldKind::Cell { ty: Type::Field },
    };
    let root = StatefulCircuit {
        source: None,
        name: "example".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "flag".into(),
            ty: Type::Boolean,
        }],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    let read = value(1, Type::Field);
    let literal = value(2, Type::Field);
    let condition = value(3, Type::Boolean);
    let mut otherwise = Vec::new();
    if let Some(operation) = branch_operation {
        otherwise.push(binding(value(8, Type::Field), operation));
    }
    otherwise.push(binding(
        unit(6),
        Operation::CellWrite {
            slot,
            value: literal.clone(),
        },
    ));
    let plan = Plan {
        parameters: vec![value(0, Type::Boolean)],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![
                binding(read.clone(), Operation::CellRead(slot)),
                binding(literal.clone(), Operation::Literal(Literal::Field([0; 32]))),
                binding(condition.clone(), Operation::Equal(read, literal.clone())),
                binding(
                    unit(4),
                    Operation::Assert {
                        condition: condition.clone(),
                        message: "must match".into(),
                    },
                ),
                binding(
                    unit(7),
                    Operation::If {
                        condition,
                        then: Block {
                            bindings: vec![binding(
                                unit(5),
                                Operation::CellWrite {
                                    slot,
                                    value: literal,
                                },
                            )],
                            result: unit(5),
                        },
                        otherwise: Block {
                            bindings: otherwise,
                            result: unit(6),
                        },
                    },
                ),
            ],
            result: unit(7),
        },
    };
    let ledger = HashMap::from([("value", &field)]);
    CheckedPlan::validate(
        plan,
        Declarations::new(&ledger, &HashMap::new()).with_root(&root),
    )
    .unwrap()
}
struct AuthorityDouble {
    source: SourceLocation,
    missing: Option<usize>,
    root_known: bool,
    coarse: bool,
}
impl AuthorityDouble {
    fn node_spans() -> Self {
        Self {
            source: SourceLocation {
                file: "retained.compact".into(),
                line: 3,
                column: 1,
            },
            missing: None,
            root_known: true,
            coarse: false,
        }
    }
}
impl SourceAuthority for AuthorityDouble {
    fn origin<'source>(&'source self, site: SourceSite<'_>) -> OriginEvidence<'source> {
        match site {
            SourceSite::Root { name } => {
                assert_eq!(name, "example");
                if !self.root_known {
                    return OriginEvidence::Unknown;
                }
            }
            SourceSite::Value {
                id,
                diagnostic_path,
            } => {
                let expected = if id == 0 {
                    "parameters[0]".to_owned()
                } else {
                    format!("checked[{id}]")
                };
                assert_eq!(diagnostic_path, expected);
                if self.missing == Some(id) {
                    return OriginEvidence::Unknown;
                }
            }
        }
        if self.coarse {
            OriginEvidence::BoundOriginalRootFallback(&self.source)
        } else {
            OriginEvidence::BoundOriginalNode(&self.source)
        }
    }
}

#[test]
fn program_borrows_checked_operations_and_preserves_order_types_and_branch_guards() {
    let checked = checked(None);
    let authority = AuthorityDouble::node_spans();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(std::ptr::eq(program.checked(), &checked));
    assert_eq!(program.target().ledger_version(), "8.1.3");
    assert_eq!(
        program.root_origin().consumed_source_bytes(),
        ConsumedSourceBytes::Unknown
    );
    assert_eq!(
        program.root_origin().snapshot_correspondence(),
        SnapshotCorrespondence::Unknown
    );
    assert_eq!(program.inputs().len(), 1);
    assert!(std::ptr::eq(
        program.inputs()[0].value(),
        &checked.plan().parameters[0]
    ));
    assert_eq!(program.inputs()[0].privacy(), Privacy::Unknown);
    assert!(matches!(
        program.inputs()[0].origin(),
        OriginEvidence::BoundOriginalNode(_)
    ));
    assert_eq!(
        program
            .effects()
            .iter()
            .map(|e| e.binding().value.id)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 7, 5, 6]
    );
    assert!(std::ptr::eq(
        program.effects()[0].binding(),
        &checked.plan().body.bindings[0]
    ));
    assert_eq!(program.effects()[0].binding().value.ty, Type::Field);
    assert_eq!(
        program.guards(),
        [
            EffectiveGuard::Root,
            EffectiveGuard::Branch {
                parent: GuardId(0),
                condition: 3,
                taken: true
            },
            EffectiveGuard::Branch {
                parent: GuardId(0),
                condition: 3,
                taken: false
            }
        ]
    );
    assert!(
        program.effects()[..5]
            .iter()
            .all(|e| e.guard() == GuardId(0))
    );
    assert_eq!(program.effects()[5].guard(), GuardId(1));
    assert_eq!(program.effects()[6].guard(), GuardId(2));
    assert!(
        program
            .effects()
            .iter()
            .all(|e| e.privacy() == Privacy::Unknown)
    );
    assert_eq!(program.effects()[3].capability(), Capability::Require);
    assert_eq!(program.requirements().len(), 5);
}

#[test]
fn unknown_source_cannot_become_bound_original_from_candidate_paths() {
    let checked = checked(None);
    let mut authority = AuthorityDouble::node_spans();
    authority.root_known = false;
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::MissingSourceAuthority { path }) if path == "root")
    );
    authority.root_known = true;
    authority.missing = Some(0);
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::MissingSourceAuthority { path }) if path == "parameters[0]")
    );
    authority.missing = Some(3);
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::MissingSourceAuthority { path }) if path == "checked[3]")
    );
}

#[test]
fn bound_original_root_fallback_is_coarse_and_never_becomes_a_node_span() {
    let checked = checked(None);
    let mut authority = AuthorityDouble::node_spans();
    authority.coarse = true;
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(matches!(
        program.root_origin(),
        OriginEvidence::BoundOriginalRootFallback(_)
    ));
    assert!(program.effects().iter().all(|effect| matches!(
        effect.origin(),
        OriginEvidence::BoundOriginalRootFallback(_)
    )));
}

#[test]
fn capability_refusal_is_typed_and_no_target_is_inferred_from_node_names() {
    let checked = checked(None);
    let authority = AuthorityDouble::node_spans();
    let target = Ledger8Target {
        capabilities: &[
            Capability::Pure,
            Capability::CellRead,
            Capability::Require,
            Capability::Select,
        ],
    };
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability { path, capability: Capability::CellWrite }) if path == "checked[5]")
    );
}

#[test]
fn both_branches_are_projected_and_unsupported_nodes_refuse_without_shape_fallback() {
    let checked = checked(Some(Operation::Add(
        value(2, Type::Field),
        value(2, Type::Field),
    )));
    let authority = AuthorityDouble::node_spans();
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::OutsideFunctionalSubset { path }) if path == "checked[8]")
    );
}

fn counter_checked() -> CheckedPlan {
    use super::super::model::{CounterSlot, LedgerId};
    let ty = Type::Unsigned {
        max: u64::MAX.to_string(),
    };
    let amount = value(
        1,
        Type::Unsigned {
            max: u16::MAX.to_string(),
        },
    );
    let field = LedgerField {
        source: None,
        id: "counter".into(),
        index: 0,
        path: vec![],
        declaration: LedgerFieldKind::Counter,
    };
    let root = StatefulCircuit {
        source: None,
        name: "example".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    let slot = CounterSlot {
        declaration: LedgerId(0),
        index: 0,
    };
    let plan = Plan {
        parameters: vec![],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![
                binding(amount.clone(), Operation::Literal(Literal::Unsigned(1))),
                binding(value(2, ty), Operation::CounterRead { slot }),
                binding(unit(3), Operation::CounterIncrement { slot, amount }),
            ],
            result: unit(3),
        },
    };
    CheckedPlan::validate(
        plan,
        Declarations::new(&HashMap::from([("counter", &field)]), &HashMap::new()).with_root(&root),
    )
    .unwrap()
}

#[test]
fn counter_capabilities_preserve_checked_impact_identity_and_missing_target_refuses() {
    let checked = counter_checked();
    let authority = AuthorityDouble::node_spans();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert_eq!(
        program
            .effects()
            .iter()
            .map(|effect| effect.capability())
            .collect::<Vec<_>>(),
        [
            Capability::Pure,
            Capability::CounterRead,
            Capability::CounterIncrement
        ]
    );
    assert!(matches!(
        program.effects()[2].binding().operation,
        Operation::CounterIncrement { .. }
    ));
    assert!(
        program
            .effects()
            .iter()
            .all(|effect| effect.privacy() == Privacy::Unknown)
    );
    let missing_increment = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::CounterRead],
    };
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, missing_increment),
        Err(ProjectionRefusal::MissingTargetCapability { path, capability: Capability::CounterIncrement }) if path == "checked[3]")
    );
}

fn witness_checked(result: Type) -> CheckedPlan {
    use super::super::model::WitnessId;
    use crate::ir::WitnessDeclaration;
    let root = StatefulCircuit {
        source: None,
        name: "example".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "enabled".into(),
            ty: Type::Boolean,
        }],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    let input = value(0, Type::Boolean);
    let witness_value = value(1, result.clone());
    let witness = WitnessDeclaration {
        source: None,
        name: "echo".into(),
        parameters: vec![Parameter {
            name: "flag".into(),
            ty: Type::Boolean,
        }],
        result,
    };
    let condition = if witness_value.ty == Type::Boolean {
        witness_value.clone()
    } else {
        input.clone()
    };
    let plan = Plan {
        parameters: vec![input.clone()],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![binding(
                unit(4),
                Operation::If {
                    condition: input.clone(),
                    then: Block {
                        bindings: vec![
                            binding(
                                witness_value,
                                Operation::Witness {
                                    declaration: WitnessId(0),
                                    arguments: vec![input.clone()],
                                },
                            ),
                            binding(
                                unit(2),
                                Operation::Assert {
                                    condition,
                                    message: "witness must accept".into(),
                                },
                            ),
                        ],
                        result: unit(2),
                    },
                    otherwise: Block {
                        bindings: vec![binding(
                            unit(3),
                            Operation::Assert {
                                condition: input,
                                message: "inactive witness branch".into(),
                            },
                        )],
                        result: unit(3),
                    },
                },
            )],
            result: unit(4),
        },
    };
    CheckedPlan::validate(
        plan,
        Declarations::new(&HashMap::new(), &HashMap::from([("echo", &witness)])).with_root(&root),
    )
    .unwrap()
}

#[test]
fn boolean_witness_uses_retained_signature_and_stays_guarded_with_unknown_privacy() {
    let checked = witness_checked(Type::Boolean);
    let authority = AuthorityDouble::node_spans();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let witness = &program.effects()[1];
    assert_eq!(witness.capability(), Capability::BooleanWitness);
    assert_eq!(witness.guard(), GuardId(1));
    assert_eq!(witness.privacy(), Privacy::Unknown);
    assert_eq!(
        program
            .effects()
            .iter()
            .filter(|effect| effect.capability() == Capability::BooleanWitness)
            .count(),
        1
    );
    assert_eq!(program.effects()[3].guard(), GuardId(2));
}

#[test]
fn valid_recorded_witness_outside_functional_signature_refuses_predictably() {
    let checked = witness_checked(Type::Field);
    let authority = AuthorityDouble::node_spans();
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::UnsupportedWitnessSignature { path, declaration: 0 }) if path == "checked[1]")
    );
}

#[test]
fn missing_witness_source_and_target_capability_are_not_inferred_from_guard_or_type() {
    let checked = witness_checked(Type::Boolean);
    let mut authority = AuthorityDouble::node_spans();
    authority.missing = Some(1);
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::MissingSourceAuthority { path }) if path == "checked[1]")
    );
    authority.missing = None;
    let target = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::Require, Capability::Select],
    };
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability { path, capability: Capability::BooleanWitness }) if path == "checked[1]")
    );
}
