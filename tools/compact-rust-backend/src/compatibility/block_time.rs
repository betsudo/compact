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

//! Conditional developer API check, not source authentication or SDK typechecking.
use super::{ValidatedRuntime, source_error};
use crate::ir::Contract;
use quote::ToTokens;
use serde_json::Value;
use std::{collections::BTreeSet, error::Error, fs};

impl ValidatedRuntime {
    /// Old/default modules retain their prior route. The caller renders before
    /// this check and publishes afterward. Actual typed compilation is separate.
    pub fn require_block_time(&self, contract: &Contract) -> Result<(), Box<dyn Error>> {
        let value = serde_json::to_value(contract)?;
        let mut pending = vec![&value];
        let mut methods = BTreeSet::new();
        while let Some(value) = pending.pop() {
            match value {
                Value::Object(object) => {
                    match object.get("kind").and_then(Value::as_str) {
                        Some("kernel_block_time_less_than") => {
                            methods.insert("kernel_block_time_less_than");
                        }
                        Some("kernel_block_time_greater_than") => {
                            methods.insert("kernel_block_time_greater_than");
                        }
                        _ => {}
                    }
                    pending.extend(object.values());
                }
                Value::Array(array) => pending.extend(array),
                _ => {}
            }
        }
        if methods.is_empty() {
            return Ok(());
        }
        let parse = |relative: &str| -> Result<syn::File, Box<dyn Error>> {
            let path = self.root().join(relative);
            let bytes = fs::read(&path)
                .map_err(|e| source_error("read block-time runtime source", &path, e))?;
            let hash = format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(&bytes));
            if self.source_sha256.get(relative) != Some(&hash) {
                return Err(source_error(
                    "block-time source fingerprint changed",
                    &path,
                    "reselect the runtime source",
                ));
            }
            syn::parse_file(std::str::from_utf8(&bytes)?)
                .map_err(|e| source_error("parse block-time runtime source", &path, e))
        };
        let root = parse("runtime-rs/src/lib.rs")?;
        for (module, owner, output) in [
            (
                "context",
                "CircuitContext",
                "::core::result::Result<super::CircuitResult<Private, ::core::primitive::bool, D>, crate::CompactError>",
            ),
            (
                "recording",
                "RecordingFrame",
                "::core::result::Result<(Self, ::core::primitive::bool), crate::CompactError>",
            ),
        ] {
            let parent = parse(&format!("runtime-rs/src/{module}.rs"))?;
            let implementation = parse(&format!("runtime-rs/src/{module}/block_time.rs"))?;
            if !external_module(&root, module, true)
                || !external_module(&parent, "block_time", false)
                || !api_methods(&implementation, owner, output, &methods)
            {
                return Err(format!("Rust block-time queries require matching runtime source: {module} must expose the selected consuming {owner} methods with u64 bound and Boolean result; ABI54 alone is insufficient").into());
            }
        }
        Ok(())
    }
}

// Keep this closed developer API check local. Unknown or conditional attributes
// cannot establish that a selected source API is available to generated code.
fn unconditional(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().all(|attribute| {
        ["doc", "allow", "warn", "deny", "forbid", "expect"]
            .iter()
            .any(|name| attribute.path().is_ident(name))
    })
}

fn external_module(file: &syn::File, name: &str, public: bool) -> bool {
    let modules: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(module) if module.ident == name => Some(module),
            _ => None,
        })
        .collect();
    unconditional(&file.attrs)
        && modules.len() == 1
        && modules[0].content.is_none()
        && unconditional(&modules[0].attrs)
        && (!public || matches!(modules[0].vis, syn::Visibility::Public(_)))
}

