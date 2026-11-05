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

//! Generate the fallible companion of a Compact witness trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, GenericParam, ItemTrait, Pat, ReturnType, TraitItem, Type};

// These two generated signature style attributes do not change the method's
// type or availability. Keep rejecting cfg/semantic attributes and broad lint
// groups; forward the permitted style exceptions to both companion surfaces.
fn supported_method_attribute(attribute: &syn::Attribute) -> bool {
    if attribute.path().is_ident("doc") {
        return true;
    }
    if !attribute.path().is_ident("allow") {
        return false;
    }
    let Ok(entries) = attribute.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    let mut lints = 0;
    let mut reason = false;
    for entry in entries {
        match entry {
            syn::Meta::Path(path)
                if path.leading_colon.is_none()
                    && path.segments.len() == 2
                    && path.segments[0].ident == "clippy"
                    && matches!(path.segments[0].arguments, syn::PathArguments::None)
                    && matches!(path.segments[1].arguments, syn::PathArguments::None)
                    && (path.segments[1].ident == "too_many_arguments"
                        || path.segments[1].ident == "type_complexity") =>
            {
                lints += 1;
            }
            syn::Meta::NameValue(value) if value.path.is_ident("reason") && !reason => {
                if !matches!(
                    value.value,
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(_),
                        ..
                    })
                ) {
                    return false;
                }
                reason = true;
            }
            _ => return false,
        }
    }
    lints > 0
}

