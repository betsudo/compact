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

//! Functional List admission and packaging from checked source nodes.
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
            file: "lists.compact".into(),
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
fn field() -> LedgerField {
    LedgerField {
        source: None,
        id: "items".into(),
        index: 0,
        path: vec![1, 3],
        declaration: LedgerFieldKind::List { ty: Type::Field },
    }
}
fn root() -> StatefulCircuit {
    let result = crate::list_head_result_type(
        &Type::Field,
        &Type::Struct {
            name: "MaybeCompact7".into(),
            fields: vec![],
        },
    );
    StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "input".into(),
            ty: Type::Field,
        }],
        actions: vec![
            StateAction::ListPushFront {
                field: "items".into(),
                index: 0,
                value: Expr::Parameter {
                    name: "input".into(),
                },
            },
            StateAction::Expression {
                value: Expr::ListLength {
                    field: "items".into(),
                    index: 0,
                },
            },
            StateAction::Expression {
                value: Expr::ListIsEmpty {
                    field: "items".into(),
                    index: 0,
                },
            },
            StateAction::ListPopFront {
                field: "items".into(),
                index: 0,
            },
            StateAction::ListReset {
                field: "items".into(),
                index: 0,
            },
        ],
        result,
        return_value: StateReturn::ListHead {
            field: "items".into(),
            index: 0,
        },
    }
}
fn checked(root: &StatefulCircuit, field: &LedgerField) -> CheckedPlan {
    super::super::check_with_calls_observed(
        root,
        &HashMap::from([("items", field)]),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", root)]),
        None,
    )
    .unwrap()
}
#[test]
fn six_operations_use_one_binding_and_precise_native_maybe_bound() {
    let root = root();
    let field = field();
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    for cap in [
        Capability::ListRead,
        Capability::ListHead,
        Capability::ListWrite,
    ] {
        assert!(program.requirements().contains(&cap));
    }
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert_eq!(rendered.lists.len(), 1);
    assert_eq!(rendered.executor_bounds.len(), 3);
    let body = rendered.body.to_token_stream().to_string();
    for method in [
        "list_push_front",
        "list_pop_front",
        "list_reset",
        "list_length",
        "list_is_empty",
        "list_head",
    ] {
        assert!(body.contains(method), "{method}");
    }
    let bounds = rendered
        .executor_bounds
        .iter()
        .map(|b| b.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(bounds.contains("crate :: types :: MaybeCompact7"));
    assert!(!bounds.contains("Option"));
}
#[test]
fn list_packaging_preserves_physical_path_and_no_list_requests() {
    let authority = Authority::new();
    let contract = Contract {
        schema_version: 6,
        type_aliases: vec![],
        ledger_fields: vec![field()],
        constructor: None,
        witnesses: vec![],
        circuits: vec![],
        stateful_circuits: vec![root()],
    };
    let text = super::super::functional_items::render_contract(&contract, &authority)
        .unwrap()
        .module
        .to_token_stream()
        .to_string();
    assert!(text.contains("ListBinding :: new"));
    assert!(text.contains("bind_lists (LISTS)"));
    assert!(text.contains(&quote::quote!(&[1u8, 3u8]).to_string()));
    let simple = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    let simple = Contract {
        stateful_circuits: vec![simple],
        // Retain a declared List: only use, not declaration, adds bindings.
        ledger_fields: vec![field()],
        ..contract
    };
    let text = super::super::functional_items::render_contract(&simple, &authority)
        .unwrap()
        .module
        .to_token_stream()
        .to_string();
    assert!(!text.contains("const LISTS"));
    assert!(!text.contains("bind_lists"));
    assert!(text.contains("with_counters (CELLS , COUNTERS)"));
}
#[test]
fn head_only_requires_head_not_general_read_or_write() {
    let mut root = root();
    root.actions.clear();
    let field = field();
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(program.requirements().contains(&Capability::ListHead));
    assert!(!program.requirements().contains(&Capability::ListRead));
    assert!(!program.requirements().contains(&Capability::ListWrite));
    let target = Ledger8Target {
        capabilities: &[
            Capability::Pure,
            Capability::ListRead,
            Capability::ListWrite,
        ],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::ListHead,
            ..
        })
    ));
}

#[test]
fn each_list_capability_is_an_explicit_target_requirement() {
    let root = root();
    let field = field();
    let checked = checked(&root, &field);
    let authority = Authority::new();
    for missing in [
        Capability::ListRead,
        Capability::ListHead,
        Capability::ListWrite,
    ] {
        let target = Ledger8Target {
            capabilities: match missing {
                Capability::ListRead => &[
                    Capability::Pure,
                    Capability::ListHead,
                    Capability::ListWrite,
                ],
                Capability::ListHead => &[
                    Capability::Pure,
                    Capability::ListRead,
                    Capability::ListWrite,
                ],
                Capability::ListWrite => {
                    &[Capability::Pure, Capability::ListRead, Capability::ListHead]
                }
                _ => unreachable!(),
            },
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),
            Err(ProjectionRefusal::MissingTargetCapability {capability,..}) if capability == missing)
        );
    }
}
