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

//! General lexical Fold ingress laws, distinct from source/runtime qualification.
use super::*;
use crate::ir::{
    Contract, Expr, Parameter, PureCircuit, ReturnPlan, StateReturn, StatefulCircuit, Type,
};

fn param(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn formal(name: &str) -> Parameter {
    Parameter {
        name: name.into(),
        ty: Type::Field,
    }
}
fn vector(length: usize) -> Type {
    Type::Vector {
        element: Box::new(Type::Field),
        length,
    }
}
fn fold(length: usize) -> Expr {
    Expr::VectorFold {
        accumulator: formal("acc"),
        items: vec![formal("left_item"), formal("right_item")],
        initial: Box::new(param("seed")),
        sources: vec![param("left"), param("right")],
        body: Box::new(ReturnPlan::Value {
            value: Expr::Add {
                left: Box::new(Expr::Multiply {
                    left: Box::new(param("acc")),
                    right: Box::new(param("factor")),
                }),
                right: Box::new(Expr::Multiply {
                    left: Box::new(param("left_item")),
                    right: Box::new(param("right_item")),
                }),
            },
        }),
        length,
    }
}
fn contract(expression: Expr, length: usize) -> Contract {
    Contract {
        schema_version: crate::ir::SCHEMA_VERSION,
        type_aliases: vec![],
        constructor: None,
        witnesses: vec![],
        circuits: vec![],
        ledger_fields: vec![],
        stateful_circuits: vec![StatefulCircuit {
            source: None,
            name: "run".into(),
            internal: false,
            parameters: vec![
                formal("seed"),
                formal("factor"),
                Parameter {
                    name: "left".into(),
                    ty: vector(length),
                },
                Parameter {
                    name: "right".into(),
                    ty: vector(length),
                },
            ],
            actions: vec![],
            result: Type::Field,
            return_value: StateReturn::Expression { value: expression },
        }],
    }
}
fn checked(c: &Contract) -> Result<CheckedPlan, PlanError> {
    check(&c.stateful_circuits[0], &HashMap::new(), &HashMap::new())
}
#[test]
fn vector_fold_ingress_captures_and_multiple_inputs_reach_pure_native_and_checked_owners() {
    for length in [0, 1, 3] {
        let mut c = contract(fold(length), length);
        let checked = checked(&c).unwrap();
        assert_eq!(fold_paths(&checked), ["return_value.value"]);
        let output = crate::render_with_capabilities(&c).unwrap();
        assert!(output.capabilities.circuits[0].recorded);
        syn::parse_file(&output.source).unwrap();
        let root = c.stateful_circuits.pop().unwrap();
        let StateReturn::Expression { value } = root.return_value else {
            unreachable!();
        };
        c.circuits.push(PureCircuit {
            source: None,
            name: root.name,
            internal: false,
            parameters: root.parameters,
            result: root.result,
            body: value,
        });
        let output = crate::render_with_capabilities(&c).unwrap();
        syn::parse_file(&output.source).unwrap();
        assert!(output.source.contains("zip") && output.source.contains("into_iter"));
    }
}
#[test]
fn vector_fold_ingress_rejects_forged_shape_types_and_missing_captures_even_when_empty() {
    for length in [0, 2] {
        for case in [
            "no_source",
            "arity",
            "duplicate",
            "normalized_duplicate",
            "body_type",
            "source_type",
            "source_length",
            "capture",
        ] {
            let mut value = fold(length);
            let Expr::VectorFold {
                accumulator,
                items,
                sources,
                body,
                ..
            } = &mut value
            else {
                unreachable!();
            };
            match case {
                "no_source" => sources.clear(),
                "arity" => {
                    items.pop();
                }
                "duplicate" => items[0].name = accumulator.name.clone(),
                "normalized_duplicate" => {
                    accumulator.name = "a$b".into();
                    items[0].name = "a_b".into();
                }
                "body_type" => {
                    **body = ReturnPlan::Value {
                        value: Expr::Boolean { value: true },
                    }
                }
                "source_type" => sources[0] = Expr::Boolean { value: true },
                "source_length" => {
                    sources[0] = Expr::Default {
                        ty: vector(length + 1),
                    }
                }
                "capture" => {
                    **body = ReturnPlan::Value {
                        value: param("escaped"),
                    }
                }
                _ => unreachable!(),
            }
            let c = contract(value, length);
            assert!(checked(&c).is_err(), "checked {length}/{case}");
            assert!(
                crate::render_with_capabilities(&c).is_err(),
                "native {length}/{case}"
            );
        }
    }
}
#[test]
fn vector_fold_ingress_seed_refusal_precedes_sources_and_reducer() {
    let mut value = fold(2);
    let Expr::VectorFold {
        initial,
        sources,
        body,
        ..
    } = &mut value
    else {
        unreachable!();
    };
    **initial = param("first");
    sources[0] = param("second");
    **body = ReturnPlan::Value {
        value: param("third"),
    };
    let c = contract(value, 2);
    assert_eq!(
        crate::render_with_capabilities(&c).err().unwrap(),
        crate::RenderError::UnknownParameter("first".into())
    );
    let Err(PlanError::Unsupported(gap)) = checked(&c) else {
        panic!("first source refusal");
    };
    assert_eq!(gap.path, "return_value.value.initial");
}
#[test]
fn vector_fold_ingress_private_tag_refuses_unknown_fields_and_missing_formals() {
    let value = fold(2);
    let mut encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(
        serde_json::from_value::<Expr>(encoded.clone()).unwrap(),
        value
    );
    encoded["unchecked"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Expr>(encoded.clone()).is_err());
    encoded.as_object_mut().unwrap().remove("unchecked");
    encoded.as_object_mut().unwrap().remove("accumulator");
    assert!(serde_json::from_value::<Expr>(encoded).is_err());
}
#[test]
fn vector_fold_ingress_body_call_edges_remain_cycle_checked_even_when_empty() {
    for length in [0, 2] {
        let mut value = fold(length);
        let Expr::VectorFold { body, .. } = &mut value else {
            unreachable!();
        };
        **body = ReturnPlan::Value {
            value: Expr::Call {
                name: "run".into(),
                arguments: ["seed", "factor", "left", "right"].map(param).to_vec(),
            },
        };
        assert!(crate::circuit_analysis::expression_contains(
            &value,
            &|node| matches!(node, Expr::Call { name, .. } if name == "run")
        ));
        let mut c = contract(value, length);
        let root = c.stateful_circuits.pop().unwrap();
        let StateReturn::Expression { value } = root.return_value else {
            unreachable!();
        };
        c.circuits.push(PureCircuit {
            source: None,
            name: root.name,
            internal: false,
            parameters: root.parameters,
            result: root.result,
            body: value,
        });
        assert_eq!(
            crate::render_with_capabilities(&c).err().unwrap(),
            crate::RenderError::RecursivePureCall("run".into())
        );
    }
}

#[test]
fn vector_fold_ingress_accepted_nested_native_shapes_keep_the_default_worker_stack() {
    for shape in ["initial", "body", "lets"] {
        let mut admitted_shapes = 0;
        for depth in [8, 24, 48, 80] {
            let mut value = Expr::FieldLiteral { value: "1".into() };
            if shape == "lets" {
                let mut body = ReturnPlan::Value {
                    value: param("acc"),
                };
                for index in 0..depth {
                    body = ReturnPlan::Let {
                        bindings: vec![crate::ir::LocalBinding {
                            name: format!("local_{index}"),
                            ty: Type::Field,
                            value: Expr::FieldLiteral { value: "2".into() },
                        }],
                        result: Box::new(body),
                    };
                }
                value = Expr::VectorFold {
                    accumulator: formal("acc"),
                    items: vec![formal("item")],
                    initial: Box::new(value),
                    sources: vec![Expr::Vector {
                        element: Type::Field,
                        elements: vec![Expr::FieldLiteral { value: "2".into() }],
                    }],
                    body: Box::new(body),
                    length: 1,
                };
            } else {
                for _ in 0..depth {
                    let (initial, body) = if shape == "initial" {
                        (value, param("acc"))
                    } else {
                        (Expr::FieldLiteral { value: "1".into() }, value)
                    };
                    value = Expr::VectorFold {
                        accumulator: formal("acc"),
                        items: vec![formal("item")],
                        initial: Box::new(initial),
                        sources: vec![Expr::Vector {
                            element: Type::Field,
                            elements: vec![Expr::FieldLiteral { value: "2".into() }],
                        }],
                        body: Box::new(ReturnPlan::Value { value: body }),
                        length: 1,
                    };
                }
            }
            let mut c = contract(value, 1);
            c.stateful_circuits[0].parameters.clear();
            let admitted = crate::resource_limits::validate(&c).is_ok();
            if admitted {
                admitted_shapes += 1;
            }
            eprintln!(
                "Fold worker shape={shape}, depth={depth}, source_preflight_admitted={admitted}"
            );
            let result = std::thread::Builder::new()
                .name(format!("fold-ingress-{shape}-{depth}"))
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    crate::render_with_capabilities(&c).map(|output| {
                        syn::parse_file(&output.source).unwrap();
                    })
                })
                .unwrap()
                .join()
                .expect("bounded Fold worker must not panic");
            if admitted {
                result.unwrap();
            } else {
                assert!(matches!(
                    result,
                    Err(crate::RenderError::ResourceLimit { .. })
                ));
            }
        }
        assert!(
            admitted_shapes > 0,
            "a nontrivial {shape} Fold must reach native lowering"
        );
    }
}
