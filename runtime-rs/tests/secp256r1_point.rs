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

use midnight_base_crypto::fab::{Aligned, AlignmentAtom, AlignmentSegment, Value, ValueAtom};
use midnight_compact_runtime::{
    BinaryHashRepr, Field, FieldRepr, FromFieldRepr, Secp256r1Base, Secp256r1Point,
};

fn base(hex: &str) -> Secp256r1Base {
    let mut bytes: [u8; 32] = hex::decode(hex).unwrap().try_into().unwrap();
    bytes.reverse();
    Secp256r1Base::from_le_bytes(bytes).unwrap()
}

fn coordinates() -> [(Secp256r1Base, Secp256r1Base); 2] {
    [
        (
            base("6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"),
            base("4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"),
        ),
        (
            Secp256r1Base::default(),
            base("66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4"),
        ),
    ]
}

fn raw(x: Secp256r1Base, y: Secp256r1Base, flag: ValueAtom) -> Value {
    let mut value = Value::from(x);
    value.0.extend(Value::from(y).0);
    value.0.push(flag);
    value
}

#[test]
fn finite_points_match_upstream_coordinates_and_five_atoms() {
    let alignment = Secp256r1Point::alignment();
    assert_eq!(
        alignment.0,
        [
            [24, 8, 24, 8]
                .map(|length| AlignmentSegment::Atom(AlignmentAtom::Bytes { length }))
                .as_slice(),
            &[AlignmentSegment::Atom(AlignmentAtom::Field)],
        ]
        .concat()
    );
    for (x, y) in coordinates() {
        let point = Secp256r1Point::from_affine(x, y).unwrap();
        assert!(!point.is_identity());
        assert_eq!(point.coordinates(), Some((x, y)));
        let value = raw(x, y, false.into());
        assert_eq!(Value::from(point), value);
        assert_eq!(Secp256r1Point::try_from(&*value).unwrap(), point);
        let mut limbs = Vec::new();
        point.field_repr(&mut limbs);
        assert_eq!(limbs.len(), 5);
        assert_eq!(Secp256r1Point::from_field_repr(&limbs), Some(point));
        let mut binary = Vec::new();
        point.binary_repr(&mut binary);
        assert_eq!(binary.len(), point.binary_len());
    }
}

#[test]
fn fab_identity_ignores_valid_coordinates_and_serializes_canonically() {
    let identity = Secp256r1Point::default();
    let zero = Secp256r1Base::default();
    assert!(identity.is_identity());
    assert_eq!(identity.coordinates(), None);
    let (x, y) = coordinates()[0];
    let ignored = raw(x, y, true.into());
    let canonical = raw(zero, zero, true.into());
    let decoded = Secp256r1Point::try_from(&*ignored).unwrap();
    assert_eq!(decoded, identity);
    assert_eq!(Value::from(decoded), canonical);
    let mut limbs = Vec::new();
    decoded.field_repr(&mut limbs);
    assert_eq!(Secp256r1Point::from_field_repr(&limbs), Some(identity));
    let mut not_canonical = Vec::new();
    x.field_repr(&mut not_canonical);
    y.field_repr(&mut not_canonical);
    not_canonical.push(Field::from(true));
    assert_eq!(Secp256r1Point::from_field_repr(&not_canonical), None);
}

#[test]
fn point_ingress_rejects_bad_width_flag_and_curve_membership() {
    let zero = Secp256r1Base::default();
    assert!(Secp256r1Point::from_affine(zero, zero).is_none());
    for bad in [
        raw(zero, zero, false.into()),
        raw(zero, zero, ValueAtom(vec![2])),
        raw(zero, zero, ValueAtom(vec![1, 0])),
        Value(vec![ValueAtom(vec![1])]),
    ] {
        assert!(Secp256r1Point::try_from(&*bad).is_err());
    }
}

#[test]
fn p256_group_preserves_canonical_identity_and_finite_coordinates() {
    use midnight_compact_runtime::Secp256r1Scalar;

    let [(x, y), (x_zero, y_zero)] = coordinates();
    let generator = Secp256r1Point::from_affine(x, y).unwrap();
    let finite_x_zero = Secp256r1Point::from_affine(x_zero, y_zero).unwrap();
    let identity = Secp256r1Point::default();
    let double = Secp256r1Point::from_affine(
        base("7cf27b188d034f7e8a52380304b51ac3c08969e277f21b35a60b48fc47669978"),
        base("07775510db8ed040293d9ac69f7430dbba7dade63ce982299e04b79d227873d1"),
    )
    .unwrap();
    let scalar = |n| {
        let mut bytes = [0; 32];
        bytes[0] = n;
        Secp256r1Scalar::from_le_bytes(bytes).unwrap()
    };
    let mut max =
        hex::decode("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550").unwrap();
    max.reverse();
    let max = Secp256r1Scalar::from_le_bytes(max.try_into().unwrap()).unwrap();
    let negated = Secp256r1Point::from_affine(x, -y).unwrap();
    assert_eq!(Secp256r1Point::mul_generator(scalar(1)), generator);
    assert_eq!(Secp256r1Point::mul_generator(scalar(2)), double);
    assert_eq!(Secp256r1Point::mul_generator(max), negated);
    assert_eq!(generator * max, negated);
    assert_eq!(generator + generator, double);
    assert_eq!(generator * scalar(2), double);
    assert_eq!(finite_x_zero * scalar(1), finite_x_zero);
    assert_eq!(identity + finite_x_zero, finite_x_zero);
    for result in [
        Secp256r1Point::mul_generator(scalar(0)),
        generator * scalar(0),
        identity * scalar(2),
        generator + negated,
    ] {
        assert_eq!(result, identity);
        assert_eq!(Value::from(result), Value::from(identity));
    }
}
