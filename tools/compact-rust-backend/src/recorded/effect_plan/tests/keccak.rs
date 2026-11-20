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

//! Keccak's FAB conversion domain is independent of persistent byte hashing.
use super::super::validate::uint;
use super::*;
use crate::ir::KeccakEncoding;

fn digest_type() -> Type {
    Type::Bytes { length: 32 }
}
fn hash(value: Expr) -> Expr {
    Expr::Keccak256 {
        encoding: Default::default(),
        value: Box::new(value),
    }
}
fn parameter() -> Expr {
    Expr::Parameter {
        name: "input".into(),
    }
}
fn root(ty: Type) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: "digest".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "input".into(),
            ty,
        }],
        actions: vec![],
        result: digest_type(),
        return_value: StateReturn::Expression {
            value: hash(parameter()),
        },
    }
}
fn checked(circuit: &StatefulCircuit) -> Result<CheckedPlan, PlanError> {
    check(circuit, &HashMap::new(), &HashMap::new())
}
fn leaves() -> Vec<Type> {
    vec![
        Type::Unit,
        Type::Boolean,
        Type::Field,
        Type::JubjubPoint,
        Type::Secp256k1Point,
        Type::OpaqueString,
        Type::OpaqueBytes,
        Type::Bytes { length: 0 },
        Type::Bytes { length: 32 },
        uint("255"),
        uint(&u128::MAX.to_string()),
        uint("340282366920938463463374607431768211456"),
        Type::Enum {
            name: "Choice".into(),
            variants: vec!["A".into(), "B".into()],
        },
    ]
}

