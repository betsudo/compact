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

use midnight_base_crypto::fab::{Aligned, Alignment, InvalidBuiltinDecode, Value, ValueSlice};
use midnight_transient_crypto::curve::EmbeddedFr;

use crate::{
    BinaryHashRepr, BoundedUint, CompactError, Field, FieldRepr, Fr, FromFieldRepr, MemWrite,
    WideUint,
};

/// A canonical scalar of the embedded Jubjub curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JubjubScalar(EmbeddedFr);

impl JubjubScalar {
    /// Decode an untrusted scalar without reducing it modulo the curve order.
    pub fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        EmbeddedFr::from_le_bytes(bytes).map(Self)
    }

    /// Explicit Compact conversion from a Field, reducing modulo the scalar order.
    pub fn from_field_reduced(value: Field) -> Self {
        let inner = EmbeddedFr::from_le_bytes_wide(&value.as_le_bytes())
            .expect("native Field fits in 32 bytes");
        Self(inner)
    }

    /// Embed the scalar's canonical value into the Compact native field.
    pub fn as_field(self) -> Field {
        Field::try_from(self.0).expect("embedded scalar fits the native field")
    }

    pub fn as_le_bytes(&self) -> Vec<u8> {
        self.0.as_le_bytes()
    }

    pub fn try_to_uint<const MAX: u128>(self) -> Result<BoundedUint<MAX>, CompactError> {
        let bytes = self.as_le_bytes();
        if bytes[16..].iter().any(|byte| *byte != 0) {
            return Err(CompactError::InvalidUnsignedValue);
        }
        BoundedUint::new(u128::from_le_bytes(
            bytes[..16].try_into().expect("low limb"),
        ))
    }

    pub fn try_to_wide_uint<const HIGH: u128, const LOW: u128>(
        self,
    ) -> Result<WideUint<HIGH, LOW>, CompactError> {
        WideUint::from_le_bytes(&self.as_le_bytes())
    }
}

impl TryFrom<Field> for JubjubScalar {
    type Error = CompactError;

    fn try_from(value: Field) -> Result<Self, Self::Error> {
        EmbeddedFr::try_from(value)
            .map(Self)
            .map_err(|()| CompactError::InvalidJubjubScalar)
    }
}

impl Aligned for JubjubScalar {
    fn alignment() -> Alignment {
        Field::alignment()
    }
}

impl From<JubjubScalar> for Value {
    fn from(value: JubjubScalar) -> Self {
        value.0.into()
    }
}

impl TryFrom<&ValueSlice> for JubjubScalar {
    type Error = InvalidBuiltinDecode;

    fn try_from(value: &ValueSlice) -> Result<Self, Self::Error> {
        let [atom] = &value.0 else {
            return Err(InvalidBuiltinDecode("JubjubScalar"));
        };
        if !atom.is_in_normal_form() {
            return Err(InvalidBuiltinDecode("JubjubScalar"));
        }
        Self::from_le_bytes(&atom.0).ok_or(InvalidBuiltinDecode("JubjubScalar"))
    }
}

impl FieldRepr for JubjubScalar {
    fn field_repr<W: MemWrite<Fr>>(&self, writer: &mut W) {
        self.as_field().field_repr(writer);
    }

    fn field_size(&self) -> usize {
        Self::FIELD_SIZE
    }
}

impl FromFieldRepr for JubjubScalar {
    const FIELD_SIZE: usize = 1;

    fn from_field_repr(repr: &[Fr]) -> Option<Self> {
        Self::try_from(Field::from_field_repr(repr)?).ok()
    }
}

impl BinaryHashRepr for JubjubScalar {
    fn binary_repr<W: MemWrite<u8>>(&self, writer: &mut W) {
        self.as_field().binary_repr(writer);
    }

    fn binary_len(&self) -> usize {
        self.as_field().binary_len()
    }
}
