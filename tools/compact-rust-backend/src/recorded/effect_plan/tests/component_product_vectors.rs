// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Forged private plans must not turn a checked source conversion into authority.
use super::*;

fn contract() -> Contract {
    serde_json::from_str(include_str!(
        "../../../../tests/component_product_vector_widening/proof-goldens/family/source-ir.json"
    ))
    .unwrap()
}
fn checked(c: &Contract, name: &str) -> CheckedPlan {
    let circuit = c
        .stateful_circuits
        .iter()
        .find(|root| root.name == name)
        .unwrap();
    check_with_calls(
        circuit,
        &c.ledger_fields.iter().map(|v| (v.id.as_str(), v)).collect(),
        &c.witnesses.iter().map(|v| (v.name.as_str(), v)).collect(),
        &c.circuits.iter().map(|v| (v.name.as_str(), v)).collect(),
        &c.stateful_circuits
            .iter()
            .map(|v| (v.name.as_str(), v))
            .collect(),
    )
    .unwrap()
}
fn conversion(plan: &mut model::Plan) -> &mut model::Binding {
    plan.body
        .bindings
        .iter_mut()
        .find(|binding| {
            matches!(
                binding.operation,
                Operation::Convert {
                    kind: model::Conversion::Coerce,
                    ..
                }
            )
        })
        .expect("the intact source contains its original final conversion")
}
fn component_fields(ty: &mut Type) -> &mut Vec<Type> {
    let Type::Vector { element, .. } = ty else {
        unreachable!()
    };
    let Type::Tuple { elements } = element.as_mut() else {
        unreachable!()
    };
    let Type::Vector { element, .. } = &mut elements[0] else {
        unreachable!()
    };
    let Type::Tuple { elements } = element.as_mut() else {
        unreachable!()
    };
    elements
}
fn leaf(ty: &mut Type) -> &mut Type {
    let Type::Vector { element, .. } = &mut component_fields(ty)[0] else {
        unreachable!()
    };
    element
}
#[test]
fn every_whole_family_root_keeps_independent_recorded_validation() {
    let c = contract();
    assert_eq!(c.stateful_circuits.len(), 14);
    assert_eq!(c.circuits.len(), 2);
    assert_eq!(c.witnesses.len(), 1);
    for root in &c.stateful_circuits {
        let checked = checked(&c, &root.name);
        assert!(
            validate::validate(checked.plan(), checked.declarations()).is_ok(),
            "{}",
            root.name
        );
    }
}
#[test]
fn forged_nested_vector_conversions_refuse_at_the_original_operation() {
    let c = contract();
    // Empty iteration must not erase the declared original child types.
    for root in ["parameter", "empty", "empty_leaf"] {
        let checked = checked(&c, root);
        assert!(validate::validate(checked.plan(), checked.declarations()).is_ok());
        for mutation in 0..5 {
            let mut forged = checked.plan().clone();
            let binding = conversion(&mut forged);
            let path = binding.path.clone();
            match mutation {
                0 => {
                    let Type::Vector { length, .. } =
                        &mut component_fields(&mut binding.value.ty)[0]
                    else {
                        unreachable!()
                    };
                    *length += 1;
                }
                1 => *leaf(&mut binding.value.ty) = Type::Unsigned { max: "1".into() },
                2 => {
                    component_fields(&mut binding.value.ty)[1] = Type::Unsigned { max: "15".into() }
                }
                3 => *leaf(&mut binding.value.ty) = Type::Boolean,
                4 => {
                    component_fields(&mut binding.value.ty).pop();
                }
                _ => unreachable!(),
            }
            // Keep the immutable original root signature and result. Validation
            // must refuse the forged operation before any later result mismatch.
            let error = validate::validate(&forged, checked.declarations()).unwrap_err();
            assert_eq!(error.path, path, "{root}/{mutation}");
            assert_eq!(
                error.detail, "operation input/output types are incompatible",
                "{root}/{mutation}"
            );
        }
    }
}
#[test]
fn forged_canonical_and_resource_annotations_cannot_hide_in_empty_leaves() {
    let c = contract();
    let checked = checked(&c, "empty_leaf");
    let mut malformed = checked.plan().clone();
    let binding = conversion(&mut malformed);
    let path = binding.path.clone();
    *leaf(&mut binding.value.ty) = Type::Unsigned { max: "00".into() };
    let error = validate::validate(&malformed, checked.declarations()).unwrap_err();
    assert_eq!(error.path, path);
    assert_eq!(
        error.detail,
        "invalid Compact Uint maximum \"00\"; expected a canonical decimal integer from 0 through 2^248 - 1"
    );

    let mut oversized = checked.plan().clone();
    let binding = conversion(&mut oversized);
    let path = binding.path.clone();
    *leaf(&mut binding.value.ty) = Type::Unsigned {
        max: "9".repeat(262145),
    };
    let preflight = crate::resource_limits::recording_type_nodes(&binding.value.ty).unwrap_err();
    assert!(matches!(
        preflight,
        crate::RenderError::ResourceLimit { .. }
    ));
    let error = validate::validate(&oversized, checked.declarations()).unwrap_err();
    assert_eq!(error.path, path);
    assert_eq!(error.detail, preflight.to_string());
}

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
