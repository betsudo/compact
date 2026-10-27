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

//! Independent loop guards and new admission of unchanged genuine prior Fold IR.
use super::{
    declarations::Declarations,
    functional::{
        Capability, FunctionalProgram, Ledger8Target, OriginEvidence, SourceAuthority, SourceSite,
    },
    model::*,
};
use crate::ir::{
    Contract, LedgerField, LedgerFieldKind, Parameter, SourceLocation, StateReturn,
    StatefulCircuit, Type,
};
use quote::ToTokens;
use std::collections::HashMap;

fn vector(length: usize) -> Type {
    Type::Vector {
        element: Box::new(Type::Field),
        length,
    }
}
fn value(id: usize, ty: Type) -> Value {
    Value { id, ty }
}
fn plan(length: usize) -> (Plan, LedgerField) {
    let initial = value(0, Type::Field);
    let sources = vec![value(1, vector(length)), value(2, vector(length))];
    let accumulator = value(3, Type::Field);
    let items = vec![value(4, Type::Field), value(5, Type::Field)];
    let sum = value(6, Type::Field);
    let product = value(7, Type::Field);
    let write = value(8, Type::Unit);
    let read = value(9, Type::Field);
    let result = value(10, Type::Field);
    let slot = CellSlot {
        declaration: LedgerId(0),
        index: 0,
    };
    let body = Block {
        bindings: vec![
            Binding {
                value: sum.clone(),
                path: "fold.body.add".into(),
                operation: Operation::Add(accumulator.clone(), items[0].clone()),
            },
            Binding {
                value: product.clone(),
                path: "fold.body.multiply".into(),
                operation: Operation::Multiply(sum, items[1].clone()),
            },
            Binding {
                value: write,
                path: "fold.body.write".into(),
                operation: Operation::CellWrite {
                    slot,
                    value: product,
                },
            },
            Binding {
                value: read.clone(),
                path: "fold.body.read".into(),
                operation: Operation::CellRead(slot),
            },
        ],
        result: read,
    };
    let plan = Plan {
        parameters: std::iter::once(initial.clone())
            .chain(sources.clone())
            .collect(),
        result_type: Type::Field,
        body: Block {
            bindings: vec![Binding {
                value: result.clone(),
                path: "fold".into(),
                operation: Operation::VectorFold {
                    initial,
                    sources,
                    accumulator: accumulator.clone(),
                    items,
                    body,
                },
            }],
            result,
        },
    };
    (
        plan,
        LedgerField {
            id: "stored".into(),
            index: 0,
            path: vec![],
            source: None,
            declaration: LedgerFieldKind::Cell { ty: Type::Field },
        },
    )
}
fn check(plan: Plan, field: &LedgerField) -> Result<CheckedPlan, super::RecordingGap> {
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: plan
            .parameters
            .iter()
            .enumerate()
            .map(|(i, v)| Parameter {
                name: format!("p{i}"),
                ty: v.ty.clone(),
            })
            .collect(),
        actions: vec![],
        result: plan.result_type.clone(),
        return_value: StateReturn::Unit,
    };
    CheckedPlan::validate(
        plan,
        Declarations::new(&HashMap::from([("stored", field)]), &HashMap::new()).with_root(&root),
    )
}
struct Authority(SourceLocation);
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, _: SourceSite<'_>) -> OriginEvidence<'a> {
        OriginEvidence::BoundOriginalNode(&self.0)
    }
}

#[test]
fn vector_fold_original_named_source_is_deliberately_new_recording_admission() {
    let contract: Contract = serde_json::from_str(include_str!(
        "../../../tests/direct_fold/named-predecessor-ir.json"
    ))
    .unwrap();
    let rendered = crate::render_with_capabilities(&contract).unwrap();
    let root = rendered
        .capabilities
        .circuits
        .iter()
        .find(|row| row.name == "run")
        .unwrap();
    assert!(
        root.recorded && root.observed_call,
        "previous genuine recording gap is now admitted"
    );
    syn::parse_file(&rendered.source).unwrap();
    assert!(rendered.source.contains("__compact_fold_accumulator"));
}

#[test]
fn vector_fold_legacy_static_seed_refusal_still_precedes_source() {
    for length in [0, 2] {
        let mut encoded: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/direct_fold/named-predecessor-ir.json"
        ))
        .unwrap();
        let root = &mut encoded["stateful_circuits"][0];
        root["parameters"][1]["ty"]["length"] = length.into();
        let fold = &mut root["actions"][0]["bindings"][0]["value"];
        fold["length"] = length.into();
        fold["initial"] = serde_json::json!({"kind":"parameter", "name":"missing_seed"});
        fold["source"] = serde_json::json!({"kind":"parameter", "name":"missing_source"});
        let c: Contract = serde_json::from_value(encoded).unwrap();
        assert_eq!(
            crate::render_with_capabilities(&c).err().unwrap(),
            crate::RenderError::Located {
                location: c.stateful_circuits[0].source.clone().unwrap(),
                error: Box::new(crate::RenderError::UnknownParameter("missing_seed".into())),
            }
        );
        let error = super::check_with_calls(
            &c.stateful_circuits[0],
            &c.ledger_fields.iter().map(|v| (v.id.as_str(), v)).collect(),
            &HashMap::new(),
            &c.circuits.iter().map(|v| (v.name.as_str(), v)).collect(),
            &c.stateful_circuits
                .iter()
                .map(|v| (v.name.as_str(), v))
                .collect(),
        )
        .err()
        .unwrap();
        let super::PlanError::Unsupported(gap) = error else {
            panic!("expected the missing seed refusal");
        };
        assert!(gap.path.ends_with(".initial"), "{}", gap.path);
    }
}

