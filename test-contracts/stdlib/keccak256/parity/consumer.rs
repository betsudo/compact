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

use cross_width::runtime::{BoundedUint, CompactError, Field, FixedBytes, FixedVector};
use serde_json::Value;
use std::io;

fn bytes<const N: usize>(value: &Value) -> FixedBytes<N> {
    let raw: Vec<u8> = serde_json::from_value(value.clone()).expect("bytes argument");
    FixedBytes::new(raw.try_into().expect("fixed-width bytes"))
}

fn field(value: &Value) -> Field {
    let mut repr: [u8; 32] = hex::decode(value.as_str().expect("hex field argument"))
        .expect("valid hex")
        .try_into()
        .expect("32-byte field");
    repr.reverse();
    Field::from_le_bytes(&repr).unwrap()
}

fn fields<const N: usize>(value: &Value) -> FixedVector<Field, N> {
    let values = value.as_array().expect("field vector argument");
    assert_eq!(values.len(), N);
    FixedVector::new(std::array::from_fn(|index| field(&values[index])))
}

fn uint64(value: &Value) -> BoundedUint<{ u64::MAX as u128 }> {
    let number = u128::from_str_radix(value.as_str().expect("hex Uint argument"), 16).unwrap();
    BoundedUint::new(number).unwrap()
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
        ("cross_width", name) => hash_bytes!(
            name,
            value,
            cross_width,
            hashBytes1,
            hashBytes32,
            hashBytes33,
            hashBytes63,
            hashBytes93,
            hashBytes94,
        ),
        ("known_vectors", name) => hash_bytes!(
            name,
            value,
            known_vectors,
            hashBytes0,
            hashBytes1,
            hashBytes2,
            hashBytes4,
            hashBytes5,
            hashBytes10,
            hashBytes18,
        ),
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
        ("trailing_zero", "hashBytes12") => trailing_zero::pure_circuits::hashBytes12(bytes(value)),
        ("field_types", "hashField") => field_types::pure_circuits::hashField(field(value)),
        ("field_types", "hashVector3") => field_types::pure_circuits::hashVector3(fields(value)),
        ("flatten_equivalence", "hashBytes12") => {
            flatten_equivalence::pure_circuits::hashBytes12(bytes(value))
        }
        ("flatten_equivalence", "hashTriple") => {
            flatten_equivalence::pure_circuits::hashTriple(flatten_equivalence::types::Triple {
                a: bytes(&value["a"]),
                b: bytes(&value["b"]),
                c: bytes(&value["c"]),
            })
        }
        ("flatten_equivalence", "hashUneven") => {
            flatten_equivalence::pure_circuits::hashUneven(flatten_equivalence::types::Uneven {
                head: bytes(&value["head"]),
                mid: bytes(&value["mid"]),
                tail: bytes(&value["tail"]),
            })
        }
        ("ordinary_types", "hashField") => ordinary_types::pure_circuits::hashField(field(value)),
        ("ordinary_types", "hashBytes32") => {
            ordinary_types::pure_circuits::hashBytes32(bytes(value))
        }
        ("ordinary_types", "hashBool") => {
            ordinary_types::pure_circuits::hashBool(value.as_bool().expect("Boolean argument"))
        }
        ("ordinary_types", "hashUint") => ordinary_types::pure_circuits::hashUint(uint64(value)),
        ("ordinary_types", "hashVector") => {
            ordinary_types::pure_circuits::hashVector(fields(value))
        }
        ("ordinary_types", "hashStruct") => {
            ordinary_types::pure_circuits::hashStruct(ordinary_types::types::Mixed {
                tag: bytes(&value["tag"]),
                count: uint64(&value["count"]),
                flag: value["flag"].as_bool().expect("Boolean flag"),
            })
        }
        _ => panic!("unexpected circuit {family}/{circuit}"),
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
