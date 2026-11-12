// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

fn main() {
    println!("cargo::rustc-check-cfg=cfg(compact_vector_product_elements_checked_sdk)");
    println!("cargo::rerun-if-env-changed=COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED");
    if std::env::var_os("CARGO_FEATURE_GENERATED_SDK").is_some()
        && let Some(root) = std::env::var_os("COMPACT_VECTOR_PRODUCT_ELEMENTS_CHECKED_GENERATED")
        && !root.is_empty()
    {
        let root = std::path::PathBuf::from(root);
        assert!(
            root.is_absolute() && root.join("contract/lib.rs").is_file(),
            "complete checked-helper SDK required"
        );
        for path in [
            root.join("contract/lib.rs"),
            root.join("contract/compact-rust-ir.json"),
            root.join("contract/checked-programs/program-0.json"),
        ] {
            assert!(
                path.is_file(),
                "complete checked-helper SDK member required"
            );
            println!("cargo::rerun-if-changed={}", path.display());
        }
        println!("cargo::rustc-cfg=compact_vector_product_elements_checked_sdk");
    }
}

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
