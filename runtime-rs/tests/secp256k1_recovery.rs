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
    BoundedUint, CompactError, FixedBytes, Secp256k1Base, Secp256k1Point, Secp256k1Scalar,
    secp256k1_ecdsa_recover,
};

fn bytes(s: &str) -> [u8; 32] {
    hex::decode(s).unwrap().try_into().unwrap()
}

fn little_endian(s: &str) -> [u8; 32] {
    let mut value = bytes(s);
    value.reverse();
    value
}

fn scalar(s: &str) -> Secp256k1Scalar {
    Secp256k1Scalar::from_le_bytes(little_endian(s)).unwrap()
}

fn point(x: &str, y: &str) -> Secp256k1Point {
    Secp256k1Point::from_affine(
        Secp256k1Base::from_le_bytes(little_endian(x)).unwrap(),
        Secp256k1Base::from_le_bytes(little_endian(y)).unwrap(),
    )
    .unwrap()
}

#[test]
fn pinned_recovery_vector_distinguishes_parity_and_accepts_high_s() {
    let digest = FixedBytes::new(bytes(
        "3bca196b80f06134b677809eec68178aa82c4abff8bb4fd456d04210b502226e",
    ));
    let r = scalar("b3b43e8fbedeca0e6f3ab04983e877669b6cfe0ef345a282ca1a964c9de07328");
    let s = scalar("46fc016cd1258862f7d182bf3b451e0b806b38a7777a74ee825559c887bd7239");
    let high_s = scalar("b903fe932eda779d082e7d40c4bae1f33a43a43f37ce2b4d3d7d04c44878cf08");
    let signer = point(
        "5cbdf0646e5db4eaa398f365f2ea7a0e3d419b7e0330e39ce92bddedcac4f9bc",
        "6aebca40ba255960a3178d6d861a54dba813d0b813fde7b5a5082628087264da",
    );
    let other = point(
        "228f64b5e0b9737e91d2114a0b98cabd2be53c53bd5745643ce736e15ef2f2fe",
        "b23658d1a8accd23e14a5f92940b65544e813f61ac5721f9c68504ed9d6bf6a1",
    );
    let id = BoundedUint::new(1).unwrap();
    let flipped = BoundedUint::new(1 ^ 1).unwrap();
    assert_eq!(secp256k1_ecdsa_recover(digest, r, s, id).unwrap(), signer);
    assert_eq!(
        secp256k1_ecdsa_recover(digest, r, s, flipped).unwrap(),
        other
    );
    assert_eq!(
        secp256k1_ecdsa_recover(digest, r, high_s, flipped).unwrap(),
        signer
    );
    assert_eq!(
        secp256k1_ecdsa_recover(digest, r, high_s, id).unwrap(),
        other
    );
    let mut seven = [0; 32];
    seven[0] = 7;
    assert_eq!(
        Secp256k1Point::mul_generator(Secp256k1Scalar::from_le_bytes(seven).unwrap()),
        signer
    );
}

#[test]
fn rejects_zero_signature_scalars_and_impossible_wrapped_coordinate() {
    let digest = FixedBytes::new(bytes(
        "3bca196b80f06134b677809eec68178aa82c4abff8bb4fd456d04210b502226e",
    ));
    let r = scalar("b3b43e8fbedeca0e6f3ab04983e877669b6cfe0ef345a282ca1a964c9de07328");
    let s = scalar("46fc016cd1258862f7d182bf3b451e0b806b38a7777a74ee825559c887bd7239");
    let zero = Secp256k1Scalar::default();
    let id = BoundedUint::new(1).unwrap();
    assert_eq!(
        secp256k1_ecdsa_recover(digest, zero, s, id),
        Err(CompactError::InvalidSecp256k1Signature)
    );
    assert_eq!(
        secp256k1_ecdsa_recover(digest, r, zero, id),
        Err(CompactError::InvalidSecp256k1Signature)
    );
    assert_eq!(
        secp256k1_ecdsa_recover(digest, r, s, BoundedUint::new(2).unwrap()),
        Err(CompactError::Secp256k1RecoveryFailed)
    );
}

#[test]
fn recovers_both_wrapped_nonce_parities() {
    let digest = FixedBytes::new(bytes(
        "0000000000000000000000000000000000000000000000000000000000000001",
    ));
    let mut two = [0; 32];
    two[0] = 2;
    let r = Secp256k1Scalar::from_le_bytes(two).unwrap();
    let s = scalar("0000000000000000000000000000000000000000000000000000000000000001");
    for (id, x, y) in [
        (
            2,
            "8a0dee06163f357e0e71688f3785eb53891144624c633ed43706b6268bb4316f",
            "e440d41220c85c709a6ceb8bb5faf8963408733e2451994f9bab1da5e0bfbff0",
        ),
        (
            3,
            "19099fbdddcdb7a14410b5c6ba73f51bd2f74a3d1cd8456fcc1da68ed6468884",
            "b07b9f7dd9ea5753e009246a1db85e7e1644bfdee276881a950ce59fdd45b45b",
        ),
    ] {
        assert_eq!(
            secp256k1_ecdsa_recover(digest, r, s, BoundedUint::new(id).unwrap()).unwrap(),
            point(x, y)
        );
    }
}
