// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use midnight_base_crypto::fab::{
    AlignedValue, Alignment, AlignmentAtom, AlignmentSegment, Value, ValueAtom, ValueSlice,
};
use midnight_compact_runtime::ledger::{CellValue, DefaultDB, read_cell, try_constructor_cell};
use midnight_compact_runtime::{
    BinaryHashRepr, CompactError, Field, FieldRepr, FromFieldRepr, HashInputValue, Secp256k1Base,
    Secp256k1Scalar,
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
fn byte_casts_preserve_canonical_values_across_the_limb_boundary() {
    let zero = [0; 32];
    let mut one = zero;
    one[0] = 1;
    let mut below_boundary = zero;
    below_boundary[..24].fill(255);
    let mut boundary = zero;
    boundary[24] = 1;
    let mut above_boundary = boundary;
    above_boundary[0] = 1;
    for input in [zero, one, below_boundary, boundary, above_boundary] {
        let base = Secp256k1Base::from_le_bytes_reduced(input);
        let scalar = Secp256k1Scalar::from_le_bytes_reduced(input);
        assert_eq!(base.to_le_bytes(), input);
        assert_eq!(scalar.to_le_bytes(), input);
        assert_eq!(Some(base), Secp256k1Base::from_le_bytes(input));
        assert_eq!(Some(scalar), Secp256k1Scalar::from_le_bytes(input));
    }
}

#[test]
fn byte_casts_reduce_each_modulus_and_keep_strict_decode_separate() {
    let zero = [0; 32];
    let mut one = zero;
    one[0] = 1;
    let base_max = le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e");
    let scalar_max = le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140");
    assert_eq!(
        Some(Secp256k1Base::from_le_bytes_reduced(base_max)),
        Secp256k1Base::from_le_bytes(base_max)
    );
    assert_eq!(
        Some(Secp256k1Scalar::from_le_bytes_reduced(scalar_max)),
        Secp256k1Scalar::from_le_bytes(scalar_max)
    );

    for (input, expected) in [
        (
            le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"),
            zero,
        ),
        (
            le_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30"),
            one,
        ),
        (
            [255; 32],
            le_bytes("00000000000000000000000000000000000000000000000000000001000003d0"),
        ),
    ] {
        assert!(Secp256k1Base::from_le_bytes(input).is_none());
        assert_eq!(
            Secp256k1Base::from_le_bytes_reduced(input).to_le_bytes(),
            expected
        );
    }
    for (input, expected) in [
        (
            le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141"),
            zero,
        ),
        (
            le_bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142"),
            one,
        ),
        (
            [255; 32],
            le_bytes("000000000000000000000000000000014551231950b75fc4402da1732fc9bebe"),
        ),
    ] {
        assert!(Secp256k1Scalar::from_le_bytes(input).is_none());
        assert_eq!(
            Secp256k1Scalar::from_le_bytes_reduced(input).to_le_bytes(),
            expected
        );
    }
}

#[test]
fn base_arithmetic_reduces_and_keeps_the_canonical_encoding() {
    let base = |hex| Secp256k1Base::from_le_bytes(le_bytes(hex)).unwrap();
    let zero = Secp256k1Base::default();
    let one = base("0000000000000000000000000000000000000000000000000000000000000001");
    let max = base("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e");
    assert_eq!(max + one, zero);
    assert_eq!(zero - one, max);
    assert_eq!(max - max, zero);
    assert_eq!(max * max, one);
    assert_eq!(max * zero, zero);
    check_representation(max + one, max.to_le_bytes());
    check_representation(max * max, [0; 32]);

    let x = base("9c4467d18df9282eb7124b451afecac5f4827503316aacda34a863e367e0c85c");
    let y = base("f58b1b91dc876f4824102f1395c6d175943cc503cbaf6092d3f319bb042f9d68");
    assert_eq!(
        x + y,
        base("91cf83636a809776db227a58b0c59c3b88bf3a06fd1a0d6d089b7d9f6c106995")
    );
    assert_eq!(
        x - y,
        base("a6b94c3fb171b8e693021c318537f9506045afff65bb4c4760b54a2763b12723")
    );
    assert_eq!(
        x * y,
        base("2ea9a138e9db7bd49a51f8cf4f2bd3ffae3b6707cfe796a3046ebabff71ccd90")
    );
}

#[test]
fn scalar_arithmetic_reduces_with_the_scalar_order() {
    let scalar = |hex| Secp256k1Scalar::from_le_bytes(le_bytes(hex)).unwrap();
    let zero = Secp256k1Scalar::default();
    let one = scalar("0000000000000000000000000000000000000000000000000000000000000001");
    let max = scalar("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140");
    assert_eq!(max + one, zero);
    assert_eq!(zero - one, max);
    assert_eq!(max - max, zero);
    assert_eq!(max * max, one);
    assert_eq!(max * zero, zero);
    check_representation(max + one, max.to_le_bytes());
    check_representation(max * max, [0; 32]);

    let x = scalar("9c4467d18df9282eb7124b451afecac5f4827503316aacda34a863e367e0c85c");
    let y = scalar("f58b1b91dc876f4824102f1395c6d175943cc503cbaf6092d3f319bb042f9d68");
    assert_eq!(
        x + y,
        scalar("91cf83636a809776db227a58b0c59c3cce105d204dd16d3148c91f119bda2483")
    );
    assert_eq!(
        x - y,
        scalar("a6b94c3fb171b8e693021c318537f94f1af48ce61503ec832087a8b533e76c35")
    );
    assert_eq!(
        x * y,
        scalar("75fa26ca654d5c64672d3951b8efbb31bc29c6a54de3af6d7e84724904a3ef0e")
    );
}

#[test]
fn base_negation_and_inverse_use_the_coordinate_field() {
    let base = |hex| Secp256k1Base::from_le_bytes(le_bytes(hex)).unwrap();
    let zero = Secp256k1Base::default();
    let one = base("0000000000000000000000000000000000000000000000000000000000000001");
    let max = base("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e");
    let zero_error = zero.inverse().unwrap_err();
    assert_eq!(zero_error, CompactError::Secp256k1BaseZeroInverse);
    assert_eq!(
        zero_error.to_string(),
        "secp256k1 base field has no inverse for 0"
    );
    assert_eq!(-zero, zero);
    assert_eq!(-one, max);
    assert_eq!(-max, one);
    assert_eq!(one.inverse().unwrap(), one);
    assert_eq!(max.inverse().unwrap(), max);

    let x = base("9c4467d18df9282eb7124b451afecac5f4827503316aacda34a863e367e0c85c");
    let negative = -x;
    let inverse = x.inverse().unwrap();
    assert_eq!(
        negative,
        base("63bb982e7206d7d148edb4bae501353a0b7d8afcce955325cb579c1b981f33d3")
    );
    assert_eq!(
        inverse,
        base("b02a2d157efd8e8e93b66c409e38291bdc7a73b12ca5ad9729f77a668025c909")
    );
    assert_eq!(x + negative, zero);
    assert_eq!(x * inverse, one);
    assert_eq!(inverse.inverse().unwrap(), x);
    check_representation(
        negative,
        le_bytes("63bb982e7206d7d148edb4bae501353a0b7d8afcce955325cb579c1b981f33d2"),
    );
    check_representation(
        inverse,
        le_bytes("b02a2d157efd8e8e93b66c409e38291bdc7a73b12ca5ad9729f77a668025c908"),
    );
}

#[test]
fn scalar_negation_and_inverse_use_the_scalar_order() {
    let scalar = |hex| Secp256k1Scalar::from_le_bytes(le_bytes(hex)).unwrap();
    let zero = Secp256k1Scalar::default();
    let one = scalar("0000000000000000000000000000000000000000000000000000000000000001");
    let max = scalar("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140");
    let zero_error = zero.inverse().unwrap_err();
    assert_eq!(zero_error, CompactError::Secp256k1ScalarZeroInverse);
    assert_eq!(
        zero_error.to_string(),
        "secp256k1 scalar field has no inverse for 0"
    );
    assert_eq!(-zero, zero);
    assert_eq!(-one, max);
    assert_eq!(-max, one);
    assert_eq!(one.inverse().unwrap(), one);
    assert_eq!(max.inverse().unwrap(), max);

    let x = scalar("9c4467d18df9282eb7124b451afecac5f4827503316aacda34a863e367e0c85c");
    let negative = -x;
    let inverse = x.inverse().unwrap();
    assert_eq!(
        negative,
        scalar("63bb982e7206d7d148edb4bae5013538c62c67e37dddf3618b29faa9685578e5")
    );
    assert_eq!(
        inverse,
        scalar("4401f18738c5f123ac5d37b243b4aaeccd97af85b56be498d008550fafd94b0b")
    );
    assert_eq!(x + negative, zero);
    assert_eq!(x * inverse, one);
    assert_eq!(inverse.inverse().unwrap(), x);
    check_representation(
        negative,
        le_bytes("63bb982e7206d7d148edb4bae5013538c62c67e37dddf3618b29faa9685578e4"),
    );
    check_representation(
        inverse,
        le_bytes("4401f18738c5f123ac5d37b243b4aaeccd97af85b56be498d008550fafd94b0a"),
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