pub(crate) fn expand(input: ItemTrait) -> syn::Result<TokenStream> {
    if input.ident != "Witnesses"
        || input.generics.params.len() != 1
        || input.generics.where_clause.is_some()
        || !input.supertraits.is_empty()
        || input.unsafety.is_some()
        || input.auto_token.is_some()
    {
        return Err(syn::Error::new_spanned(
            &input,
            "expected generated trait Witnesses<Private> without bounds",
        ));
    }
    let Some(GenericParam::Type(private)) = input.generics.params.first() else {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "expected a Private type parameter",
        ));
    };
    if private.ident != "Private" || !private.bounds.is_empty() || private.default.is_some() {
        return Err(syn::Error::new_spanned(
            private,
            "expected an unbounded Private type parameter without a default",
        ));
    }

    let visibility = &input.vis;
    let mut fallible_methods = Vec::new();
    let mut adapter_methods = Vec::new();
    for item in &input.items {
        let TraitItem::Fn(method) = item else {
            return Err(syn::Error::new_spanned(item, "expected witness method"));
        };
        if method.default.is_some()
            || !method.sig.generics.params.is_empty()
            || method.sig.generics.where_clause.is_some()
            || method.sig.constness.is_some()
            || method.sig.asyncness.is_some()
            || method.sig.unsafety.is_some()
            || method.sig.abi.is_some()
            || method.sig.variadic.is_some()
            || method
                .attrs
                .iter()
                .any(|attr| !supported_method_attribute(attr))
        {
            return Err(syn::Error::new_spanned(
                method,
                "expected a generated witness method signature",
            ));
        }
        let mut inputs = method.sig.inputs.iter();
        if !matches!(inputs.next(), Some(FnArg::Receiver(receiver)) if receiver.reference.is_some() && receiver.mutability.is_none())
        {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "expected a shared &self receiver",
            ));
        }
        let mut argument_names = Vec::new();
        for input in inputs {
            let FnArg::Typed(parameter) = input else {
                return Err(syn::Error::new_spanned(input, "expected typed argument"));
            };
            let Pat::Ident(name) = parameter.pat.as_ref() else {
                return Err(syn::Error::new_spanned(
                    &parameter.pat,
                    "expected named witness argument",
                ));
            };
            if name.by_ref.is_some() || name.mutability.is_some() || name.subpat.is_some() {
                return Err(syn::Error::new_spanned(
                    name,
                    "expected a plain witness argument name",
                ));
            }
            argument_names.push(&name.ident);
        }
        if !argument_names
            .first()
            .is_some_and(|name| *name == "context")
        {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "expected a named context argument",
            ));
        }
        let ReturnType::Type(_, output) = &method.sig.output else {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "expected a witness output type",
            ));
        };
        let Type::Tuple(pair) = output.as_ref() else {
            return Err(syn::Error::new_spanned(
                output,
                "expected a (Private, Value) witness output",
            ));
        };
        if pair.elems.len() != 2
            || !matches!(pair.elems.first(), Some(Type::Path(path)) if path.qself.is_none() && path.path.is_ident("Private"))
        {
            return Err(syn::Error::new_spanned(
                output,
                "expected a (Private, Value) witness output",
            ));
        }
        let mut fallible_signature = method.sig.clone();
        fallible_signature.output = syn::parse_quote!(-> ::core::result::Result<#output, ::midnight_compact_runtime::CompactError>);
        let name = &method.sig.ident;
        let attrs = &method.attrs;
        let style_attrs = attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("allow"));
        fallible_methods.push(quote! { #(#attrs)* #fallible_signature; });
        adapter_methods.push(quote! {
            #(#style_attrs)*
            #fallible_signature {
                ::core::result::Result::Ok(<W as Witnesses<Private>>::#name(self, #(#argument_names),*))
            }
        });
    }

    Ok(quote! {
        #input
        /// Witness methods that can propagate ledger projection failures.
        #visibility trait TryWitnesses<Private> {
            #(#fallible_methods)*
        }
        impl<Private, W: Witnesses<Private>> TryWitnesses<Private> for W {
            #(#adapter_methods)*
        }
    })
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::ToTokens;
    use syn::{ItemTrait, parse_quote};

    #[test]
    fn rejects_bounded_and_defaulted_private_state() {
        let bounded: ItemTrait = parse_quote!(
            pub trait Witnesses<Private: Clone> {}
        );
        assert!(expand(bounded).is_err());
        let defaulted: ItemTrait = parse_quote!(
            pub trait Witnesses<Private = ()> {}
        );
        assert!(expand(defaulted).is_err());
    }

    #[test]
    fn rejects_unsupported_witness_method_signatures() {
        for input in [
            "pub trait Witnesses<Private> { const fn secret(&self, context: u64) -> (Private, ()); }",
            "pub trait Witnesses<Private> { extern \"C\" fn secret(&self, context: u64) -> (Private, ()); }",
            "pub trait Witnesses<Private> { fn secret(&self, context: u64) -> (Private, ()) where Private: Clone; }",
            "pub trait Witnesses<Private> { fn secret(&self, seed: u64) -> (Private, ()); }",
            "pub trait Witnesses<Private> { fn secret(&self, context: u64) -> u64; }",
        ] {
            let trait_item: ItemTrait = syn::parse_str(input).unwrap();
            assert!(expand(trait_item).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn permits_only_narrow_signature_style_exceptions() {
        for attribute in [
            "#[allow(clippy::type_complexity, reason = \"preserves Compact carriers\")]",
            "#[allow(clippy::too_many_arguments)]",
            "#[allow(clippy::too_many_arguments, clippy::type_complexity)]",
        ] {
            let input: ItemTrait = syn::parse_str(&format!(
                "pub trait Witnesses<Private> {{ {attribute} fn secret(&self, context: u64) -> (Private, u64); }}"
            )).unwrap();
            let file: syn::File = syn::parse2(expand(input).unwrap()).unwrap();
            let [
                syn::Item::Trait(ordinary),
                syn::Item::Trait(fallible),
                syn::Item::Impl(adapter),
            ] = file.items.as_slice()
            else {
                panic!("companion surfaces changed");
            };
            let syn::TraitItem::Fn(ordinary) = &ordinary.items[0] else {
                panic!();
            };
            let syn::TraitItem::Fn(fallible) = &fallible.items[0] else {
                panic!();
            };
            let syn::ImplItem::Fn(adapter) = &adapter.items[0] else {
                panic!();
            };
            assert_eq!(ordinary.attrs.len(), 1);
            let attributes = |attrs: &[syn::Attribute]| {
                attrs
                    .iter()
                    .map(|attribute| attribute.to_token_stream().to_string())
                    .collect::<Vec<_>>()
            };
            assert_eq!(attributes(&fallible.attrs), attributes(&ordinary.attrs));
            assert_eq!(attributes(&adapter.attrs), attributes(&ordinary.attrs));
            assert_eq!(
                fallible.sig.inputs.to_token_stream().to_string(),
                ordinary.sig.inputs.to_token_stream().to_string()
            );
            assert_eq!(
                adapter.sig.to_token_stream().to_string(),
                fallible.sig.to_token_stream().to_string()
            );
        }
        for attribute in [
            "#[cfg(any())]",
            "#[inline]",
            "#[expect(clippy::type_complexity)]",
            "#[allow(clippy::all)]",
            "#[allow(dead_code)]",
            "#[allow()]",
            "#[allow(clippy::type_complexity, dead_code)]",
            "#[allow(clippy::type_complexity, reason = 1)]",
            "#[allow(clippy::type_complexity, reason = \"a\", reason = \"b\")]",
            "#[allow(reason = \"no lint\")]",
        ] {
            let input: ItemTrait = syn::parse_str(&format!(
                "pub trait Witnesses<Private> {{ {attribute} fn secret(&self, context: u64) -> (Private, u64); }}"
            )).unwrap();
            assert!(expand(input).is_err(), "accepted unsupported {attribute}");
        }
    }

    #[test]
    fn preserves_method_docs_on_both_public_traits() {
        let input: ItemTrait = parse_quote! {
            pub trait Witnesses<Private> {
                /// Reads the secret value.
                fn secret(&self, context: u64) -> (Private, u64);
            }
        };
        let source = expand(input).unwrap().to_string();
        assert_eq!(source.matches("Reads the secret value.").count(), 2);
    }
}

#[cfg(test)]
#[path = "tests/witness_bridge.rs"]
mod validation_tests;
