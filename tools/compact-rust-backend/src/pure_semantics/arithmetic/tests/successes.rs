// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Complete arithmetic-family successes through a syntax-free consumer.
use super::*;

#[test]
fn no_syntax_complete_family_visits_both_sources_once_before_output() {
    let left = parameter("left");
    let right = parameter("right");
    for kind in [
        ArithmeticKind::Add,
        ArithmeticKind::Subtract,
        ArithmeticKind::Multiply,
    ] {
        let field = Type::Field;
        let mut children = LeafChildren {
            bindings: HashMap::from([("left", (&field, ())), ("right", (&field, ()))]),
            visits: vec![],
        };
        let mut sink = Consumer::default();
        assert_eq!(
            fold_arithmetic(
                ArithmeticSource::Field {
                    kind,
                    left: &left,
                    right: &right
                },
                &mut children,
                &mut sink
            )
            .unwrap()
            .ty,
            Type::Field
        );
        assert_eq!(
            children.visits,
            [
                std::ptr::from_ref(&left) as usize,
                std::ptr::from_ref(&right) as usize
            ]
        );
        assert!(
            matches!(sink.facts.as_slice(), [ArithmeticFacts::Field(actual)] if *actual == kind)
        );
        let small = uint("15");
        let larger = uint("255");
        let mut children = LeafChildren {
            bindings: HashMap::from([("left", (&small, ())), ("right", (&larger, ()))]),
            visits: vec![],
        };
        let mut sink = Consumer::default();
        assert_eq!(
            fold_arithmetic(
                ArithmeticSource::Unsigned {
                    kind,
                    max: "65535",
                    left: &left,
                    right: &right
                },
                &mut children,
                &mut sink
            )
            .unwrap()
            .ty,
            uint("65535")
        );
        assert_eq!(children.visits.len(), 2);
        assert!(
            matches!(sink.facts.as_slice(), [ArithmeticFacts::Unsigned(UnsignedArithmeticStrategy::Small { kind: actual, left: 15, right: 255, result: 65535 })] if *actual == kind)
        );
    }
    let boolean = Type::Boolean;
    for negated in [false, true] {
        let mut children = LeafChildren {
            bindings: HashMap::from([("left", (&boolean, ())), ("right", (&boolean, ()))]),
            visits: vec![],
        };
        let mut sink = Consumer::default();
        assert_eq!(
            fold_arithmetic(
                ArithmeticSource::Equality {
                    negated,
                    left: &left,
                    right: &right
                },
                &mut children,
                &mut sink
            )
            .unwrap()
            .ty,
            Type::Boolean
        );
        assert!(
            matches!(sink.facts.as_slice(), [ArithmeticFacts::Equality { negated: actual, unit_result: None }] if *actual == negated)
        );
        assert_eq!(children.visits.len(), 2);
    }
    let unsigned = uint("255");
    for operator in [
        ComparisonOperator::Less,
        ComparisonOperator::LessEqual,
        ComparisonOperator::Greater,
        ComparisonOperator::GreaterEqual,
    ] {
        let mut children = LeafChildren {
            bindings: HashMap::from([("left", (&unsigned, ())), ("right", (&unsigned, ()))]),
            visits: vec![],
        };
        let mut sink = Consumer::default();
        assert_eq!(
            fold_arithmetic(
                ArithmeticSource::Ordered {
                    operator,
                    left: &left,
                    right: &right
                },
                &mut children,
                &mut sink
            )
            .unwrap()
            .ty,
            Type::Boolean
        );
        assert!(
            matches!(sink.facts.as_slice(), [ArithmeticFacts::Ordered(actual)] if *actual == operator)
        );
        assert_eq!(children.visits.len(), 2);
    }
}

#[test]
fn unit_equality_preserves_two_child_visits_and_static_type_checks() {
    for negated in [false, true] {
        let left = Expr::Parameter {
            name: "left".into(),
        };
        let right = Expr::Parameter {
            name: "right".into(),
        };
        let unit = Type::Unit;
        for right_ty in [Type::Unit, Type::Boolean] {
            let mut children = LeafChildren {
                bindings: HashMap::from([("left", (&unit, ())), ("right", (&right_ty, ()))]),
                visits: vec![],
            };
            let mut sink = Consumer::default();
            let result = fold_arithmetic(
                ArithmeticSource::Equality {
                    negated,
                    left: &left,
                    right: &right,
                },
                &mut children,
                &mut sink,
            );
            assert_eq!(children.visits.len(), 2);
            if right_ty == Type::Unit {
                assert_eq!(result.unwrap().ty, Type::Boolean);
                assert!(
                    matches!(sink.facts.as_slice(), [ArithmeticFacts::Equality { unit_result: Some(value), .. }] if *value == !negated)
                );
            } else {
                assert!(matches!(
                    result,
                    Err(RenderError::TypeMismatch {
                        expected: Type::Unit,
                        actual: Type::Boolean
                    })
                ));
                assert!(sink.facts.is_empty());
            }
        }
    }
}
