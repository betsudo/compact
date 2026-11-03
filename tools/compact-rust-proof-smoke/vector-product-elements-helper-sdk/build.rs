// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

fn main() {
    println!("cargo::rustc-check-cfg=cfg(compact_vector_product_elements_helper_sdk)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED");
    if std::env::var_os("CARGO_FEATURE_GENERATED_SDK").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "complete helper SDK required"
        );
        for path in [
            root.join("contract/lib.rs"),
            root.join("contract/compact-rust-ir.json"),
            root.join("contract/checked-programs/program-0.json"),
        ] {
            assert!(path.is_file(), "complete helper SDK member required");
            println!("cargo::rerun-if-changed={}", path.display());
        }
        println!("cargo::rustc-cfg=compact_vector_product_elements_helper_sdk");
    }
}
