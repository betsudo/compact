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

//! Actual source lowering/checking owns qualified input and destination shapes.
use super::super::{DiagnosticPlanError, model::CheckedPlan};
use super::*;
use crate::ir::{
    Expr, LedgerField, LedgerFieldKind, Parameter, StateAction, StateReturn, StatefulCircuit,
};
use quote::ToTokens;
use std::collections::HashMap;
pub(in crate::recorded::effect_plan) fn fixture(
    set: bool,
    invalid_destination: bool,
) -> Result<CheckedPlan, DiagnosticPlanError> {
    let ty = if invalid_destination {
        Type::Boolean
    } else {
        crate::coin_shapes::qualified_coin_type()
    };
    let field = LedgerField {
        source: None,
        id: "coins".into(),
        index: 0,
        path: vec![1, 2],
        declaration: if set {
            LedgerFieldKind::Set { ty }
        } else {
            LedgerFieldKind::Cell { ty }
        },
    };
    let coin = Expr::Parameter {
        name: "coin".into(),
    };
    let recipient = Expr::Parameter {
        name: "recipient".into(),
    };
    let action = if set {
        StateAction::SetInsertCoin {
            field: "coins".into(),
            index: 0,
            coin,
            recipient,
        }
    } else {
        StateAction::CellWriteCoin {
            field: "coins".into(),
            index: 0,
            coin,
            recipient,
        }
    };
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![
            Parameter {
                name: "coin".into(),
                ty: crate::coin_shapes::shielded_coin_type(),
            },
            Parameter {
                name: "recipient".into(),
                ty: crate::coin_shapes::shielded_recipient_type(),
            },
        ],
        actions: vec![action],
        result: Type::Unit,
        return_value: StateReturn::Unit,
    };
    super::super::check_with_calls_observed(
        &root,
        &HashMap::from([("coins", &field)]),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", &root)]),
        None,
    )
}
fn binding(checked: &CheckedPlan) -> &Binding {
    checked
        .plan()
        .body
        .bindings
        .iter()
        .find(|binding| access(binding).is_some())
        .unwrap()
}
#[test]
fn qualified_storage_selects_only_its_explicit_ledger8_capability() {
    for set in [false, true] {
        let checked = fixture(set, false).unwrap();
        let binding = binding(&checked);
        assert_eq!(
            access(binding),
            Some(if set {
                QualifiedAccess::SetInsert
            } else {
                QualifiedAccess::CellWrite
            })
        );
        let result = render(binding, checked.declarations(), syn::parse_quote!(executor))
            .unwrap()
            .unwrap();
        let bound = result.bound.to_token_stream().to_string();
        assert!(bound.contains(if set {
            "QualifiedSetInsert"
        } else {
            "QualifiedCellWrite"
        }));
        assert!(bound.contains("crate :: types :: QualifiedShieldedCoinInfo"));
        match result.slot {
            QualifiedSlot::Cell(slot) => {
                assert!(!set);
                assert_eq!(slot.id, slot.declaration.0 as u32);
            }
            QualifiedSlot::Set(slot) => {
                assert!(set);
                assert_eq!(slot.id, slot.declaration.0 as u32);
            }
        }
        let expression = result.expression.to_token_stream().to_string();
        assert_eq!(expression.matches("coin_info_from_compact").count(), 1);
        assert_eq!(expression.matches("coin_recipient_from_compact").count(), 1);
        assert!(!expression.contains("com_indices"));
        assert!(!expression.contains("create_zswap"));
        assert!(!expression.contains("clone"));
        assert!(expression.contains("crate :: runtime :: ledger"));
    }
}
#[test]
fn wrong_destination_shapes_remain_original_checked_refusals() {
    for set in [false, true] {
        assert!(fixture(set, true).is_err());
    }
}
#[test]
fn malformed_slot_identity_reuses_existing_destination_owner() {
    for set in [false, true] {
        let checked = fixture(set, false).unwrap();
        let mut binding = binding(&checked).clone();
        match &mut binding.operation {
            Operation::CellWriteCoin { slot, .. } => slot.index = 1,
            Operation::SetInsertCoin { slot, .. } => slot.index = 1,
            _ => unreachable!(),
        }
        let error = match render(
            &binding,
            checked.declarations(),
            syn::parse_quote!(executor),
        ) {
            Err(error) => error,
            _ => panic!("changed declaration accepted"),
        };
        assert_eq!(error.path, binding.path);
        assert!(error.detail.contains("index mismatch"));
    }
}
#[test]
fn shared_conversion_expressions_keep_the_original_qualified_syntax() {
    let checked = fixture(false, false).unwrap();
    let binding = binding(&checked);
    let Operation::CellWriteCoin {
        coin, recipient, ..
    } = &binding.operation
    else {
        unreachable!()
    };
    assert_eq!(coin.id, 0);
    assert_eq!(recipient.id, 1);
    let actual_coin = super::super::render::effect_values::coin_info(coin);
    let actual_recipient = super::super::render::effect_values::recipient(recipient);
    let expected_coin: syn::Expr = syn::parse_quote!(runtime::ledger::coin_info_from_compact(
        __compact_effect_0.nonce,
        __compact_effect_0.color,
        __compact_effect_0.value.value()
    ));
    let expected_recipient: syn::Expr =
        syn::parse_quote!(runtime::ledger::coin_recipient_from_compact(
            __compact_effect_1.is_left,
            __compact_effect_1.left.bytes,
            __compact_effect_1.right.bytes
        ));
    assert_eq!(
        actual_coin.to_token_stream().to_string(),
        expected_coin.to_token_stream().to_string()
    );
    assert_eq!(
        actual_recipient.to_token_stream().to_string(),
        expected_recipient.to_token_stream().to_string()
    );
}
