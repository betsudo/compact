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

//! Typed witness signatures and mechanical forwarding, separate from execution.
use super::{
    declarations::Declarations, functional_paths::rust_type, functional_render::FunctionalWitness,
    invalid, model::Value,
};
use crate::{
    RecordingGap, RenderError,
    ir::{Type, WitnessDeclaration},
};
use quote::{ToTokens, format_ident};
use std::collections::BTreeMap;

/// Checked declaration identity remains authoritative. Exact opaque output
/// transport is a separate adapter; never silently use aligned conversion for it.
pub(super) fn supported(
    declaration: Option<&WitnessDeclaration>,
    arguments: &[Value],
    result: &Type,
) -> bool {
    !crate::pure_semantics::hash_encoding::contains_opaque(result)
        && declaration.is_some_and(|declaration| {
            declaration.result == *result
                && declaration.parameters.len() == arguments.len()
                && declaration
                    .parameters
                    .iter()
                    .zip(arguments)
                    .all(|(parameter, argument)| parameter.ty == argument.ty)
        })
}

/// Preserve the original single-argument surface, using tuples only to carry
/// multiple checked arguments. Zero arguments use Unit.
pub(super) fn argument_type(types: &[Type]) -> Result<syn::Type, RenderError> {
    match types {
        [] => Ok(syn::parse_quote!(())),
        [ty] => rust_type(ty),
        // Reuse the canonical carrier spelling: a single tuple argument and
        // multiple arguments can have the same Rust type and must share an impl.
        _ => rust_type(&Type::Tuple {
            elements: types.to_vec(),
        }),
    }
}
pub(super) fn argument_value(values: Vec<syn::Expr>) -> syn::Expr {
    match values.as_slice() {
        [] => syn::parse_quote!(()),
        [value] => value.clone(),
        _ => syn::parse_quote!((#(#values,)*)),
    }
}

struct HandlerGroup {
    arguments: syn::Type,
    result: syn::Type,
    cases: Vec<syn::Arm>,
}

/// Equal emitted Rust signatures share one impl. Declaration IDs still select
/// distinct callbacks, including zero-vs-Unit and single-product-vs-many args.
pub(super) fn adapters(
    declarations: &Declarations,
    witnesses: &[FunctionalWitness],
) -> Result<Vec<syn::Item>, RecordingGap> {
    let mut groups = BTreeMap::<(String, String), HandlerGroup>::new();
    for witness in witnesses {
        let declaration = declarations
            .witness(witness.declaration)
            .ok_or_else(|| invalid("functional.witness", "checked witness declaration missing"))?;
        let types = declaration
            .parameters
            .iter()
            .map(|parameter| parameter.ty.clone())
            .collect::<Vec<_>>();
        let arguments = argument_type(&types)
            .map_err(|error| invalid("functional.witness", error.to_string()))?;
        let result = rust_type(&declaration.result)
            .map_err(|error| invalid("functional.witness", error.to_string()))?;
        let key = (
            arguments.to_token_stream().to_string(),
            result.to_token_stream().to_string(),
        );
        let method = crate::ident(&declaration.name)
            .map_err(|error| invalid("functional.witness", error.to_string()))?;
        let names = (0..types.len())
            .map(|index| format_ident!("__compact_witness_arg_{index}"))
            .collect::<Vec<_>>();
        let unpack: Option<syn::Stmt> = match names.as_slice() {
            [] => Some(syn::parse_quote!(let () = arguments;)),
            [name] => Some(syn::parse_quote!(let #name = arguments;)),
            _ => Some(syn::parse_quote!(let (#(#names,)*) = arguments;)),
        };
        let id = witness.id;
        let arm: syn::Arm = syn::parse_quote! {
            #id => {
                #unpack
                self.0.#method(context.witness_context_with(crate::ledger_contract::LedgerView {
                    state: context.query.state.get_ref(), meter,
                }), #(#names),*)
            }
        };
        groups
            .entry(key)
            .or_insert_with(|| HandlerGroup {
                arguments,
                result,
                cases: Vec::new(),
            })
            .cases
            .push(arm);
    }
    Ok(groups.into_values().map(|HandlerGroup { arguments, result, cases }| {
        syn::parse_quote! {
            impl<Private, W: crate::ledger_contract::TryWitnesses<Private>>
                crate::runtime::functional::ledger8::WitnessHandler<Private, crate::runtime::ledger::DefaultDB, #arguments, #result>
                for WitnessAdapter<'_, W>
            {
                fn call(&self, id: crate::runtime::functional::WitnessId, arguments: #arguments,
                    context: &crate::runtime::context::CircuitContext<Private>,
                    meter: &crate::runtime::context::WitnessReadMeter<'_>,
                ) -> Result<(Private, #result), crate::runtime::CompactError> {
                    match id.index() {
                        #(#cases,)*
                        _ => Err(crate::runtime::CompactError::InvalidLedgerCell("unknown checked witness identity".into())),
                    }
                }
            }
        }
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Parameter;
    fn declaration(result: Type) -> WitnessDeclaration {
        WitnessDeclaration {
            source: None,
            name: "typed".into(),
            parameters: vec![Parameter {
                name: "input".into(),
                ty: Type::Field,
            }],
            result,
        }
    }
    #[test]
    fn checked_signature_mismatch_refuses_before_adapter_generation() {
        let declaration = declaration(Type::Field);
        let argument = Value {
            id: 0,
            ty: Type::Field,
        };
        assert!(supported(
            Some(&declaration),
            std::slice::from_ref(&argument),
            &Type::Field
        ));
        assert!(!supported(
            None,
            std::slice::from_ref(&argument),
            &Type::Field
        ));
        assert!(!supported(Some(&declaration), &[], &Type::Field));
        assert!(!supported(
            Some(&declaration),
            &[Value {
                id: 0,
                ty: Type::Boolean
            }],
            &Type::Field
        ));
        assert!(!supported(Some(&declaration), &[argument], &Type::Boolean));
    }
    #[test]
    fn exact_opaque_results_are_not_silently_encoded_as_aligned_values() {
        for result in [
            Type::OpaqueBytes,
            Type::OpaqueString,
            Type::Tuple {
                elements: vec![Type::Field, Type::OpaqueBytes],
            },
        ] {
            let declaration = declaration(result.clone());
            assert!(!supported(
                Some(&declaration),
                &[Value {
                    id: 0,
                    ty: Type::Field
                }],
                &result
            ));
        }
    }
}
