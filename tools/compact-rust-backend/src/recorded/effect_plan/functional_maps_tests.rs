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

//! Scalar Maps project actual checked plans; structural Maps refuse explicitly.
use super::*;
use crate::ir::{
    Contract, Expr, LedgerField, LedgerFieldKind, Parameter, StateAction, StateReturn,
    StatefulCircuit, Type,
};
use quote::ToTokens;
use std::collections::HashMap;
struct Authority(SourceLocation);
impl Authority {
    fn new() -> Self {
        Self(SourceLocation {
            file: "maps.compact".into(),
            line: 1,
            column: 1,
        })
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, _: SourceSite<'_>) -> OriginEvidence<'a> {
        OriginEvidence::BoundOriginalNode(&self.0)
    }
}
impl super::super::functional_items::FunctionalSources for Authority {
    fn root<'a>(&'a self, _: &StatefulCircuit) -> Option<&'a SourceLocation> {
        Some(&self.0)
    }
}
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn field(key: Type, value: Type) -> LedgerField {
    LedgerField {
        source: None,
        id: "entries".into(),
        index: 0,
        path: vec![1, 2],
        declaration: LedgerFieldKind::Map { key, value },
    }
}
fn root(key: Type, value: Type) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![
            Parameter {
                name: "key".into(),
                ty: key,
            },
            Parameter {
                name: "next".into(),
                ty: value,
            },
        ],
        actions: vec![
            StateAction::Expression {
                value: Expr::MapIsEmpty {
                    field: "entries".into(),
                    index: 0,
                },
            },
            StateAction::MapInsert {
                field: "entries".into(),
                index: 0,
                key: input("key"),
                value: input("next"),
            },
            StateAction::Expression {
                value: Expr::MapMember {
                    field: "entries".into(),
                    index: 0,
                    key: Box::new(input("key")),
                },
            },
            StateAction::Expression {
                value: Expr::MapLookup {
                    field: "entries".into(),
                    index: 0,
                    key: Box::new(input("key")),
                },
            },
            StateAction::MapRemove {
                field: "entries".into(),
                index: 0,
                key: input("key"),
            },
            StateAction::MapInsertDefault {
                field: "entries".into(),
                index: 0,
                key: input("key"),
            },
            StateAction::MapReset {
                field: "entries".into(),
                index: 0,
            },
        ],
        result: Type::Unsigned {
            max: "18446744073709551615".into(),
        },
        return_value: StateReturn::MapSize {
            field: "entries".into(),
            index: 0,
        },
    }
}
fn checked(root: &StatefulCircuit, field: &LedgerField) -> CheckedPlan {
    let fields = HashMap::from([("entries", field)]);
    let calls = HashMap::from([("root", root)]);
    super::super::check_with_calls_observed(
        root,
        &fields,
        &HashMap::new(),
        &HashMap::new(),
        &calls,
        None,
    )
    .unwrap()
}
#[test]
fn all_eight_scalar_map_operations_have_typed_bounds_and_one_logical_binding() {
    let field = field(Type::Boolean, Type::Field);
    let root = root(Type::Boolean, Type::Field);
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(program.requirements().contains(&Capability::MapRead));
    assert!(program.requirements().contains(&Capability::MapWrite));
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert_eq!(rendered.maps.len(), 1);
    let bounds: Vec<_> = rendered
        .executor_bounds
        .iter()
        .map(|b| b.to_token_stream().to_string())
        .collect();
    assert_eq!(bounds.len(), 4);
    assert!(bounds.iter().any(|b| b.contains("MapRead")));
    assert!(bounds.iter().any(|b| b.contains("MapWrite")));
    assert!(bounds.iter().any(|b| b.contains("MapLookup")));
    assert!(bounds.iter().any(|b| b.contains("MapRemove")));
    let source = rendered.body.to_token_stream().to_string();
    for method in [
        "map_member",
        "map_lookup",
        "map_size",
        "map_is_empty",
        "map_insert",
        "map_insert_default",
        "map_remove",
        "map_reset",
    ] {
        assert!(source.contains(method), "missing {method}");
    }
    assert!(source.contains("__compact_map_size as u128"));
    assert!(source.contains("18446744073709551615"));
}
#[test]
fn scalar_owned_key_and_value_use_existing_codec_eligibility() {
    let field = field(Type::OpaqueString, Type::OpaqueBytes);
    let root = root(Type::OpaqueString, Type::OpaqueBytes);
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let rendered = super::super::functional_render::render(&program)
        .unwrap()
        .body
        .to_token_stream()
        .to_string();
    assert!(rendered.contains("OpaqueString"));
    assert!(rendered.contains("OpaqueBytes"));
    assert!(rendered.contains("clone"));
}
#[test]
fn nested_map_membership_preserves_structural_type_without_requesting_scalar_lookup() {
    let field = field(
        Type::Boolean,
        Type::LedgerMap {
            key: Box::new(Type::Boolean),
            value: Box::new(Type::Field),
        },
    );
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "key".into(),
            ty: Type::Boolean,
        }],
        actions: vec![],
        result: Type::Boolean,
        return_value: StateReturn::Expression {
            value: Expr::MapMember {
                field: "entries".into(),
                index: 0,
                key: Box::new(input("key")),
            },
        },
    };
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert_eq!(
        program.requirements().iter().copied().collect::<Vec<_>>(),
        [Capability::MapRead]
    );
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert_eq!(rendered.executor_bounds.len(), 1);
    let bound = rendered.executor_bounds[0].to_token_stream().to_string();
    assert!(bound.contains("MapRead < bool , crate :: runtime :: slots :: MapNode < bool , crate :: runtime :: Field > >"));
    assert!(!bound.contains("MapLookup"));
    assert!(!bound.contains("MapWrite"));
}

