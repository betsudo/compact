// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use midnight_base_crypto::fab::{
    AlignedValue, Alignment, AlignmentAtom, AlignmentSegment, Value, ValueAtom, ValueSlice,
};
use midnight_compact_runtime::ledger::{CellValue, DefaultDB, read_cell, try_constructor_cell};
use midnight_compact_runtime::{
    BinaryHashRepr, Field, FieldRepr, FromFieldRepr, HashInputValue, Secp256k1Base, Secp256k1Scalar,
};

fn le_bytes(hex: &str) -> [u8; 32] {
    let mut bytes: [u8; 32] = hex::decode(hex).unwrap().try_into().unwrap();
    bytes.reverse();
    bytes
}

fn check_representation<T>(value: T, shifted: [u8; 32])
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
    let encoded = AlignedValue::from(value);
    assert_eq!(
        encoded.alignment,
        Alignment(vec![
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 24 }),
            AlignmentSegment::Atom(AlignmentAtom::Bytes { length: 8 }),
        ])
    );
    assert_eq!(encoded.value, expected);
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

#[test]
fn base_field_round_trips_the_upstream_shifted_fab_values() {
    let max = le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e");
    let before_max = le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d");
    let modulus = le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
    check_representation(Secp256k1Base::default(), max);
    check_representation(Secp256k1Base::from_le_bytes(max).unwrap(), before_max);
    assert_eq!(Secp256k1Base::default().to_le_bytes(), [0; 32]);
    assert_eq!(
        Secp256k1Base::from_le_bytes(max).unwrap().to_le_bytes(),
        max
    );
    assert!(Secp256k1Base::from_le_bytes(modulus).is_none());

    let mut one = [0; 32];
    one[0] = 1;
    check_representation(Secp256k1Base::from_le_bytes(one).unwrap(), [0; 32]);

    let mut middle = [0; 32];
    middle[..8].copy_from_slice(&12_345_678_901_234_567_890_u64.to_le_bytes());
    let mut shifted = [0; 32];
    shifted[..8].copy_from_slice(&12_345_678_901_234_567_889_u64.to_le_bytes());
    check_representation(Secp256k1Base::from_le_bytes(middle).unwrap(), shifted);
}

#[test]
fn scalar_field_checks_its_own_bound_and_carries_across_the_two_limbs() {
    let max = le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140");
    let before_max = le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd036413f");
    let modulus = le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
    check_representation(Secp256k1Scalar::default(), max);
    check_representation(Secp256k1Scalar::from_le_bytes(max).unwrap(), before_max);
    assert_eq!(Secp256k1Scalar::default().to_le_bytes(), [0; 32]);
    assert_eq!(
        Secp256k1Scalar::from_le_bytes(max).unwrap().to_le_bytes(),
        max
    );
    assert!(Secp256k1Scalar::from_le_bytes(modulus).is_none());
    let base = Secp256k1Base::from_le_bytes(modulus).unwrap();
    // The same width does not admit a base value whose shifted encoding
    // exceeds the scalar field's bound.
    let base_max = le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e");
    let large_base = Secp256k1Base::from_le_bytes(base_max).unwrap();
    let cell = try_constructor_cell::<_, DefaultDB>(large_base).unwrap();
    assert!(read_cell::<Secp256k1Scalar, _>(&cell).is_err());
    assert_eq!(base.to_le_bytes(), modulus);

    let mut boundary = [0; 32];
    boundary[24] = 1;
    let mut below_boundary = [0; 32];
    below_boundary[..24].fill(255);
    check_representation(
        Secp256k1Scalar::from_le_bytes(boundary).unwrap(),
        below_boundary,
    );
    let mut above_boundary = boundary;
    above_boundary[0] = 1;
    check_representation(
        Secp256k1Scalar::from_le_bytes(above_boundary).unwrap(),
        boundary,
    );
}

#[test]
fn foreign_field_decoders_refuse_malformed_and_noncanonical_inputs() {
    for atoms in [
        vec![vec![]],
        vec![vec![], vec![], vec![]],
        vec![vec![1; 25], vec![]],
        vec![vec![], vec![1; 9]],
        vec![vec![1, 0], vec![]],
        vec![vec![], vec![0]],
        vec![vec![255; 24], vec![255; 8]],
    ] {
        let value = Value(atoms.into_iter().map(ValueAtom).collect());
        assert!(Secp256k1Base::try_from(&*value).is_err());
        assert!(Secp256k1Scalar::try_from(&*value).is_err());
    }
    let zero = Field::from(0u64);
    let mut overwide_low_bytes = [0; 25];
    overwide_low_bytes[24] = 1;
    let low_overflow = Field::from_le_bytes(&overwide_low_bytes).unwrap();
    let high_overflow = Field::from(1u128 << 64);
    for fields in [
        vec![zero],
        vec![zero, zero, zero],
        vec![low_overflow, zero],
        vec![zero, high_overflow],
    ] {
        assert!(Secp256k1Base::from_field_repr(&fields).is_none());
        assert!(Secp256k1Scalar::from_field_repr(&fields).is_none());
    }
}
