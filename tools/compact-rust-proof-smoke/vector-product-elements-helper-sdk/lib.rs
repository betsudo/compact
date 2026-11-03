// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

// A complete unedited generated SDK is a crate root: its crate:: paths must
// resolve inside this package, independently of the proof runner's other SDK.
#[cfg(compact_vector_product_elements_helper_sdk)]
include!(concat!(
    env!("COMPACT_VECTOR_PRODUCT_ELEMENTS_HELPER_GENERATED"),
    "/contract/lib.rs"
));
