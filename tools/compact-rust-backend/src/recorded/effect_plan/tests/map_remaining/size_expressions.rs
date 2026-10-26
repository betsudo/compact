// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Expression ingress shares the accepted direct Map metadata authority.
use super::*;

fn size(field: &str, index: u8) -> Expr {
    Expr::MapSize {
        field: field.into(),
        index,
    }
}
fn expression(value: Type) -> Contract {
    let mut c = contract(value, 2, true);
    c.stateful_circuits[0].return_value = StateReturn::Expression {
        value: size("table", 0),
    };
    c
}

#[test]
fn map_size_expression_reuses_direct_metadata_for_scalar_and_whole_adt() {
    for value in [
        Type::Field,
        Type::LedgerMap {
            key: Box::new(Type::Boolean),
            value: Box::new(Type::Field),
        },
    ] {
        let direct = checked(&contract(value.clone(), 2, true)).unwrap();
        let c = expression(value);
        let nested = checked(&c).unwrap();
        assert_eq!(nested.plan().body.bindings.len(), 1);
        assert!(matches!(
            nested.plan().body.bindings[0].operation,
            Operation::MapSize { .. }
        ));
        assert_eq!(
            nested.plan().body.result.ty,
            validate::uint("18446744073709551615")
        );
        let direct = render(&direct).unwrap().steps;
        let nested = render(&nested).unwrap().steps;
        assert_eq!(
            quote::quote!(#(#direct)*).to_string(),
            quote::quote!(#(#nested)*).to_string()
        );
        crate::render_with_capabilities(&c).unwrap();
    }
}

#[test]
fn map_size_expression_preserves_queries_around_mutation_and_static_branch_checks() {
    let mut c = expression(Type::Field);
    c.stateful_circuits[0].actions = vec![
        StateAction::Expression {
            value: size("table", 0),
        },
        StateAction::MapInsert {
            field: "table".into(),
            index: 0,
            key: key(),
            value: Expr::FieldLiteral { value: "1".into() },
        },
    ];
    let plan = checked(&c).unwrap();
    assert_eq!(plan.plan().body.bindings.len(), 4);
    assert!(matches!(
        plan.plan().body.bindings[0].operation,
        Operation::MapSize { .. }
    ));
    assert!(matches!(
        plan.plan().body.bindings[2].operation,
        Operation::MapInsert { .. }
    ));
    assert!(matches!(
        plan.plan().body.bindings[3].operation,
        Operation::MapSize { .. }
    ));
    c.stateful_circuits[0].actions.clear();
    c.stateful_circuits[0].return_value = StateReturn::Expression {
        value: Expr::If {
            condition: Box::new(Expr::Boolean { value: false }),
            then: Box::new(size("missing", 0)),
            otherwise: Box::new(Expr::UnsignedLiteral {
                value: "0".into(),
                max: u64::MAX.to_string(),
            }),
        },
    };
    assert!(
        checked(&c).is_err(),
        "inactive malformed query must be checked"
    );
}

#[test]
fn map_size_expression_refuses_source_and_independent_plan_forgeries() {
    let c = expression(Type::Field);
    let plan = checked(&c).unwrap();
    for wrong_type in [false, true] {
        let mut forged = plan.plan().clone();
        if wrong_type {
            forged.body.bindings[0].value.ty = Type::Boolean;
            forged.body.result.ty = Type::Boolean;
            forged.result_type = Type::Boolean;
        } else {
            let Operation::MapSize { slot } = &mut forged.body.bindings[0].operation else {
                unreachable!()
            };
            slot.index = 1;
        }
        assert!(validate::validate(&forged, plan.declarations()).is_err());
    }
    for (name, index) in [("missing", 0), ("table", 1)] {
        let mut bad = c.clone();
        bad.stateful_circuits[0].return_value = StateReturn::Expression {
            value: size(name, index),
        };
        assert!(checked(&bad).is_err());
        assert!(crate::render(&bad).is_err());
    }
    let mut wrong_kind = c.clone();
    wrong_kind.ledger_fields[0].declaration = LedgerFieldKind::Counter;
    assert!(checked(&wrong_kind).is_err());
    assert!(crate::render(&wrong_kind).is_err());
}

#[test]
fn map_size_expression_remains_effectful_in_pure_source() {
    let mut c = expression(Type::Field);
    c.circuits.push(PureCircuit {
        name: "pure_size".into(),
        source: None,
        internal: false,
        parameters: vec![],
        result: validate::uint("18446744073709551615"),
        body: size("table", 0),
    });
    assert_eq!(
        crate::render(&c).unwrap_err(),
        crate::RenderError::EffectfulExpression
    );
}
