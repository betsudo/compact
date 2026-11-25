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

//! Kernel invariants verified independently of source lowering.
use super::*;
fn bytes_type() -> Type {
    Type::Bytes { length: 32 }
}
fn amount_type() -> Type {
    Type::Unsigned {
        max: u64::MAX.to_string(),
    }
}

#[test]
fn balance_queries_record_typed_token_and_refuse_wrong_domain() {
    let mut token_type = crate::coin_shapes::token_type();
    if let Type::Struct { name, .. } = &mut token_type {
        *name = "EitherCompact3".into();
    }
    let token = value(0, token_type);
    let amount = value(
        1,
        Type::Unsigned {
            max: u128::MAX.to_string(),
        },
    );
    let source = StatefulCircuit {
        source: None,
        name: "balance".into(),
        internal: false,
        parameters: vec![
            Parameter {
                name: "token".into(),
                ty: token.ty.clone(),
            },
            Parameter {
                name: "amount".into(),
                ty: amount.ty.clone(),
            },
        ],
        actions: vec![],
        result: amount.ty.clone(),
        return_value: StateReturn::Expression {
            value: Expr::KernelBalance {
                token: Box::new(Expr::Parameter {
                    name: "token".into(),
                }),
            },
        },
    };
    let validate_balance = |plan| {
        let types = type_environment::TypeEnvironment::from_sources(
            &source,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        validate::check(
            plan,
            declarations::Declarations::default().with_type_environment(types),
        )
    };
    for (operation, ty, method) in [
        (
            Operation::KernelBalance {
                token: token.clone(),
            },
            amount.ty.clone(),
            "kernel_balance",
        ),
        (
            Operation::KernelBalanceLessThan {
                token: token.clone(),
                amount: amount.clone(),
            },
            Type::Boolean,
            "kernel_balance_less_than",
        ),
        (
            Operation::KernelBalanceGreaterThan {
                token: token.clone(),
                amount: amount.clone(),
            },
            Type::Boolean,
            "kernel_balance_greater_than",
        ),
    ] {
        let original = Plan {
            parameters: vec![token.clone(), amount.clone()],
            result_type: ty.clone(),
            body: Block {
                bindings: vec![binding(2, ty.clone(), operation)],
                result: value(2, ty),
            },
        };
        let checked = validate_balance(original.clone()).unwrap();
        let rendered = render(&checked).unwrap();
        let steps = rendered.steps;
        let source = quote::quote!(#(#steps)*).to_string();
        assert!(source.contains(method), "{method}");
        let mut bad = original;
        bad.parameters[0].ty = bytes_type();
        assert!(validate_balance(bad).is_err(), "{method} token mismatch");
    }
}
fn value(id: usize, ty: Type) -> Value {
    Value { id, ty }
}
fn binding(id: usize, ty: Type, operation: Operation) -> Binding {
    Binding {
        value: value(id, ty),
        operation,
        path: format!("kernel[{id}]"),
    }
}
fn plan(operation: Operation) -> Plan {
    Plan {
        parameters: vec![value(0, bytes_type()), value(1, amount_type())],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![binding(2, Type::Unit, operation)],
            result: value(2, Type::Unit),
        },
    }
}
fn validate(plan: Plan) -> Result<CheckedPlan, RecordingGap> {
    validate::check(plan, declarations::Declarations::default())
}

#[test]
fn mint_event_requires_exact_bytes81_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitShieldedMint(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 81 })).unwrap();
    for ty in [
        Type::Bytes { length: 80 },
        Type::Bytes { length: 82 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn receive_event_requires_exact_bytes578_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitShieldedReceive(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 578 })).unwrap();
    for ty in [
        Type::Bytes { length: 577 },
        Type::Bytes { length: 579 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn unshielded_spend_event_requires_exact_bytes145_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitUnshieldedSpend(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 145 })).unwrap();
    for ty in [
        Type::Bytes { length: 144 },
        Type::Bytes { length: 146 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn pause_events_require_zero_byte_payloads_in_the_checked_plan() {
    for event in [
        Operation::EmitPaused as fn(Value) -> Operation,
        Operation::EmitUnpaused,
    ] {
        let event_plan = |ty| {
            let input = value(0, ty);
            let mut plan = plan(event(input.clone()));
            plan.parameters[0] = input;
            plan
        };
        validate(event_plan(Type::Bytes { length: 0 })).unwrap();
        assert!(validate(event_plan(Type::Bytes { length: 1 })).is_err());
        assert!(validate(event_plan(Type::Field)).is_err());
    }
}

#[test]
fn misc_event_requires_exact_bytes288_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitMisc(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 288 })).unwrap();
    for ty in [
        Type::Bytes { length: 287 },
        Type::Bytes { length: 289 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn unshielded_burn_event_requires_exact_bytes113_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitUnshieldedBurn(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 113 })).unwrap();
    for ty in [
        Type::Bytes { length: 112 },
        Type::Bytes { length: 114 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn unshielded_receive_event_requires_exact_bytes145_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitUnshieldedReceive(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 145 })).unwrap();
    for ty in [
        Type::Bytes { length: 144 },
        Type::Bytes { length: 146 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn unshielded_mint_event_requires_exact_bytes80_in_the_checked_plan() {
    let event_plan = |ty: Type| {
        let input = value(0, ty);
        let mut plan = plan(Operation::EmitUnshieldedMint(input.clone()));
        plan.parameters[0] = input;
        plan
    };
    validate(event_plan(Type::Bytes { length: 80 })).unwrap();
    for ty in [
        Type::Bytes { length: 79 },
        Type::Bytes { length: 81 },
        Type::Field,
    ] {
        assert!(validate(event_plan(ty)).is_err());
    }
}

#[test]
fn all_closed_claim_kinds_require_bytes32_live_input_and_unit_output() {
    for kind in [
        KernelClaimKind::Nullifier,
        KernelClaimKind::CoinSpend,
        KernelClaimKind::CoinReceive,
    ] {
        let original = plan(Operation::KernelClaim {
            kind,
            value: value(0, bytes_type()),
        });
        validate(original.clone()).unwrap();
        for mode in 0..4 {
            let mut bad = original.clone();
            match mode {
                0 => {
                    if let Operation::KernelClaim { value, .. } =
                        &mut bad.body.bindings[0].operation
                    {
                        *value = Value {
                            id: 1,
                            ty: amount_type(),
                        };
                    }
                }
                1 => {
                    if let Operation::KernelClaim { value, .. } =
                        &mut bad.body.bindings[0].operation
                    {
                        value.id = 99;
                    }
                }
                2 => {
                    if let Operation::KernelClaim { value, .. } =
                        &mut bad.body.bindings[0].operation
                    {
                        value.ty = Type::Bytes { length: 31 };
                    }
                }
                3 => {
                    bad.body.bindings[0].value.ty = Type::Boolean;
                    bad.body.result.ty = Type::Boolean;
                    bad.result_type = Type::Boolean;
                }
                _ => unreachable!(),
            }
            assert!(validate(bad).is_err(), "claim mutation {mode}");
        }
    }
}
#[test]
fn mint_requires_exact_uint64_domain_order_and_unit_result() {
    let original = plan(Operation::KernelMintShielded {
        domain: value(0, bytes_type()),
        amount: value(1, amount_type()),
    });
    validate(original.clone()).unwrap();
    for mode in 0..7 {
        let mut bad = original.clone();
        match mode {
            0 | 1 => {
                let ty = Type::Unsigned {
                    max: if mode == 0 {
                        "255".into()
                    } else {
                        (u64::MAX as u128 + 1).to_string()
                    },
                };
                bad.parameters[1].ty = ty.clone();
                if let Operation::KernelMintShielded { amount, .. } =
                    &mut bad.body.bindings[0].operation
                {
                    amount.ty = ty;
                }
            }
            2 => {
                if let Operation::KernelMintShielded { domain, amount } =
                    &mut bad.body.bindings[0].operation
                {
                    std::mem::swap(domain, amount);
                }
            }
            3 => {
                if let Operation::KernelMintShielded { amount, .. } =
                    &mut bad.body.bindings[0].operation
                {
                    amount.id = 99;
                }
            }
            4 => {
                if let Operation::KernelMintShielded { amount, .. } =
                    &mut bad.body.bindings[0].operation
                {
                    amount.ty = Type::Field;
                }
            }
            5 => {
                bad.body.bindings[0].value.ty = Type::Boolean;
                bad.body.result.ty = Type::Boolean;
                bad.result_type = Type::Boolean;
            }
            6 => {
                bad.parameters.remove(0);
            }
            _ => unreachable!(),
        }
        assert!(validate(bad).is_err(), "mint mutation {mode}");
    }
}
#[test]
fn kernel_cannot_use_a_value_from_a_sibling_branch_scope() {
    let unit = |id| binding(id, Type::Unit, Operation::Literal(Literal::Unit));
    let bad = Plan {
        parameters: vec![value(0, Type::Boolean)],
        result_type: Type::Unit,
        body: Block {
            bindings: vec![
                binding(
                    1,
                    Type::Unit,
                    Operation::If {
                        condition: value(0, Type::Boolean),
                        then: Block {
                            bindings: vec![
                                binding(
                                    2,
                                    bytes_type(),
                                    Operation::Literal(Literal::Bytes(vec![0; 32])),
                                ),
                                unit(3),
                            ],
                            result: value(3, Type::Unit),
                        },
                        otherwise: Block {
                            bindings: vec![unit(4)],
                            result: value(4, Type::Unit),
                        },
                    },
                ),
                binding(
                    5,
                    Type::Unit,
                    Operation::KernelClaim {
                        kind: KernelClaimKind::Nullifier,
                        value: value(2, bytes_type()),
                    },
                ),
            ],
            result: value(5, Type::Unit),
        },
    };
    assert_eq!(validate(bad).unwrap_err().path, "kernel[5]");
}
fn root(value: Expr) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: Type::Unit,
        return_value: StateReturn::Expression { value },
    }
}
#[test]
fn mint_witness_operands_evaluate_once_in_domain_then_amount_order() {
    let witnesses = [
        WitnessDeclaration {
            source: None,
            name: "domain".into(),
            parameters: vec![],
            result: bytes_type(),
        },
        WitnessDeclaration {
            source: None,
            name: "amount".into(),
            parameters: vec![],
            result: amount_type(),
        },
    ];
    let maps = witnesses.iter().map(|w| (w.name.as_str(), w)).collect();
    let witness = |name: &str| Expr::WitnessCall {
        name: name.into(),
        arguments: vec![],
    };
    let checked = check(
        &root(Expr::KernelMintShielded {
            domain: Box::new(witness("domain")),
            amount: Box::new(witness("amount")),
        }),
        &HashMap::new(),
        &maps,
    )
    .unwrap();
    let bindings = &checked.plan().body.bindings;
    assert_eq!(bindings.len(), 3);
    assert!(matches!(bindings[0].operation, Operation::Witness { .. }));
    assert!(matches!(bindings[1].operation, Operation::Witness { .. }));
    assert_eq!(bindings[0].path, "return_value.value.domain");
    assert_eq!(bindings[1].path, "return_value.value.amount");
    let Operation::KernelMintShielded { domain, amount } = &bindings[2].operation else {
        panic!("mint follows both witnesses")
    };
    assert_eq!(domain, &bindings[0].value);
    assert_eq!(amount, &bindings[1].value);
    let rendered = render(&checked).unwrap();
    let steps = rendered.steps;
    let source = quote::quote!(#(#steps)*).to_string();
    assert_eq!(source.matches("try_witness_metered").count(), 2);
    assert_eq!(source.matches("kernel_mint_shielded").count(), 1);
    let error = check(
        &root(Expr::KernelMintShielded {
            domain: Box::new(witness("missing_domain")),
            amount: Box::new(witness("missing_amount")),
        }),
        &HashMap::new(),
        &maps,
    )
    .unwrap_err();
    let PlanError::Unsupported(gap) = error else {
        panic!("source refusal")
    };
    assert_eq!(gap.path, "return_value.value.domain");
}

#[test]
fn unshielded_kernel_effects_keep_exact_types_and_source_operand_order() {
    use crate::ir::KernelUnshieldedEffectKind as Kind;
    for (kind, key_ty, amount_ty, method) in [
        (
            Kind::Mint,
            bytes_type(),
            amount_type(),
            "kernel_mint_unshielded",
        ),
        (
            Kind::Inputs,
            super::super::kernel_shapes::token_type(),
            Type::Unsigned {
                max: u128::MAX.to_string(),
            },
            "kernel_inc_unshielded_inputs",
        ),
        (
            Kind::Outputs,
            super::super::kernel_shapes::token_type(),
            Type::Unsigned {
                max: u128::MAX.to_string(),
            },
            "kernel_inc_unshielded_outputs",
        ),
        (
            Kind::Claim,
            super::super::kernel_shapes::token_type(),
            Type::Unsigned {
                max: u128::MAX.to_string(),
            },
            "kernel_claim_unshielded_coin_spend",
        ),
    ] {
        let claim = kind == Kind::Claim;
        let mut witnesses = vec![
            WitnessDeclaration {
                source: None,
                name: "key".into(),
                parameters: vec![],
                result: key_ty.clone(),
            },
            WitnessDeclaration {
                source: None,
                name: "amount".into(),
                parameters: vec![],
                result: amount_ty.clone(),
            },
        ];
        if claim {
            // The compiler gives distinct generic instantiations distinct Rust
            // names when both occur in one contract.
            let mut recipient_type = super::super::kernel_shapes::recipient_type();
            let Type::Struct { name, .. } = &mut recipient_type else {
                unreachable!()
            };
            *name = "EitherCompact1".into();
            witnesses.push(WitnessDeclaration {
                source: None,
                name: "recipient".into(),
                parameters: vec![],
                result: recipient_type,
            });
        }
        let maps = witnesses.iter().map(|w| (w.name.as_str(), w)).collect();
        let witness = |name: &str| {
            Box::new(Expr::WitnessCall {
                name: name.into(),
                arguments: vec![],
            })
        };
        let checked = check(
            &root(Expr::KernelUnshieldedEffect {
                effect: kind,
                key: witness("key"),
                recipient: claim.then(|| witness("recipient")),
                amount: witness("amount"),
            }),
            &HashMap::new(),
            &maps,
        )
        .unwrap();
        let bindings = &checked.plan().body.bindings;
        let expected = if claim {
            vec!["key", "recipient", "amount"]
        } else {
            vec!["key", "amount"]
        };
        assert_eq!(bindings.len(), expected.len() + 1, "{kind:?}");
        for (binding, name) in bindings.iter().zip(&expected) {
            assert!(matches!(binding.operation, Operation::Witness { .. }));
            assert_eq!(binding.path, format!("return_value.value.{name}"));
        }
        let Operation::KernelUnshieldedEffect {
            kind: observed,
            key,
            recipient,
            amount,
        } = &bindings.last().unwrap().operation
        else {
            panic!("missing unshielded effect")
        };
        assert_eq!(*observed, kind);
        assert_eq!(key, &bindings[0].value);
        assert_eq!(amount, &bindings[expected.len() - 1].value);
        assert_eq!(recipient.as_ref(), claim.then_some(&bindings[1].value));
        let rendered = render(&checked).unwrap();
        let steps = rendered.steps;
        let source = quote::quote!(#(#steps)*).to_string();
        assert_eq!(
            source.matches("try_witness_metered").count(),
            expected.len()
        );
        assert_eq!(source.matches(method).count(), 1, "{kind:?}");
        assert_eq!(
            source.matches("token_type_from_compact").count(),
            usize::from(!matches!(kind, Kind::Mint))
        );
    }
}

#[test]
fn unshielded_claim_cardinality_and_operand_types_refuse_before_render() {
    use crate::ir::KernelUnshieldedEffectKind as Kind;
    let witnesses = [
        WitnessDeclaration {
            source: None,
            name: "key".into(),
            parameters: vec![],
            result: super::super::kernel_shapes::token_type(),
        },
        WitnessDeclaration {
            source: None,
            name: "amount".into(),
            parameters: vec![],
            result: Type::Unsigned {
                max: u128::MAX.to_string(),
            },
        },
        WitnessDeclaration {
            source: None,
            name: "wrong_recipient".into(),
            parameters: vec![],
            result: bytes_type(),
        },
    ];
    let maps = witnesses.iter().map(|w| (w.name.as_str(), w)).collect();
    let witness = |name: &str| {
        Box::new(Expr::WitnessCall {
            name: name.into(),
            arguments: vec![],
        })
    };
    for recipient in [None, Some(witness("wrong_recipient"))] {
        assert!(
            check(
                &root(Expr::KernelUnshieldedEffect {
                    effect: Kind::Claim,
                    key: witness("key"),
                    recipient,
                    amount: witness("amount"),
                }),
                &HashMap::new(),
                &maps
            )
            .is_err()
        );
    }
    assert!(
        check(
            &root(Expr::KernelUnshieldedEffect {
                effect: Kind::Mint,
                key: witness("key"),
                recipient: None,
                amount: witness("amount"),
            }),
            &HashMap::new(),
            &maps
        )
        .is_err()
    );
    assert!(
        check(
            &root(Expr::KernelUnshieldedEffect {
                effect: Kind::Inputs,
                key: witness("key"),
                recipient: Some(witness("wrong_recipient")),
                amount: witness("amount"),
            }),
            &HashMap::new(),
            &maps
        )
        .is_err()
    );
}

#[test]
fn block_time_queries_record_uint64_bound_and_refuse_wrong_domain() {
    let bound = value(0, amount_type());
    let source = StatefulCircuit {
        source: None,
        name: "blockTime".into(),
        internal: false,
        parameters: vec![Parameter {
            name: "time".into(),
            ty: bound.ty.clone(),
        }],
        actions: vec![],
        result: Type::Boolean,
        return_value: StateReturn::Expression {
            value: Expr::KernelBlockTimeLessThan {
                bound: Box::new(Expr::Parameter {
                    name: "time".into(),
                }),
            },
        },
    };
    let validate_time = |plan| {
        let types = type_environment::TypeEnvironment::from_sources(
            &source,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap();
        validate::check(
            plan,
            declarations::Declarations::default().with_type_environment(types),
        )
    };
    for (operation, method) in [
        (
            Operation::KernelBlockTimeLessThan {
                bound: bound.clone(),
            },
            "kernel_block_time_less_than",
        ),
        (
            Operation::KernelBlockTimeGreaterThan {
                bound: bound.clone(),
            },
            "kernel_block_time_greater_than",
        ),
    ] {
        let original = Plan {
            parameters: vec![bound.clone()],
            result_type: Type::Boolean,
            body: Block {
                bindings: vec![binding(1, Type::Boolean, operation)],
                result: value(1, Type::Boolean),
            },
        };
        let checked = validate_time(original.clone()).unwrap();
        let rendered = render(&checked).unwrap();
        let steps = rendered.steps;
        assert!(quote::quote!(#(#steps)*).to_string().contains(method));
        let mut bad = original;
        bad.parameters[0].ty = Type::Unsigned { max: "255".into() };
        assert!(validate_time(bad).is_err());
    }
}
