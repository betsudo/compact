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

//! Arithmetic syntax from checked facts and two already checked child outputs.
//! The exact binary AST preserves child association without extra parentheses.
use super::leaf::NativeLeafSink;
use crate::{
    ir::ComparisonOperator,
    pure_semantics::{arithmetic::ArithmeticFacts, numeric::ArithmeticKind, sink::ArithmeticSink},
    value_lowering::render_unsigned_arithmetic,
};

impl ArithmeticSink for NativeLeafSink {
    fn arithmetic(
        &mut self,
        facts: ArithmeticFacts,
        left: syn::Expr,
        right: syn::Expr,
    ) -> syn::Expr {
        match facts {
            ArithmeticFacts::Field(kind) => {
                let op = match kind {
                    ArithmeticKind::Add => syn::BinOp::Add(syn::token::Plus::default()),
                    ArithmeticKind::Subtract => syn::BinOp::Sub(syn::token::Minus::default()),
                    ArithmeticKind::Multiply => syn::BinOp::Mul(syn::token::Star::default()),
                };
                syn::Expr::Binary(syn::ExprBinary {
                    attrs: vec![],
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            ArithmeticFacts::Unsigned(strategy) => {
                render_unsigned_arithmetic(strategy, left, right)
            }
            ArithmeticFacts::Equality {
                negated,
                unit_result,
            } => {
                if let Some(result) = unit_result {
                    return crate::value_lowering::unit_comparison_syntax(left, right, result);
                }
                // Interpolating a nested comparison into parse_quote! loses its
                // expression grouping and panics on valid Boolean equality.
                let op = if negated {
                    syn::BinOp::Ne(syn::token::Ne::default())
                } else {
                    syn::BinOp::Eq(syn::token::EqEq::default())
                };
                syn::Expr::Binary(syn::ExprBinary {
                    attrs: vec![],
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            ArithmeticFacts::Ordered(operator) => match operator {
                ComparisonOperator::Less => syn::parse_quote!(#left.value() < #right.value()),
                ComparisonOperator::LessEqual => syn::parse_quote!(#left.value() <= #right.value()),
                ComparisonOperator::Greater => syn::parse_quote!(#left.value() > #right.value()),
                ComparisonOperator::GreaterEqual => {
                    syn::parse_quote!(#left.value() >= #right.value())
                }
            },
        }
    }
}
