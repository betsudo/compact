// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
fn main(){
 let _ = small_unsigned::types::Input {left:true,right:helper::runtime::BoundedUint::new(1).unwrap()}; // expected-error
}
