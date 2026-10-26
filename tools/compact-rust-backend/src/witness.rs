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

//! Typed witness declarations and contract ledger projection syntax.

use std::collections::{HashMap, HashSet};

use proc_macro2::Span;

use crate::ir::{LedgerField, LedgerFieldKind, WitnessDeclaration};
use crate::{RenderError, located};
use crate::{
    native_header,
    source_diagnostics::{Owner, RefusalObserver, RefusalSite},
};

pub(crate) struct WitnessSyntax<'a> {
    pub declarations: HashMap<&'a str, &'a WitnessDeclaration>,
    pub trait_methods: Vec<syn::TraitItemFn>,
    pub ledger_view_methods: Vec<syn::ImplItemFn>,
}

pub(crate) fn build<'a>(
    witnesses: &'a [WitnessDeclaration],
    ledger_fields: &[&LedgerField],
    observer: Option<&dyn RefusalObserver>,
) -> Result<WitnessSyntax<'a>, RenderError> {
    let mut declarations = HashMap::new();
    let mut trait_methods = Vec::new();
    for witness in witnesses {
        located(witness.source.as_ref(), || {
            let name = native_header::identifier(
                &witness.name,
                RefusalSite::field(Owner::Witness(witness), "name"),
                observer,
            )?;
            if declarations
                .insert(witness.name.as_str(), witness)
                .is_some()
            {
                if let Some(observer) = observer {
                    observer.refused(RefusalSite::field(Owner::Witness(witness), "name"));
                }
                return Err(RenderError::DuplicateWitness(witness.name.clone()));
            }
            let mut args = Vec::<syn::FnArg>::new();
            let mut parameter_names = HashSet::new();
            for (index, parameter) in witness.parameters.iter().enumerate() {
                native_header::identifier(
                    &parameter.name,
                    RefusalSite::field(Owner::Parameter(parameter), "name"),
                    observer,
                )?;
                if !parameter_names.insert(parameter.name.as_str()) {
                    if let Some(observer) = observer {
                        observer.refused(RefusalSite::field(Owner::Parameter(parameter), "name"));
                    }
                    return Err(RenderError::DuplicateParameter(parameter.name.clone()));
                }
                let parameter_name =
                    syn::Ident::new(&format!("__compact_param_{index}"), Span::call_site());
                let ty = native_header::carrier(&parameter.ty, observer)?;
                args.push(syn::parse_quote!(#parameter_name: #ty));
            }
            let result = native_header::carrier(&witness.result, observer)?;
            trait_methods.push(syn::parse_quote! {
                fn #name(
                    &self,
                    context: runtime::context::WitnessContext<'_, Private, LedgerView<'_>>,
                    #(#args),*
                ) -> (Private, #result);
            });
            Ok(())
        })?;
    }

    let mut ledger_view_methods = Vec::new();
    if !witnesses.is_empty() {
        for field in ledger_fields {
            located(field.source.as_ref(), || {
                let name = native_header::identifier(
                    &field.id,
                    RefusalSite::field(Owner::Ledger(field), "id"),
                    observer,
                )?;
                match &field.declaration {
                    LedgerFieldKind::Cell { ty } => {
                        let ty = native_header::carrier(ty, observer)?;
                        ledger_view_methods.push(syn::parse_quote! {
                            pub fn #name(&self) -> Result<#ty, runtime::CompactError> {
                                crate::ledger_slots::#name.witness_read(self.meter)
                            }
                        });
                    }
                    LedgerFieldKind::Counter => {
                        let max = syn::LitInt::new(&u64::MAX.to_string(), Span::call_site());
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::BoundedUint<#max>, runtime::CompactError> {
                            let value = crate::ledger_slots::#name.witness_read(self.meter)?;
                            runtime::BoundedUint::<#max>::new(value as u128)
                        }
                    });
                    }
                    LedgerFieldKind::Set { ty } => {
                        let ty = native_header::carrier(ty, observer)?;
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::ledger::MeteredSetView<'a, #ty, runtime::ledger::DefaultDB>, runtime::CompactError> {
                            crate::ledger_slots::#name.witness_view(self.meter)
                        }
                    });
                    }
                    LedgerFieldKind::Map { key, value } => {
                        // Keys remain checked even when the scalar view is omitted.
                        let key = native_header::carrier(key, observer)?;
                        // Structural Map values have no CellValue codec. As with
                        // PublicState, omit their scalar-only witness view;
                        // key-only circuit operations still use the typed slot.
                        if matches!(value, crate::ir::Type::LedgerMap { .. }) {
                            return Ok(());
                        }
                        let value = native_header::carrier(value, observer)?;
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::ledger::MeteredMapView<'a, #key, #value, runtime::ledger::DefaultDB>, runtime::CompactError> {
                            crate::ledger_slots::#name.witness_view(self.meter)
                        }
                    });
                    }
                    LedgerFieldKind::List { ty } => {
                        let ty = native_header::carrier(ty, observer)?;
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::ledger::MeteredListView<'a, #ty, runtime::ledger::DefaultDB>, runtime::CompactError> {
                            crate::ledger_slots::#name.witness_view(self.meter)
                        }
                    });
                    }
                    LedgerFieldKind::HistoricMerkleTree { .. } => {
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::ledger::MeteredHistoricMerkleTreeView<'a, crate::types::MerkleTreeDigest, runtime::ledger::DefaultDB>, runtime::CompactError> {
                            crate::ledger_slots::#name.witness_view(self.meter)
                        }
                    });
                    }
                    LedgerFieldKind::MerkleTree { .. } => {
                        ledger_view_methods.push(syn::parse_quote! {
                        pub fn #name(&self) -> Result<runtime::ledger::MeteredMerkleTreeView<'a, crate::types::MerkleTreeDigest, runtime::ledger::DefaultDB>, runtime::CompactError> {
                            crate::ledger_slots::#name.witness_view(self.meter)
                        }
                    });
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(WitnessSyntax {
        declarations,
        trait_methods,
        ledger_view_methods,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Type;

    #[test]
    fn nested_map_does_not_hide_supported_witness_ledger_views() {
        let field = |id: &str, index, declaration| LedgerField {
            id: id.into(),
            index,
            path: vec![1, index],
            source: None,
            declaration,
        };
        let nested = field(
            "outer",
            0,
            LedgerFieldKind::Map {
                key: Type::Boolean,
                value: Type::LedgerMap {
                    key: Box::new(Type::Field),
                    value: Box::new(Type::Field),
                },
            },
        );
        let scalar = field(
            "scalar",
            1,
            LedgerFieldKind::Map {
                key: Type::Boolean,
                value: Type::Field,
            },
        );
        let cell = field("stored", 2, LedgerFieldKind::Cell { ty: Type::Field });
        let witness = WitnessDeclaration {
            source: None,
            name: "mark".into(),
            parameters: vec![],
            result: Type::Field,
        };
        let witnesses = [witness];
        let syntax = build(&witnesses, &[&nested, &scalar, &cell], None).unwrap();
        assert_eq!(syntax.declarations.len(), 1);
        assert_eq!(syntax.trait_methods.len(), 1);
        let names: Vec<_> = syntax
            .ledger_view_methods
            .iter()
            .map(|method| method.sig.ident.to_string())
            .collect();
        assert_eq!(names, ["scalar", "stored"]);
        let methods = syntax.ledger_view_methods;
        let output = quote::quote!(#(#methods)*).to_string();
        assert!(output.contains("MeteredMapView"));
        assert!(output.contains("scalar . witness_view (self . meter)"));
        assert!(output.contains("stored . witness_read (self . meter)"));
        assert!(!output.contains("MapNode"));
    }

    #[test]
    fn scalar_and_structural_map_views_refuse_non_value_key() {
        let invalid_key = Type::LedgerMap {
            key: Box::new(Type::Field),
            value: Box::new(Type::Field),
        };
        let witnesses = [WitnessDeclaration {
            source: None,
            name: "mark".into(),
            parameters: vec![],
            result: Type::Field,
        }];
        for value in [Type::Field, invalid_key.clone()] {
            let field = LedgerField {
                id: "invalid".into(),
                index: 0,
                path: vec![],
                source: None,
                declaration: LedgerFieldKind::Map {
                    key: invalid_key.clone(),
                    value,
                },
            };
            let error = match build(&witnesses, &[&field], None) {
                Ok(_) => panic!("invalid Map key was accepted"),
                Err(error) => error,
            };
            assert!(
                matches!(error, RenderError::UnsupportedLedgerValueType(ty) if ty == invalid_key)
            );
        }
    }
}
