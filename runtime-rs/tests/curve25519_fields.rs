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

use ff::{Field as _, PrimeField};
use midnight_base_crypto::fab::{AlignedValue, Value, ValueAtom};
use midnight_compact_runtime::{
    BinaryHashRepr, Curve25519Base, Curve25519Scalar, Field, FieldRepr, FromFieldRepr,
    ledger::{DefaultDB, read_cell, try_constructor_cell},
};
use midnight_curves::curve25519::{Fp, Scalar};

#[test]
fn base_and_scalar_use_distinct_sdk_moduli() {
    let base_max: [u8; 32] = (-Fp::ONE).to_repr().into();
    let scalar_max: [u8; 32] = (-Scalar::ONE).to_repr();
    assert!(Curve25519Base::from_le_bytes(base_max).is_some());
    assert!(Curve25519Scalar::from_le_bytes(scalar_max).is_some());
    assert!(Curve25519Scalar::from_le_bytes(base_max).is_none());
    let mut base_order = base_max;
    base_order[0] += 1;
    let mut scalar_order = scalar_max;
    scalar_order[0] += 1;
    assert!(Curve25519Base::from_le_bytes(base_order).is_none());
    assert!(Curve25519Scalar::from_le_bytes(scalar_order).is_none());
    assert_eq!(
        Curve25519Base::from_le_bytes_reduced(base_order),
        Curve25519Base::default()
    );
    assert_eq!(
        Curve25519Scalar::from_le_bytes_reduced(scalar_order),
        Curve25519Scalar::default()
    );
    assert_eq!(
        Curve25519Scalar::from_le_bytes_reduced([255; 32]).to_le_bytes(),
        Scalar::from_bytes_mod_order([255; 32]).to_repr(),
    );
    assert_eq!(Curve25519Base::default().binary_vec(), base_max);
    assert_eq!(
        Curve25519Base::default().field_vec(),
        [
            Field::from_le_bytes(&base_max[..24]).unwrap(),
            Field::from_le_bytes(&base_max[24..]).unwrap(),
        ]
    );
}

#[test]
fn scalar_atom_boundary_matches_canonical_storage_and_hashing() {
    let mut at_boundary = [0; 32];
    at_boundary[25] = 0x10;
    let below = Curve25519Scalar::from_le_bytes(at_boundary).unwrap();
    at_boundary[0] = 1;
    let above = Curve25519Scalar::from_le_bytes(at_boundary).unwrap();
    let mut low_max = [255; 26];
    low_max[25] = 15;
    let expected = [
        (
            below,
            Value(vec![ValueAtom::from(&low_max[..]), 0u8.into()]),
        ),
        (above, Value(vec![0u8.into(), 1u8.into()])),
    ];
    for (value, atoms) in expected {
        let aligned = AlignedValue::from(value);
        assert_eq!(aligned.value, atoms);
        let fields = value.field_vec();
        assert_eq!(fields[0], Field::from_le_bytes(&atoms.0[0].0).unwrap());
        assert_eq!(fields[1], Field::from_le_bytes(&atoms.0[1].0).unwrap());
        assert_eq!(Curve25519Scalar::from_field_repr(&fields), Some(value));
        let mut binary = vec![0; 33];
        binary[..atoms.0[0].0.len()].copy_from_slice(&atoms.0[0].0);
        binary[26..26 + atoms.0[1].0.len()].copy_from_slice(&atoms.0[1].0);
        assert_eq!(value.binary_vec(), binary);
        let cell = try_constructor_cell::<_, DefaultDB>(value).unwrap();
        assert_eq!(read_cell::<Curve25519Scalar, _>(&cell).unwrap(), value);
    }
}

#[test]
fn scalar_refuses_limb_padding_and_out_of_field_encodings() {
    let mut low_padding = [0; 26];
    low_padding[25] = 0x10;
    let mut high_padding = [0; 7];
    high_padding[6] = 0x08;
    let mut out_of_field = [0; 7];
    out_of_field[6] = 0x02; // 2^253 after the 204-bit shift.
    for (low, high) in [
        (&low_padding[..], &[][..]),
        (&[][..], &high_padding[..]),
        (&[][..], &out_of_field[..]),
    ] {
        let atoms = Value(vec![ValueAtom::from(low), ValueAtom::from(high)]);
        assert!(Curve25519Scalar::try_from(&*atoms).is_err());
        let fields = [
            Field::from_le_bytes(low).unwrap(),
            Field::from_le_bytes(high).unwrap(),
        ];
        assert!(Curve25519Scalar::from_field_repr(&fields).is_none());
    }
}
