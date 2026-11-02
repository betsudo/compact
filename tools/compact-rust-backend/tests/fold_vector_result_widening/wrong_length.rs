// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use small::{pure_circuits as c, runtime as r};
fn main() {
    let input = r::FixedVector::new([r::BoundedUint::<255>::new(1).unwrap(); 3]);
    let seed = r::FixedVector::new([r::BoundedUint::<255>::new(1).unwrap(); 2]);
    let _ = c::value(input, seed); // expected-error
}
