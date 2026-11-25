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

//! Host AST/default-contract controls. Actual old-runtime compilation is a separate gate.
use compact_rust_backend::{
    RenderOptions, RenderedContract, RustCapabilityReport, RustCircuitCapability, ir::Contract,
    render_with_capabilities, render_with_standalone_proof_data,
};
use quote::ToTokens;
use serde_json::json;
use syn::visit_mut::VisitMut;

#[test]
fn old_public_struct_literals_keep_their_exact_field_contract() {
    let _ = RenderOptions {
        apply_frontend_proof_metadata: false,
        require_recording: false,
    };
    let circuit = RustCircuitCapability {
        name: "old".into(),
        source: None,
        recorded: false,
        observed_call: false,
        proof_required: None,
        recording_status: None,
        recording_unavailable: None,
        observed_call_unavailable: None,
    };
    let rendered = RenderedContract {
        source: String::new(),
        capabilities: RustCapabilityReport {
            schema_version: 2,
            circuits: vec![circuit],
        },
    };
    let value = serde_json::to_value(&rendered.capabilities).unwrap();
    assert!(value["circuits"][0].get("standalone_proof_data").is_none());
    assert!(
        value["circuits"][0]
            .get("standalone_proof_data_unavailable")
            .is_none()
    );
}

#[test]
fn default_counter_output_has_no_helper_or_extra_capability_fields() {
    let contract: Contract = serde_json::from_str(include_str!(
        "../fixtures/upstream-672/adt-counter/compact-rust-ir.json"
    ))
    .unwrap();
    let default = render_with_capabilities(&contract).unwrap();
    let opted = render_with_standalone_proof_data(&contract).unwrap();
    assert!(!default.source.contains("runtime::proof_data"));
    let old_report = serde_json::to_value(&default.capabilities).unwrap();
    for circuit in old_report["circuits"].as_array().unwrap() {
        assert!(circuit.get("standalone_proof_data").is_none());
        assert!(circuit.get("standalone_proof_data_unavailable").is_none());
    }
    struct StripHelpers;
    impl VisitMut for StripHelpers {
        fn visit_item_impl_mut(&mut self, item: &mut syn::ItemImpl) {
            item.items.retain(|item| {
                !matches!(item, syn::ImplItem::Fn(method)
                if method.sig.ident.to_string().ends_with("_proof_data"))
            });
            syn::visit_mut::visit_item_impl_mut(self, item);
        }
    }
    let default = syn::parse_file(&default.source).unwrap();
    let mut opted = syn::parse_file(&opted.source).unwrap();
    StripHelpers.visit_file_mut(&mut opted);
    assert_eq!(
        default.to_token_stream().to_string(),
        opted.to_token_stream().to_string()
    );
}

#[test]
fn every_direct_native_and_recorded_time_operation_links_its_domain_version() {
    for kind in [
        "kernel_block_time_less_than",
        "kernel_block_time_greater_than",
    ] {
        let contract: Contract = serde_json::from_value(json!({
            "schema_version":20,"ledger_fields":[],"circuits":[],
            "stateful_circuits":[{"name":"time","parameters":[],"actions":[],
                "result":{"kind":"boolean"},"return_value":{"kind":"expression","value":{
                    "kind":kind,"bound":{"kind":"unsigned_literal","value":"1","max":"18446744073709551615"}
                }}}]
        })).unwrap();
        struct Calls {
            native: usize,
            recorded: usize,
        }
        impl Calls {
            fn check(&mut self, block: &syn::Block) {
                let source = block.to_token_stream().to_string();
                if !source.contains(". kernel_block_time_") {
                    return;
                }
                assert!(
                    source.contains(
                        "const _ : [() ; 1] = [() ; runtime :: BLOCK_TIME_API_VERSION as usize]"
                    ),
                    "{source}"
                );
                if source.contains("context . kernel_block_time_") {
                    self.native += 1;
                }
                if source.contains("frame . kernel_block_time_") {
                    self.recorded += 1;
                }
            }
        }
        impl VisitMut for Calls {
            fn visit_item_fn_mut(&mut self, item: &mut syn::ItemFn) {
                self.check(&item.block);
                syn::visit_mut::visit_item_fn_mut(self, item);
            }
            fn visit_impl_item_fn_mut(&mut self, item: &mut syn::ImplItemFn) {
                self.check(&item.block);
                syn::visit_mut::visit_impl_item_fn_mut(self, item);
            }
        }
        let mut file =
            syn::parse_file(&render_with_capabilities(&contract).unwrap().source).unwrap();
        let mut calls = Calls {
            native: 0,
            recorded: 0,
        };
        calls.visit_file_mut(&mut file);
        assert!(calls.native > 0 && calls.recorded > 0);
    }
}
