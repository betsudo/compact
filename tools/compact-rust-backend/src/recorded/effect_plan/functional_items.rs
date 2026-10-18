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

//! Immutable request packaging. The checked graph owns semantics; this module
//! only gives its single interpreter-polymorphic body a developer-facing home.
use super::functional_paths::rust_type;
use super::{functional::FunctionalProgram, functional_render, invalid};
use crate::{RecordingGap, ident};
use std::collections::BTreeMap;

/// Rust syntax and its review-only assets travel together until publication.
/// Asset names are private locators, never circuit or execution identities.
pub(crate) struct FunctionalArtifacts {
    pub module: syn::ItemMod,
    pub assets: BTreeMap<String, String>,
}

#[cfg(test)]
#[path = "functional_artifact_tests.rs"]
mod artifact_tests;

/// Issued by retained frontend ingress for the original borrowed circuit only.
/// This is an occurrence/snapshot binding, not consumed-source authentication.
pub(crate) trait FunctionalSources {
    fn root<'a>(
        &'a self,
        original: &crate::ir::StatefulCircuit,
    ) -> Option<&'a crate::ir::SourceLocation>;
}

struct RootOrigin<'a> {
    name: &'a str,
    source: &'a crate::ir::SourceLocation,
}
impl super::functional::SourceAuthority for RootOrigin<'_> {
    fn origin<'source>(
        &'source self,
        site: super::functional::SourceSite<'_>,
    ) -> super::functional::OriginEvidence<'source> {
        match site {
            super::functional::SourceSite::Root { name } if name != self.name => {
                super::functional::OriginEvidence::Unknown
            }
            super::functional::SourceSite::Root { .. } => {
                super::functional::OriginEvidence::BoundOriginalRootFallback(self.source)
            }
            super::functional::SourceSite::Value {
                id,
                diagnostic_path,
            } => {
                // These labels are diagnostic only; the original-root binding
                // below intentionally makes no exact per-node span assertion.
                let _diagnostic_labels = (id, diagnostic_path);
                super::functional::OriginEvidence::BoundOriginalRootFallback(self.source)
            }
        }
    }
}

/// Retains the original circuit -> checked graph association within this call.
/// Coarse fallback is explicit: no operation's exact source span is invented.
pub(crate) fn render_contract(
    contract: &crate::ir::Contract,
    sources: &dyn FunctionalSources,
) -> Result<FunctionalArtifacts, RecordingGap> {
    let ledger = contract
        .ledger_fields
        .iter()
        .map(|field| (field.id.as_str(), field))
        .collect();
    let witnesses = contract
        .witnesses
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let pure = contract
        .circuits
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let circuits = contract
        .stateful_circuits
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect();
    let mut items = Vec::new();
    let mut assets = BTreeMap::new();
    for (ordinal, original) in contract
        .stateful_circuits
        .iter()
        .filter(|item| !item.internal)
        .enumerate()
    {
        let source = sources.root(original).ok_or_else(|| {
            invalid(
                "functional.source",
                format!("no retained original source binding for {}", original.name),
            )
        })?;
        let checked =
            super::check_with_calls_observed(original, &ledger, &witnesses, &pure, &circuits, None)
                .map_err(|error| invalid("functional.check", format!("{error:?}")))?;
        let origin = RootOrigin {
            name: &original.name,
            source,
        };
        let program = FunctionalProgram::project(
            &checked,
            &origin,
            super::functional::Ledger8Target::PINNED,
        )
        .map_err(|error| {
            invalid(
                "functional.projection",
                format!(
                    "{error:?}; coarse original root at {}:{}:{} (not an exact failed-node span)",
                    source.file, source.line, source.column,
                ),
            )
        })?;
        // Only a compiler-assigned ordinal enters the filename. Source names,
        // including normalized collisions and path characters, are data only.
        let asset_path = format!("checked-programs/program-{ordinal}.json");
        items.push(render(&program, &asset_path)?);
        assets.insert(asset_path, super::functional_metadata::metadata(&program));
    }
    let module = syn::parse_quote! {
        /// Experimental immutable requests over the compiler's checked effect graph.
        pub mod functional {
            #(#items)*
        }
    };
    Ok(FunctionalArtifacts { module, assets })
}

