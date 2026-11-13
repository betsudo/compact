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

use midnight_base_crypto::fab::{Value, ValueAtom};
use midnight_compact_runtime::{
    BinaryHashRepr, Curve25519Base, Curve25519Point, FieldRepr, FromFieldRepr,
    ledger::{DefaultDB, read_cell, try_constructor_cell},
};

#[test]
fn identity_has_four_shifted_base_atoms_and_roundtrips() {
    let zero = Curve25519Base::default();
    let mut bytes = [0; 32];
    bytes[0] = 1;
    let one = Curve25519Base::from_le_bytes(bytes).unwrap();
    let point = Curve25519Point::from_affine(zero, one).unwrap();
    assert_eq!(point, Curve25519Point::default());
    assert_eq!(point.coordinates(), (zero, one));
    let mut expected = Value::from(zero);
    expected.0.extend(Value::from(one).0);
    assert_eq!(Value::from(point), expected);
    assert_eq!(Curve25519Point::try_from(&*expected).unwrap(), point);
    assert_eq!(Curve25519Point::from_field_repr(&point.field_vec()), Some(point));
    assert_eq!(point.binary_vec(), [zero.binary_vec(), one.binary_vec()].concat());
    let cell = try_constructor_cell::<_, DefaultDB>(point).unwrap();
    assert_eq!(read_cell::<Curve25519Point, _>(&cell).unwrap(), point);
}

#[test]
fn rejects_invalid_and_small_order_coordinates() {
    let zero = Curve25519Base::default();
    let mut bytes = [0xff; 32];
    bytes[0] = 0xec;
    bytes[31] = 0x7f;
    let minus_one = Curve25519Base::from_le_bytes(bytes).unwrap();
    assert!(Curve25519Point::from_affine(zero, zero).is_none());
    assert!(Curve25519Point::from_affine(zero, minus_one).is_none());
    let mut atoms = Value::from(zero);
    atoms.0.extend(Value::from(minus_one).0);
    assert!(Curve25519Point::try_from(&*atoms).is_err());
    atoms.0.push(ValueAtom::from(0u8));
    assert!(Curve25519Point::try_from(&*atoms).is_err());
}
