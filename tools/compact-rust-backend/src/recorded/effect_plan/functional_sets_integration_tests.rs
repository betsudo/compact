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

//! Set projection and packaging from genuine checked plans.
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
            file: "sets.compact".into(),
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
        id: "entries".into(),
        index: 0,
        path: vec![1, 2],
        declaration: LedgerFieldKind::Set { ty: Type::Field },
    }
}
fn input() -> Expr {
    Expr::Parameter {
        name: "value".into(),
    }
}
fn root() -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "value".into(),
            ty: Type::Field,
        }],
        actions: vec![
            StateAction::Expression {
                value: Expr::SetIsEmpty {
                    field: "entries".into(),
                    index: 0,
                },
            },
            StateAction::SetInsert {
                field: "entries".into(),
                index: 0,
                value: input(),
            },
            StateAction::Expression {
                value: Expr::SetMember {
                    field: "entries".into(),
                    index: 0,
                    value: Box::new(input()),
                },
            },
            StateAction::SetRemove {
                field: "entries".into(),
                index: 0,
                value: input(),
            },
            StateAction::SetReset {
                field: "entries".into(),
                index: 0,
            },
        ],
        result: Type::Unsigned {
            max: u64::MAX.to_string(),
        },
        return_value: StateReturn::SetSize {
            field: "entries".into(),
            index: 0,
        },
    }
}
fn checked(root: &StatefulCircuit, field: &LedgerField) -> CheckedPlan {
    super::super::check_with_calls_observed(
        root,
        &HashMap::from([("entries", field)]),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", root)]),
        None,
    )
    .unwrap()
}
#[test]
fn six_operations_share_one_slot_and_precise_capabilities() {
    let root = root();
    let field = field();
    let checked = checked(&root, &field);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    assert!(program.requirements().contains(&Capability::SetRead));
    assert!(program.requirements().contains(&Capability::SetWrite));
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert_eq!(rendered.sets.len(), 1);
    let bounds = rendered
        .executor_bounds
        .iter()
        .map(|b| b.to_token_stream().to_string())
        .collect::<Vec<_>>();
    assert_eq!(bounds.len(), 2);
    assert!(bounds.iter().any(|b| b.contains("SetRead")));
    assert!(bounds.iter().any(|b| b.contains("SetWrite")));
    let body = rendered.body.to_token_stream().to_string();
    for method in [
        "set_member",
        "set_insert",
        "set_remove",
        "set_reset",
        "set_size",
        "set_is_empty",
    ] {
        assert!(body.contains(method), "missing {method}");
    }
}
#[test]
fn set_binding_uses_full_physical_path_and_no_set_packaging_stays_minimal() {
    let contract = Contract {
        schema_version: 6,
        type_aliases: vec![],
        ledger_fields: vec![field()],
        constructor: None,
        witnesses: vec![],
        circuits: vec![],
        stateful_circuits: vec![root()],
    };
    let authority = Authority::new();
    let text = super::super::functional_items::render_contract(&contract, &authority)
        .unwrap()
        .module
        .to_token_stream()
        .to_string();
    assert!(text.contains("SetBinding :: new"));
    assert!(text.contains("bind_sets (SETS)"));
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
    let simple = Contract {
        stateful_circuits: vec![simple],
        ledger_fields: vec![],
        ..contract
    };
    let text = super::super::functional_items::render_contract(&simple, &authority)
        .unwrap()
        .module
        .to_token_stream()
        .to_string();
    assert!(!text.contains("const SETS"));
    assert!(!text.contains("bind_sets"));
    assert!(text.contains("with_counters (CELLS , COUNTERS)"));
}
#[test]
fn target_must_explicitly_accept_each_set_capability() {
    let root = root();
    let field = field();
    let checked = checked(&root, &field);
    let authority = Authority::new();
    for missing in [Capability::SetRead, Capability::SetWrite] {
        let target = Ledger8Target {
            capabilities: match missing {
                Capability::SetRead => &[Capability::Pure, Capability::SetWrite],
                Capability::SetWrite => &[Capability::Pure, Capability::SetRead],
                _ => unreachable!(),
            },
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability {capability,..}) if capability==missing)
        );
    }
}
#[test]
fn read_only_and_write_only_sets_do_not_require_the_other_capability() {
    let field = field();
    let authority = Authority::new();
    for write in [false, true] {
        let mut root = root();
        root.actions = if write {
            vec![StateAction::SetInsert {
                field: "entries".into(),
                index: 0,
                value: input(),
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
        assert_eq!(
            program.requirements().contains(&Capability::SetWrite),
            write
        );
        assert_eq!(
            program.requirements().contains(&Capability::SetRead),
            !write
        );
    }
}

#[test]
fn genuine_checked_coin_storage_requires_qualified_not_scalar_capability() {
    let contract: Contract = serde_json::from_str(include_str!(
        "../../../tests/qualified-coin-set-schema20-ir.json"
    ))
    .unwrap();
    let root = &contract.stateful_circuits[0];
    let fields = contract
        .ledger_fields
        .iter()
        .map(|f| (f.id.as_str(), f))
        .collect();
    let witnesses = contract
        .witnesses
        .iter()
        .map(|w| (w.name.as_str(), w))
        .collect();
    let pure = contract
        .circuits
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let calls = contract
        .stateful_circuits
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let checked =
        super::super::check_with_calls_observed(root, &fields, &witnesses, &pure, &calls, None)
            .unwrap();
    let binding = checked
        .plan()
        .body
        .bindings
        .iter()
        .find(|b| matches!(b.operation, Operation::SetInsertCoin { .. }))
        .unwrap();
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let effect = program
        .effects()
        .iter()
        .find(|effect| effect.binding().path == binding.path)
        .unwrap();
    assert_eq!(effect.capability(), Capability::QualifiedSetInsert);
    let capabilities = Ledger8Target::PINNED
        .capabilities
        .iter()
        .copied()
        .filter(|cap| *cap != Capability::QualifiedSetInsert)
        .collect::<Vec<_>>();
    let target = Ledger8Target {
        capabilities: Box::leak(capabilities.into_boxed_slice()),
    };
    assert!(
        matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability{path,capability:Capability::QualifiedSetInsert})if path==binding.path)
    );
}