#[test]
fn map_target_capability_is_required_before_emission() {
    let field = field(Type::Boolean, Type::Field);
    let root = root(Type::Boolean, Type::Field);
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let target = Ledger8Target {
        capabilities: &[Capability::Pure],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::MapRead,
            ..
        })
    ));
}
#[test]
fn map_bindings_keep_full_physical_paths_and_nonmap_packaging_stays_unchanged() {
    let contract = Contract {
        schema_version: 6,
        type_aliases: vec![],
        ledger_fields: vec![field(Type::Boolean, Type::Field)],
        constructor: None,
        witnesses: vec![],
        circuits: vec![],
        stateful_circuits: vec![root(Type::Boolean, Type::Field)],
    };
    let authority = Authority::new();
    let artifacts = super::super::functional_items::render_contract(&contract, &authority).unwrap();
    let text = artifacts.module.to_token_stream().to_string();
    assert!(text.contains("MapBinding :: new"));
    assert!(text.contains("with_maps (CELLS , COUNTERS , MAPS)"));
    assert!(text.contains(&quote::quote!(&[1u8, 2u8]).to_string()));
    let simple = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    let contract = Contract {
        stateful_circuits: vec![simple],
        ledger_fields: vec![],
        ..contract
    };
    let text = super::super::functional_items::render_contract(&contract, &authority)
        .unwrap()
        .module
        .to_token_stream()
        .to_string();
    assert!(!text.contains("const MAPS"));
    assert!(text.contains("with_counters (CELLS , COUNTERS)"));
}
#[test]
fn readonly_and_writeonly_maps_do_not_request_the_other_capability() {
    let field = field(Type::Boolean, Type::Field);
    let authority = Authority::new();
    for write in [false, true] {
        let mut root = root(Type::Boolean, Type::Field);
        root.actions = if write {
            vec![StateAction::MapInsert {
                field: "entries".into(),
                index: 0,
                key: input("key"),
                value: input("next"),
            }]
        } else {
            vec![]
        };
        if write {
            root.result = Type::Unit;
            root.return_value = StateReturn::Unit;
        }
        let checked = checked(&root, &field);
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        let rendered = super::super::functional_render::render(&program).unwrap();
        let bounds = rendered
            .executor_bounds
            .iter()
            .map(|b| b.to_token_stream().to_string())
            .collect::<Vec<_>>();
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].contains("MapWrite"), write);
        assert_eq!(bounds[0].contains("MapRead"), !write);
    }
}

