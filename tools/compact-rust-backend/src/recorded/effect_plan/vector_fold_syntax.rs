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

//! Iterator zip syntax for a previously validated fixed-vector Fold.
use super::{RecordingGap, invalid, model::Value};
use crate::ir::Type;

pub(super) fn formal(value: &Value) -> syn::Pat {
    if value.ty == Type::Unit {
        syn::parse_quote!(_)
    } else {
        let name = super::functional_render::name(value);
        syn::parse_quote!(#name)
    }
}
pub(super) fn iteration(
    sources: &[Value],
    items: &[Value],
    read: fn(&Value) -> syn::Expr,
    path: &str,
) -> Result<(syn::Pat, syn::Expr), RecordingGap> {
    if sources.is_empty() || sources.len() != items.len() {
        return Err(invalid(path, "checked Fold iterable/formal mismatch"));
    }
    let input = read(&sources[0]);
    let mut iterator: syn::Expr = syn::parse_quote!((#input).into_array().into_iter());
    let mut pattern = formal(&items[0]);
    for (source, item) in sources.iter().zip(items).skip(1) {
        let input = read(source);
        let item = formal(item);
        iterator = syn::parse_quote!((#iterator).zip((#input).into_array()));
        pattern = syn::parse_quote!((#pattern, #item));
    }
    Ok((pattern, iterator))
}
