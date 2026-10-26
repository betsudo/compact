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

// The direct VectorFoldCall child is a native-valid recording refusal; fixed Field
// commitments themselves are supported. Do not replace this with malformed types.
// Direct legacy_selected traces below preserve the staged predecessor priority laws.
// Current public/helper emission assertions, where present, use algebra.ordinary.
use super::*;

#[test]
fn deposit_owner_keeps_priority_and_stops_later_families() {
    let c = source();
    let row = legacy_selected_trace(&c, root(&c));
    assert_eq!(row["body_owner"], "algebra.guarded-deposit");
    let a = attempts(&row);
    assert_eq!(
        a.iter()
            .filter(|&&s| s == "algebra.guarded-deposit")
            .count(),
        1
    );
    assert!(!a.contains(&"selector.guarded-deposit"));
    assert!(!a.contains(&"selector.reset-payout"));
    let at = a
        .iter()
        .position(|&s| s == "algebra.guarded-deposit")
        .unwrap();
    assert_eq!(a[at - 1], "algebra.voting-commit");
    assert!(!a.contains(&"selector.funded-mint"));
}

#[test]
fn earlier_composite_and_receive_owners_prevent_deposit_attempts() {
    for (json, name, owner) in [
        (
            include_str!("../../../../tests/field-observation-schema20-ir.json"),
            "snapshot",
            "algebra.composite-values",
        ),
        (
            include_str!("../../../../tests/shielded-receive-schema20-ir.json"),
            "accept",
            "algebra.shielded-receive",
        ),
    ] {
        let c: Contract = serde_json::from_str(json).unwrap();
        let r = c.stateful_circuits.iter().find(|r| r.name == name).unwrap();
        let row = legacy_selected_trace(&c, r);
        assert_eq!(row["body_owner"], owner);
        assert!(!attempts(&row).contains(&"algebra.guarded-deposit"));
    }
}

#[test]
fn fold_successor_guarded_deposit_stops_staged_suffix() {
    let mut c = source();
    root_mut(&mut c).actions.insert(
        0,
        StateAction::Expression {
            value: Expr::Default {
                ty: Type::OpaqueString,
            },
        },
    );
    assert!(Context::new(&c).old(root(&c)).is_none());
    assert!(Context::new(&c).domain(root(&c)).unwrap().is_some());
    crate::recorded::test_support::register_unsupported_fold(&mut c);
    // Keep the positive default above, then exercise the direct fold boundary.
    root_mut(&mut c).actions[0] = StateAction::Expression {
        value: Expr::PersistentCommit {
            value: Box::new(crate::recorded::test_support::unsupported_fixed_input()),
            opening: Box::new(Expr::BytesLiteral { bytes: vec![0; 32] }),
        },
    };
    let maps = Context::new(&c);
    assert!(maps.old(root(&c)).is_none());
    assert!(maps.domain(root(&c)).unwrap().is_some());
    let row = legacy_selected_trace(&c, root(&c));
    let a = attempts(&row);
    // The staged oracle uses current checked children. New admission stops at
    // its successful algebra owner; the frozen old shape still declined above.
    assert_eq!(a.last().copied(), row["body_owner"].as_str());
    assert!(a.last().is_some_and(|route| route.starts_with("algebra.")));
    assert!(!a.contains(&"selector.guarded-deposit"));
    let output = crate::render_with_capabilities(&c).unwrap();
    let cap = output
        .capabilities
        .circuits
        .iter()
        .find(|r| r.name == "set_topic")
        .unwrap();
    assert!(cap.recorded && cap.observed_call);
    crate::recorded::test_support::assert_checked_fold_successor(
        &c,
        root(&c),
        RecordingGap::action(&root(&c).actions[0], "actions[0]".into()),
        "actions[0].value.value",
    );
}
