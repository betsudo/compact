// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use midnight_base_crypto::fab::{
    Aligned, Alignment, AlignmentAtom, AlignmentSegment, InvalidBuiltinDecode, Value, ValueAtom,
    ValueSlice,
};
use midnight_curves::k256::K256Affine;
use midnight_transient_crypto::curve::FR_BYTES;

use crate::{BinaryHashRepr, FieldRepr, Fr, FromFieldRepr, MemWrite, Secp256k1Base};

/// A checked secp256k1 point. The default value is the identity.
#[derive(Clone, Copy, Debug)]
pub struct Secp256k1Point {
    x: Secp256k1Base,
    y: Secp256k1Base,
    identity: bool,
}

impl Secp256k1Point {
    /// Construct a finite point only if the coordinates lie on secp256k1.
    pub fn from_affine(x: Secp256k1Base, y: Secp256k1Base) -> Option<Self> {
        K256Affine::from_xy(x.into_inner(), y.into_inner())?;
        Some(Self {
            x,
            y,
            identity: false,
        })
    }

    pub fn is_identity(&self) -> bool {
        self.identity
    }

    /// Return the coordinates of a finite point.
    pub fn coordinates(&self) -> Option<(Secp256k1Base, Secp256k1Base)> {
        (!self.identity).then_some((self.x, self.y))
    }

    fn encoding_coordinates(&self) -> (Secp256k1Base, Secp256k1Base) {
        self.coordinates().unwrap_or_default()
    }
}

impl Default for Secp256k1Point {
    fn default() -> Self {
        Self {
            x: Secp256k1Base::default(),
            y: Secp256k1Base::default(),
            identity: true,
        }
    }
}

impl PartialEq for Secp256k1Point {
    fn eq(&self, other: &Self) -> bool {
        self.coordinates() == other.coordinates()
    }
}

impl Eq for Secp256k1Point {}

impl Aligned for Secp256k1Point {
    fn alignment() -> Alignment {
        let mut alignment = Secp256k1Base::alignment().0;
        alignment.extend(Secp256k1Base::alignment().0);
        alignment.push(AlignmentSegment::Atom(AlignmentAtom::Field));
        Alignment(alignment)
    }
}

impl From<Secp256k1Point> for Value {
    fn from(point: Secp256k1Point) -> Self {
        let (x, y) = point.encoding_coordinates();
        let mut value = Self::from(x);
        value.0.extend(Self::from(y).0);
        value.0.push(ValueAtom::from(point.identity));
        value
    }
}

impl TryFrom<&ValueSlice> for Secp256k1Point {
    type Error = InvalidBuiltinDecode;

    fn try_from(value: &ValueSlice) -> Result<Self, Self::Error> {
        let [_, _, _, _, flag] = &value.0 else {
            return Err(InvalidBuiltinDecode("Secp256k1Point"));
        };
        if !flag.is_in_normal_form() {
            return Err(InvalidBuiltinDecode("Secp256k1Point"));
        }
        let x = Secp256k1Base::try_from(&value[..2])?;
        let y = Secp256k1Base::try_from(&value[2..4])?;
        let identity = bool::try_from(flag).map_err(|_| InvalidBuiltinDecode("Secp256k1Point"))?;
        if identity {
            Ok(Self { x, y, identity })
        } else {
            // A typed Rust value can enter a generated circuit without another
            // argument check. Validate finite points as part of FAB ingress.
            Self::from_affine(x, y).ok_or(InvalidBuiltinDecode("Secp256k1Point"))
        }
    }
}

impl FieldRepr for Secp256k1Point {
    fn field_repr<W: MemWrite<Fr>>(&self, writer: &mut W) {
        let (x, y) = self.encoding_coordinates();
        x.field_repr(writer);
        y.field_repr(writer);
        writer.write(&[Fr::from(self.identity)]);
    }

    fn field_size(&self) -> usize {
        Self::FIELD_SIZE
    }
}

impl FromFieldRepr for Secp256k1Point {
    const FIELD_SIZE: usize = 2 * Secp256k1Base::FIELD_SIZE + 1;

    fn from_field_repr(repr: &[Fr]) -> Option<Self> {
        let [_, _, _, _, flag] = repr else {
            return None;
        };
        let x = Secp256k1Base::from_field_repr(&repr[..2])?;
        let y = Secp256k1Base::from_field_repr(&repr[2..4])?;
        // ZKIR requires a point on the curve or the canonical identity encoding.
        if *flag == Fr::from(false) {
            Self::from_affine(x, y)
        } else if *flag == Fr::from(true) && (x, y) == Default::default() {
            Some(Self::default())
        } else {
            None
        }
    }
}

impl BinaryHashRepr for Secp256k1Point {
    fn binary_repr<W: MemWrite<u8>>(&self, writer: &mut W) {
        let (x, y) = self.encoding_coordinates();
        x.binary_repr(writer);
        y.binary_repr(writer);
        Fr::from(self.identity).binary_repr(writer);
    }

    fn binary_len(&self) -> usize {
        self.x.binary_len() + self.y.binary_len() + FR_BYTES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fab_decode_retains_ignored_identity_coordinates() {
        let mut bytes = [0; 32];
        bytes[0] = 4;
        let four = Secp256k1Base::from_le_bytes(bytes).unwrap();
        let mut value = Value::from(four);
        value.0.extend(Value::from(four).0);
        value.0.push(ValueAtom::from(true));

        let point = Secp256k1Point::try_from(&*value).unwrap();
        assert_eq!((point.x, point.y), (four, four));
        assert_eq!(point, Secp256k1Point::default());
        assert_eq!(Value::from(point), Value::from(Secp256k1Point::default()));
    }
}
