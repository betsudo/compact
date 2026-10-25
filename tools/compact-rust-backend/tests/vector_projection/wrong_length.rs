// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
fn main() {
    let _ = vector_fixture::pure_circuits::first(vector_fixture::runtime::FixedVector([
        vector_fixture::runtime::Field::from(1_u64),
        vector_fixture::runtime::Field::from(2_u64),
        vector_fixture::runtime::Field::from(3_u64),
    ]));
}
