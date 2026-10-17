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

//! Qualify generated type paths without rewriting source argument expressions.
use syn::visit_mut::{self, VisitMut};

pub(super) fn rust_type(ty: &crate::ir::Type) -> Result<syn::Type, crate::RenderError> {
    let mut ty = crate::rust_type(ty)?;
    AbsoluteRuntime.visit_type_mut(&mut ty);
    Ok(ty)
}

struct AbsoluteRuntime;
impl VisitMut for AbsoluteRuntime {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if path.segments.len() > 1
            && path
                .segments
                .first()
                .is_some_and(|part| part.ident == "runtime")
        {
            path.segments.insert(0, syn::parse_quote!(crate));
        }
        visit_mut::visit_path_mut(self, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn nested_runtime_types_are_absolute_but_source_type_names_are_preserved() {
        let mut ty: syn::Type = syn::parse_quote!((
            runtime::FixedVector<runtime::Field, 2>,
            crate::types::runtime
        ));
        AbsoluteRuntime.visit_type_mut(&mut ty);
        assert_eq!(
            ty.to_token_stream().to_string(),
            "(crate :: runtime :: FixedVector < crate :: runtime :: Field , 2 > , crate :: types :: runtime)"
        );
    }
}