fn api_methods(file: &syn::File, owner: &str, output: &str, methods: &BTreeSet<&str>) -> bool {
    if !unconditional(&file.attrs) {
        return false;
    }
    let expected: syn::Type = syn::parse_str(output).expect("fixed API result type");
    let expected_self: syn::Type =
        syn::parse_str(&format!("super::{owner}<Private, D>")).expect("fixed API owner");
    let expected_generics: syn::Generics = syn::parse_quote!(<Private, D: crate::ledger::DB>);
    let expected_bound: syn::Type = syn::parse_quote!(::core::primitive::u64);
    methods.iter().all(|name| {
        let candidates: Vec<_> = file.items.iter().filter_map(|item| match item {
            syn::Item::Impl(item) if item.trait_.is_none() && item.unsafety.is_none()
                && item.defaultness.is_none() && unconditional(&item.attrs)
                && same_api_type(&item.self_ty, &expected_self)
                && item.generics.to_token_stream().to_string() == expected_generics.to_token_stream().to_string()
                && item.generics.where_clause.is_none() => Some(item),
            _ => None,
        }).flat_map(|item| item.items.iter()).filter_map(|item| match item {
            syn::ImplItem::Fn(method) if method.sig.ident == *name => Some(method),
            _ => None,
        }).collect();
        if candidates.len() != 1 { return false; }
        let method = candidates[0];let signature = &method.sig;
        matches!(method.vis, syn::Visibility::Public(_)) && unconditional(&method.attrs)
            && signature.constness.is_none() && signature.asyncness.is_none()
            && signature.unsafety.is_none() && signature.abi.is_none()
            && signature.generics.params.is_empty() && signature.generics.where_clause.is_none()
            && signature.variadic.is_none() && signature.inputs.len() == 2
            && matches!(signature.inputs.first(), Some(syn::FnArg::Receiver(receiver))
                if receiver.reference.is_none() && receiver.colon_token.is_none() && unconditional(&receiver.attrs))
            && matches!(signature.inputs.iter().nth(1), Some(syn::FnArg::Typed(argument))
                if unconditional(&argument.attrs)
                    && same_api_type(&argument.ty, &expected_bound))
            && matches!(&signature.output, syn::ReturnType::Type(_, ty)
                if same_api_type(ty, &expected))
    })
}

// Compare only the closed API type forms above. Generic and tuple punctuation
// may contain an optional trailing comma; path anchors and argument types must
// still match exactly. This does not resolve aliases or authenticate a body.
fn same_api_type(actual: &syn::Type, expected: &syn::Type) -> bool {
    match (actual, expected) {
        (syn::Type::Path(actual), syn::Type::Path(expected)) => {
            actual.qself.is_none()
                && expected.qself.is_none()
                && actual.path.leading_colon.is_some() == expected.path.leading_colon.is_some()
                && actual.path.segments.len() == expected.path.segments.len()
                && actual
                    .path
                    .segments
                    .iter()
                    .zip(&expected.path.segments)
                    .all(|(actual, expected)| {
                        actual.ident == expected.ident
                            && match (&actual.arguments, &expected.arguments) {
                                (syn::PathArguments::None, syn::PathArguments::None) => true,
                                (
                                    syn::PathArguments::AngleBracketed(actual),
                                    syn::PathArguments::AngleBracketed(expected),
                                ) => {
                                    actual.colon2_token.is_some() == expected.colon2_token.is_some()
                                        && actual.args.len() == expected.args.len()
                                        && actual.args.iter().zip(&expected.args).all(
                                            |(actual, expected)| match (actual, expected) {
                                                (
                                                    syn::GenericArgument::Type(actual),
                                                    syn::GenericArgument::Type(expected),
                                                ) => same_api_type(actual, expected),
                                                _ => false,
                                            },
                                        )
                                }
                                _ => false,
                            }
                    })
        }
        (syn::Type::Tuple(actual), syn::Type::Tuple(expected)) => {
            actual.elems.len() == expected.elems.len()
                && actual
                    .elems
                    .iter()
                    .zip(&expected.elems)
                    .all(|(actual, expected)| same_api_type(actual, expected))
        }
        _ => false,
    }
}

#[cfg(test)]
mod attribute_tests {
    use super::unconditional;

    #[test]
    fn block_time_api_allows_only_unconditional_documentation_and_lints() {
        assert!(unconditional(&[]));
        for attribute in [
            syn::parse_quote!(#[doc = "API documentation"]),
            syn::parse_quote!(#[allow(dead_code)]),
            syn::parse_quote!(#[warn(unused)]),
            syn::parse_quote!(#[deny(unused)]),
            syn::parse_quote!(#[forbid(unsafe_code)]),
            syn::parse_quote!(#[expect(dead_code)]),
        ] {
            assert!(unconditional(&[attribute]));
        }
        for attribute in [
            syn::parse_quote!(#[cfg(any())]),
            syn::parse_quote!(#[cfg_attr(all(), allow(dead_code))]),
            syn::parse_quote!(#[derive(Clone)]),
            syn::parse_quote!(#[unknown_attribute]),
        ] {
            assert!(!unconditional(&[attribute]));
        }
    }
}
