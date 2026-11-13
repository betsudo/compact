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

use group::Group;
use midnight_base_crypto::fab::{Aligned, Alignment, InvalidBuiltinDecode, Value, ValueSlice};
use midnight_curves::curve25519::{Curve25519Affine, Curve25519Subgroup};

use crate::{BinaryHashRepr, CompactError, Curve25519Base, Curve25519Scalar, FieldRepr, Fr, FromFieldRepr, MemWrite};

/// A point in the prime-order subgroup of the Curve25519 Edwards curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Curve25519Point(Curve25519Subgroup);

impl Curve25519Point {
    /// Accept canonical coordinates only when they describe a subgroup point.
    pub fn from_affine(x: Curve25519Base, y: Curve25519Base) -> Option<Self> {
        let affine = Curve25519Affine::from_xy(x.into_inner(), y.into_inner())?;
        Curve25519Subgroup::from_edwards(affine.to_edwards()).map(Self)
    }

    /// Multiply the standard Edwards generator by a canonical scalar.
    pub fn mul_generator(scalar: Curve25519Scalar) -> Self {
        Self(Curve25519Subgroup::generator() * scalar.into_inner())
    }

    pub fn x(self) -> Result<Curve25519Base, CompactError> {
        Ok(self.coordinates().0)
    }

    pub fn y(self) -> Result<Curve25519Base, CompactError> {
        Ok(self.coordinates().1)
    }

    /// Return both affine coordinates, including (0, 1) for identity.
    pub fn coordinates(self) -> (Curve25519Base, Curve25519Base) {
        let affine = Curve25519Affine::from_edwards(*self.0.inner());
        (
            Curve25519Base::from_inner(*affine.x()),
            Curve25519Base::from_inner(*affine.y()),
        )
    }
}

impl std::ops::Add for Curve25519Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Mul<Curve25519Scalar> for Curve25519Point {
    type Output = Self;

    fn mul(self, scalar: Curve25519Scalar) -> Self {
        Self(self.0 * scalar.into_inner())
    }
}

impl Aligned for Curve25519Point {
    fn alignment() -> Alignment {
        let mut alignment = Curve25519Base::alignment().0;
        alignment.extend(Curve25519Base::alignment().0);
        Alignment(alignment)
    }
}

impl From<Curve25519Point> for Value {
    fn from(point: Curve25519Point) -> Self {
        let (x, y) = point.coordinates();
        let mut value = Self::from(x);
        value.0.extend(Self::from(y).0);
        value
    }
}

impl TryFrom<&ValueSlice> for Curve25519Point {
    type Error = InvalidBuiltinDecode;

    fn try_from(value: &ValueSlice) -> Result<Self, Self::Error> {
        if value.0.len() != 4 {
            return Err(InvalidBuiltinDecode("Curve25519Point"));
        }
        let x = Curve25519Base::try_from(&value[..2])?;
        let y = Curve25519Base::try_from(&value[2..])?;
        Self::from_affine(x, y).ok_or(InvalidBuiltinDecode("Curve25519Point"))
    }
}

impl FieldRepr for Curve25519Point {
    fn field_repr<W: MemWrite<Fr>>(&self, writer: &mut W) {
        let (x, y) = self.coordinates();
        x.field_repr(writer);
        y.field_repr(writer);
    }

    fn field_size(&self) -> usize {
        Self::FIELD_SIZE
    }
}

impl FromFieldRepr for Curve25519Point {
    const FIELD_SIZE: usize = 2 * Curve25519Base::FIELD_SIZE;

    fn from_field_repr(repr: &[Fr]) -> Option<Self> {
        if repr.len() != Self::FIELD_SIZE {
            return None;
        }
        let (x, y) = repr.split_at(Curve25519Base::FIELD_SIZE);
        Self::from_affine(
            Curve25519Base::from_field_repr(x)?,
            Curve25519Base::from_field_repr(y)?,
        )
    }
}

impl BinaryHashRepr for Curve25519Point {
    fn binary_repr<W: MemWrite<u8>>(&self, writer: &mut W) {
        let (x, y) = self.coordinates();
        x.binary_repr(writer);
        y.binary_repr(writer);
    }

    fn binary_len(&self) -> usize {
        64
    }
}
