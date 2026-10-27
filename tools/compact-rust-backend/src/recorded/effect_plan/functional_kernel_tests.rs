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

//! genuine checked circuit effects; constructor execution is not certified.
use super::*;
use crate::ir::{
    Expr, KernelClaimKind, NativeWitnessBuiltin, Parameter, StateReturn, StatefulCircuit, Type,
};
use quote::ToTokens;
use std::collections::HashMap;
struct Authority {
    location: SourceLocation,
    missing: Option<usize>,
}
impl Authority {
    fn new() -> Self {
        Self {
            location: SourceLocation {
                file: "kernel.compact".into(),
                line: 1,
                column: 1,
            },
            missing: None,
        }
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, site: SourceSite<'_>) -> OriginEvidence<'a> {
        if matches!(site,SourceSite::Value {id,..} if self.missing==Some(id)) {
            OriginEvidence::Unknown
        } else {
            OriginEvidence::BoundOriginalNode(&self.location)
        }
    }
}
fn input(name: &str) -> Expr {
    Expr::Parameter { name: name.into() }
}
fn cases() -> Vec<(Expr, Type, Capability, &'static str)> {
    vec![
        (
            Expr::NativeWitnessCall {
                builtin: NativeWitnessBuiltin::OwnPublicKey,
            },
            NativeWitnessBuiltin::OwnPublicKey.result_type(),
            Capability::ExecutionIdentity,
            "ExecutionIdentity",
        ),
        (
            Expr::KernelSelf {
                ty: crate::ir::contract_address_type(),
            },
            crate::ir::contract_address_type(),
            Capability::KernelAddress,
            "KernelAddress",
        ),
        (
            Expr::KernelClaim {
                claim: KernelClaimKind::Nullifier,
                value: Box::new(input("bytes")),
            },
            Type::Unit,
            Capability::KernelClaims,
            "KernelClaims",
        ),
        (
            Expr::KernelMintShielded {
                domain: Box::new(input("bytes")),
                amount: Box::new(input("amount")),
            },
            Type::Unit,
            Capability::ShieldedMint,
            "ShieldedMint",
        ),
        (
            Expr::CreateZswapInput {
                coin: Box::new(input("qualified")),
            },
            Type::Unit,
            Capability::ZswapInput,
            "ZswapInput",
        ),
        (
            Expr::CreateZswapOutput {
                coin: Box::new(input("coin")),
                recipient: Box::new(input("recipient")),
            },
            Type::Unit,
            Capability::ZswapOutput,
            "ZswapOutput",
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
                name: "bytes".into(),
                ty: Type::Bytes { length: 32 },
            },
            Parameter {
                name: "amount".into(),
                ty: Type::Unsigned {
                    max: u64::MAX.to_string(),
                },
            },
            Parameter {
                name: "coin".into(),
                ty: crate::coin_shapes::shielded_coin_type(),
            },
            Parameter {
                name: "qualified".into(),
                ty: crate::coin_shapes::qualified_coin_type(),
            },
            Parameter {
                name: "recipient".into(),
                ty: crate::coin_shapes::shielded_recipient_type(),
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
#[test]
fn six_circuit_effects_have_only_their_official_ledger8_capability() {
    for (expression, result, capability, trait_name) in cases() {
        let root = root(expression, result);
        let checked = checked(&root);
        let authority = Authority::new();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        assert_eq!(program.requirements().len(), 1);
        assert!(program.requirements().contains(&capability));
        let rendered = super::super::functional_render::render(&program).unwrap();
        assert_eq!(rendered.executor_bounds.len(), 1);
        assert!(
            rendered.executor_bounds[0]
                .to_token_stream()
                .to_string()
                .contains(&format!("ledger8 :: {trait_name}"))
        );
        let target = Ledger8Target {
            capabilities: &[Capability::Pure],
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability {capability:actual,..}) if actual==capability)
        );
    }
}
#[test]
fn effect_identity_still_requires_original_source_occurrence() {
    for (expression, result, _, _) in cases() {
        let root = root(expression, result);
        let checked = checked(&root);
        let binding = checked.plan().body.bindings.last().unwrap();
        let mut authority = Authority::new();
        authority.missing = Some(binding.value.id);
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,Ledger8Target::PINNED),Err(ProjectionRefusal::MissingSourceAuthority {path}) if path==binding.path)
        );
    }
}
#[test]
fn ordinary_coin_conversion_preserves_reuse_and_has_no_allocation_shortcut() {
    let (expression, result, ..) = cases().pop().unwrap();
    let root = root(expression, result);
    let checked = checked(&root);
    let binding = checked.plan().body.bindings.last().unwrap();
    let Operation::CreateZswapOutput { coin, recipient } = &binding.operation else {
        unreachable!()
    };
    let info = super::super::render::effect_values::coin_info(coin)
        .to_token_stream()
        .to_string();
    let recipient = super::super::render::effect_values::recipient(recipient)
        .to_token_stream()
        .to_string();
    assert!(info.contains("coin_info_from_compact"));
    assert!(!info.contains("clone"));
    assert!(!info.contains("qualify"));
    assert!(recipient.contains("coin_recipient_from_compact"));
    assert!(!recipient.contains("com_indices"));
}
#[test]
fn all_three_claim_kinds_use_the_original_official_wrappers() {
    for (kind, variant, wrapper) in [
        (KernelClaimKind::Nullifier, "Nullifier", "CoinNullifier"),
        (KernelClaimKind::CoinSpend, "CoinSpend", "CoinCommitment"),
        (
            KernelClaimKind::CoinReceive,
            "CoinReceive",
            "CoinCommitment",
        ),
    ] {
        let root = root(
            Expr::KernelClaim {
                claim: kind,
                value: Box::new(input("bytes")),
            },
            Type::Unit,
        );
        let checked = checked(&root);
        let binding = checked.plan().body.bindings.last().unwrap();
        let Operation::KernelClaim { value, .. } = &binding.operation else {
            unreachable!()
        };
        let syntax = super::super::render::effect_values::claim(kind, value)
            .to_token_stream()
            .to_string();
        assert!(syntax.contains(variant));
        assert!(syntax.contains(wrapper));
        assert!(syntax.contains("HashOutput"));
    }
}

#[test]
fn selected_circuit_intents_keep_the_existing_guard_and_lazy_conversion_boundary() {
    let (output, _, _, _) = cases().pop().unwrap();
    let root = root(
        Expr::If {
            condition: Box::new(Expr::Boolean { value: true }),
            then: Box::new(output),
            otherwise: Box::new(Expr::Unit),
        },
        Type::Unit,
    );
    let checked = checked(&root);
    let authority = Authority::new();
    let program = FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
    let output = program
        .effects()
        .iter()
        .find(|effect| {
            matches!(
                effect.binding().operation,
                Operation::CreateZswapOutput { .. }
            )
        })
        .unwrap();
    assert_ne!(output.guard(), GuardId(0));
    let rendered = super::super::functional_render::render(&program)
        .unwrap()
        .body
        .to_token_stream()
        .to_string();
    assert!(rendered.find("bind").unwrap() < rendered.find("coin_info_from_compact").unwrap());
    assert!(!rendered.contains("set_zswap_output_start"));
    assert!(!rendered.contains("com_indices"));
}
