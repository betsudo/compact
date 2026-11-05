// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Whole-source outer-Vector conversions retain independent plan validation.
use super::*;

fn contract(role: &str) -> Contract {
    let text = match role {
        "original" => include_str!(
            "../../../../tests/outer_vector_product_semantics/proof-goldens/original/source-ir.json"
        ),
        "family" => include_str!(
            "../../../../tests/outer_vector_product_semantics/proof-goldens/family/source-ir.json"
        ),
        _ => unreachable!(),
    };
    serde_json::from_str(text).unwrap()
}
fn checked(c: &Contract, name: &str) -> CheckedPlan {
    let root = c
        .stateful_circuits
        .iter()
        .find(|root| root.name == name)
        .unwrap();
    check_with_calls(
        root,
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
fn conversion(body: &mut model::Block) -> &mut model::Binding {
    body.bindings
        .iter_mut()
        .rev()
        .find(|binding| {
            matches!(
                binding.operation,
                Operation::Convert {
                    kind: model::Conversion::Coerce,
                    ..
                }
            )
        })
        .expect("whole source retains its final checked conversion")
}
fn elements(ty: &mut Type) -> &mut Vec<Type> {
    let Type::Vector { element, .. } = ty else {
        unreachable!()
    };
    let Type::Tuple { elements } = element.as_mut() else {
        unreachable!()
    };
    elements
}
fn child(ty: &mut Type, last: bool) -> &mut Vec<Type> {
    let Type::Tuple { elements } = &mut elements(ty)[usize::from(last)] else {
        unreachable!()
    };
    elements
}
fn leaf(ty: &mut Type, empty_leaf: bool) -> &mut Type {
    let field = &mut child(ty, false)[0];
    if empty_leaf {
        let Type::Vector { element, .. } = field else {
            unreachable!()
        };
        element
    } else {
        field
    }
}
#[test]
fn every_original_and_whole_family_root_is_independently_validated() {
    let declarations: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/outer_vector_product_semantics/sources.json"
    ))
    .unwrap();
    let mut roots = 0;
    for role in ["original", "family"] {
        let c = contract(role);
        let spec = declarations
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["role"] == role)
            .unwrap();
        let names = c
            .stateful_circuits
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(serde_json::to_value(&names).unwrap(), spec["entries"]);
        assert_eq!(c.circuits.len(), if role == "family" { 2 } else { 0 });
        assert_eq!(c.witnesses.len(), usize::from(role == "family"));
        for name in names {
            let plan = checked(&c, name);
            assert!(
                validate::validate(plan.plan(), plan.declarations()).is_ok(),
                "{role}.{name}"
            );
            roots += 1;
        }
    }
    assert_eq!(roots, 16);
}
#[test]
fn dimension_arity_narrowing_and_foreign_children_refuse_at_conversion() {
    let c = contract("family");
    for root in ["parameter", "empty", "empty_leaf", "last_child"] {
        let checked = checked(&c, root);
        assert!(validate::validate(checked.plan(), checked.declarations()).is_ok());
        for mutation in 0..6 {
            let mut forged = checked.plan().clone();
            let binding = conversion(&mut forged.body);
            let path = binding.path.clone();
            let ty = &mut binding.value.ty;
            match mutation {
                0 => {
                    let Type::Vector { length, .. } = ty else {
                        unreachable!()
                    };
                    *length += 1;
                }
                1 => {
                    elements(ty).pop();
                }
                2 => {
                    child(ty, root == "last_child").pop();
                }
                3 | 4 => {
                    let target = if root == "last_child" {
                        &mut child(ty, true)[0]
                    } else {
                        leaf(ty, root == "empty_leaf")
                    };
                    *target = if mutation == 3 {
                        Type::Unsigned { max: "1".into() }
                    } else {
                        Type::Boolean
                    };
                }
                5 => {
                    // A tuple's shape never grants a structurally similar nominal identity.
                    elements(ty)[usize::from(root == "last_child")] = Type::Struct {
                        name: "Foreign".into(),
                        fields: vec![StructField {
                            name: "value".into(),
                            ty: Type::Unsigned {
                                max: "65535".into(),
                            },
                        }],
                    };
                }
                _ => unreachable!(),
            }
            let error = validate::validate(&forged, checked.declarations()).unwrap_err();
            assert_eq!(error.path, path, "{root}/{mutation}");
            assert_eq!(
                error.detail,
                if mutation == 5 {
                    "struct definition is not canonical source type"
                } else {
                    "operation input/output types are incompatible"
                },
                "{root}/{mutation}"
            );
        }
    }
}
#[test]
fn operand_aliases_cannot_forge_scope_type_or_self_reference() {
    let c = contract("family");
    for root in ["parameter", "empty", "empty_leaf"] {
        let checked = checked(&c, root);
        for mutation in 0..3 {
            let mut forged = checked.plan().clone();
            let binding = conversion(&mut forged.body);
            let path = binding.path.clone();
            let id = binding.value.id;
            let Operation::Convert { value, .. } = &mut binding.operation else {
                unreachable!()
            };
            match mutation {
                0 => value.id = usize::MAX,
                1 => value.id = id,
                2 => {
                    *leaf(&mut value.ty, root == "empty_leaf") = Type::Unsigned {
                        max: "65535".into(),
                    }
                }
                _ => unreachable!(),
            }
            let error = validate::validate(&forged, checked.declarations()).unwrap_err();
            assert_eq!(error.path, path, "{root}/{mutation}");
            assert_eq!(
                error.detail,
                "value is out of scope or has an inconsistent type"
            );
        }
    }
}
#[test]
fn empty_children_retain_canonical_and_resource_checks() {
    let c = contract("family");
    for root in ["empty", "empty_leaf"] {
        let checked = checked(&c, root);
        let mut forged = checked.plan().clone();
        let binding = conversion(&mut forged.body);
        let path = binding.path.clone();
        *leaf(&mut binding.value.ty, root == "empty_leaf") = Type::Unsigned { max: "00".into() };
        let error = validate::validate(&forged, checked.declarations()).unwrap_err();
        assert_eq!(error.path, path);
        assert_eq!(
            error.detail,
            "invalid Compact Uint maximum \"00\"; expected a canonical decimal integer from 0 through 2^248 - 1"
        );
        let mut oversized = checked.plan().clone();
        let binding = conversion(&mut oversized.body);
        let path = binding.path.clone();
        *leaf(&mut binding.value.ty, root == "empty_leaf") = Type::Unsigned {
            max: "9".repeat(262145),
        };
        let preflight =
            crate::resource_limits::recording_type_nodes(&binding.value.ty).unwrap_err();
        assert!(matches!(
            preflight,
            crate::RenderError::ResourceLimit { .. }
        ));
        let error = validate::validate(&oversized, checked.declarations()).unwrap_err();
        assert_eq!(error.path, path);
        assert_eq!(error.detail, preflight.to_string());
    }
}
#[test]
fn statically_dead_branch_keeps_independent_child_validation() {
    let mut raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/outer_vector_product_semantics/proof-goldens/family/source-ir.json"
    ))
    .unwrap();
    let root = raw["stateful_circuits"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["name"] == "branch")
        .unwrap();
    root["return_value"]["body"]["condition"] = serde_json::json!({"kind":"boolean","value":true});
    let c: Contract = serde_json::from_value(raw).unwrap();
    let checked = checked(&c, "branch");
    assert!(validate::validate(checked.plan(), checked.declarations()).is_ok());
    let mut forged = checked.plan().clone();
    let binding = forged
        .body
        .bindings
        .iter_mut()
        .find(|binding| matches!(binding.operation, Operation::If { .. }))
        .unwrap();
    let Operation::If { otherwise, .. } = &mut binding.operation else {
        unreachable!()
    };
    let binding = conversion(otherwise);
    let path = binding.path.clone();
    *leaf(&mut binding.value.ty, false) = Type::Boolean;
    let error = validate::validate(&forged, checked.declarations()).unwrap_err();
    assert_eq!(error.path, path);
    assert_eq!(
        error.detail,
        "operation input/output types are incompatible"
    );
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
