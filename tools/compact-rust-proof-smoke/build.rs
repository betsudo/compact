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
}
