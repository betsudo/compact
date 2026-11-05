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

//! Zero's singleton type admits a wide product without narrowing its operand.
use super::*;
const WIDE: &str = "1606938044258990275541962092341162602522202993782792835301375";
fn reference(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn product(left: Expr, right: Expr) -> Expr {
    Expr::UnsignedMultiply {
        max: "0".into(),
        left: Box::new(left),
        right: Box::new(right),
    }
}
fn root(left: &str, right: &str) -> StatefulCircuit {
    StatefulCircuit {
        source: None,
        internal: false,
        name: "zero".into(),
        parameters: vec![
            Parameter {
                name: "left".into(),
                ty: validate::uint(left),
            },
            Parameter {
                name: "right".into(),
                ty: validate::uint(right),
            },
        ],
        actions: vec![StateAction::Expression {
            value: product(reference("left"), reference("right")),
        }],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    }
}
fn authority(source: &StatefulCircuit) -> declarations::Declarations {
    let types = type_environment::TypeEnvironment::from_sources(
        source,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    )
    .unwrap();
    declarations::Declarations::default()
        .with_type_environment(types)
        .with_root(source)
}
fn multiplication(plan: &mut Plan) -> &mut Binding {
    plan.body
        .bindings
        .iter_mut()
        .find(|b| matches!(b.operation, Operation::UnsignedMultiply(..)))
        .unwrap()
}
#[test]
fn wide_zero_plan_accepts_both_orders_and_preserves_zero_result_type() {
    for (left, right) in [(WIDE, "0"), ("0", WIDE)] {
        let source = root(left, right);
        let plan = check(&source, &HashMap::new(), &HashMap::new()).unwrap();
        validate::check(plan.plan().clone(), authority(&source)).unwrap();
        assert_eq!(
            multiplication(&mut plan.plan().clone()).value.ty,
            validate::uint("0")
        );
        let steps = render(&plan).unwrap().steps;
        let syntax = quote::quote!({ #(#steps)* }).to_string();
        assert!(!syntax.contains("narrow_wide_uint"));
        assert!(
            syntax.contains("BoundedUint :: < 0 > :: new (0)"),
            "{syntax}"
        );
    }
}
#[test]
fn wide_zero_validator_rejects_forged_bounds_and_ssa_references() {
    for (left_max, right_max) in [(WIDE, "0"), ("0", WIDE)] {
        let source = root(left_max, right_max);
        let original = check(&source, &HashMap::new(), &HashMap::new())
            .unwrap()
            .plan()
            .clone();
        for side in [0, 1] {
            for mode in ["missing", "annotation", "self", "malformed"] {
                let mut forged = original.clone();
                let binding = multiplication(&mut forged);
                let Operation::UnsignedMultiply(left, right) = &mut binding.operation else {
                    unreachable!()
                };
                let value = if side == 0 { left } else { right };
                match mode {
                    "missing" => value.id = 999,
                    "annotation" => value.ty = validate::uint("1"),
                    "self" => value.id = binding.value.id,
                    "malformed" => value.ty = validate::uint("00"),
                    _ => unreachable!(),
                }
                assert_eq!(
                    validate::check(forged, authority(&source))
                        .unwrap_err()
                        .path,
                    "actions[0].value"
                );
            }
        }
        for result in [WIDE, "00"] {
            let mut forged = original.clone();
            multiplication(&mut forged).value.ty = validate::uint(result);
            assert_eq!(
                validate::check(forged, authority(&source))
                    .unwrap_err()
                    .path,
                "actions[0].value"
            );
        }
        // Change the declared zero parameter and its use coherently: the
        // operation law must refuse, even though SSA use-site types agree.
        let zero_side = usize::from(left_max != "0");
        let mut source = source.clone();
        source.parameters[zero_side].ty = validate::uint("1");
        let mut forged = original;
        forged.parameters[zero_side].ty = validate::uint("1");
        let Operation::UnsignedMultiply(left, right) = &mut multiplication(&mut forged).operation
        else {
            unreachable!()
        };
        (if zero_side == 0 { left } else { right }).ty = validate::uint("1");
        assert_eq!(
            validate::check(forged, authority(&source))
                .unwrap_err()
                .path,
            "actions[0].value"
        );
    }
}
#[test]
fn wide_zero_witnesses_remain_once_left_then_right_in_the_checked_plan() {
    for (left_max, right_max) in [(WIDE, "0"), ("0", WIDE)] {
        let declarations: Vec<_> = [("left", left_max), ("right", right_max)]
            .into_iter()
            .map(|(name, max)| WitnessDeclaration {
                source: None,
                name: name.into(),
                parameters: vec![],
                result: validate::uint(max),
            })
            .collect();
        let env = declarations.iter().map(|d| (d.name.as_str(), d)).collect();
        let witness = |name: &str| Expr::WitnessCall {
            name: name.into(),
            arguments: vec![],
        };
        let mut source = root(left_max, right_max);
        source.parameters.clear();
        source.actions = vec![StateAction::Expression {
            value: product(witness("left"), witness("right")),
        }];
        let plan = check(&source, &HashMap::new(), &env).unwrap();
        let bindings = &plan.plan().body.bindings;
        for (position, declaration) in [(0, WitnessId(0)), (1, WitnessId(1))] {
            assert!(
                matches!(bindings[position].operation, Operation::Witness { declaration: d, .. } if d == declaration)
            );
        }
        let Operation::UnsignedMultiply(left, right) = &bindings[2].operation else {
            panic!("multiplication after both witnesses")
        };
        assert_eq!(left, &bindings[0].value);
        assert_eq!(right, &bindings[1].value);
        let steps = render(&plan).unwrap().steps;
        let text = quote::quote!({ #(#steps)* }).to_string();
        assert_eq!(text.matches("try_witness_metered").count(), 2);
        assert!(
            text.rfind("try_witness_metered").unwrap()
                < text.rfind("BoundedUint :: < 0 > :: new (0)").unwrap()
        );
    }
}

#[test]
fn rounded_small_result_carrier_does_not_require_narrowing_the_wide_operand() {
    for result in ["1", "255", "340282366920938463463374607431768211455"] {
        let mut source = root(WIDE, "0");
        let StateAction::Expression {
            value: Expr::UnsignedMultiply { max, .. },
        } = &mut source.actions[0]
        else {
            unreachable!()
        };
        *max = result.into();
        let plan = check(&source, &HashMap::new(), &HashMap::new()).unwrap();
        validate::check(plan.plan().clone(), authority(&source)).unwrap();
        assert_eq!(
            multiplication(&mut plan.plan().clone()).value.ty,
            validate::uint(result)
        );
        render(&plan).unwrap();
    }
}
