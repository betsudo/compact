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

use midnight_base_crypto::fab::{
    Aligned, AlignedValue, Alignment, AlignmentAtom, AlignmentSegment, Value, ValueAtom,
};
use midnight_compact_runtime::ledger::{DefaultDB, StateValue, read_cell, try_constructor_cell};
use midnight_compact_runtime::{
    BinaryHashRepr, Field, FieldRepr, FromFieldRepr, HashInputValue, Secp256k1Base, Secp256k1Point,
    persistent_hash, transient_hash,
};

fn base(hex: &str) -> Secp256k1Base {
    let mut bytes: [u8; 32] = hex::decode(hex).unwrap().try_into().unwrap();
    bytes.reverse();
    Secp256k1Base::from_le_bytes(bytes).unwrap()
}

fn generator() -> (Secp256k1Base, Secp256k1Base) {
    (
        base("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"),
        base("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"),
    )
}

fn encoded(x: Secp256k1Base, y: Secp256k1Base, identity: bool) -> Value {
    let mut value = Value::from(x);
    value.0.extend(Value::from(y).0);
    value.0.push(ValueAtom::from(identity));
    value
}

#[test]
fn finite_point_matches_the_five_atom_alignment_and_fixed_width_hash_input() {
    let (x, y) = generator();
    let point = Secp256k1Point::from_affine(x, y).unwrap();
    assert!(!point.is_identity());
    assert_eq!(point.coordinates(), Some((x, y)));
    assert_eq!(Secp256k1Point::from_affine(x, y), Some(point),);
    assert_ne!(Secp256k1Point::from_affine(x, -y).unwrap(), point);

    let aligned = AlignedValue::from(point);
    assert_eq!(
        aligned.alignment,
        Alignment(vec![
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 24 }),
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 8 }),
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 24 }),
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 8 }),
            AlignmentSegment::Atom(AlignmentAtom::Field),
        ])
    );
    assert_eq!(aligned.value, encoded(x, y, false));
    assert_eq!(point.into_hash_value(), aligned.value);
    assert_eq!(Secp256k1Point::try_from(&*aligned.value).unwrap(), point);
    let cell = try_constructor_cell::<_, DefaultDB>(point).unwrap();
    assert_eq!(read_cell::<Secp256k1Point, _>(&cell).unwrap(), point);

    let raw = hex::decode(concat!(
        "9717f8165b81f259d928ce2ddbfc9b02070b87ce9562a055acbbdcf97e66be79b",
        "7d410fb8fd0479c195485a648b417fda808110efcfba45d65c4a32677da3a48",
        "0000000000000000000000000000000000000000000000000000000000000000",
    ))
    .unwrap();
    assert_eq!(point.binary_len(), 96);
    assert_eq!(point.binary_vec(), raw);
    assert_eq!(
        hex::encode(persistent_hash(point).0),
        "1840ba9f76d31bc5f6006d98439803c896e53af3d612e5e65ed35025373362f3",
    );
    let expected_fields = [
        &raw[..24],
        &raw[24..32],
        &raw[32..56],
        &raw[56..64],
        &raw[64..],
    ]
    .map(|bytes| Field::from_le_bytes(bytes).unwrap());
    assert_eq!(point.field_vec(), expected_fields);
    assert_eq!(
        Secp256k1Point::from_field_repr(&expected_fields),
        Some(point)
    );
}

#[test]
fn all_structural_identity_encodings_compare_hash_and_store_canonically() {
    let zero = Secp256k1Base::default();
    let default = Secp256k1Point::default();
    assert!(default.is_identity());
    assert_eq!(default.coordinates(), None);
    let four = base("0000000000000000000000000000000000000000000000000000000000000004");

    let (x, y) = generator();
    let canonical = encoded(zero, zero, true);
    let fields = default.field_vec();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[4], Field::from(true));
    assert_eq!(Secp256k1Point::from_field_repr(&fields), Some(default));
    assert_eq!(
        hex::encode(persistent_hash(default).0),
        "6cda5ab3e840212d3a7857a1306f4903089e66b72a5413218979f7511a083620",
    );
    for (x, y) in [(zero, zero), (four, four), (x, y)] {
        let point = Secp256k1Point::try_from(&*encoded(x, y, true)).unwrap();
        assert_eq!(point, default);
        assert_eq!(Value::from(point), canonical);
        assert_eq!(point.into_hash_value(), canonical);
        assert_eq!(point.field_vec(), fields);
        assert_eq!(point.binary_vec(), default.binary_vec());
        assert_eq!(persistent_hash(point), persistent_hash(default));
        assert_eq!(transient_hash(point), transient_hash(default));
        let cell = try_constructor_cell::<_, DefaultDB>(point).unwrap();
        assert_eq!(
            Value::from(read_cell::<Secp256k1Point, _>(&cell).unwrap()),
            canonical
        );
    }

    let mut noncanonical_fields = fields;
    noncanonical_fields[..2].copy_from_slice(&four.field_vec());
    assert!(Secp256k1Point::from_field_repr(&noncanonical_fields).is_none());
}

#[test]
fn off_curve_finite_points_cannot_enter_the_typed_rust_api() {
    let zero = Secp256k1Base::default();
    assert!(Secp256k1Point::from_affine(zero, zero).is_none());
    let input = encoded(zero, zero, false);
    assert!(Secp256k1Point::try_from(&*input).is_err());
    let raw_cell = StateValue::<DefaultDB>::from(
        AlignedValue::new(input, Secp256k1Point::alignment()).unwrap(),
    );
    assert!(read_cell::<Secp256k1Point, _>(&raw_cell).is_err());

    let mut fields = Secp256k1Point::default().field_vec();
    fields[4] = Field::from(false);
    assert!(Secp256k1Point::from_field_repr(&fields).is_none());
}

#[test]
fn point_decoders_refuse_wrong_widths_flags_and_out_of_range_coordinates() {
    let identity = Secp256k1Point::default();
    let canonical = Value::from(identity);
    assert!(Secp256k1Point::try_from(&canonical[..4]).is_err());
    let mut too_many = canonical.clone();
    too_many.0.push(ValueAtom::from(false));
    assert!(Secp256k1Point::try_from(&*too_many).is_err());
    for flag in [vec![2], vec![0], vec![1, 0]] {
        let mut value = canonical.clone();
        value.0[4] = ValueAtom(flag);
        assert!(Secp256k1Point::try_from(&*value).is_err());
    }
    let mut invalid_coordinate = canonical;
    invalid_coordinate.0[2] = ValueAtom(vec![255; 24]);
    invalid_coordinate.0[3] = ValueAtom(vec![255; 8]);
    assert!(Secp256k1Point::try_from(&*invalid_coordinate).is_err());

    let mut fields = identity.field_vec();
    assert!(Secp256k1Point::from_field_repr(&fields[..4]).is_none());
    fields[4] = Field::from(2u64);
    assert!(Secp256k1Point::from_field_repr(&fields).is_none());
}
