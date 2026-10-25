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

//! Fixed vector projection has a distinct independently checked plan operation.
use super::*;
fn vector(element: Type, length: usize) -> Type {
    Type::Vector {
        element: Box::new(element),
        length,
    }
}
fn reference(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn project(value: Expr, index: usize) -> Expr {
    Expr::TupleIndex {
        value: Box::new(value),
        index,
    }
}
fn source(element: Type, length: usize, index: usize) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: "selected".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "input".into(),
            ty: vector(element.clone(), length),
        }],
        actions: vec![],
        result: element,
        return_value: StateReturn::Expression {
            value: project(reference("input"), index),
        },
    }
}
fn checked(source: &StatefulCircuit) -> CheckedPlan {
    check(source, &HashMap::new(), &HashMap::new()).unwrap()
}
fn projection(plan: &mut Plan) -> &mut Binding {
    plan.body
        .bindings
        .iter_mut()
        .find(|b| matches!(b.operation, Operation::VectorProject { .. }))
        .unwrap()
}
#[test]
fn vector_projection_first_last_singleton_and_owned_leaf_are_checked_and_rendered() {
    for (element, length, index) in [
        (Type::Field, 2, 0),
        (Type::Field, 2, 1),
        (Type::Boolean, 1, 0),
        (Type::OpaqueString, 2, 1),
    ] {
        let c = source(element.clone(), length, index);
        let checked = checked(&c);
        let body = &checked.plan().body;
        assert_eq!(body.result.ty, element);
        assert_eq!(body.bindings.len(), 1);
        let Operation::VectorProject {
            value,
            index: actual,
        } = &body.bindings[0].operation
        else {
            panic!("distinct vector operation")
        };
        assert_eq!(value, &checked.plan().parameters[0]);
        assert_eq!(*actual, index);
        validate::validate(checked.plan(), checked.declarations()).unwrap();
        render(&checked).unwrap();
        let contract = Contract {
            schema_version: SCHEMA_VERSION,
            type_aliases: vec![],
            ledger_fields: vec![],
            constructor: None,
            witnesses: vec![],
            circuits: vec![],
            stateful_circuits: vec![c],
        };
        let text = crate::render(&contract).unwrap();
        assert!(text.contains("pub mod recorded"));
    }
}
#[test]
fn vector_projection_operand_witness_precedes_selection_exactly_once() {
    let witness = WitnessDeclaration {
        source: None,
        name: "values".into(),
        parameters: vec![],
        result: vector(Type::Field, 2),
    };
    let mut c = source(Type::Field, 2, 1);
    c.return_value = StateReturn::Expression {
        value: project(
            Expr::WitnessCall {
                name: "values".into(),
                arguments: vec![],
            },
            1,
        ),
    };
    let checked = check(&c, &HashMap::new(), &HashMap::from([("values", &witness)])).unwrap();
    let body = &checked.plan().body;
    assert_eq!(body.bindings.len(), 2);
    assert!(matches!(
        body.bindings[0].operation,
        Operation::Witness { .. }
    ));
    let Operation::VectorProject { value, index } = &body.bindings[1].operation else {
        panic!("selection after witness")
    };
    assert_eq!(value, &body.bindings[0].value);
    assert_eq!(*index, 1);
    render(&checked).unwrap();
}
#[test]
fn vector_projection_source_refuses_out_of_bounds_and_empty_vectors() {
    for (length, index) in [(0, 0), (2, 2), (2, usize::MAX)] {
        let Err(PlanError::Unsupported(gap)) = check(
            &source(Type::Field, length, index),
            &HashMap::new(),
            &HashMap::new(),
        ) else {
            panic!("invalid vector index admitted")
        };
        assert_eq!(gap.detail, "vector projection index is outside its length");
    }
}
#[test]
fn independent_vector_validation_rejects_kind_bounds_length_type_and_scope_mutations() {
    let checked = checked(&source(Type::Field, 2, 1));
    for mutation in 0..7 {
        let mut forged = checked.plan().clone();
        let binding = projection(&mut forged);
        let Operation::VectorProject { value, index } = &mut binding.operation else {
            unreachable!()
        };
        match mutation {
            0 => *index = 2,
            1 => *index = usize::MAX,
            2 => {
                value.ty = Type::Tuple {
                    elements: vec![Type::Field; 2],
                }
            }
            3 => value.ty = vector(Type::Field, 1),
            4 => value.id = usize::MAX,
            5 => binding.value.ty = Type::Boolean,
            6 => value.ty = vector(Type::Boolean, 2),
            _ => unreachable!(),
        }
        assert!(
            validate::validate(&forged, checked.declarations()).is_err(),
            "mutation {mutation}"
        );
    }
    let mut forged = checked.plan().clone();
    let binding = projection(&mut forged);
    let Operation::VectorProject { value, index } = &binding.operation else {
        unreachable!()
    };
    binding.operation = Operation::TupleProject {
        value: value.clone(),
        index: *index,
    };
    assert!(validate::validate(&forged, checked.declarations()).is_err());
}

#[test]
fn vector_projection_binds_original_operand_and_index_beyond_type_scope_checks() {
    let mut c = source(Type::Field, 2, 0);
    c.parameters.push(Parameter {
        name: "other".into(),
        ty: vector(Type::Field, 2),
    });
    let checked = checked(&c);
    // Both operands are genuinely in scope and have exactly the same type.
    for swap_operand in [false, true] {
        let mut forged = checked.plan().clone();
        let other = forged.parameters[1].clone();
        let binding = projection(&mut forged);
        let Operation::VectorProject { value, index } = &mut binding.operation else {
            unreachable!()
        };
        if swap_operand {
            *value = other;
        } else {
            *index = 1;
        }
        assert!(validate::validate(&forged, checked.declarations()).is_err());
    }
    let declarations = declarations::Declarations::default().with_root(&c);
    assert!(
        validate::validate(checked.plan(), &declarations).is_err(),
        "a candidate operation cannot mint its own source occurrence binding"
    );
}
