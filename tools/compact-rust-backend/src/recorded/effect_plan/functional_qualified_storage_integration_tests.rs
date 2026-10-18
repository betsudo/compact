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

//! Explicit authority doubles qualify projection mechanics, never frontend issuance.
use super::*;
use crate::ir::{
    Contract, Expr, LedgerField, LedgerFieldKind, Parameter, SourceLocation, StateAction,
    StateReturn, StatefulCircuit,
};
use quote::ToTokens;
struct Authority {
    location: SourceLocation,
    missing: Option<usize>,
}
impl Authority {
    fn new() -> Self {
        Self {
            location: SourceLocation {
                file: "qualified.compact".into(),
                line: 2,
                column: 1,
            },
            missing: None,
        }
    }
}
impl SourceAuthority for Authority {
    fn origin<'a>(&'a self, site: SourceSite<'_>) -> OriginEvidence<'a> {
        if matches!(site,SourceSite::Value{id,..}if self.missing==Some(id)) {
            OriginEvidence::Unknown
        } else {
            OriginEvidence::BoundOriginalNode(&self.location)
        }
    }
}
impl super::super::functional_items::FunctionalSources for Authority {
    fn root<'a>(&'a self, _: &StatefulCircuit) -> Option<&'a SourceLocation> {
        Some(&self.location)
    }
}
#[test]
fn qualified_storage_requires_precise_target_and_source_before_emission() {
    for set in [false, true] {
        let checked =
            super::super::functional_qualified_storage::tests::fixture(set, false).unwrap();
        let authority = Authority::new();
        let capability = if set {
            Capability::QualifiedSetInsert
        } else {
            Capability::QualifiedCellWrite
        };
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        assert_eq!(
            program.requirements(),
            &std::collections::BTreeSet::from([Capability::Pure, capability])
        );
        let rendered = super::super::functional_render::render(&program).unwrap();
        assert_eq!(rendered.executor_bounds.len(), 1);
        assert_eq!(rendered.slots.len(), usize::from(!set));
        assert_eq!(rendered.sets.len(), usize::from(set));
        let target = Ledger8Target {
            capabilities: &[Capability::Pure, Capability::CellWrite, Capability::SetWrite],
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability{capability:actual,..})if actual==capability)
        );
        let effect = program.effects().iter().find(|effect| effect.capability() == capability).expect("qualified storage occurrence");
        let missing = Authority {
            missing: Some(effect.binding().value.id),
            ..Authority::new()
        };
        assert!(
            matches!(FunctionalProgram::project(&checked,&missing,Ledger8Target::PINNED),Err(ProjectionRefusal::MissingSourceAuthority{path})if path==effect.binding().path)
        );
    }
}
#[test]
fn qualified_bindings_keep_existing_tables_full_paths_and_canonical_types() {
    for set in [false, true] {
        let ty = crate::coin_shapes::qualified_coin_type();
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
            result: crate::ir::Type::Unit,
            return_value: StateReturn::Unit,
        };
        let contract = Contract {
            schema_version: crate::SCHEMA_VERSION,
            type_aliases: vec![],
            ledger_fields: vec![field],
            constructor: None,
            witnesses: vec![],
            circuits: vec![],
            stateful_circuits: vec![root],
        };
        let authority = Authority::new();
        let output =
            super::super::functional_items::render_contract(&contract, &authority).unwrap();
        let text = output.module.to_token_stream().to_string();
        assert!(text.contains(&quote::quote!(&[1u8, 2u8]).to_string()));
        assert!(text.contains(if set {
            "SetBinding :: new"
        } else {
            "CellBinding :: new"
        }));
        assert!(text.contains(if set {
            "ledger8 :: QualifiedSetInsert"
        } else {
            "ledger8 :: QualifiedCellWrite"
        }));
        assert!(!text.contains("const MAPS"));
        assert!(!text.contains("create_zswap_output"));
        assert!(!text.contains("com_indices"));
    }
}
