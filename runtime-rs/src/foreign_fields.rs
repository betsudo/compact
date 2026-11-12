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

use std::cmp::Ordering;
use std::ops::{Add, Mul, Neg, Sub};

use ff::{Field, PrimeField};
use midnight_base_crypto::fab::{
    Aligned, Alignment, AlignmentAtom, AlignmentSegment, InvalidBuiltinDecode, Value, ValueAtom,
    ValueSlice,
};
use midnight_curves::{k256, p256};
use midnight_transient_crypto::fab::AlignmentExt;

use crate::{BinaryHashRepr, CompactError, FieldRepr, Fr, FromFieldRepr, MemWrite};

// The pinned Compact runtime's maximum values, in little-endian order.
const K1_BASE_MAX: [u8; 32] = [
    0x2e, 0xfc, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];
const K1_SCALAR_MAX: [u8; 32] = [
    0x40, 0x41, 0x36, 0xd0, 0x8c, 0x5e, 0xd2, 0xbf, 0x3b, 0xa0, 0x48, 0xaf, 0xe6, 0xdc, 0xae, 0xba,
    0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

const R1_BASE_MAX: [u8; 32] = [
    0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
];
const R1_SCALAR_MAX: [u8; 32] = [
    0x50, 0x25, 0x63, 0xfc, 0xc2, 0xca, 0xb9, 0xf3, 0x84, 0x9e, 0x17, 0xa7, 0xad, 0xfa, 0xe6, 0xbc,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
];

const LOW_BYTES: usize = 24;
const HIGH_BYTES: usize = 8;

/// A canonical element of the secp256k1 coordinate field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256k1Base([u8; 32]);

/// A canonical element of the secp256k1 scalar field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256k1Scalar([u8; 32]);

/// A canonical element of the secp256r1 coordinate field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256r1Base([u8; 32]);

/// A canonical element of the secp256r1 scalar field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256r1Scalar([u8; 32]);

fn compare_le(left: &[u8; 32], right: &[u8; 32]) -> Ordering {
    left.iter().rev().cmp(right.iter().rev())
}

fn reduce(bytes: [u8; 32], max: [u8; 32]) -> [u8; 32] {
    // Both moduli exceed 2^255, so a 256-bit input needs at most one subtraction.
    // The initial borrow subtracts the extra one in the modulus (max + 1).
    let mut remainder = bytes;
    let mut borrow = true;
    for (byte, limit) in remainder.iter_mut().zip(max) {
        let (difference, first_borrow) = byte.overflowing_sub(limit);
        let (difference, second_borrow) = difference.overflowing_sub(u8::from(borrow));
        *byte = difference;
        borrow = first_borrow || second_borrow;
    }
    if borrow { bytes } else { remainder }
}

// Compact's FAB representation stores (value - 1) modulo the field order.
fn shifted(mut value: [u8; 32], max: [u8; 32]) -> [u8; 32] {
    if value == [0; 32] {
        return max;
    }
    for byte in &mut value {
        let (result, borrow) = byte.overflowing_sub(1);
        *byte = result;
        if !borrow {
            break;
        }
    }
    value
}

fn decode(value: &ValueSlice, max: [u8; 32]) -> Option<[u8; 32]> {
    let [low, high] = &value.0 else {
        return None;
    };
    if low.0.len() > LOW_BYTES
        || high.0.len() > HIGH_BYTES
        || !low.is_in_normal_form()
        || !high.is_in_normal_form()
    {
        return None;
    }
    let mut value = [0; 32];
    value[..low.0.len()].copy_from_slice(&low.0);
    value[LOW_BYTES..LOW_BYTES + high.0.len()].copy_from_slice(&high.0);
    match compare_le(&value, &max) {
        Ordering::Greater => None,
        Ordering::Equal => Some([0; 32]),
        Ordering::Less => {
            for byte in &mut value {
                let (result, carry) = byte.overflowing_add(1);
                *byte = result;
                if !carry {
                    break;
                }
            }
            Some(value)
        }
    }
}

macro_rules! impl_foreign_field_arithmetic {
    ($field:ident, $inner:ty) => {
        impl $field {
            pub(super) fn into_inner(self) -> $inner {
                let mut bytes = self.0;
                bytes.reverse();
                <$inner>::from_repr(bytes.into()).expect("field element is canonical")
            }

            pub(super) fn from_inner(value: $inner) -> Self {
                let mut bytes: [u8; 32] = value.to_repr().into();
                bytes.reverse();
                Self(bytes)
            }
        }

        impl Add for $field {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self::from_inner(self.into_inner() + rhs.into_inner())
            }
        }

        impl Sub for $field {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self::from_inner(self.into_inner() - rhs.into_inner())
            }
        }

        impl Mul for $field {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self {
                Self::from_inner(self.into_inner() * rhs.into_inner())
            }
        }
    };
}

