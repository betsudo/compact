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

//! Call composition uses genuine lowering/certification and checked lexical scopes.
use super::*;
use crate::ir::{
    Expr, LedgerField, LedgerFieldKind, Parameter, PureCircuit, StateAction, StateReturn,
    StatefulCircuit, Type,
};
use quote::ToTokens;
use std::collections::HashMap;

struct Authority {
    location: SourceLocation,
    missing: Option<usize>,
}
impl Authority {
    fn new() -> Self {
        Self {
            location: SourceLocation {
                file: "calls.compact".into(),
                line: 1,
                column: 1,
            },
            missing: None,
        }
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, site: SourceSite<'_>) -> OriginEvidence<'a> {
        if matches!(site,SourceSite::Value{id,..} if self.missing==Some(id)) {
            OriginEvidence::Unknown
        } else {
            OriginEvidence::BoundOriginalNode(&self.location)
        }
    }
}
fn parameter(name: &str, ty: Type) -> Parameter {
    Parameter {
        name: name.into(),
        ty,
    }
}
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        name: name.into(),
        arguments: args,
    }
}
fn circuit(name: &str, parameters: Vec<Parameter>, result: Type, value: Expr) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: name.into(),
        internal: name != "root",
        parameters,
        actions: vec![],
        result,
        return_value: StateReturn::Expression { value },
    }
}
fn checked(circuits: &[StatefulCircuit], pure: &[PureCircuit]) -> CheckedPlan {
    let field = LedgerField {
        source: None,
        id: "stored".into(),
        index: 0,
        path: vec![0],
        declaration: LedgerFieldKind::Cell { ty: Type::Field },
    };
    let fields = HashMap::from([("stored", &field)]);
    let circuits_by_name = circuits.iter().map(|c| (c.name.as_str(), c)).collect();
    let pure_by_name = pure.iter().map(|c| (c.name.as_str(), c)).collect();
    super::super::check_with_calls_observed(
        &circuits[0],
        &fields,
        &HashMap::new(),
        &pure_by_name,
        &circuits_by_name,
        None,
    )
    .unwrap()
}
fn branch_call() -> CheckedPlan {
    let helper = circuit(
        "helper",
        vec![],
        Type::Field,
        Expr::CellRead {
            field: "stored".into(),
            index: 0,
        },
    );
    let mut root = circuit(
        "root",
        vec![parameter("enabled", Type::Boolean)],
        Type::Unit,
        Expr::Unit,
    );
    root.return_value = StateReturn::Unit;
    root.actions = vec![StateAction::If {
        condition: input("enabled"),
        then: Box::new(StateAction::CircuitCall {
            name: "helper".into(),
            arguments: vec![],
        }),
        otherwise: Box::new(StateAction::Sequence { actions: vec![] }),
    }];
    checked(&[root, helper], &[])
}
#[test]
fn nested_call_effects_keep_parent_guard_and_collect_executor_capabilities() {
    let checked = branch_call();
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let call = program
        .effects()
        .iter()
        .find(|e| matches!(e.binding().operation, Operation::Call { .. }))
        .unwrap();
    let read = program
        .effects()
        .iter()
        .find(|e| matches!(e.binding().operation, Operation::CellRead(_)))
        .unwrap();
    assert_eq!(call.capability(), Capability::Call);
    assert_eq!(call.guard(), read.guard());
    assert_ne!(call.guard(), GuardId(0));
    assert!(program.requirements().contains(&Capability::Call));
    assert!(program.requirements().contains(&Capability::CellRead));
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert_eq!(rendered.slots.len(), 1);
    assert!(
        rendered
            .executor_bounds
            .iter()
            .any(|bound| bound.to_token_stream().to_string().contains("CellRead"))
    );
    assert!(
        rendered
            .body
            .to_token_stream()
            .to_string()
            .contains(". read (")
    );
}
#[test]
fn nested_call_missing_authority_and_missing_target_capability_refuse() {
    let checked = branch_call();
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let nested = program
        .effects()
        .iter()
        .find(|e| matches!(e.binding().operation, Operation::CellRead(_)))
        .unwrap()
        .binding()
        .value
        .id;
    let mut missing = Authority::new();
    missing.missing = Some(nested);
    assert!(matches!(
        FunctionalProgram::project(&checked, &missing, Ledger8Target::PINNED),
        Err(ProjectionRefusal::MissingSourceAuthority { .. })
    ));
    let target = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::Select, Capability::CellRead],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::Call,
            ..
        })
    ));
}
#[test]
fn two_calls_retain_noncopy_arguments_and_keep_fresh_formal_scopes() {
    let ty = Type::OpaqueString;
    let helper = circuit(
        "helper",
        vec![parameter("item", ty.clone())],
        ty.clone(),
        input("item"),
    );
    let mut root = circuit(
        "root",
        vec![parameter("original", ty.clone())],
        ty.clone(),
        input("original"),
    );
    root.actions = (0..2)
        .map(|_| StateAction::CircuitCall {
            name: "helper".into(),
            arguments: vec![input("original")],
        })
        .collect();
    let checked = checked(&[root, helper], &[]);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let formals: Vec<_> = program
        .effects()
        .iter()
        .filter_map(|effect| match &effect.binding().operation {
            Operation::Call { parameters, .. } => Some(parameters[0].id),
            _ => None,
        })
        .collect();
    assert_eq!(formals.len(), 2);
    assert_ne!(formals[0], formals[1]);
    let rendered = super::super::functional_render::render(&program)
        .unwrap()
        .body
        .to_token_stream()
        .to_string();
    for formal in formals {
        assert!(rendered.contains(&format!("let __compact_effect_{formal} : crate :: runtime :: OpaqueString = (__compact_effect_0) . clone ()")));
    }
    assert!(rendered.contains("try_returning"));
}
#[test]
fn checked_pure_calls_reuse_the_same_certified_syntax_as_ordinary_recording() {
    let pure = PureCircuit {
        source: None,
        name: "identity".into(),
        internal: true,
        parameters: vec![parameter("item", Type::Field)],
        result: Type::Field,
        body: input("item"),
    };
    let root = circuit(
        "root",
        vec![parameter("original", Type::Field)],
        Type::Field,
        call("identity", vec![input("original")]),
    );
    let checked = checked(&[root], &[pure]);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let binding = program
        .effects()
        .iter()
        .find(|e| matches!(e.binding().operation, Operation::PureCall { .. }))
        .unwrap()
        .binding();
    let expression =
        super::super::render::pure::certified_call(binding, checked.declarations(), |value| {
            let name = quote::format_ident!("__compact_effect_{}", value.id);
            syn::parse_quote!(#name)
        })
        .unwrap();
    let expected: syn::Expr =
        syn::parse_quote!(crate::pure_circuits::identity(__compact_effect_0)?);
    assert_eq!(
        expression.to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert!(rendered.executor_bounds.is_empty());
    assert_eq!(
        rendered
            .body
            .to_token_stream()
            .to_string()
            .matches("pure_circuits :: identity")
            .count(),
        1
    );
}
#[test]
fn certified_unit_calls_keep_one_fallible_statement_without_a_unit_local() {
    for result in [Type::Unit, Type::Tuple { elements: vec![] }] {
        let pure = PureCircuit {
            source: None,
            name: "validate".into(),
            internal: true,
            parameters: vec![parameter("accepted", Type::Boolean)],
            result: result.clone(),
            body: Expr::Sequence {
                steps: vec![Expr::Assert {
                    condition: Box::new(input("accepted")),
                    message: "unit guard".into(),
                }],
                value: Box::new(if result == Type::Unit {
                    Expr::Unit
                } else {
                    Expr::Tuple { elements: vec![] }
                }),
            },
        };
        let root = circuit(
            "root",
            vec![parameter("accepted", Type::Boolean)],
            result,
            call("validate", vec![input("accepted")]),
        );
        let checked = checked(&[root], &[pure]);
        let authority = Authority::new();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        let rendered = super::super::functional_render::render(&program)
            .unwrap()
            .body
            .to_token_stream()
            .to_string();
        assert_eq!(rendered.matches("pure_circuits :: validate").count(), 1);
        assert!(rendered.contains("pure_circuits :: validate (__compact_effect_0) ? ; Ok (())"));
        assert!(
            !rendered
                .contains("let __compact_functional_result = crate :: pure_circuits :: validate")
        );
    }
}

#[test]
fn unsupported_operations_inside_call_bodies_are_not_hidden_by_composition() {
    let helper = circuit(
        "helper",
        vec![],
        Type::Bytes { length: 32 },
        Expr::Keccak256 {
            value: Box::new(Expr::FieldLiteral { value: "7".into() }),
        },
    );
    let root = circuit(
        "root",
        vec![],
        Type::Bytes { length: 32 },
        call("helper", vec![]),
    );
    let checked = checked(&[root, helper], &[]);
    let Operation::Call { body, .. } = &checked.plan().body.bindings.last().unwrap().operation
    else {
        unreachable!()
    };
    let unsupported = body
        .bindings
        .iter()
        .find(|binding| matches!(binding.operation, Operation::Keccak256(_)))
        .unwrap();
    let authority = Authority::new();
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED),
        Err(ProjectionRefusal::OutsideFunctionalSubset { path }) if path==unsupported.path
    ));
}

#[test]
fn calls_skip_exact_unit_aliases_but_retain_empty_tuple_formals() {
    for ty in [Type::Unit, Type::Tuple { elements: vec![] }] {
        let helper = circuit(
            "helper",
            vec![parameter("item", ty.clone())],
            ty.clone(),
            input("item"),
        );
        let root = circuit(
            "root",
            vec![parameter("original", ty.clone())],
            ty.clone(),
            call("helper", vec![input("original")]),
        );
        let checked = checked(&[root, helper], &[]);
        let authority = Authority::new();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        let formal = program
            .effects()
            .iter()
            .find_map(|effect| {
                if let Operation::Call { parameters, .. } = &effect.binding().operation {
                    Some(parameters[0].id)
                } else {
                    None
                }
            })
            .unwrap();
        let rendered = super::super::functional_render::render(&program)
            .unwrap()
            .body
            .to_token_stream()
            .to_string();
        let alias = format!("let __compact_effect_{formal} : ()");
        assert_eq!(rendered.contains(&alias), ty != Type::Unit);
    }
}
