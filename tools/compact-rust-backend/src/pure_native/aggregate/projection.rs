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

use crate::{
    naming::checked_ident,
    pure_semantics::{identifier::CheckedName, ownership::Ownership},
    value_lowering::retained_value_with_ownership,
};

pub(super) fn field(value: syn::Expr, field: CheckedName, ownership: Ownership) -> syn::Expr {
    let name = checked_ident(&field);
    retained_value_with_ownership(syn::parse_quote!((#value).#name), ownership)
}
pub(super) fn tuple(value: syn::Expr, index: usize) -> syn::Expr {
    let index = syn::Index::from(index);
    syn::parse_quote!((#value).#index)
}

/// The semantic owner has checked this literal against the fixed vector length.
/// Preserve the original aggregate and retain only an owned selected element.
pub(crate) fn vector(value: syn::Expr, index: usize, ownership: Ownership) -> syn::Expr {
    let index = syn::Index::from(index);
    retained_value_with_ownership(syn::parse_quote!((#value).0[#index]), ownership)
}