macro_rules! impl_foreign_field_unary {
    ($field:ident, $zero_inverse:ident) => {
        impl $field {
            /// Return the multiplicative inverse, or a field-specific error for zero.
            pub fn inverse(self) -> Result<Self, CompactError> {
                Option::from(Field::invert(&self.into_inner()))
                    .map(Self::from_inner)
                    .ok_or(CompactError::$zero_inverse)
            }
        }

        impl Neg for $field {
            type Output = Self;

            fn neg(self) -> Self {
                Self::from_inner(-self.into_inner())
            }
        }
    };
}

macro_rules! impl_foreign_field {
    ($field:ident, $max:ident) => {
        impl $field {
            /// Decode a canonical little-endian field element.
            pub fn from_le_bytes(bytes: [u8; 32]) -> Option<Self> {
                compare_le(&bytes, &$max).is_le().then_some(Self(bytes))
            }

            /// Reduce an unsigned little-endian integer modulo this field's order.
            pub fn from_le_bytes_reduced(bytes: [u8; 32]) -> Self {
                Self(reduce(bytes, $max))
            }

            /// Return the field element's canonical little-endian bytes.
            pub fn to_le_bytes(self) -> [u8; 32] {
                self.0
            }
        }

        impl Aligned for $field {
            fn alignment() -> Alignment {
                Alignment(vec![
                    AlignmentSegment::Atom(AlignmentAtom::Bytes {
                        length: LOW_BYTES as u32,
                    }),
                    AlignmentSegment::Atom(AlignmentAtom::Bytes {
                        length: HIGH_BYTES as u32,
                    }),
                ])
            }
        }

        impl From<$field> for Value {
            fn from(value: $field) -> Self {
                let value = shifted(value.0, $max);
                Value(vec![
                    ValueAtom::from(&value[..LOW_BYTES]),
                    ValueAtom::from(&value[LOW_BYTES..]),
                ])
            }
        }

        impl TryFrom<&ValueSlice> for $field {
            type Error = InvalidBuiltinDecode;

            fn try_from(value: &ValueSlice) -> Result<Self, Self::Error> {
                decode(value, $max)
                    .map(Self)
                    .ok_or(InvalidBuiltinDecode(stringify!($field)))
            }
        }

        impl FieldRepr for $field {
            fn field_repr<W: MemWrite<Fr>>(&self, writer: &mut W) {
                let value = shifted(self.0, $max);
                let low =
                    Fr::from_le_bytes(&value[..LOW_BYTES]).expect("192-bit limb fits a Field");
                let high =
                    Fr::from_le_bytes(&value[LOW_BYTES..]).expect("64-bit limb fits a Field");
                writer.write(&[low, high]);
            }

            fn field_size(&self) -> usize {
                Self::FIELD_SIZE
            }
        }

        impl FromFieldRepr for $field {
            const FIELD_SIZE: usize = 2;

            fn from_field_repr(repr: &[Fr]) -> Option<Self> {
                if repr.len() != Self::FIELD_SIZE {
                    return None;
                }
                let value = Self::alignment().parse_field_repr(repr)?;
                // The field parser supplies width-padded byte atoms.
                let value = Value(
                    value
                        .value
                        .0
                        .into_iter()
                        .map(ValueAtom::normalize)
                        .collect(),
                );
                Self::try_from(&*value).ok()
            }
        }

        impl BinaryHashRepr for $field {
            fn binary_repr<W: MemWrite<u8>>(&self, writer: &mut W) {
                writer.write(&shifted(self.0, $max));
            }

            fn binary_len(&self) -> usize {
                32
            }
        }
    };
}

impl_foreign_field!(Secp256k1Base, K1_BASE_MAX);
impl_foreign_field!(Secp256k1Scalar, K1_SCALAR_MAX);
impl_foreign_field!(Secp256r1Base, R1_BASE_MAX);
impl_foreign_field!(Secp256r1Scalar, R1_SCALAR_MAX);

impl_foreign_field_arithmetic!(Secp256k1Base, k256::Fp);
impl_foreign_field_arithmetic!(Secp256k1Scalar, k256::Fq);
impl_foreign_field_arithmetic!(Secp256r1Base, p256::Fp);
impl_foreign_field_arithmetic!(Secp256r1Scalar, p256::Fq);

impl_foreign_field_unary!(Secp256k1Base, Secp256k1BaseZeroInverse);
impl_foreign_field_unary!(Secp256k1Scalar, Secp256k1ScalarZeroInverse);
impl_foreign_field_unary!(Secp256r1Base, Secp256r1BaseZeroInverse);
impl_foreign_field_unary!(Secp256r1Scalar, Secp256r1ScalarZeroInverse);
