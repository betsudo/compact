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

//! Genuine checked graph fixtures, separate from source-issuance controls.
use super::super::model::CheckedPlan;
use super::*;
use crate::ir::{
    Expr, LedgerField, LedgerFieldKind, Parameter, StateAction, StateReturn, StatefulCircuit,
};
use quote::ToTokens;
use std::collections::HashMap;
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
pub(in crate::recorded::effect_plan) fn fixture(
    kind: usize,
    historic: bool,
) -> Result<CheckedPlan, super::super::DiagnosticPlanError> {
    let field = LedgerField {
        source: None,
        id: "tree".into(),
        index: 0,
        path: vec![1, 2],
        declaration: if historic {
            LedgerFieldKind::HistoricMerkleTree {
                depth: 3,
                ty: Type::Field,
            }
        } else {
            LedgerFieldKind::MerkleTree {
                depth: 3,
                ty: Type::Field,
            }
        },
    };
    let root_type = super::super::validate::merkle::root_type();
    let parameters = vec![
        Parameter {
            name: "value".into(),
            ty: Type::Field,
        },
        Parameter {
            name: "position".into(),
            ty: Type::Unsigned {
                max: u64::MAX.to_string(),
            },
        },
        Parameter {
            name: "hash".into(),
            ty: Type::Bytes { length: 32 },
        },
        Parameter {
            name: "root".into(),
            ty: root_type,
        },
    ];
    let mut actions = vec![];
    if kind < 7 {
        let suffix = [
            "insert",
            "insert_index",
            "insert_index_default",
            "insert_hash",
            "insert_hash_index",
            "reset_to_default",
            "reset_history",
        ][kind];
        let mut action = serde_json::json!({"kind":format!("{}merkle_{suffix}",if historic{"historic_"}else{""}),"field":"tree","index":0});
        if kind <= 1 {
            action["value"] = serde_json::to_value(input("value")).unwrap();
        }
        if matches!(kind, 1 | 2 | 4) {
            action["position"] = serde_json::to_value(input("position")).unwrap();
        }
        if matches!(kind, 3 | 4) {
            action["hash"] = serde_json::to_value(input("hash")).unwrap();
        }
        // Plain history reset has no source IR variant; its checked-invalid control
        // is constructed below by mutating an otherwise checked historic operation.
        actions.push(serde_json::from_value::<StateAction>(action).unwrap());
    }
    let return_value = match (kind, historic) {
        (7, false) => StateReturn::MerkleCheckRoot {
            field: "tree".into(),
            index: 0,
            root: input("root"),
        },
        (7, true) => StateReturn::HistoricMerkleCheckRoot {
            field: "tree".into(),
            index: 0,
            root: input("root"),
        },
        (8, false) => StateReturn::MerkleIsFull {
            field: "tree".into(),
            index: 0,
        },
        (8, true) => StateReturn::HistoricMerkleIsFull {
            field: "tree".into(),
            index: 0,
        },
        _ => StateReturn::Unit,
    };
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters,
        actions,
        result: if kind >= 7 { Type::Boolean } else { Type::Unit },
        return_value,
    };
    super::super::check_with_calls_observed(
        &root,
        &HashMap::from([("tree", &field)]),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", &root)]),
        None,
    )
}
fn operation(checked: &CheckedPlan) -> &Binding {
    checked
        .plan()
        .body
        .bindings
        .iter()
        .find(|binding| matches!(binding.operation, Operation::Merkle { .. }))
        .unwrap()
}
#[test]
fn nine_forms_select_precise_traits_and_keep_typed_arguments() {
    for historic in [false, true] {
        for (kind, access, method, cap) in [
            (0, MerkleAccess::Insert, "merkle_insert", "MerkleInsert"),
            (
                1,
                MerkleAccess::Insert,
                "merkle_insert_index",
                "MerkleInsert",
            ),
            (
                2,
                MerkleAccess::Insert,
                "merkle_insert_default",
                "MerkleInsert",
            ),
            (
                3,
                MerkleAccess::InsertHash,
                "merkle_insert_hash",
                "MerkleInsertHash",
            ),
            (
                4,
                MerkleAccess::InsertHash,
                "merkle_insert_hash_index",
                "MerkleInsertHash",
            ),
            (5, MerkleAccess::Reset, "merkle_reset", "MerkleReset"),
            (
                6,
                MerkleAccess::ResetHistory,
                "merkle_reset_history",
                "MerkleResetHistory",
            ),
            (
                7,
                MerkleAccess::CheckRoot,
                "merkle_check_root",
                "MerkleCheckRoot",
            ),
            (8, MerkleAccess::Inspect, "merkle_is_full", "MerkleInspect"),
        ] {
            if kind == 6 && !historic {
                continue;
            }
            let checked = fixture(kind, historic).unwrap();
            let binding = operation(&checked);
            let tree = declaration(binding, checked.declarations())
                .unwrap()
                .unwrap();
            assert_eq!(tree.access, access);
            assert_eq!(tree.slot.depth, 3);
            assert_eq!(tree.slot.historic, historic);
            let rendered = render(
                binding,
                checked.declarations(),
                syn::parse_quote!(executor),
                |value| {
                    let id = value.id;
                    let ident = quote::format_ident!("operand_{id}");
                    syn::parse_quote!(#ident)
                },
            )
            .unwrap()
            .unwrap();
            assert_eq!(rendered.slot.id, rendered.slot.declaration.0 as u32);
            assert_eq!(rendered.slot.depth, 3);
            assert_eq!(rendered.slot.historic, historic);
            let expression = rendered.expression.to_token_stream().to_string();
            assert!(expression.contains(method));
            assert!(
                !expression.contains(" as "),
                "checked index/hash/root carriers must not be recast"
            );
            let bound = rendered.bound.to_token_stream().to_string();
            assert!(bound.starts_with(&format!("crate :: runtime :: functional :: {cap} <")));
            if kind == 7 {
                assert!(bound.contains("crate :: types :: MerkleTreeDigest"));
            }
        }
    }
}
#[test]
fn declaration_identity_depth_and_history_errors_come_from_existing_owner() {
    let checked = fixture(8, false).unwrap();
    let original = operation(&checked);
    for change in [0, 1, 2, 3] {
        let mut binding = original.clone();
        let Operation::Merkle { slot, .. } = &mut binding.operation else {
            unreachable!()
        };
        match change {
            0 => slot.index = 1,
            1 => slot.depth = 4,
            2 => slot.historic = true,
            _ => slot.declaration = LedgerId(99),
        }
        let expected =
            super::super::validate::merkle::leaf(checked.declarations(), *slot, &binding.path)
                .unwrap_err();
        let actual = match declaration(&binding, checked.declarations()) {
            Err(error) => error,
            _ => panic!("mutated declaration accepted"),
        };
        assert_eq!(actual.path, expected.path);
        assert_eq!(actual.detail, expected.detail);
    }
}
#[test]
fn malformed_root_and_plain_history_reset_stay_checked_refusals() {
    for historic in [false, true] {
        let checked = fixture(7, historic).unwrap();
        let mut plan = checked.plan().clone();
        let binding = plan
            .body
            .bindings
            .iter_mut()
            .find(|binding| matches!(binding.operation, Operation::Merkle { .. }))
            .unwrap();
        let Operation::Merkle {
            operation: MerkleOperation::CheckRoot { root },
            ..
        } = &mut binding.operation
        else {
            unreachable!()
        };
        root.ty = Type::Field;
        // Reusing a checked value ID with a false annotation remains an original
        // SSA/type refusal, before functional projection can be invoked.
        assert!(super::super::validate::validate(&plan, checked.declarations()).is_err());
    }
    let checked = fixture(5, false).unwrap();
    let mut plan = checked.plan().clone();
    let binding = plan
        .body
        .bindings
        .iter_mut()
        .find(|binding| matches!(binding.operation, Operation::Merkle { .. }))
        .unwrap();
    let Operation::Merkle { operation, .. } = &mut binding.operation else {
        unreachable!()
    };
    *operation = MerkleOperation::ResetHistory;
    assert!(super::super::validate::validate(&plan, checked.declarations()).is_err());
}

#[test]
fn unrelated_operations_do_not_evaluate_merkle_operands() {
    let checked = fixture(8, false).unwrap();
    let mut binding = operation(&checked).clone();
    binding.operation = Operation::Literal(super::super::model::Literal::Boolean(true));
    assert!(
        render(
            &binding,
            checked.declarations(),
            syn::parse_quote!(executor),
            |_| panic!("unrelated operand evaluated")
        )
        .unwrap()
        .is_none()
    );
}