fn render(
    program: &FunctionalProgram<'_, '_>,
    metadata_path: &str,
) -> Result<syn::ItemMod, RecordingGap> {
    let checked = program.checked();
    let root = checked.declarations().root().ok_or_else(|| {
        invalid(
            "functional.root",
            "checked functional root declaration is missing",
        )
    })?;
    let module = ident(&root.name).map_err(|e| invalid("functional.root", e.to_string()))?;
    let result =
        rust_type(&root.result).map_err(|e| invalid("functional.result", e.to_string()))?;
    let allocated =
        crate::naming::parameter_idents(&root.parameters, &["self", "__compact_executor"]);
    let parameters = root
        .parameters
        .iter()
        .zip(allocated)
        .map(|(parameter, name)| {
            let ty = rust_type(&parameter.ty)
                .map_err(|e| invalid("functional.parameters", e.to_string()))?;
            Ok((name, ty))
        })
        .collect::<Result<Vec<_>, RecordingGap>>()?;
    let names: Vec<_> = parameters.iter().map(|(name, _)| name).collect();
    let types: Vec<_> = parameters.iter().map(|(_, ty)| ty).collect();
    let locals: Vec<_> = parameters
        .iter()
        .enumerate()
        .map(|(index, (name, _))| {
            let local = quote::format_ident!("__compact_param_{index}");
            syn::parse_quote!(let #local = self.#name;)
        })
        .collect::<Vec<syn::Stmt>>();
    let rendered = functional_render::render(program)?;
    let slot_bindings = rendered
        .slots
        .iter()
        .map(|slot| {
            let id = slot.id;
            let declaration = checked
                .declarations()
                .field(slot.declaration)
                .ok_or_else(|| invalid("functional.slot", "checked Cell declaration missing"))?;
            let path = declaration.physical_path();
            // Physical bindings come only from the checked declaration registry.
            Ok(
                quote::quote!(crate::runtime::functional::ledger8::CellBinding::new(
                    crate::runtime::functional::SlotId::new(#id), &[#(#path),*],
                )),
            )
        })
        .collect::<Result<Vec<_>, RecordingGap>>()?;
    let counter_bindings = rendered
        .counters
        .iter()
        .map(|slot| {
            let declaration = checked
                .declarations()
                .field(slot.declaration)
                .ok_or_else(|| invalid("functional.slot", "checked Counter declaration missing"))?;
            let path = declaration.physical_path();
            let id = slot.id;
            Ok(
                quote::quote!(crate::runtime::functional::ledger8::CounterBinding::new(
                    crate::runtime::functional::SlotId::new(#id), &[#(#path),*],
                )),
            )
        })
        .collect::<Result<Vec<_>, RecordingGap>>()?;
    let map_bindings = rendered
        .maps
        .iter()
        .map(|slot| {
            let declaration = checked
                .declarations()
                .field(slot.declaration)
                .ok_or_else(|| invalid("functional.slot", "checked Map declaration missing"))?;
            let path = declaration.physical_path();
            let id = slot.id;
            Ok(
                quote::quote!(crate::runtime::functional::ledger8::MapBinding::new(
                    crate::runtime::functional::SlotId::new(#id), &[#(#path),*],
                )),
            )
        })
        .collect::<Result<Vec<_>, RecordingGap>>()?;
    // Keep non-Map request packaging byte-stable.
    let has_maps = !map_bindings.is_empty();
    let map_constants = if !has_maps {
        quote::quote!()
    } else {
        quote::quote!(const MAPS: &[crate::runtime::functional::ledger8::MapBinding] = &[#(#map_bindings),*];)
    };
    let ledger_bindings = if !has_maps {
        quote::quote!(
            crate::runtime::functional::ledger8::Ledger8Bindings::with_counters(CELLS, COUNTERS)
        )
    } else {
        quote::quote!(
            crate::runtime::functional::ledger8::Ledger8Bindings::with_maps(CELLS, COUNTERS, MAPS)
        )
    };
    let witness_adapters =
        super::functional_witnesses::adapters(checked.declarations(), &rendered.witnesses)?;
    let body = rendered.body;
    let bounds = rendered.executor_bounds;
    let (witness_generic, witness_argument, native, recorded) = if rendered.witnesses.is_empty() {
        (
            quote::quote!(),
            quote::quote!(),
            quote::quote!(crate::runtime::functional::ledger8::NativeLedger8::new(
                context,
                ledger8_bindings()?
            )),
            quote::quote!(crate::runtime::functional::ledger8::RecordedLedger8::new(
                context,
                ledger8_bindings()?
            )),
        )
    } else {
        (
            quote::quote!(, W: crate::ledger_contract::TryWitnesses<Private>),
            quote::quote!(, witnesses: &W),
            quote::quote!(
                crate::runtime::functional::ledger8::NativeLedger8::new(
                    context,
                    ledger8_bindings()?
                )
                .with_witnesses(WitnessAdapter(witnesses))
            ),
            quote::quote!(
                crate::runtime::functional::ledger8::RecordedLedger8::new(
                    context,
                    ledger8_bindings()?
                )
                .with_witnesses(WitnessAdapter(witnesses))
            ),
        )
    };
    let default: Option<syn::ItemImpl> = if names.is_empty() {
        Some(syn::parse_quote! {
            impl Default for Program {
                fn default() -> Self { Self::new() }
            }
        })
    } else {
        None
    };
    let source_module_lint: Option<syn::Attribute> = (module == "functional").then(|| {
        syn::parse_quote!(
            #[allow(clippy::module_inception, reason = "preserves the Compact circuit name")]
        )
    });
    Ok(syn::parse_quote! {
        #source_module_lint
        pub mod #module {

            /// Review metadata only; never consumed as execution authority.
            pub const CHECKED_PROGRAM: &str = include_str!(#metadata_path);

            const CELLS: &[crate::runtime::functional::ledger8::CellBinding] = &[#(#slot_bindings),*];
            const COUNTERS: &[crate::runtime::functional::ledger8::CounterBinding] = &[#(#counter_bindings),*];
            #map_constants

            /// Adapts this entry's retained witness identities to existing callbacks.
            pub struct WitnessAdapter<'a, W>(pub &'a W);
            #(#witness_adapters)*

            pub fn ledger8_bindings()
                -> Result<crate::runtime::functional::ledger8::Ledger8Bindings<'static>, crate::runtime::CompactError>
            {
                #ledger_bindings
            }

            /// Immutable checked request. Creating it performs no ledger or witness work.
            /// Execution consumes the request and its selected interpreter.
            pub struct Program {
                #(#names: #types),*
            }

            #default

            impl Program {
                pub fn new(#(#names: #types),*) -> Self {
                    Self { #(#names),* }
                }

                /// Execute with this program's compiler-issued ledger8 bindings.
                pub fn execute_native<Private #witness_generic>(
                    self, context: crate::runtime::context::CircuitContext<Private> #witness_argument,
                ) -> Result<crate::runtime::context::CircuitResult<Private, #result>, crate::runtime::CompactError> {
                    self.run(#native)
                }

                /// Record with the same checked body and compiler-issued bindings.
                pub fn execute_recorded<Private #witness_generic>(
                    self, context: crate::runtime::context::CircuitContext<Private> #witness_argument,
                ) -> Result<crate::runtime::recording::RecordedCircuitResult<Private, #result>, crate::runtime::CompactError> {
                    self.run(#recorded)
                }

                /// One checked body shared by native and recorded target adapters.
                pub fn run<E>(self, __compact_executor: E)
                    -> Result<E::Outcome<#result>, crate::runtime::CompactError>
                where E: crate::runtime::functional::Executor #(+ #bounds)*
                {
                    #(#locals)*
                    (#body).finish()
                }
            }
        }
    })
}
