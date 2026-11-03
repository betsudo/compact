// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

// Compile this complete unedited SDK at its own crate root: generated crate::
// paths must resolve inside this package, independently of every other SDK.
#[cfg(compact_vector_component_products_sdk)]
include!(concat!(
    env!("COMPACT_VECTOR_COMPONENT_PRODUCTS_GENERATED"),
    "/witness_after/contract/lib.rs"
));
