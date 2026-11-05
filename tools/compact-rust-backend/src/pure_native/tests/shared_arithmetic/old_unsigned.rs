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

//! Test-only helper from signed 5c995cff. The added strategy arm below retains
//! its historical refusal; it must not adopt the new zero-product behavior.
use crate::pure_semantics::numeric::{
    ArithmeticKind, UnsignedArithmeticStrategy, checked_unsigned_arithmetic,
};
use crate::{RenderError, ir::Expr};
use proc_macro2::Span;

pub(crate) fn unsigned_arithmetic_syntax(
    operation: &Expr,
    left: syn::Expr,
    right: syn::Expr,
    left_max: &str,
    right_max: &str,
    result_max: &str,
) -> Result<syn::Expr, RenderError> {
    let kind = match operation {
        Expr::UnsignedAdd { .. } => ArithmeticKind::Add,
        Expr::UnsignedSubtract { .. } => ArithmeticKind::Subtract,
        Expr::UnsignedMultiply { .. } => ArithmeticKind::Multiply,
        _ => unreachable!("unsigned arithmetic caller"),
    };
    Ok(
        match checked_unsigned_arithmetic(kind, left_max, right_max, result_max)? {
            UnsignedArithmeticStrategy::WideZeroProduct { .. } => {
                // The historical adapter shares the strategy type, so map the
                // new variant back to its original first-wide-bound refusal.
                return Err(RenderError::UnsupportedWideUnsignedOperation {
                    operation: "multiplication",
                    max: if left_max == "0" { right_max } else { left_max }.into(),
                });
            }
            UnsignedArithmeticStrategy::WideAdd { high, low } => {
                let high = syn::LitInt::new(&format!("{high}u128"), Span::call_site());
                let low = syn::LitInt::new(&format!("{low}u128"), Span::call_site());
                syn::parse_quote!(runtime::add_wide_unsigned::<#high,#low,_,_>(#left,#right)?)
            }
            UnsignedArithmeticStrategy::Small {
                kind,
                left: left_bound,
                right: right_bound,
                result,
            } => {
                let left_max = syn::LitInt::new(&left_bound.to_string(), Span::call_site());
                let right_max = syn::LitInt::new(&right_bound.to_string(), Span::call_site());
                let result_max = syn::LitInt::new(&result.to_string(), Span::call_site());
                let helper: syn::Path = match kind {
                    ArithmeticKind::Add => syn::parse_quote!(runtime::add_unsigned),
                    ArithmeticKind::Subtract => syn::parse_quote!(runtime::subtract_unsigned),
                    ArithmeticKind::Multiply => syn::parse_quote!(runtime::multiply_unsigned),
                };
                syn::parse_quote!(#helper::<#left_max,#right_max,#result_max>(#left,#right)?)
            }
        },
    )
}
