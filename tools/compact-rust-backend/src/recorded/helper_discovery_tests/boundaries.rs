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
use super::*;

#[test]
fn direct_fold_successor_seeds_helpers_under_current_rendering_dependencies() {
    let mut c = fixture(
        Type::Field,
        StateReturn::Expression {
            value: crate::recorded::test_support::unsupported_fixed_input(),
        },
        true,
    );
    crate::recorded::test_support::register_unsupported_fold(&mut c);
    let output = crate::render_with_capabilities(&c).unwrap();
    assert!(
        output
            .capabilities
            .circuits
            .iter()
            .find(|v| v.name == "root")
            .unwrap()
            .recorded
    );
    let (names, items) = Context::new(&c).plan(&c, false).unwrap();
    assert!(names.contains("callee") && !items.is_empty());
    let (old_names, old_items) = Context::new(&c).plan(&c, true).unwrap();
    // The frozen collector/planner delegates rendering to the current owner.
    // It therefore sees the same new semantic admission, not a frozen refusal.
    assert_eq!(old_names, names);
    assert_eq!(old_items.len(), items.len());
    let cx = Context::new(&c);
    assert!(matches!(
        crate::recorded::generic_reference::render_recorded_helper(
            cx.circuits["callee"],
            &cx.ledger,
            &cx.witnesses,
            &cx.pure,
            &cx.circuits,
            &HashSet::new()
        )
        .unwrap(),
        RecordingOutcome::Unsupported(_)
    ));
}

#[test]
fn native_missing_declarations_and_arity_errors_precede_helper_publication() {
    let mut c = fixture(Type::Unit, StateReturn::Unit, true);
    let StateAction::CircuitCall { name, .. } = &mut c.stateful_circuits[1].actions[0] else {
        unreachable!()
    };
    *name = "missing".into();
    assert!(crate::render_with_capabilities(&c).is_err());
    let mut c = fixture(Type::Unit, StateReturn::Unit, true);
    let StateAction::CircuitCall { arguments, .. } = &mut c.stateful_circuits[1].actions[0] else {
        unreachable!()
    };
    arguments.push(value(&Type::Field));
    assert!(crate::render_with_capabilities(&c).is_err());
}

#[test]
fn constructor_only_calls_do_not_seed_ordinary_recording_helpers() {
    let mut c = fixture(
        Type::Boolean,
        StateReturn::Expression {
            value: value(&Type::Boolean),
        },
        true,
    );
    c.stateful_circuits.pop();
    c.constructor = Some(crate::ir::Constructor {
        source: None,
        parameters: vec![],
        steps: vec![crate::ir::ConstructorStep::Expression {
            value: Expr::Call {
                name: "callee".into(),
                arguments: vec![],
            },
        }],
    });
    // Native validates the constructor call independently of ordinary roots.
    crate::render_with_capabilities(&c).unwrap();
    let (names, items) = Context::new(&c).plan(&c, false).unwrap();
    assert!(names.is_empty() && items.is_empty());
}
