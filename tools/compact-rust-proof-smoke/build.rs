// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

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
}