#[test]
fn nested_map_removal_uses_structural_remove_not_scalar_write() {
    let field = field(
        Type::Boolean,
        Type::LedgerMap {
            key: Box::new(Type::Boolean),
            value: Box::new(Type::Field),
        },
    );
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "key".into(),
            ty: Type::Boolean,
        }],
        actions: vec![StateAction::MapRemove {
            field: "entries".into(),
            index: 0,
            key: input("key"),
        }],
        result: Type::Boolean,
        return_value: StateReturn::Expression {
            value: Expr::MapMember {
                field: "entries".into(),
                index: 0,
                key: Box::new(input("key")),
            },
        },
    };
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(program.requirements().contains(&Capability::MapRemove));
    assert!(!program.requirements().contains(&Capability::MapWrite));
    let rendered = super::super::functional_render::render(&program).unwrap();
    let bounds = rendered
        .executor_bounds
        .iter()
        .map(|b| b.to_token_stream().to_string())
        .collect::<Vec<_>>();
    assert_eq!(bounds.len(), 2);
    assert!(bounds.iter().any(|b| b.contains("MapRemove")));
    assert!(
        rendered
            .body
            .to_token_stream()
            .to_string()
            .contains("map_remove")
    );
}

#[test]
fn nested_scalar_operations_refuse_at_the_declaration_before_operand_evaluation() {
    let nested = Type::LedgerMap {
        key: Box::new(Type::Boolean),
        value: Box::new(Type::Field),
    };
    let field = field(Type::Boolean, nested.clone());
    for operation in ["lookup", "insert", "insert_default", "reset"] {
        // A nested Map cannot be an SSA argument. A lookup denotes the same
        // declared nested type, but is itself intentionally not a scalar codec.
        // Every mutation must refuse its declaration before visiting operands.
        let lookup = Expr::MapLookup {
            field: "entries".into(),
            index: 0,
            key: Box::new(input("key")),
        };
        let actions = match operation {
            "lookup" => vec![StateAction::Expression {
                value: lookup.clone(),
            }],
            "insert" => vec![StateAction::MapInsert {
                field: "entries".into(),
                index: 0,
                key: input("key"),
                value: lookup.clone(),
            }],
            "insert_default" => vec![StateAction::MapInsertDefault {
                field: "entries".into(),
                index: 0,
                key: input("key"),
            }],
            "reset" => vec![StateAction::MapReset {
                field: "entries".into(),
                index: 0,
            }],
            _ => unreachable!(),
        };
        let root = StatefulCircuit {
            source: None,
            name: "root".into(),
            internal: false,
            parameters: vec![Parameter {
                name: "key".into(),
                ty: Type::Boolean,
            }],
            actions,
            result: Type::Unit,
            return_value: StateReturn::Unit,
        };
        let fields = HashMap::from([("entries", &field)]);
        let circuits = HashMap::from([("root", &root)]);
        let cause = super::super::check_with_calls_observed(
            &root,
            &fields,
            &HashMap::new(),
            &HashMap::new(),
            &circuits,
            None,
        )
        .unwrap_err();
        let cause = match cause {
            super::super::DiagnosticPlanError::Unsupported(rejection) => rejection.gap,
            other => panic!("expected declaration lowering refusal: {other:?}"),
        };
        let path = if operation == "lookup" {
            "actions[0].value"
        } else {
            "actions[0]"
        };
        assert_eq!(
            cause,
            super::super::invalid(path, "type is outside checked recording value domain"),
            "{operation}"
        );
    }
}

#[test]
fn lookup_and_remove_request_their_own_target_capabilities() {
    let field = field(Type::Boolean, Type::Field);
    let authority = Authority::new();
    let mut lookup = root(Type::Boolean, Type::Field);
    lookup.actions.clear();
    lookup.result = Type::Field;
    lookup.return_value = StateReturn::Expression {
        value: Expr::MapLookup {
            field: "entries".into(),
            index: 0,
            key: Box::new(input("key")),
        },
    };
    let checked_lookup = checked(&lookup, &field);
    let target = Ledger8Target {
        capabilities: &[
            Capability::Pure,
            Capability::MapRead,
            Capability::MapRemove,
            Capability::MapWrite,
        ],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked_lookup, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::MapLookup,
            ..
        })
    ));
    let mut remove = lookup;
    remove.actions = vec![StateAction::MapRemove {
        field: "entries".into(),
        index: 0,
        key: input("key"),
    }];
    remove.result = Type::Unit;
    remove.return_value = StateReturn::Unit;
    let checked_remove = checked(&remove, &field);
    let target = Ledger8Target {
        capabilities: &[
            Capability::Pure,
            Capability::MapRead,
            Capability::MapLookup,
            Capability::MapWrite,
        ],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked_remove, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::MapRemove,
            ..
        })
    ));
}
