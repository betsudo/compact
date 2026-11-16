// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use super::*;
use crate::pure_semantics::identifier::{check_local_name, check_name};

#[test]
fn compact_underscore_local_is_addressable_without_admitting_other_underscore_identifiers() {
    assert!(check_name("_").is_err());
    let checked = check_local_name("_").unwrap();
    assert_eq!(allocate(&checked, "_", []).to_string(), "__compact_local__");
    assert_eq!(
        check_local_name(" _").unwrap_err(),
        crate::RenderError::InvalidIdentifier(" _".into())
    );
}

#[test]
fn accepted_words_produce_safe_symbols_without_copying_source_trivia_or_raw_markers() {
    for (source, expected) in [
        ("x", "__compact_local_x"),
        ("x$0", "__compact_local_x_0"),
        ("r#value", "__compact_local_value"),
        ("type", "__compact_local_type"),
        ("gen", "__compact_local_gen"),
        ("x /* comment */", "__compact_local_x"),
    ] {
        let checked = check_name(source).unwrap();
        assert_eq!(allocate(&checked, source, []).to_string(), expected);
    }
}

#[test]
fn distinct_raw_keys_and_raw_native_spellings_block_but_same_key_shadow_does_not() {
    let checked = check_name("x").unwrap();
    let base: syn::Ident = syn::parse_quote!(__compact_local_x);
    let raw: syn::Ident = syn::parse_quote!(r#__compact_local_x);
    let first: syn::Ident = syn::parse_quote!(__compact_local_x_1);
    assert_eq!(
        allocate(&checked, "x", [("x", &base)]).to_string(),
        "__compact_local_x"
    );
    assert_eq!(
        allocate(&checked, "x", [("other", &raw)]).to_string(),
        "__compact_local_x_1"
    );
    let occupied = [("other", &base), ("another", &first), ("x", &base)];
    assert_eq!(
        allocate(&checked, "x", occupied).to_string(),
        "__compact_local_x_2"
    );
    assert_eq!(
        allocate(&checked, "x", occupied.into_iter().rev()).to_string(),
        "__compact_local_x_2"
    );
    // Lowest-free choice may change when the live namespace changes; previous
    // same-key suffix is not pinned as an allocation promise.
    assert_eq!(
        allocate(&checked, "x", [("other", &base), ("x", &first)]).to_string(),
        "__compact_local_x_1"
    );
    assert_eq!(
        allocate(&checked, "x", [("x", &first)]).to_string(),
        "__compact_local_x"
    );
}
