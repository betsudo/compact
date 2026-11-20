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

use std::fmt::Debug;

use ff::PrimeField;
use midnight_base_crypto::fab::{
    AlignedValue, Alignment, AlignmentAtom, AlignmentSegment, Value, ValueAtom, ValueSlice,
};
use midnight_compact_runtime::{
    BinaryHashRepr, Field, FieldRepr, FromFieldRepr, HashInputValue, Secp256r1Base,
    Secp256r1Scalar,
    ledger::{CellValue, DefaultDB, read_cell, try_constructor_cell},
};
use midnight_curves::p256::{Fp, Fq};

fn le_bytes(hex: &str) -> [u8; 32] {
    let mut bytes: [u8; 32] = hex::decode(hex).unwrap().try_into().unwrap();
    bytes.reverse();
    bytes
}

fn check_codec<T>(value: T, shifted: [u8; 32])
where
    T: BinaryHashRepr + CellValue + Copy + Debug + Eq + FieldRepr + FromFieldRepr + HashInputValue,
    Value: From<T>,
    for<'a> T: TryFrom<&'a ValueSlice>,
    for<'a> <T as TryFrom<&'a ValueSlice>>::Error: Debug,
{
    let expected = Value(vec![
        ValueAtom::from(&shifted[..24]),
        ValueAtom::from(&shifted[24..]),
    ]);
    let aligned = AlignedValue::from(value);
    assert_eq!(aligned.value, expected);
    assert_eq!(
        aligned.alignment,
        Alignment(vec![
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 24 }),
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 8 }),
        ])
    );
    assert_eq!(value.into_hash_value(), expected);
    assert_eq!(T::try_from(&*expected).unwrap(), value);
    let fields = [
        Field::from_le_bytes(&shifted[..24]).unwrap(),
        Field::from_le_bytes(&shifted[24..]).unwrap(),
    ];
    assert_eq!(value.field_vec(), fields);
    assert_eq!(T::from_field_repr(&fields), Some(value));
    assert_eq!(value.binary_vec(), shifted);
    let cell = try_constructor_cell::<_, DefaultDB>(value).unwrap();
    assert_eq!(read_cell::<T, _>(&cell).unwrap(), value);
}

fn check_boundary<T: Eq + Debug, S: PrimeField<Repr: From<[u8; 32]>>>(
    checked: impl Fn([u8; 32]) -> Option<T>,
    canonical_maximum: [u8; 32],
    modulus: [u8; 32],
) {
    let sdk = |mut bytes: [u8; 32]| -> bool {
        bytes.reverse();
        S::from_repr(bytes.into()).is_some().into()
    };
    for (bytes, accepted) in [
        ([0; 32], true),
        (canonical_maximum, true),
        (modulus, false),
        ([255; 32], false),
    ] {
        assert_eq!(checked(bytes).is_some(), accepted);
        assert_eq!(sdk(bytes), accepted);
    }
}

#[test]
fn base_carrier_uses_p256_modulus_and_shifted_atoms() {
    let max = le_bytes("ffffffff00000001000000000000000000000000fffffffffffffffffffffffe");
    let modulus = le_bytes("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff");
    check_boundary::<_, Fp>(Secp256r1Base::from_le_bytes, max, modulus);
    check_codec(Secp256r1Base::default(), max);
    let value = Secp256r1Base::from_le_bytes(max).unwrap();
    assert_eq!(value.to_le_bytes(), max);
    let mut shifted = max;
    shifted[0] -= 1;
    check_codec(value, shifted);
    let mut value = [0; 32];
    value[0] = 1;
    check_codec(Secp256r1Base::from_le_bytes(value).unwrap(), [0; 32]);
}

#[test]
fn scalar_carrier_preserves_borrow_and_refuses_malformed_atoms() {
    let max = le_bytes("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550");
    let modulus = le_bytes("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");
    check_boundary::<_, Fq>(Secp256r1Scalar::from_le_bytes, max, modulus);
    check_codec(Secp256r1Scalar::default(), max);
    let mut boundary = [0; 32];
    boundary[24] = 1;
    let mut shifted = [0; 32];
    shifted[..24].fill(255);
    let value = Secp256r1Scalar::from_le_bytes(boundary).unwrap();
    assert_eq!(value.to_le_bytes(), boundary);
    check_codec(value, shifted);
    let empty = ValueAtom::from(&[][..]);
    for malformed in [
        vec![empty.clone()],
        vec![ValueAtom::from(&[1; 25][..]), empty.clone()],
        vec![empty.clone(), ValueAtom::from(&[1; 9][..])],
        vec![ValueAtom(vec![1, 0]), empty.clone()],
        vec![
            ValueAtom::from(&modulus[..24]),
            ValueAtom::from(&modulus[24..]),
        ],
    ] {
        let malformed = Value(malformed);
        assert!(Secp256r1Scalar::try_from(&*malformed).is_err());
    }
}

#[test]
fn p256_byte_casts_reduce_only_at_the_cast_boundary() {
    let mut one = [0; 32];
    one[0] = 1;
    for (modulus, reduce, checked, all_ones) in [
        (
            le_bytes("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff"),
            (|bytes| Secp256r1Base::from_le_bytes_reduced(bytes).to_le_bytes())
                as fn([u8; 32]) -> [u8; 32],
            (|bytes| Secp256r1Base::from_le_bytes(bytes).is_some()) as fn([u8; 32]) -> bool,
            le_bytes("00000000fffffffeffffffffffffffffffffffff000000000000000000000000"),
        ),
        (
            le_bytes("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551"),
            (|bytes| Secp256r1Scalar::from_le_bytes_reduced(bytes).to_le_bytes())
                as fn([u8; 32]) -> [u8; 32],
            (|bytes| Secp256r1Scalar::from_le_bytes(bytes).is_some()) as fn([u8; 32]) -> bool,
            le_bytes("00000000ffffffff00000000000000004319055258e8617b0c46353d039cdaae"),
        ),
    ] {
        let mut below = modulus;
        below[0] -= 1;
        let mut above = modulus;
        if above[0] == u8::MAX {
            for byte in &mut above {
                let (next, carry) = byte.overflowing_add(1);
                *byte = next;
                if !carry {
                    break;
                }
            }
        } else {
            above[0] += 1;
        }
        assert!(checked(below));
        assert!(!checked(modulus));
        assert!(!checked(above));
        assert_eq!(reduce(below), below);
        assert_eq!(reduce(modulus), [0; 32]);
        assert_eq!(reduce(above), one);
        assert_eq!(reduce([u8::MAX; 32]), all_ones);
    }
}
