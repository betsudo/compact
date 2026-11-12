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
    Aligned, Alignment, AlignmentAtom, AlignmentSegment, InvalidBuiltinDecode, Value, ValueAtom,
    ValueSlice,
};
use midnight_curves::p256::affine_from_xy;
use midnight_transient_crypto::curve::FR_BYTES;

use crate::{BinaryHashRepr, FieldRepr, Fr, FromFieldRepr, MemWrite, Secp256r1Base};

/// A checked NIST P-256 point. The default value is the canonical identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Secp256r1Point(Option<(Secp256r1Base, Secp256r1Base)>);

impl Secp256r1Point {
    /// Construct a finite point only if the coordinates lie on P-256.
    pub fn from_affine(x: Secp256r1Base, y: Secp256r1Base) -> Option<Self> {
        affine_from_xy(x.into_inner(), y.into_inner())?;
        Some(Self(Some((x, y))))
    }

    pub fn is_identity(&self) -> bool {
        self.0.is_none()
    }

    /// Return the coordinates of a finite point.
    pub fn coordinates(&self) -> Option<(Secp256r1Base, Secp256r1Base)> {
        self.0
    }
}

impl Aligned for Secp256r1Point {
    fn alignment() -> Alignment {
        let mut alignment = Secp256r1Base::alignment().0;
        alignment.extend(Secp256r1Base::alignment().0);
        alignment.push(AlignmentSegment::Atom(AlignmentAtom::Field));
        Alignment(alignment)
    }
}

impl From<Secp256r1Point> for Value {
    fn from(point: Secp256r1Point) -> Self {
        let (x, y) = point.0.unwrap_or_default();
        let mut value = Self::from(x);
        value.0.extend(Self::from(y).0);
        value.0.push(ValueAtom::from(point.is_identity()));
        value
    }
}

impl TryFrom<&ValueSlice> for Secp256r1Point {
    type Error = InvalidBuiltinDecode;

    fn try_from(value: &ValueSlice) -> Result<Self, Self::Error> {
        let [_, _, _, _, flag] = &value.0 else {
            return Err(InvalidBuiltinDecode("Secp256r1Point"));
        };
        if !flag.is_in_normal_form() {
            return Err(InvalidBuiltinDecode("Secp256r1Point"));
        }
        let identity = bool::try_from(flag).map_err(|_| InvalidBuiltinDecode("Secp256r1Point"))?;
        let x = Secp256r1Base::try_from(&value[..2])?;
        let y = Secp256r1Base::try_from(&value[2..4])?;
        if identity {
            Ok(Self::default())
        } else {
            Self::from_affine(x, y).ok_or(InvalidBuiltinDecode("Secp256r1Point"))
        }
    }
}

impl FieldRepr for Secp256r1Point {
    fn field_repr<W: MemWrite<Fr>>(&self, writer: &mut W) {
        let (x, y) = self.0.unwrap_or_default();
        x.field_repr(writer);
        y.field_repr(writer);
        writer.write(&[Fr::from(self.is_identity())]);
    }

    fn field_size(&self) -> usize {
        Self::FIELD_SIZE
    }
}

impl FromFieldRepr for Secp256r1Point {
    const FIELD_SIZE: usize = 2 * Secp256r1Base::FIELD_SIZE + 1;

    fn from_field_repr(repr: &[Fr]) -> Option<Self> {
        let [_, _, _, _, flag] = repr else {
            return None;
        };
        let x = Secp256r1Base::from_field_repr(&repr[..2])?;
        let y = Secp256r1Base::from_field_repr(&repr[2..4])?;
        // Native ZKIR inputs require canonical identity coordinates.
        if *flag == Fr::from(false) {
            Self::from_affine(x, y)
        } else if *flag == Fr::from(true) && (x, y) == Default::default() {
            Some(Self::default())
        } else {
            None
        }
    }
}

impl BinaryHashRepr for Secp256r1Point {
    fn binary_repr<W: MemWrite<u8>>(&self, writer: &mut W) {
        let (x, y) = self.0.unwrap_or_default();
        x.binary_repr(writer);
        y.binary_repr(writer);
        Fr::from(self.is_identity()).binary_repr(writer);
    }

    fn binary_len(&self) -> usize {
        let (x, y) = self.0.unwrap_or_default();
        x.binary_len() + y.binary_len() + FR_BYTES
    }
}
