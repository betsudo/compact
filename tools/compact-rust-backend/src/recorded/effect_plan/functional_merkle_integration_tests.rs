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

//! Projection and packaging controls use an explicit source-authority double.
use super::*;
use crate::ir::{
    Contract, LedgerField, LedgerFieldKind, SourceLocation, StateReturn, StatefulCircuit,
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
                file: "merkle.compact".into(),
                line: 3,
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
fn merkle_caps_source_refusals_and_minimal_executor_bounds_are_precise() {
    for (kind, cap, trait_name) in [
        (0, Capability::MerkleInsert, "MerkleInsert"),
        (3, Capability::MerkleInsertHash, "MerkleInsertHash"),
        (5, Capability::MerkleReset, "MerkleReset"),
        (6, Capability::MerkleResetHistory, "MerkleResetHistory"),
        (7, Capability::MerkleCheckRoot, "MerkleCheckRoot"),
        (8, Capability::MerkleInspect, "MerkleInspect"),
    ] {
        let checked = super::super::functional_merkle::tests::fixture(kind, true).unwrap();
        let authority = Authority::new();
        let program =
            FunctionalProgram::project(&checked, &authority, Ledger8Target::PINNED).unwrap();
        assert!(program.requirements().contains(&cap));
        let rendered = super::super::functional_render::render(&program).unwrap();
        assert_eq!(rendered.merkle.len(), 1);
        assert_eq!(rendered.executor_bounds.len(), 1);
        assert!(
            rendered.executor_bounds[0]
                .to_token_stream()
                .to_string()
                .contains(trait_name)
        );
        let target = Ledger8Target { capabilities: &[] };
        assert!(
            matches!(FunctionalProgram::project(&checked,&authority,target),Err(ProjectionRefusal::MissingTargetCapability{capability,..})if capability==cap)
        );
        let id = program.effects()[0].binding().value.id;
        let missing = Authority {
            missing: Some(id),
            ..Authority::new()
        };
        assert!(matches!(
            FunctionalProgram::project(&checked, &missing, Ledger8Target::PINNED),
            Err(ProjectionRefusal::MissingSourceAuthority { .. })
        ));
    }
}
#[test]
fn merkle_packaging_keeps_full_path_depth_history_and_unused_output() {
    let field = LedgerField {
        source: None,
        id: "tree".into(),
        index: 0,
        path: vec![1, 2],
        declaration: LedgerFieldKind::HistoricMerkleTree {
            depth: 3,
            ty: crate::ir::Type::Field,
        },
    };
    let root = StatefulCircuit {
        source: None,
        name: "root".into(),
        internal: false,
        parameters: vec![],
        actions: vec![],
        result: crate::ir::Type::Boolean,
        return_value: StateReturn::HistoricMerkleIsFull {
            field: "tree".into(),
            index: 0,
        },
    };
    let mut contract = Contract {
        schema_version: crate::SCHEMA_VERSION,
        type_aliases: vec![],
        ledger_fields: vec![field],
        constructor: None,
        witnesses: vec![],
        circuits: vec![],
        stateful_circuits: vec![root],
    };
    let authority = Authority::new();
    let rendered = super::super::functional_items::render_contract(&contract, &authority).unwrap();
    let text = rendered.module.to_token_stream().to_string();
    assert!(text.contains("MerkleBinding :: new"));
    assert!(text.contains(&quote::quote!(&[1u8, 2u8], 3u8, true).to_string()));
    assert!(text.contains("bind_merkle (MERKLE)"));
    contract.stateful_circuits[0].result = crate::ir::Type::Unit;
    contract.stateful_circuits[0].return_value = StateReturn::Unit;
    let rendered = super::super::functional_items::render_contract(&contract, &authority).unwrap();
    let text = rendered.module.to_token_stream().to_string();
    assert!(!text.contains("const MERKLE"));
    assert!(!text.contains("bind_merkle"));
    assert!(text.contains("with_counters (CELLS , COUNTERS)"));
}
