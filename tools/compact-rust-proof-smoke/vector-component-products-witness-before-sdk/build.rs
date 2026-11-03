// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

fn main() {
    println!("cargo::rustc-check-cfg=cfg(compact_vector_component_products_sdk)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED");
    if std::env::var_os("CARGO_FEATURE_GENERATED_SDK").is_some()
        && let Some(pool) = std::env::var_os("COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED")
        && !pool.is_empty()
    {
        let pool = std::path::PathBuf::from(pool);
        assert!(pool.is_absolute(), "absolute complete SDK pool required");
        let root = pool.join("witness_before/contract");
        for relative in [
            "lib.rs",
            "compact-rust-ir.json",
            "rust-capabilities.json",
            "checked-programs/program-0.json",
        ] {
            let path = root.join(relative);
            assert!(
                path.is_file(),
                "complete witness_before SDK member required"
            );
            println!("cargo::rerun-if-changed={}", path.display());
        }
        println!("cargo::rustc-cfg=compact_vector_component_products_sdk");
    }
}