#[test]
fn keccak_accepts_canonical_fab_leaves_and_owned_products() {
    for ty in leaves() {
        for input in [
            ty.clone(),
            Type::Vector {
                element: Box::new(ty.clone()),
                length: 2,
            },
            Type::Struct {
                name: "Payload".into(),
                fields: vec![StructField {
                    name: "value".into(),
                    ty,
                }],
            },
        ] {
            assert!(
                validate::keccak_input_type(&input, Default::default()),
                "{input:?}"
            );
            let plan = checked(&root(input)).unwrap();
            assert!(matches!(
                plan.plan().body.bindings.last().unwrap().operation,
                Operation::Keccak256(..)
            ));
            let steps = render(&plan).unwrap().steps;
            assert_eq!(
                quote::quote!(#(#steps)*)
                    .to_string()
                    .matches("keccak256")
                    .count(),
                1
            );
        }
    }
}

#[test]
fn aligned_keccak_keeps_its_encoding_and_refuses_nested_opaque_input() {
    let aligned = |ty| {
        let mut circuit = root(ty);
        let StateReturn::Expression {
            value: Expr::Keccak256 { encoding, .. },
        } = &mut circuit.return_value
        else {
            panic!("expected a Keccak return");
        };
        *encoding = KeccakEncoding::Aligned;
        circuit
    };
    for ty in [Type::Bytes { length: 12 }, Type::Secp256k1Point] {
        let plan = checked(&aligned(ty)).unwrap();
        assert!(matches!(
            plan.plan().body.bindings.last().unwrap().operation,
            Operation::Keccak256(_, KeccakEncoding::Aligned)
        ));
        let steps = render(&plan).unwrap().steps;
        assert!(
            quote::quote!(#(#steps)*)
                .to_string()
                .contains("runtime :: keccak256_aligned")
        );
    }

    let opaque_vector = Type::Vector {
        length: 2,
        element: Box::new(Type::OpaqueString),
    };
    assert!(checked(&root(opaque_vector.clone())).is_ok());
    assert!(checked(&aligned(opaque_vector)).is_err());
}

#[test]
fn keccak_tuple_and_named_representation_limits_are_distinct() {
    for n in [0, 1, 8, 11] {
        checked(&root(Type::Tuple {
            elements: vec![Type::Field; n],
        }))
        .unwrap();
    }
    let large = Type::Tuple {
        elements: vec![Type::Field; 12],
    };
    assert!(validate::value_type(&large));
    assert!(!validate::keccak_input_type(&large, Default::default()));
    assert!(checked(&root(large)).is_err());
    for n in [8, 9] {
        let named = Type::Struct {
            name: "Product".into(),
            fields: vec![StructField {
                name: "tuple".into(),
                ty: Type::Tuple {
                    elements: vec![Type::Field; n],
                },
            }],
        };
        assert_eq!(
            validate::keccak_input_type(&named, Default::default()),
            n == 8
        );
        assert_eq!(checked(&root(named)).is_ok(), n == 8);
    }
    let ledger = Type::LedgerMap {
        key: Box::new(Type::Field),
        value: Box::new(Type::Field),
    };
    assert!(!validate::keccak_input_type(&ledger, Default::default()));
    assert!(!validate::keccak_input_type(
        &uint("01"),
        KeccakEncoding::NormalizedAtoms
    ));
    // Aligned hashes retain their independently reviewed fixed-input boundary.
    for ty in [Type::OpaqueBytes, Type::OpaqueString] {
        assert!(validate::keccak_input_type(&ty, Default::default()));
        assert!(!validate::fixed_aligned_fab_type(&ty));
    }
}

fn validation_plan() -> Plan {
    let input = Value {
        id: 0,
        ty: Type::Field,
    };
    let output = Value {
        id: 1,
        ty: digest_type(),
    };
    Plan {
        parameters: vec![input.clone()],
        result_type: digest_type(),
        body: Block {
            bindings: vec![Binding {
                value: output.clone(),
                operation: Operation::Keccak256(input, Default::default()),
                path: "hash".into(),
            }],
            result: output,
        },
    }
}

#[test]
fn keccak_validator_checks_exact_operand_authority_and_result() {
    validate::check(validation_plan(), declarations::Declarations::default()).unwrap();
    for mode in 0..5 {
        let mut plan = validation_plan();
        match mode {
            0 => {
                if let Operation::Keccak256(input, _) = &mut plan.body.bindings[0].operation {
                    input.id = 99;
                }
            }
            1 => {
                if let Operation::Keccak256(input, _) = &mut plan.body.bindings[0].operation {
                    input.id = 1;
                }
            }
            2 => {
                if let Operation::Keccak256(input, _) = &mut plan.body.bindings[0].operation {
                    input.ty = Type::Boolean;
                }
            }
            3 => {
                plan.body.bindings[0].value.ty = Type::Field;
                plan.body.result.ty = Type::Field;
                plan.result_type = Type::Field;
            }
            4 => {
                let unsupported = Type::Tuple {
                    elements: vec![Type::Field; 12],
                };
                plan.parameters[0].ty = unsupported.clone();
                if let Operation::Keccak256(input, _) = &mut plan.body.bindings[0].operation {
                    input.ty = unsupported;
                }
            }
            _ => unreachable!(),
        }
        assert!(
            validate::check(plan, declarations::Declarations::default()).is_err(),
            "forgery {mode}"
        );
    }
}

#[test]
fn keccak_preserves_lazy_branch_and_evaluates_effectful_operand_once() {
    let witness = WitnessDeclaration {
        source: None,
        name: "next".into(),
        parameters: vec![],
        result: Type::Field,
    };
    let mut circuit = root(Type::Boolean);
    circuit.return_value = StateReturn::Expression {
        value: Expr::If {
            condition: Box::new(parameter()),
            then: Box::new(hash(Expr::WitnessCall {
                name: "next".into(),
                arguments: vec![],
            })),
            otherwise: Box::new(Expr::BytesLiteral { bytes: vec![0; 32] }),
        },
    };
    let witnesses = HashMap::from([("next", &witness)]);
    let checked = check(&circuit, &HashMap::new(), &witnesses).unwrap();
    let steps = render(&checked).unwrap().steps;
    let tokens = quote::quote!(#(#steps)*).to_string();
    assert_eq!(tokens.matches("keccak256").count(), 1);
    assert_eq!(tokens.matches("try_witness_metered").count(), 1);
    assert!(tokens.find("if").unwrap() < tokens.find("keccak256").unwrap());
    assert!(tokens.find("try_witness_metered").unwrap() < tokens.find("keccak256").unwrap());
}

#[test]
fn keccak_refuses_first_bad_operand_before_later_product_members() {
    let mut circuit = root(Type::Field);
    circuit.return_value = StateReturn::Expression {
        value: hash(Expr::Tuple {
            elements: vec![
                Expr::Parameter {
                    name: "missing".into(),
                },
                Expr::FieldCast {
                    value: Box::new(Expr::Boolean { value: true }),
                },
            ],
        }),
    };
    let PlanError::Unsupported(gap) = checked(&circuit).unwrap_err() else {
        panic!("expected source refusal")
    };
    assert_eq!(gap.path, "return_value.value.value.elements[0]");
}

#[test]
fn keccak_cannot_replace_named_source_definitions_with_consistent_annotations() {
    for named in [
        Type::Struct {
            name: "Payload".into(),
            fields: vec![StructField {
                name: "value".into(),
                ty: Type::Field,
            }],
        },
        Type::Enum {
            name: "State".into(),
            variants: vec!["Ready".into(), "Done".into()],
        },
    ] {
        let source = root(named);
        let environment = super::super::type_environment::TypeEnvironment::from_sources(
            &source,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        let mut plan = checked(&source).unwrap().plan().clone();
        match &mut plan.parameters[0].ty {
            Type::Struct { fields, .. } => fields[0].name = "forged".into(),
            Type::Enum { variants, .. } => variants[0] = "Forged".into(),
            _ => unreachable!(),
        }
        let forged = plan.parameters[0].ty.clone();
        let Operation::Keccak256(input, _) = &mut plan.body.bindings[0].operation else {
            unreachable!()
        };
        input.ty = forged;
        assert!(
            validate::check(
                plan,
                declarations::Declarations::default().with_type_environment(environment)
            )
            .is_err()
        );
    }
}

#[test]
fn keccak_cannot_use_a_value_owned_by_only_one_branch() {
    let condition = Value {
        id: 0,
        ty: Type::Boolean,
    };
    let output = Value {
        id: 4,
        ty: digest_type(),
    };
    let branch = Binding {
        value: field(3),
        path: "branch".into(),
        operation: Operation::If {
            condition: condition.clone(),
            then: Block {
                bindings: vec![literal(1)],
                result: field(1),
            },
            otherwise: Block {
                bindings: vec![literal(2)],
                result: field(2),
            },
        },
    };
    let mut plan = Plan {
        parameters: vec![condition],
        result_type: digest_type(),
        body: Block {
            bindings: vec![
                branch,
                Binding {
                    value: output.clone(),
                    operation: Operation::Keccak256(field(3), Default::default()),
                    path: "hash".into(),
                },
            ],
            result: output,
        },
    };
    validate::check(plan.clone(), declarations::Declarations::default()).unwrap();
    plan.body.bindings[1].operation = Operation::Keccak256(field(1), Default::default());
    assert!(validate::check(plan, declarations::Declarations::default()).is_err());
}

#[path = "keccak/export.rs"]
mod export;
#[path = "keccak/source.rs"]
mod source;
