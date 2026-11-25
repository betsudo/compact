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

use compact_rust_backend::ir::{Contract, Expr, LocalBinding, Parameter, Type};
use compact_rust_backend::transient_provenance::{Finding, Limits, ParameterDependency, analyze};

fn source_contract(fixture: &str) -> Contract {
    serde_json::from_str(fixture).unwrap()
}

fn names(dependencies: &[ParameterDependency]) -> Vec<&str> {
    dependencies.iter().map(|item| item.name.as_str()).collect()
}

fn finding<'a>(
    report: &'a compact_rust_backend::transient_provenance::Report,
    circuit: &str,
) -> &'a Finding {
    report
        .findings
        .iter()
        .find(|item| item.circuit == circuit)
        .unwrap()
}

#[test]
fn source_helpers_instantiate_symbolic_caller_parameters() {
    let contract = source_contract(include_str!("transient-provenance-helpers-source-ir.json"));
    let report = analyze(&contract, Limits::default());
    assert_eq!(report, analyze(&contract, Limits::default()));

    let forwarded = finding(&report, "helper_forwarder");
    assert_eq!(names(&forwarded.parameter_dependencies), ["value"]);
    assert!(
        forwarded
            .parameter_dependencies
            .iter()
            .all(|item| item.circuit == "helper_forwarder")
    );
    assert_eq!(forwarded.routes[0].producer.circuit, "helper_forwarder");

    let produced = finding(&report, "helper_producer");
    assert_eq!(names(&produced.parameter_dependencies), ["value"]);
    assert_eq!(produced.routes[0].producer.circuit, "derive");

    let mixed = finding(&report, "helper_mixed");
    assert_eq!(names(&mixed.parameter_dependencies), ["extra", "value"]);
    assert!(mixed.has_unknown_input);
    assert!(
        mixed
            .parameter_dependencies
            .iter()
            .all(|item| item.circuit == "helper_mixed")
    );
}

#[test]
fn source_products_and_branches_separate_data_and_control_parameters() {
    let products = source_contract(include_str!(
        "transient-provenance-aggregates-source-ir.json"
    ));
    let report = analyze(&products, Limits::default());
    assert_eq!(
        names(&finding(&report, "struct_hit").parameter_dependencies),
        ["value"]
    );
    assert_eq!(
        names(&finding(&report, "whole_product").parameter_dependencies),
        ["other", "value"]
    );

    let state = source_contract(include_str!(
        "transient-provenance-state-sinks-source-ir.json"
    ));
    let report = analyze(&state, Limits::default());
    let control = report
        .findings
        .iter()
        .filter(|item| item.circuit == "control_hit")
        .collect::<Vec<_>>();
    assert_eq!(control.len(), 2);
    assert_eq!(names(&control[0].parameter_dependencies), ["other"]);
    assert_eq!(names(&control[1].parameter_dependencies), ["value"]);
    assert_eq!(names(&control[0].control_parameter_dependencies), ["value"]);
    assert_eq!(names(&control[1].control_parameter_dependencies), ["value"]);
}

#[test]
fn symbolic_parameter_budget_is_explicit() {
    let contract = source_contract(include_str!("transient-provenance-helpers-source-ir.json"));
    let report = analyze(
        &contract,
        Limits {
            max_parameters: 1,
            ..Limits::default()
        },
    );
    assert!(report.truncated);
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| { entry.circuit == "helper_mixed" && entry.reason == "parameter limit" })
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|item| item.circuit == "helper_mixed")
    );
}

#[test]
fn unknown_witness_result_keeps_possible_argument_dependencies() {
    let mut contract = source_contract(include_str!("transient-provenance-source-ir.json"));
    let mixed = contract
        .circuits
        .iter_mut()
        .find(|circuit| circuit.name == "mixed")
        .unwrap();
    mixed.body = Expr::PersistentHash {
        value: Box::new(Expr::Add {
            left: Box::new(Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }),
            right: Box::new(Expr::WitnessCall {
                name: "supplied".into(),
                arguments: vec![Expr::Parameter {
                    name: "other".into(),
                }],
            }),
        }),
    };
    let report = analyze(&contract, Limits::default());
    let mixed = finding(&report, "mixed");
    assert_eq!(names(&mixed.parameter_dependencies), ["other", "value"]);
    assert!(mixed.has_unknown_input);
    assert_eq!(mixed.routes.len(), 1);
}

#[test]
fn ordinary_assertion_gate_is_explicitly_unavailable() {
    let mut contract = source_contract(include_str!("transient-provenance-helpers-source-ir.json"));
    contract
        .circuits
        .retain(|circuit| circuit.name == "helper_forwarder");
    let circuit = &mut contract.circuits[0];
    circuit.parameters.push(Parameter {
        name: "enabled".into(),
        ty: Type::Boolean,
    });
    circuit.body = Expr::Sequence {
        steps: vec![Expr::Assert {
            condition: Box::new(Expr::Parameter {
                name: "enabled".into(),
            }),
            message: "gate".into(),
        }],
        value: Box::new(Expr::PersistentHash {
            value: Box::new(Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }),
        }),
    };

    let report = analyze(&contract, Limits::default());
    let sink = finding(&report, "helper_forwarder");
    assert_eq!(names(&sink.parameter_dependencies), ["value"]);
    assert!(sink.control_parameter_dependencies.is_empty());
    assert!(report.unavailable.iter().any(|entry| {
        entry.circuit == "helper_forwarder"
            && entry.reason == "assertion execution control unavailable"
    }));
}

#[test]
fn repeated_product_aliases_stop_at_component_budget() {
    let mut contract = source_contract(include_str!("transient-provenance-helpers-source-ir.json"));
    contract
        .circuits
        .retain(|circuit| circuit.name == "helper_forwarder");
    let circuit = &mut contract.circuits[0];
    let mut bindings = Vec::new();
    let mut element_type = Type::Field;
    for index in 0..10 {
        let element = if index == 0 {
            Expr::TransientHash {
                value: Box::new(Expr::Parameter {
                    name: "value".into(),
                }),
            }
        } else {
            Expr::Parameter {
                name: format!("t{}", index - 1),
            }
        };
        element_type = Type::Tuple {
            elements: vec![element_type.clone(), element_type],
        };
        bindings.push(LocalBinding {
            name: format!("t{index}"),
            ty: element_type.clone(),
            value: Expr::Tuple {
                elements: vec![element.clone(), element],
            },
        });
    }
    circuit.body = Expr::Let {
        bindings,
        body: Box::new(Expr::PersistentHash {
            value: Box::new(Expr::Parameter { name: "t9".into() }),
        }),
    };
    let limits = Limits {
        max_component_nodes: 8,
        ..Limits::default()
    };
    let report = analyze(&contract, limits);
    assert_eq!(report, analyze(&contract, limits));
    assert!(report.truncated);
    assert!(
        report
            .unavailable
            .iter()
            .any(|entry| entry.reason == "aggregate component limit")
    );
}
