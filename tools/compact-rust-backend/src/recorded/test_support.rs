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

//! Historical Fold inputs and explicit successor admission checks.
//! The legacy helper names/reducer identity retain original test inputs. Direct
//! named Fold is now admitted; opaque commitments remain IR-only policy controls.
use crate::ir::{Contract, Expr, Parameter, PureCircuit, Type};

pub(crate) fn fixed_map_input() -> Expr {
    Expr::VectorMap {
        parameter: Parameter {
            name: "sentinel_item".into(),
            ty: Type::Field,
        },
        source: Box::new(Expr::Vector {
            element: Type::Field,
            elements: vec![Expr::FieldLiteral { value: "1".into() }],
        }),
        body: Box::new(Expr::Parameter {
            name: "sentinel_item".into(),
        }),
        result: Type::Field,
        length: 1,
    }
}

/// Genuine source-qualified direct fold. Each complete contract must include
/// the exact declaration below; a missing callee is not a refusal control.
const FOLD_STEP: &str = "recording_refusal_fold_step";
pub(crate) fn unsupported_fixed_input() -> Expr {
    Expr::VectorFoldCall {
        name: FOLD_STEP.into(),
        initial: Box::new(Expr::FieldLiteral { value: "0".into() }),
        source: Box::new(Expr::Tuple {
            elements: vec![Expr::FieldLiteral { value: "1".into() }],
        }),
        accumulator: Type::Field,
        element: Type::Field,
        length: 1,
    }
}
pub(crate) fn register_unsupported_fold(contract: &mut Contract) {
    assert!(!contract.circuits.iter().any(|c| c.name == FOLD_STEP));
    contract.circuits.push(PureCircuit {
        source: None,
        name: FOLD_STEP.into(),
        internal: true,
        parameters: ["acc", "item"]
            .into_iter()
            .map(|name| Parameter {
                name: name.into(),
                ty: Type::Field,
            })
            .collect(),
        result: Type::Field,
        body: Expr::Add {
            left: Box::new(Expr::Parameter { name: "acc".into() }),
            right: Box::new(Expr::Parameter {
                name: "item".into(),
            }),
        },
    });
}
pub(crate) fn with_unsupported_fold(mut contract: Contract) -> Contract {
    register_unsupported_fold(&mut contract);
    contract
}

pub(crate) fn unsupported_commit() -> Expr {
    Expr::PersistentCommit {
        value: Box::new(unsupported_fixed_input()),
        opening: Box::new(Expr::BytesLiteral { bytes: vec![0; 32] }),
    }
}

/// Keeps PersistentCommit's concrete typed-policy provenance. The Compact
/// frontend rejects opaque persistent hashing; never call this a source gap.
pub(crate) fn opaque_commit_policy() -> Expr {
    Expr::PersistentCommit {
        value: Box::new(Expr::Default {
            ty: Type::OpaqueString,
        }),
        opening: Box::new(Expr::BytesLiteral { bytes: vec![0; 32] }),
    }
}

#[test]
fn direct_fold_control_is_native_valid_and_newly_admitted_at_the_actual_child() {
    let mut contract: crate::ir::Contract = serde_json::from_str(include_str!(
        "../../tests/opaque-string-map-schema20-ir.json"
    ))
    .unwrap();
    register_unsupported_fold(&mut contract);
    let root = contract
        .stateful_circuits
        .iter_mut()
        .find(|c| c.name == "ensure")
        .unwrap();
    root.actions.clear();
    root.result = Type::Bytes { length: 32 };
    root.return_value = crate::ir::StateReturn::Expression {
        value: unsupported_commit(),
    };
    // A whole native render and actual typed loop qualify the changed boundary.
    let output = crate::render_with_capabilities(&contract).unwrap();
    let capability = output
        .capabilities
        .circuits
        .iter()
        .find(|c| c.name == "ensure")
        .unwrap();
    assert!(capability.recorded && capability.observed_call);
    let root = contract
        .stateful_circuits
        .iter()
        .find(|c| c.name == "ensure")
        .unwrap();
    let checked = super::effect_plan::check_with_calls(
        root,
        &std::collections::HashMap::new(),
        &std::collections::HashMap::new(),
        &contract
            .circuits
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect(),
        &std::collections::HashMap::new(),
    )
    .unwrap();
    assert_eq!(
        super::effect_plan::fold_paths(&checked),
        ["return_value.value.value"]
    );
}

/// Keep the complete old refusal, then qualify intentional current admission at
/// the same original Fold path. No malformed replacement control is substituted.
pub(crate) fn assert_checked_fold_successor(
    contract: &Contract,
    root: &crate::ir::StatefulCircuit,
    historical: crate::recorded::RecordingGap,
    expected_path: &str,
) {
    use crate::recorded::{RecordingOutcome, effect_plan, generic_reference};
    use std::collections::{HashMap, HashSet};
    let ledger = contract
        .ledger_fields
        .iter()
        .map(|v| (v.id.as_str(), v))
        .collect::<HashMap<_, _>>();
    let witnesses = contract
        .witnesses
        .iter()
        .map(|v| (v.name.as_str(), v))
        .collect();
    let pure = contract
        .circuits
        .iter()
        .map(|v| (v.name.as_str(), v))
        .collect();
    let calls = contract
        .stateful_circuits
        .iter()
        .map(|v| (v.name.as_str(), v))
        .collect();
    let old = generic_reference::render_recorded_circuit(
        root,
        &ledger,
        &witnesses,
        &pure,
        &calls,
        &HashSet::new(),
    )
    .unwrap();
    assert!(matches!(old, RecordingOutcome::Unsupported(gap) if gap == historical));
    let checked =
        effect_plan::check_with_calls_observed(root, &ledger, &witnesses, &pure, &calls, None)
            .unwrap();
    assert!(
        effect_plan::fold_paths(&checked)
            .iter()
            .any(|path| path == expected_path),
        "expected original Fold path {expected_path}"
    );
    let output = crate::render_with_capabilities(contract).unwrap();
    let cap = output
        .capabilities
        .circuits
        .iter()
        .find(|v| v.name == root.name)
        .unwrap();
    assert!(cap.recorded && cap.observed_call);
    assert!(cap.recording_unavailable.is_none() && cap.observed_call_unavailable.is_none());
}
