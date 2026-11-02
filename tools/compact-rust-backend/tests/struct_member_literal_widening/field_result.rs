// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
use helper::runtime::{BoundedUint,FixedVector};
fn main()->Result<(),Box<dyn std::error::Error>>{
let input=small_field::types::Input{left:BoundedUint::new(0)?,right:BoundedUint::new(255)?};
let _: FixedVector<BoundedUint<65535>,2> = small_field::pure_circuits::value(input)?; // expected-error
Ok(())
}
