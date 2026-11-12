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

//! Host-side recovery for a Compact witness.

use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

use crate::{
    BoundedUint, CompactError, FixedBytes, Secp256k1Base, Secp256k1Point, Secp256k1Scalar,
};

/// Recover a finite secp256k1 public key from an ECDSA signature and a 32-byte prehash.
///
/// The low bit of the recovery ID selects the parity of the nonce point's y
/// coordinate. The high bit selects whether its x coordinate was reduced
/// modulo the scalar order. Both low-s and high-s signatures are accepted.
pub fn secp256k1_ecdsa_recover(
    digest: FixedBytes<32>,
    r: Secp256k1Scalar,
    s: Secp256k1Scalar,
    recovery_id: BoundedUint<3>,
) -> Result<Secp256k1Point, CompactError> {
    let mut r = r.to_le_bytes();
    let mut s = s.to_le_bytes();
    r.reverse();
    s.reverse();
    let signature =
        Signature::from_scalars(r, s).map_err(|_| CompactError::InvalidSecp256k1Signature)?;
    let id = recovery_id.value();
    let id = RecoveryId::new(id & 1 != 0, id & 2 != 0);
    let recovered = VerifyingKey::recover_from_prehash(&digest.into_array(), &signature, id)
        .map_err(|_| CompactError::Secp256k1RecoveryFailed)?;
    let point = recovered.to_sec1_point(false);
    let coordinate = |value: Option<&[u8]>| {
        let mut bytes: [u8; 32] = value?.try_into().ok()?;
        bytes.reverse();
        Secp256k1Base::from_le_bytes(bytes)
    };
    let x =
        coordinate(point.x().map(|x| x.as_slice())).ok_or(CompactError::Secp256k1RecoveryFailed)?;
    let y =
        coordinate(point.y().map(|y| y.as_slice())).ok_or(CompactError::Secp256k1RecoveryFailed)?;
    Secp256k1Point::from_affine(x, y).ok_or(CompactError::Secp256k1RecoveryFailed)
}
