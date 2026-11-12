// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use midnight_base_crypto::fab::{
    Aligned, Alignment, AlignmentAtom, AlignmentSegment, InvalidBuiltinDecode, Value, ValueAtom,
    ValueSlice,
};
use midnight_transient_crypto::fab::AlignmentExt;

use crate::{BinaryHashRepr, FieldRepr, Fr, FromFieldRepr, MemWrite};

// The pinned Compact runtime's maximum values, in little-endian order.
const BASE_MAX: [u8; 32] = [
    0x2e, 0xfc, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];
const SCALAR_MAX: [u8; 32] = [
    0x40, 0x41, 0x36, 0xd0, 0x8c, 0x5e, 0xd2, 0xbf, 0x3b, 0xa0, 0x48, 0xaf, 0xe6, 0xdc, 0xae, 0xba,
    0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

const LOW_BYTES: usize = 24;
const HIGH_BYTES: usize = 8;

/// A canonical element of the secp256k1 coordinate field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256k1Base([u8; 32]);

/// A canonical element of the secp256k1 scalar field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256k1Scalar([u8; 32]);

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

macro_rules! impl_secp256k1_field {
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

impl_secp256k1_field!(Secp256k1Base, BASE_MAX);
impl_secp256k1_field!(Secp256k1Scalar, SCALAR_MAX);
