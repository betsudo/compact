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

//! One structured loop carries a successful frame and accumulator per item.
use super::*;

pub(super) fn render(
    binding: &Binding,
    declarations: &Declarations,
) -> Result<syn::Stmt, RecordingGap> {
    let Operation::VectorFold {
        initial,
        sources,
        accumulator,
        items,
        body,
    } = &binding.operation
    else {
        return Err(invalid(&binding.path, "expected checked VectorFold"));
    };
    let output = name(&binding.value);
    let ty = rust_type(&binding.value.ty).map_err(|e| invalid(&binding.path, e.to_string()))?;
    let initial = value(initial);
    let formal = name(accumulator);
    let (pattern, iterator) =
        super::super::vector_fold_syntax::iteration(sources, items, value, &binding.path)?;
    let body_steps = block(body, declarations)?;
    let result = value(&body.result);
    // Validation has checked the whole body even when no runtime step remains.
    // Unit carries no payload; an effect-free body need not move the frame out
    // and immediately back into itself. Keep operand/iteration evaluation.
    if binding.value.ty == crate::ir::Type::Unit && body_steps.is_empty() {
        return Ok(syn::parse_quote!(for #pattern in #iterator {}));
    }

    if binding.value.ty == crate::ir::Type::Unit {
        Ok(syn::parse_quote! {
            let frame = {
                // Unit has no payload alias; its producer already ran.
                let mut __compact_fold_frame = frame;
                for #pattern in #iterator {
                    let frame = __compact_fold_frame;
                    #(#body_steps)*
                    __compact_fold_frame = frame;
                }
                __compact_fold_frame
            };
        })
    } else {
        Ok(syn::parse_quote! {
            let (frame, #output): (_, #ty) = {
                let mut __compact_fold_accumulator: #ty = #initial;
                let mut __compact_fold_frame = frame;
                for #pattern in #iterator {
                    let frame = __compact_fold_frame;
                    let #formal: #ty = __compact_fold_accumulator;
                    #(#body_steps)*
                    __compact_fold_accumulator = #result;
                    __compact_fold_frame = frame;
                }
                (__compact_fold_frame, __compact_fold_accumulator)
            };
        })
    }
}
