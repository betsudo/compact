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
    let checked = checked(Some(Operation::TransientHash(value(2, Type::Field))));
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
fn field_witness_keeps_checked_signature_guard_and_explicit_target_requirement() {
    let checked = witness_checked(Type::Field);
    let authority = AuthorityDouble::node_spans();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert_eq!(program.effects()[1].capability(), Capability::TypedWitness);
    assert_eq!(program.effects()[1].guard(), GuardId(1));
    assert_eq!(program.effects()[1].privacy(), Privacy::Unknown);
    let target = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::Require, Capability::Select],
    };
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability { path, capability: Capability::TypedWitness }) if path == "checked[1]")
    );
}

#[test]
fn opaque_witness_output_requires_explicit_exact_target_capability() {
    let checked = witness_checked(Type::OpaqueBytes);
    let authority = AuthorityDouble::node_spans();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert_eq!(program.effects()[1].capability(), Capability::ExactWitness);
    assert_eq!(program.effects()[1].guard(), GuardId(1));
    assert_eq!(program.effects()[1].privacy(), Privacy::Unknown);
    let target = Ledger8Target {
        capabilities: &[
            Capability::Pure,
            Capability::Require,
            Capability::Select,
            Capability::TypedWitness,
        ],
    };
    assert!(
        matches!(FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability { path, capability: Capability::ExactWitness }) if path == "checked[1]")
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

#[test]
fn witness_rendering_selects_exact_mode_for_nested_opaque_results_only() {
    use quote::ToTokens;
    for (result, exact) in [
        (Type::Field, false),
        (
            Type::Tuple {
                elements: vec![Type::Field, Type::OpaqueBytes],
            },
            true,
        ),
    ] {
        let checked = witness_checked(result);
        let authority = AuthorityDouble::node_spans();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        let rendered = super::super::functional_render::render(&program).unwrap();
        let body = rendered.body.to_token_stream().to_string();
        let bounds = rendered
            .executor_bounds
            .iter()
            .map(ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(body.contains(". witness_in ("), exact);
        assert_eq!(body.contains(". witness ("), !exact);
        assert_eq!(bounds.contains("ExactWitnessOutput"), exact);
    }
}

// These plans use a literal at SSA0, unlike the earlier parameter-based fixtures.
struct CounterAuthority {
    source: SourceLocation,
    missing: Option<usize>,
}
impl CounterAuthority {
    fn new() -> Self {
        Self {
            source: SourceLocation {
                file: "retained-counter.compact".into(),
                line: 2,
                column: 1,
            },
            missing: None,
        }
    }
}
impl SourceAuthority for CounterAuthority {
    fn origin<'source>(&'source self, site: SourceSite<'_>) -> OriginEvidence<'source> {
        if matches!(site, SourceSite::Value { id, .. } if self.missing == Some(id)) {
            OriginEvidence::Unknown
        } else {
            OriginEvidence::BoundOriginalNode(&self.source)
        }
    }
}

fn remaining_counter_checked(
    kind: usize,
    invalid_operand: bool,
) -> Result<CheckedPlan, crate::RecordingGap> {
    use super::super::model::{CounterSlot, LedgerId};
    let slot = CounterSlot {
        declaration: LedgerId(0),
        index: 0,
    };
    let field = LedgerField {
        source: None,
        id: "counter".into(),
        index: 0,
        path: vec![1, 2],
        declaration: LedgerFieldKind::Counter,
    };
    let operand_type = Type::Unsigned {
        max: if invalid_operand {
            "255".into()
        } else if kind == 0 {
            u64::MAX.to_string()
        } else {
            u16::MAX.to_string()
        },
    };
    let operand = value(0, operand_type);
    let result = value(1, if kind == 0 { Type::Boolean } else { Type::Unit });
    let operation = match kind {
        0 => Operation::CounterLessThan {
            slot,
            threshold: operand.clone(),
        },
        1 => Operation::CounterDecrement {
            slot,
            amount: operand.clone(),
        },
        2 => Operation::CounterReset { slot },
        _ => unreachable!(),
    };
    let root = StatefulCircuit {
        source: None,
        name: "example".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: result.ty.clone(),
        return_value: if kind == 0 {
            StateReturn::Expression {
                value: crate::ir::Expr::Boolean { value: false },
            }
        } else {
            StateReturn::Unit
        },
    };
    let mut bindings = vec![];
    if kind != 2 {
        bindings.push(binding(
            operand,
            Operation::Literal(Literal::Unsigned(if invalid_operand {
                1
            } else if kind == 0 {
                u64::MAX as u128
            } else {
                u16::MAX as u128
            })),
        ));
    }
    bindings.push(binding(result.clone(), operation));
    CheckedPlan::validate(
        Plan {
            parameters: vec![],
            result_type: result.ty.clone(),
            body: Block { bindings, result },
        },
        Declarations::new(&HashMap::from([("counter", &field)]), &HashMap::new()).with_root(&root),
    )
}

#[test]
fn remaining_counter_operations_request_only_their_own_capability_and_slot() {
    use quote::ToTokens;
    for (kind, capability, method, trait_name) in [
        (
            0,
            Capability::CounterCompare,
            "counter_less_than",
            "CounterCompare",
        ),
        (
            1,
            Capability::CounterDecrement,
            "decrement",
            "CounterDecrement",
        ),
        (2, Capability::CounterReset, "reset_counter", "CounterReset"),
    ] {
        let checked = remaining_counter_checked(kind, false).unwrap();
        let authority = CounterAuthority::new();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        assert!(program.requirements().contains(&capability));
        let rendered = super::super::functional_render::render(&program).unwrap();
        assert_eq!(rendered.counters.len(), 1);
        assert_eq!(rendered.counters[0].declaration.0, 0);
        let bounds = rendered
            .executor_bounds
            .iter()
            .map(ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            bounds,
            vec![format!("crate :: runtime :: functional :: {trait_name}")]
        );
        let body = rendered.body.to_token_stream().to_string();
        assert!(body.contains(&format!(". {method} (")));
        assert!(!body.contains("CounterIncrement"));
        if kind == 0 {
            assert!(body.contains("as u64"));
        }
        if kind == 1 {
            assert!(body.contains("as u16"));
        }
    }
}

#[test]
fn remaining_counter_capability_and_source_refusals_stay_precise() {
    for (kind, capability) in [
        (0, Capability::CounterCompare),
        (1, Capability::CounterDecrement),
        (2, Capability::CounterReset),
    ] {
        let checked = remaining_counter_checked(kind, false).unwrap();
        let authority = CounterAuthority::new();
        let target = Ledger8Target {
            capabilities: &[Capability::Pure],
        };
        assert!(
            matches!(FunctionalProgram::project(&checked, &authority, target),
            Err(ProjectionRefusal::MissingTargetCapability { path, capability: actual }) if path == "checked[1]" && actual == capability)
        );
    }
}

#[test]
fn remaining_counter_bad_operand_bounds_keep_the_original_checked_refusal() {
    for kind in [0, 1] {
        let error = remaining_counter_checked(kind, true).unwrap_err();
        assert_eq!(error.path, "checked[1]");
        assert_eq!(
            error.detail,
            "operation input/output types are incompatible"
        );
    }
}

#[test]
fn remaining_counter_nodes_do_not_bypass_source_authority() {
    for kind in 0..3 {
        let checked = remaining_counter_checked(kind, false).unwrap();
        let authority = CounterAuthority {
            missing: Some(1),
            ..CounterAuthority::new()
        };
        assert!(
            matches!(FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
            Err(ProjectionRefusal::MissingSourceAuthority { path }) if path == "checked[1]")
        );
    }
}
