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

//! Generic functional Map bodies are projected from genuine checked source nodes.
use super::*;
use crate::ir::{Expr, Parameter, PureCircuit, StateReturn, StatefulCircuit, Type};
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
                file: "vector-map.compact".into(),
                line: 1,
                column: 1,
            },
            missing: None,
        }
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, site: SourceSite<'_>) -> OriginEvidence<'a> {
        if matches!(site,SourceSite::Value {id,..} if self.missing==Some(id)) {
            OriginEvidence::Unknown
        } else {
            OriginEvidence::BoundOriginalNode(&self.location)
        }
    }
}
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn vector(element: Type, length: usize) -> Type {
    Type::Vector {
        element: Box::new(element),
        length,
    }
}
fn mapped(
    source: Expr,
    formal: &str,
    element: Type,
    body: Expr,
    result: Type,
    length: usize,
) -> Expr {
    Expr::VectorMap {
        parameter: Parameter {
            name: formal.into(),
            ty: element,
        },
        source: Box::new(source),
        body: Box::new(body),
        result,
        length,
    }
}
fn root(element: Type, result: Type, length: usize, body: Expr) -> StatefulCircuit {
    StatefulCircuit {
        name: "root".into(),
        source: None,
        internal: false,
        parameters: vec![
            Parameter {
                name: "source".into(),
                ty: vector(element.clone(), length),
            },
            Parameter {
                name: "capture".into(),
                ty: element.clone(),
            },
            Parameter {
                name: "flag".into(),
                ty: Type::Boolean,
            },
        ],
        actions: vec![],
        result: vector(result.clone(), length),
        return_value: StateReturn::Expression {
            value: mapped(input("source"), "item", element, body, result, length),
        },
    }
}
fn checked(root: &StatefulCircuit, pure: &[PureCircuit]) -> CheckedPlan {
    let pure = pure.iter().map(|p| (p.name.as_str(), p)).collect();
    super::super::check_with_calls_observed(
        root,
        &HashMap::new(),
        &HashMap::new(),
        &pure,
        &HashMap::from([("root", root)]),
        None,
    )
    .unwrap()
}
fn map_binding(checked: &CheckedPlan) -> &Binding {
    checked
        .plan()
        .body
        .bindings
        .iter()
        .find(|b| matches!(b.operation, Operation::VectorMap { .. }))
        .unwrap()
}
#[test]
fn recursive_map_if_certified_call_and_assert_fold_without_executor_capture() {
    let pure = PureCircuit {
        name: "identity".into(),
        source: None,
        internal: true,
        parameters: vec![Parameter {
            name: "value".into(),
            ty: Type::Field,
        }],
        result: Type::Field,
        body: input("value"),
    };
    let inner = mapped(
        Expr::Vector {
            elements: vec![input("item"), input("capture")],
            element: Type::Field,
        },
        "inner",
        Type::Field,
        Expr::Sequence {
            steps: vec![Expr::Assert {
                condition: Box::new(input("flag")),
                message: "mapped guard".into(),
            }],
            value: Box::new(Expr::Call {
                name: "identity".into(),
                arguments: vec![input("inner")],
            }),
        },
        Type::Field,
        2,
    );
    let body = Expr::If {
        condition: Box::new(input("flag")),
        then: Box::new(inner.clone()),
        otherwise: Box::new(inner),
    };
    let root = root(Type::Field, vector(Type::Field, 2), 3, body);
    let checked = checked(&root, &[pure]);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    for capability in [
        Capability::VectorMap,
        Capability::Pure,
        Capability::Require,
        Capability::Select,
    ] {
        assert!(program.requirements().contains(&capability));
    }
    assert!(program.guards().len() > 1);
    let binding = map_binding(&checked);
    let expression =
        super::super::functional_pure_blocks::vector_map(binding, checked.declarations())
            .unwrap()
            .to_token_stream()
            .to_string();
    assert_eq!(expression.matches("try_map").count(), 3);
    assert!(expression.contains("pure_circuits :: identity"));
    assert!(expression.contains("mapped guard"));
    assert!(expression.contains("AssertionFailed"));
    assert!(!expression.contains("frame"));
    assert!(!expression.contains("__compact_execution"));
    assert!(!expression.contains("move |"));
    let _ = super::super::functional_render::render(&program).unwrap();
}
#[test]
fn empty_maps_still_require_body_authority_and_require_capability() {
    let body = Expr::Sequence {
        steps: vec![Expr::Assert {
            condition: Box::new(Expr::Boolean { value: true }),
            message: "zero".into(),
        }],
        value: Box::new(input("item")),
    };
    let root = root(Type::Field, Type::Field, 0, body);
    let checked = checked(&root, &[]);
    let binding = map_binding(&checked);
    let Operation::VectorMap { body, .. } = &binding.operation else {
        unreachable!()
    };
    let assertion = body
        .bindings
        .iter()
        .find(|b| matches!(b.operation, Operation::Assert { .. }))
        .unwrap();
    let mut authority = Authority::new();
    authority.missing = Some(assertion.value.id);
    assert!(
        matches!(FunctionalProgram::project(&checked,&authority,Ledger8Target::PINNED),Err(ProjectionRefusal::MissingSourceAuthority {path}) if path==assertion.path)
    );
    let authority = Authority::new();
    let target = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::VectorMap],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::Require,
            ..
        })
    ));
    let target = Ledger8Target {
        capabilities: &[Capability::Pure, Capability::Require],
    };
    assert!(matches!(
        FunctionalProgram::project(&checked, &authority, target),
        Err(ProjectionRefusal::MissingTargetCapability {
            capability: Capability::VectorMap,
            ..
        })
    ));
}
#[test]
fn context_free_crypto_is_not_silently_granted_by_map_admission() {
    let root = root(
        Type::Field,
        Type::Field,
        0,
        Expr::TransientHash {
            value: Box::new(input("item")),
        },
    );
    let checked = checked(&root, &[]);
    let authority = Authority::new();
    let Operation::VectorMap { body, .. } = &map_binding(&checked).operation else {
        unreachable!()
    };
    let hash = body
        .bindings
        .iter()
        .find(|b| matches!(b.operation, Operation::TransientHash(_)))
        .unwrap();
    assert!(
        matches!(FunctionalProgram::project(&checked,&authority,Ledger8Target::PINNED),Err(ProjectionRefusal::OutsideFunctionalSubset {path}) if path==hash.path)
    );
}
#[test]
fn owned_source_and_captures_are_retained_inside_non_move_mapping_closure() {
    let element = Type::OpaqueString;
    let result = Type::Tuple {
        elements: vec![element.clone(); 3],
    };
    let mut root = root(
        element.clone(),
        result.clone(),
        2,
        Expr::Tuple {
            elements: vec![input("item"), input("capture"), input("capture")],
        },
    );
    let StateReturn::Expression { value: mapped } = root.return_value.clone() else {
        unreachable!()
    };
    root.result = Type::Tuple {
        elements: vec![vector(result, 2), vector(element, 2)],
    };
    root.return_value = StateReturn::Expression {
        value: Expr::Tuple {
            elements: vec![mapped, input("source")],
        },
    };
    let checked = checked(&root, &[]);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let expression = super::super::functional_pure_blocks::vector_map(
        map_binding(&checked),
        checked.declarations(),
    )
    .unwrap()
    .to_token_stream()
    .to_string();
    assert!(expression.matches("clone ()").count() >= 4, "{expression}");
    assert!(!expression.contains("move |"));
    let rendered = super::super::functional_render::render(&program).unwrap();
    assert!(rendered.executor_bounds.is_empty());
}
#[test]
fn unit_producers_execute_but_empty_tuple_aliases_are_retained() {
    for result in [Type::Unit, Type::Tuple { elements: vec![] }] {
        let body = Expr::Sequence {
            steps: vec![Expr::Assert {
                condition: Box::new(input("flag")),
                message: "unit producer".into(),
            }],
            value: Box::new(if result == Type::Unit {
                Expr::Unit
            } else {
                Expr::Tuple { elements: vec![] }
            }),
        };
        let root = root(Type::Field, result.clone(), 1, body);
        let checked = checked(&root, &[]);
        let expression = super::super::functional_pure_blocks::vector_map(
            map_binding(&checked),
            checked.declarations(),
        )
        .unwrap()
        .to_token_stream()
        .to_string();
        assert!(expression.contains("unit producer"));
        let Operation::VectorMap { body, .. } = &map_binding(&checked).operation else {
            unreachable!()
        };
        let alias = format!("let __compact_effect_{}", body.result.id);
        assert_eq!(
            expression.contains(&alias),
            result != Type::Unit,
            "{expression}"
        );
    }
}
