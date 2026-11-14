// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

#[test]
fn prepared_unit_views_are_distinct_without_any_source_or_type_checker() {
    let assertion = control::assertion(syn::parse_quote!(flag), "guard", std::iter::empty());
    let fragment = fragment::lexical(
        vec![syn::parse_quote!(let saved: bool = flag;)],
        fragment::sequence(
            assertion.into_views().1,
            NativeFragment::unit(syn::parse_quote!(()), Vec::new()),
        ),
    );
    let (expression, statements) = fragment.into_views();
    let expected_expression: syn::Expr = syn::parse_quote!({
        let saved: bool = flag;
        {
            if !(flag) {
                return Err(runtime::CompactError::AssertionFailed("guard".to_owned()));
            }
            ()
        }
    });
    let expected_statement: syn::Stmt = syn::parse_quote!({
        let saved: bool = flag;
        if !(flag) {
            return Err(runtime::CompactError::AssertionFailed("guard".to_owned()));
        }
    });
    assert_eq!(
        expression.to_token_stream().to_string(),
        expected_expression.to_token_stream().to_string()
    );
    assert_eq!(
        quote::quote!(#(#statements)*).to_string(),
        expected_statement.to_token_stream().to_string()
    );
}

#[test]
fn prepared_equal_arms_keep_one_fallible_condition_and_single_selected_call() {
    let arm = || {
        NativeFragment::unit(
            syn::parse_quote!(check()?),
            vec![syn::parse_quote!(check()?;)],
        )
    };
    let fragment = control::conditional(
        IfFacts {
            equal_arms: true,
            evaluate_condition: true,
            unit_result: true,
        },
        syn::parse_quote!(probe()?),
        arm(),
        arm(),
        std::iter::empty(),
    );
    let (expression, statements) = fragment.into_views();
    let expected_expression: syn::Expr = syn::parse_quote!({
        let _ = probe()?;
        check()?
    });
    let expected_statements: syn::Block = syn::parse_quote!({
        let _ = probe()?;
        check()?;
    });
    assert_eq!(
        expression.to_token_stream().to_string(),
        expected_expression.to_token_stream().to_string()
    );
    let expected = expected_statements.stmts;
    assert_eq!(
        quote::quote!(#(#statements)*).to_string(),
        quote::quote!(#(#expected)*).to_string()
    );
}

#[test]
fn equality_groups_an_if_operand_in_a_lexical_block() {
    use crate::pure_semantics::sink::ArithmeticSink;

    let mut syntax = super::super::leaf::NativeLeafSink::new();
    let equality = syntax.arithmetic(
        crate::pure_semantics::arithmetic::ArithmeticFacts::Equality {
            negated: false,
            unit_result: None,
        },
        syn::parse_quote!(if flag { left } else { right }),
        syn::parse_quote!(expected),
    );
    let fragment = fragment::lexical(
        vec![syn::parse_quote!(let flag: bool = input;)],
        NativeFragment::Value(equality),
    );
    let expected: syn::Expr = syn::parse_quote!({
        let flag: bool = input;
        (if flag { left } else { right }) == expected
    });
    assert_eq!(
        fragment.into_expression().to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
}
