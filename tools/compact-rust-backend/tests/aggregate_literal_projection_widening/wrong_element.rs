// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use vector_unsigned::{pure_circuits as c, runtime as r};
fn main() {
    let input = r::FixedVector::new([true, false]);
    let _ = c::value(input); // expected-error
}
