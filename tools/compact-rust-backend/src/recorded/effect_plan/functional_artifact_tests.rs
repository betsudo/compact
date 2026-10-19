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

//! Asset ownership controls use an authority double, not frontend issuance.
use super::*;
use crate::ir::{Contract, SourceLocation, StatefulCircuit};
use quote::ToTokens;

struct BoundRoot(SourceLocation);
impl FunctionalSources for BoundRoot {
    fn root<'a>(&'a self, _: &StatefulCircuit) -> Option<&'a SourceLocation> {
        Some(&self.0)
    }
}

fn contract(names: &[&str]) -> Contract {
    serde_json::from_value(serde_json::json!({
        "schema_version": crate::SCHEMA_VERSION,
        "ledger_fields": [],
        "witnesses": [],
        "circuits": [],
        "stateful_circuits": names.iter().map(|name| serde_json::json!({
            "name": name, "actions": [], "result": {"kind": "unit"}
        })).collect::<Vec<_>>()
    }))
    .unwrap()
}

fn sources() -> BoundRoot {
    BoundRoot(SourceLocation {
        file: "fixture.compact".into(),
        line: 1,
        column: 1,
    })
}

#[test]
fn review_constants_reference_their_own_relative_assets_and_keep_str_api() {
    // Locator isolation only: these normalized collisions bypass the real
    // frontend namespace check, so this synthetic module is not a valid crate.
    let contract = contract(&["runtime", "functional", "a$b", "a_b", "match"]);
    let sources = sources();
    let rendered = render_contract(&contract, &sources).unwrap();
    let modules = &rendered.module.content.as_ref().unwrap().1;
    assert_eq!(modules.len(), contract.stateful_circuits.len());
    assert_eq!(rendered.assets.len(), modules.len());
    for (index, (item, original)) in modules.iter().zip(&contract.stateful_circuits).enumerate() {
        let syn::Item::Mod(module) = item else {
            panic!("expected generated entry module");
        };
        let constant = module
            .content
            .as_ref()
            .unwrap()
            .1
            .iter()
            .find_map(|item| match item {
                syn::Item::Const(item) if item.ident == "CHECKED_PROGRAM" => Some(item),
                _ => None,
            })
            .unwrap();
        assert_eq!(constant.ty.to_token_stream().to_string(), "& str");
        let syn::Expr::Macro(inclusion) = constant.expr.as_ref() else {
            panic!("review data must be included, not embedded in Rust syntax");
        };
        assert!(inclusion.mac.path.is_ident("include_str"));
        let path = syn::parse2::<syn::LitStr>(inclusion.mac.tokens.clone())
            .unwrap()
            .value();
        assert_eq!(path, format!("checked-programs/program-{index}.json"));
        let metadata: serde_json::Value =
            serde_json::from_str(rendered.assets.get(&path).unwrap()).unwrap();
        assert_eq!(metadata["root"], original.name);
        assert_eq!(metadata["purpose"], "review_only_not_execution_authority");
        assert_eq!(metadata["root_origin"]["consumed_source_bytes"], "unknown");
    }
    assert_eq!(
        rendered.assets,
        render_contract(&contract, &sources).unwrap().assets
    );
}

#[test]
fn internal_entries_do_not_allocate_unreferenced_assets() {
    let mut contract = contract(&["helper", "visible", "another"]);
    contract.stateful_circuits[0].internal = true;
    let rendered = render_contract(&contract, &sources()).unwrap();
    assert_eq!(
        rendered
            .assets
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "checked-programs/program-0.json",
            "checked-programs/program-1.json"
        ]
    );
    let roots: Vec<serde_json::Value> = rendered
        .assets
        .values()
        .map(|text| serde_json::from_str::<serde_json::Value>(text).unwrap()["root"].clone())
        .collect();
    assert_eq!(roots, vec!["visible", "another"]);
}
