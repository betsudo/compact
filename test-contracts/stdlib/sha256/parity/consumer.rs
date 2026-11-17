// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use multi_length::runtime::{CompactError, FixedBytes};
use serde_json::Value;
use std::io;

fn bytes<const N: usize>(value: &Value) -> FixedBytes<N> {
    let raw: Vec<u8> = serde_json::from_value(value.clone()).expect("bytes argument");
    FixedBytes::new(raw.try_into().expect("fixed-width bytes"))
}

macro_rules! hash_bytes {
    ($name:expr, $value:expr, $contract:ident, $($circuit:ident),+ $(,)?) => {
        match $name {
            $(stringify!($circuit) => $contract::pure_circuits::$circuit(bytes($value)),)+
            name => panic!("unexpected byte circuit {name}"),
        }
    };
}

fn digest(family: &str, circuit: &str, value: &Value) -> Result<FixedBytes<32>, CompactError> {
    match (family, circuit) {
        ("multi_length", name) => hash_bytes!(
            name,
            value,
            multi_length,
            hashBytes1,
            hashBytes32,
            hashBytes33,
            hashBytes34,
            hashBytes35,
            hashBytes36,
            hashBytes37,
            hashBytes38,
            hashBytes39,
            hashBytes40,
            hashBytes41,
            hashBytes42,
            hashBytes43,
            hashBytes44,
            hashBytes45,
            hashBytes46,
            hashBytes47,
            hashBytes48,
            hashBytes49,
            hashBytes50,
            hashBytes51,
            hashBytes52,
            hashBytes53,
            hashBytes54,
            hashBytes55,
            hashBytes56,
            hashBytes57,
            hashBytes58,
            hashBytes59,
            hashBytes60,
            hashBytes61,
            hashBytes62,
            hashBytes63,
            hashBytes93,
            hashBytes94,
            hashBytes376,
            hashBytes1024,
        ),
        ("trailing_zero", name) => hash_bytes!(name, value, trailing_zero, hashBytes12,),
        _ => panic!("unexpected fixture {family}/{circuit}"),
    }
}

fn main() {
    let [family, circuit]: [String; 2] = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .try_into()
        .expect("expected fixture and circuit names");
    let value = serde_json::from_reader(io::stdin().lock()).expect("JSON argument");
    println!(
        "{}",
        hex::encode(digest(&family, &circuit, &value).unwrap().0)
    );
}
