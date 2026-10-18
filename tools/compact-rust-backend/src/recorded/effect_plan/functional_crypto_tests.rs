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

//! Checked crypto syntax against authenticated predecessor expression goldens.
use super::*;
use crate::ir::{Expr, Parameter, StateReturn, StatefulCircuit, Type};
use quote::ToTokens;
use std::collections::HashMap;
struct Authority(SourceLocation);
impl Authority {
    fn new() -> Self {
        Self(SourceLocation {
            file: "crypto.compact".into(),
            line: 1,
            column: 1,
        })
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, _: SourceSite<'_>) -> OriginEvidence<'a> {
        OriginEvidence::BoundOriginalNode(&self.0)
    }
}
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn cases() -> Vec<(&'static str, Expr, Type, Option<Capability>)> {
    let bytes = Type::Bytes { length: 32 };
    vec![
        (
            "persistent_hash",
            Expr::PersistentHash {
                value: Box::new(input("bytes")),
            },
            bytes.clone(),
            Some(Capability::Hashing),
        ),
        (
            "persistent_commit",
            Expr::PersistentCommit {
                value: Box::new(input("bytes")),
                opening: Box::new(input("bytes")),
            },
            bytes.clone(),
            Some(Capability::Hashing),
        ),
        (
            "transient_hash",
            Expr::TransientHash {
                value: Box::new(input("opaque")),
            },
            Type::Field,
            Some(Capability::Hashing),
        ),
        (
            "transient_commit",
            Expr::TransientCommit {
                value: Box::new(input("opaque")),
                opening: Box::new(input("field")),
            },
            Type::Field,
            Some(Capability::Hashing),
        ),
        (
            "hash_to_curve",
            Expr::HashToCurve {
                value: Box::new(input("opaque")),
            },
            Type::JubjubPoint,
            Some(Capability::Hashing),
        ),
        (
            "field_to_bytes",
            Expr::FieldToBytes32 {
                value: Box::new(input("field")),
            },
            bytes.clone(),
            Some(Capability::FieldEncoding),
        ),
        (
            "degrade",
            Expr::DegradeToTransient {
                value: Box::new(input("bytes")),
            },
            Type::Field,
            Some(Capability::FieldEncoding),
        ),
        (
            "upgrade",
            Expr::UpgradeFromTransient {
                value: Box::new(input("field")),
            },
            bytes.clone(),
            Some(Capability::FieldEncoding),
        ),
        (
            "point_x",
            Expr::JubjubPointX {
                value: Box::new(input("point")),
            },
            Type::Field,
            Some(Capability::Jubjub),
        ),
        (
            "point_y",
            Expr::JubjubPointY {
                value: Box::new(input("point")),
            },
            Type::Field,
            Some(Capability::Jubjub),
        ),
        (
            "scalar",
            Expr::JubjubScalarFromNative {
                value: Box::new(input("field")),
            },
            Type::Field,
            Some(Capability::Jubjub),
        ),
        (
            "generator",
            Expr::EcMulGenerator {
                scalar: Box::new(input("field")),
            },
            Type::JubjubPoint,
            Some(Capability::Jubjub),
        ),
        (
            "neg",
            Expr::EcNeg {
                value: Box::new(input("point")),
            },
            Type::JubjubPoint,
            Some(Capability::Jubjub),
        ),
        (
            "construct",
            Expr::ConstructJubjubPoint {
                x: Box::new(input("field")),
                y: Box::new(input("field")),
            },
            Type::JubjubPoint,
            Some(Capability::Jubjub),
        ),
        (
            "mul",
            Expr::EcMul {
                point: Box::new(input("point")),
                scalar: Box::new(input("field")),
            },
            Type::JubjubPoint,
            Some(Capability::Jubjub),
        ),
        (
            "add",
            Expr::EcAdd {
                left: Box::new(input("point")),
                right: Box::new(input("point")),
            },
            Type::JubjubPoint,
            Some(Capability::Jubjub),
        ),
        (
            "keccak",
            Expr::Keccak256 {
                value: Box::new(input("bytes")),
            },
            bytes,
            None,
        ),
    ]
}
fn root(expression: Expr, result: Type) -> StatefulCircuit {
    StatefulCircuit {
        name: "root".into(),
        source: None,
        internal: false,
        parameters: vec![
            Parameter {
                name: "field".into(),
                ty: Type::Field,
            },
            Parameter {
                name: "bytes".into(),
                ty: Type::Bytes { length: 32 },
            },
            Parameter {
                name: "point".into(),
                ty: Type::JubjubPoint,
            },
            Parameter {
                name: "opaque".into(),
                ty: Type::OpaqueString,
            },
        ],
        actions: vec![],
        result,
        return_value: StateReturn::Expression { value: expression },
    }
}
fn checked(root: &StatefulCircuit) -> CheckedPlan {
    super::super::check_with_calls_observed(
        root,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::from([("root", root)]),
        None,
    )
    .unwrap()
}
fn operand(input: &Value) -> syn::Expr {
    let name = quote::format_ident!("__compact_effect_{}", input.id);
    crate::retained_value(syn::parse_quote!(#name), &input.ty)
}
#[test]
fn seventeen_original_crypto_expression_tokens_are_preserved() {
    let goldens: serde_json::Value =
        serde_json::from_str(include_str!("crypto-expression-goldens.json")).unwrap();
    assert_eq!(goldens.as_object().unwrap().len(), 17);
    for (name, expression, result, _) in cases() {
        let root = root(expression, result);
        let checked = checked(&root);
        let binding = checked.plan().body.bindings.last().unwrap();
        let expected: syn::Expr = syn::parse_str(goldens[name].as_str().unwrap()).unwrap();
        let actual = super::super::render::crypto::expression(binding, operand)
            .unwrap()
            .unwrap();
        // The predecessor CLI applies prettyplease, which may insert a
        // trailing call comma. Compare through that SAME existing output owner
        // rather than introducing a bespoke token-normalization algorithm.
        let output = |expression: syn::Expr| {
            let file: syn::File = syn::parse_quote!(fn probe() { let _ = #expression; });
            prettyplease::unparse(&file)
        };
        assert_eq!(output(actual), output(expected), "{name}");
    }
}
#[test]
fn admitted_checked_crypto_has_one_precise_group_and_no_executor_bounds() {
    for (name, expression, result, capability) in cases() {
        let root = root(expression, result);
        let checked = checked(&root);
        let authority = Authority::new();
        let Some(capability) = capability else {
            continue;
        };
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        assert_eq!(program.requirements().len(), 1, "{name}");
        assert!(program.requirements().contains(&capability));
        let rendered = super::super::functional_render::render(&program).unwrap();
        assert!(rendered.executor_bounds.is_empty());
        let target = Ledger8Target {
            capabilities: &[Capability::Pure],
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability {capability:actual,..}) if actual==capability),
            "{name}"
        );
    }
}
#[test]
fn exact_dynamic_hash_input_wrapper_is_retained_once() {
    for (name, expression, result, _) in cases().into_iter().filter(|(name, ..)| {
        matches!(
            *name,
            "transient_hash" | "transient_commit" | "hash_to_curve"
        )
    }) {
        let root = root(expression, result);
        let checked = checked(&root);
        let binding = checked.plan().body.bindings.last().unwrap();
        let syntax = super::super::render::crypto::expression(binding, operand)
            .unwrap()
            .unwrap()
            .to_token_stream()
            .to_string();
        assert_eq!(
            syntax.matches("exact_hash_input").count(),
            1,
            "{name}: {syntax}"
        );
    }
}
#[test]
fn native_keccak_syntax_does_not_grant_pinned_zkir_v2_functional_admission() {
    let (_, expression, result, _) = cases().pop().unwrap();
    let root = root(expression, result);
    let checked = checked(&root);
    let authority = Authority::new();
    let binding = checked.plan().body.bindings.last().unwrap();
    assert!(
        super::super::render::crypto::expression(binding, operand)
            .unwrap()
            .is_some()
    );
    assert!(
        matches!(FunctionalProgram::project(&checked,&authority,Ledger8Target::PINNED),Err(ProjectionRefusal::OutsideFunctionalSubset {path}) if path==binding.path)
    );
}

#[test]
fn crypto_capability_never_replaces_original_value_authority() {
    struct DenyValue {
        location: SourceLocation,
        id: usize,
    }
    impl SourceAuthority for DenyValue {
        fn origin<'a>(&'a self, site: SourceSite<'_>) -> OriginEvidence<'a> {
            if matches!(site, SourceSite::Value {id,..} if id==self.id) {
                OriginEvidence::Unknown
            } else {
                OriginEvidence::BoundOriginalNode(&self.location)
            }
        }
    }
    for (name, expression, result, capability) in cases() {
        if capability.is_none() {
            continue;
        }
        let root = root(expression, result);
        let checked = checked(&root);
        let binding = checked.plan().body.bindings.last().unwrap();
        let authority = DenyValue {
            location: Authority::new().0,
            id: binding.value.id,
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,Ledger8Target::PINNED),
            Err(ProjectionRefusal::MissingSourceAuthority {path}) if path==binding.path),
            "{name}"
        );
    }
}
