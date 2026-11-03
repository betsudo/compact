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

// Default/all-features checks need no generated SDK. The explicit gate selects
// an absolute current bundle; Cargo tracks this env and every actual include.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(compact_fold_vector_result_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_FOLD_VECTOR_RESULT_GENERATED");
    if std::env::var_os("CARGO_FEATURE_FOLD_VECTOR_RESULT_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_FOLD_VECTOR_RESULT_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_FOLD_VECTOR_RESULT_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_fold_vector_result_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_vector_alias_source_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_ALIAS_SOURCE_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_ALIAS_SOURCE_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_ALIAS_SOURCE_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_ALIAS_SOURCE_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_vector_alias_source_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_aggregate_literal_projection_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED");
    if std::env::var_os("CARGO_FEATURE_AGGREGATE_LITERAL_PROJECTION_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_AGGREGATE_LITERAL_PROJECTION_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_aggregate_literal_projection_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_struct_member_literal_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STRUCT_MEMBER_LITERAL_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STRUCT_MEMBER_LITERAL_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STRUCT_MEMBER_LITERAL_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STRUCT_MEMBER_LITERAL_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_struct_member_literal_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_direct_stateful_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_DIRECT_STATEFUL_VECTOR_LITERAL_RETURNS").is_some()
        && let Some(root) = std::env::var_os("COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_DIRECT_STATEFUL_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_direct_stateful_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_vector_value_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_VECTOR_VALUE_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_VECTOR_VALUE_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_VECTOR_VALUE_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_VECTOR_VALUE_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_vector_value_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_local_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_LOCAL_VECTOR_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_LOCAL_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_local_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_stateful_product_vector_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED");
    if std::env::var_os("CARGO_FEATURE_STATEFUL_PRODUCT_VECTOR_WIDENING").is_some()
        && let Some(root) = std::env::var_os("COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_STATEFUL_PRODUCT_VECTOR_GENERATED must name the complete generated SDK"
        );
        println!("cargo::rustc-cfg=compact_stateful_product_vector_generated");
    }
    println!("cargo::rustc-check-cfg=cfg(compact_vector_product_elements_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_PRODUCT_ELEMENTS").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_GENERATED must name the complete generated SDK"
        );
        let helper = std::path::PathBuf::from(
            std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED")
                .expect("paired complete helper SDK is required"),
        );
        assert!(
            helper.is_absolute() && helper.join("contract/lib.rs").is_file(),
            "COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED must name the complete helper SDK"
        );
        let expected_helper = root
            .parent()
            .expect("candidate SDK pool required")
            .join("helper");
        assert_eq!(
            helper.canonicalize().expect("helper SDK must exist"),
            expected_helper
                .canonicalize()
                .expect("paired sibling helper SDK must exist"),
            "helper SDK must be the candidate pool's complete helper sibling"
        );
        println!("cargo::rustc-cfg=compact_vector_product_elements_generated");
    }

    println!("cargo::rustc-check-cfg=cfg(compact_vector_component_products_generated)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_VECTOR_COMPONENT_PRODUCTS").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(pool.is_absolute(), "absolute complete SDK pool required");
        for name in ["witness_before", "witness_after", "inner_product_single"] {
            for relative in [
                "lib.rs",
                "compact-rust-ir.json",
                "rust-capabilities.json",
                "checked-programs/program-0.json",
            ] {
                let path = pool.join(name).join("contract").join(relative);
                assert!(path.is_file(), "complete selected SDK member required");
                println!("cargo::rerun-if-changed={}", path.display());
            }
        }
        println!("cargo::rustc-cfg=compact_vector_component_products_generated");
    }
}
