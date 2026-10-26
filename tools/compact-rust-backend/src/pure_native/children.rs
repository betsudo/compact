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

//! Transitional native traversal environment, outside shared semantic core.
//! It borrows scopes/declarations without rebuilding them per child. The complete
//! generic Expr visitor will replace this adapter before body CheckSink cutover.
use super::leaf::NativeLeafSink;
use crate::{
    RenderError,
    ir::{Expr, PureCircuit, Type},
    pure_semantics::{children::Children, sink::Typed},
};
use std::collections::HashMap;

pub(crate) struct NativeChildren<'call, 'ir> {
    bindings: &'call HashMap<&'ir str, (&'ir Type, syn::Ident)>,
    declarations: &'call HashMap<&'ir str, &'ir PureCircuit>,
}

impl<'call, 'ir> NativeChildren<'call, 'ir> {
    pub(crate) fn new(
        bindings: &'call HashMap<&'ir str, (&'ir Type, syn::Ident)>,
        declarations: &'call HashMap<&'ir str, &'ir PureCircuit>,
    ) -> Self {
        Self {
            bindings,
            declarations,
        }
    }
}

impl Children<NativeLeafSink> for NativeChildren<'_, '_> {
    fn visit(
        &mut self,
        source: &Expr,
        _: &mut NativeLeafSink,
    ) -> Result<Typed<syn::Expr>, RenderError> {
        let (output, ty) = crate::expression_with_calls(source, self.bindings, self.declarations)?;
        Ok(Typed { output, ty })
    }
}

impl crate::pure_semantics::children::ScopedChildren<NativeLeafSink> for NativeChildren<'_, '_> {
    fn visit_bound(
        &mut self,
        source: &Expr,
        source_key: &str,
        ty: &Type,
        binding: &syn::Ident,
        _: &mut NativeLeafSink,
    ) -> Result<Typed<syn::Expr>, RenderError> {
        // Reborrow inherited entries into this shorter body scope. Only binding
        // symbols are cloned, matching the old map overlay; no Type is cloned.
        let mut scope: HashMap<&str, (&Type, syn::Ident)> = self.bindings.clone();
        scope.insert(source_key, (ty, binding.clone()));
        let (output, ty) = crate::expression_with_calls(source, &scope, self.declarations)?;
        Ok(Typed { output, ty })
    }
    fn visit_bound_many(
        &mut self,
        source: &Expr,
        bindings: &[(&str, &Type, &syn::Ident)],
        _: &mut NativeLeafSink,
    ) -> Result<Typed<syn::Expr>, RenderError> {
        let mut scope: HashMap<&str, (&Type, syn::Ident)> = self.bindings.clone();
        for (key, ty, binding) in bindings {
            scope.insert(key, (ty, (*binding).clone()));
        }
        let (output, ty) = crate::expression_with_calls(source, &scope, self.declarations)?;
        Ok(Typed { output, ty })
    }
}
