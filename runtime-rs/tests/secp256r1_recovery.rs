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

use midnight_compact_runtime::{
    BoundedUint, CompactError, FixedBytes, Secp256r1Base, Secp256r1Point, Secp256r1Scalar,
    secp256r1_ecdsa_recover,
};

fn bytes(s: &str) -> [u8; 32] {
    hex::decode(s).unwrap().try_into().unwrap()
}

fn little_endian(s: &str) -> [u8; 32] {
    let mut value = bytes(s);
    value.reverse();
    value
}

fn scalar(s: &str) -> Secp256r1Scalar {
    Secp256r1Scalar::from_le_bytes(little_endian(s)).unwrap()
}

fn point(x: &str, y: &str) -> Secp256r1Point {
    Secp256r1Point::from_affine(
        Secp256r1Base::from_le_bytes(little_endian(x)).unwrap(),
        Secp256r1Base::from_le_bytes(little_endian(y)).unwrap(),
    )
    .unwrap()
}

// Compact upstream 672bd0f, deterministic P-256 vector for the key 7.
#[test]
fn pinned_recovery_vector_distinguishes_parity_and_accepts_high_s() {
    let digest = FixedBytes::new(bytes(
        "3bca196b80f06134b677809eec68178aa82c4abff8bb4fd456d04210b502226e",
    ));
    let r = scalar("0ec69bf32d8bd19b906e7345c0420be3beb650724cde76981283e3aaa68ea694");
    let s = scalar("0953fe03f4f50653c666349cbe653f660c1e0764e1002a22f614d2152b6e94ae");
    let high_s = scalar("f6ac01fb0b0af9ad3999cb63419ac099b0c8f348c6177461fda4f8add0f490a3");
    let signer = point(
        "8e533b6fa0bf7b4625bb30667c01fb607ef9f8b8a80fef5b300628703187b2a3",
        "73eb1dbde03318366d069f83a6f5900053c73633cb041b21c55e1a86c1f400b4",
    );
    let other = point(
        "3cc7a016c2d9693fa9dae341244f610a86d720fed07a7a4ad94790cb5dc6820a",
        "8183f15a3d1893c8bb37150c8b7fdff1f319d0810213485add22ae3c0033dcbd",
    );
    let id = BoundedUint::new(0).unwrap();
    let flipped = BoundedUint::new(1).unwrap();
    assert_eq!(secp256r1_ecdsa_recover(digest, r, s, id).unwrap(), signer);
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, s, flipped).unwrap(),
        other
    );
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, high_s, flipped).unwrap(),
        signer
    );
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, high_s, id).unwrap(),
        other
    );
    let mut seven = [0; 32];
    seven[0] = 7;
    assert_eq!(
        Secp256r1Point::mul_generator(Secp256r1Scalar::from_le_bytes(seven).unwrap()),
        signer
    );
}

#[test]
fn rejects_zero_signature_scalars_and_impossible_wrapped_coordinate() {
    let digest = FixedBytes::new(bytes(
        "3bca196b80f06134b677809eec68178aa82c4abff8bb4fd456d04210b502226e",
    ));
    let r = scalar("0ec69bf32d8bd19b906e7345c0420be3beb650724cde76981283e3aaa68ea694");
    let s = scalar("0953fe03f4f50653c666349cbe653f660c1e0764e1002a22f614d2152b6e94ae");
    let zero = Secp256r1Scalar::default();
    let id = BoundedUint::new(0).unwrap();
    assert_eq!(
        secp256r1_ecdsa_recover(digest, zero, s, id),
        Err(CompactError::InvalidSecp256r1Signature)
    );
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, zero, id),
        Err(CompactError::InvalidSecp256r1Signature)
    );
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, s, BoundedUint::new(2).unwrap()),
        Err(CompactError::Secp256r1RecoveryFailed)
    );
    // This sum fits in 256 bits but x = r + n equals the base-field modulus.
    let r = scalar("000000000000000000000000000000004319055358e8617b0c46353d039cdaae");
    assert_eq!(
        secp256r1_ecdsa_recover(digest, r, s, BoundedUint::new(2).unwrap()),
        Err(CompactError::Secp256r1RecoveryFailed)
    );
}

#[test]
fn recovers_both_wrapped_nonce_parities() {
    let digest = FixedBytes::new(bytes(
        "0000000000000000000000000000000000000000000000000000000000000001",
    ));
    // x = n + 3 has a valid P-256 point for each parity.
    let r = scalar("0000000000000000000000000000000000000000000000000000000000000003");
    let s = scalar("0000000000000000000000000000000000000000000000000000000000000001");
    for (id, x, y) in [
        (
            2,
            "abefd53035325dcb20b6967a2c284a9bfc679b8446682523b32baf41120d76e4",
            "b4d5ee3eb39ab3dc059bc7cb8c8fcf1f842cee7d729602a7f15eaa653d81872d",
        ),
        (
            3,
            "ab835d9808d0b3e93199f38d0c1c9a5ab8c9bf62516ffbf37b037ea17f2fbd41",
            "e99c2933ec5b6de96409c9c5ebe6a33842583f16805b96074e9a977b125d29cd",
        ),
    ] {
        assert_eq!(
            secp256r1_ecdsa_recover(digest, r, s, BoundedUint::new(id).unwrap()).unwrap(),
            point(x, y)
        );
    }
}

#[test]
fn rejects_a_recovered_identity() {
    let mut digest = [0; 32];
    digest[31] = 1;
    let one = scalar("0000000000000000000000000000000000000000000000000000000000000001");
    let (x, y) = Secp256r1Point::mul_generator(one).coordinates().unwrap();
    let r = Secp256r1Scalar::from_le_bytes(x.to_le_bytes()).unwrap();
    let id = BoundedUint::new(u128::from(y.to_le_bytes()[0] & 1)).unwrap();
    assert_eq!(
        secp256r1_ecdsa_recover(FixedBytes::new(digest), r, one, id),
        Err(CompactError::Secp256r1RecoveryFailed)
    );
}

#[test]
fn typed_inputs_reject_out_of_range_recovery_id_and_scalar() {
    assert!(BoundedUint::<3>::new(4).is_err());
    assert!(
        Secp256r1Scalar::from_le_bytes(little_endian(
            "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551"
        ))
        .is_none()
    );
}