#[test]
fn vector_fold_multi_input_effectful_body_has_one_owned_loop_and_static_inventory() {
    for length in [0, 1, 3] {
        let (plan, field) = plan(length);
        let checked = check(plan, &field).unwrap();
        assert!(!super::context_free::bindings(
            &checked.plan().body.bindings
        ));
        assert_eq!(super::has_public_program(&checked), length > 0);
        let ordinary = super::render(&checked).unwrap();
        let tokens = ordinary
            .steps
            .iter()
            .map(ToTokens::to_token_stream)
            .collect::<proc_macro2::TokenStream>()
            .to_string();
        assert_eq!(tokens.matches("for ").count(), 1);
        assert!(tokens.contains(". zip ("));
        let authority = Authority(SourceLocation {
            file: "typed-loop-test".into(),
            line: 1,
            column: 1,
        });
        let projected =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        for capability in [
            Capability::VectorFold,
            Capability::CellRead,
            Capability::CellWrite,
        ] {
            assert!(projected.requirements().contains(&capability));
        }
        let rendered = super::functional_render::render(&projected).unwrap();
        let tokens = rendered.body.to_token_stream().to_string();
        assert_eq!(tokens.matches("for ").count(), 1);
        assert!(tokens.contains("__compact_fold_execution . bind"));
        let metadata: serde_json::Value =
            serde_json::from_str(&super::functional_metadata::metadata(&projected)).unwrap();
        assert!(
            metadata["fold_body_semantics"]
                .as_str()
                .unwrap()
                .contains("zero times")
        );
        assert_eq!(
            projected.effects().len(),
            5,
            "loop and four body occurrences, including empty inputs; not an execution count"
        );
    }
}

#[test]
fn vector_fold_forged_lengths_types_formals_and_capture_leak_refuse_even_empty() {
    for length in [0, 2] {
        for mutation in 0..9 {
            let (mut input, field) = plan(length);
            let Operation::VectorFold {
                initial,
                sources,
                accumulator,
                items,
                body,
            } = &mut input.body.bindings[0].operation
            else {
                unreachable!()
            };
            match mutation {
                0 => sources.clear(),
                1 => {
                    items.pop();
                }
                2 => sources[1].ty = vector(length + 1),
                3 => items[1].ty = Type::Boolean,
                4 => initial.ty = Type::Boolean,
                5 => body.result.ty = Type::Boolean,
                6 => items[0].id = accumulator.id,
                7 => accumulator.id = sources[0].id,
                8 => {
                    body.bindings[0].operation =
                        Operation::Add(value(999, Type::Field), items[0].clone())
                }
                _ => unreachable!(),
            }
            assert!(
                check(input, &field).is_err(),
                "length={length}, mutation={mutation}"
            );
        }
    }
}

#[test]
fn vector_fold_unit_accumulator_is_not_an_empty_tuple_payload() {
    let initial = value(0, Type::Unit);
    let source = value(1, vector(0));
    let accumulator = value(2, Type::Unit);
    let item = value(3, Type::Field);
    let result = value(4, Type::Unit);
    let plan = Plan {
        parameters: vec![initial.clone(), source.clone()],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![Binding {
                value: result.clone(),
                path: "fold".into(),
                operation: Operation::VectorFold {
                    initial,
                    sources: vec![source],
                    accumulator: accumulator.clone(),
                    items: vec![item],
                    body: Block {
                        bindings: vec![],
                        result: accumulator,
                    },
                },
            }],
            result,
        },
    };
    let (_, field) = self::plan(0);
    let checked = check(plan.clone(), &field).unwrap();
    let steps = super::render(&checked).unwrap().steps;
    syn::parse2::<syn::File>(quote::quote!(fn syntax_only() { #(#steps)* })).unwrap();
    let authority = Authority(SourceLocation {
        file: "unit-loop-test".into(),
        line: 1,
        column: 1,
    });
    let projected =
        FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    super::functional_render::render(&projected).unwrap();
    let mut forged = plan;
    let Operation::VectorFold { body, .. } = &mut forged.body.bindings[0].operation else {
        unreachable!()
    };
    body.result.ty = Type::Tuple { elements: vec![] };
    assert!(
        check(forged, &field).is_err(),
        "Unit and empty tuple keep distinct checked types"
    );
}
