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

use midnight_base_crypto::fab::ValueAtom;
use midnight_compact_runtime::{
    BinaryHashRepr, BoundedUint, CompactError, Field, FieldRepr, FromFieldRepr, JubjubScalar,
    fab::{AlignedValue, Value},
    ledger::{CellValue, DefaultDB, read_cell, try_constructor_cell},
};

const ORDER: &str = "b72cf7d65e0e97d08210c8cc932068a6003b3401013b6706a9af3365eab47d0e";

#[test]
fn canonical_scalar_matches_field_fab_and_rejects_noncanonical_atoms() {
    let mut order: [u8; 32] = hex::decode(ORDER).unwrap().try_into().unwrap();
    let modulus = Field::from_le_bytes(&order).unwrap();
    assert!(JubjubScalar::from_le_bytes(&order).is_none());
    assert_eq!(
        JubjubScalar::try_from(modulus),
        Err(CompactError::InvalidJubjubScalar)
    );
    assert_eq!(
        JubjubScalar::from_field_reduced(modulus),
        JubjubScalar::default()
    );
    assert!(JubjubScalar::from_field_repr(&[modulus]).is_none());
    assert!(JubjubScalar::decode_cell_value(&Value::from(modulus)).is_err());

    order[0] -= 1;
    let max = Field::from_le_bytes(&order).unwrap();
    for field in [Field::from(0u64), Field::from(1u64), max] {
        let value = JubjubScalar::try_from(field).unwrap();
        let aligned = AlignedValue::from(value);
        assert_eq!(aligned, AlignedValue::from(field));
        assert_eq!(value.field_vec(), vec![field]);
        assert_eq!(JubjubScalar::from_field_repr(&[field]), Some(value));
        let mut binary = Vec::new();
        value.binary_repr(&mut binary);
        assert_eq!(binary, field.as_le_bytes());
        let cell = try_constructor_cell::<JubjubScalar, DefaultDB>(value).unwrap();
        assert_eq!(read_cell::<JubjubScalar, DefaultDB>(&cell).unwrap(), value);
    }
    assert!(JubjubScalar::decode_cell_value(&Value(vec![ValueAtom(vec![1, 0])])).is_err());
}

#[test]
fn uint_cast_is_checked_in_both_carriers() {
    let max = JubjubScalar::try_from(Field::from(u128::from(u64::MAX))).unwrap();
    assert_eq!(
        max.try_to_uint::<{ u64::MAX as u128 }>(),
        BoundedUint::new(u128::from(u64::MAX))
    );
    let overflow = JubjubScalar::try_from(Field::from(1u128 << 64)).unwrap();
    assert!(overflow.try_to_uint::<{ u64::MAX as u128 }>().is_err());
    let mut bytes = [0; 32];
    bytes[16] = 1;
    let wide = JubjubScalar::from_le_bytes(&bytes).unwrap();
    assert!(wide.try_to_uint::<{ u128::MAX }>().is_err());
    assert!(wide.try_to_wide_uint::<1, 0>().is_ok());
}
