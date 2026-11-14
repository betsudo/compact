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

use ecdsa_verify::{
    runtime::{
        BoundedUint, CompactError, FixedBytes, Secp256k1Base, Secp256k1Point, Secp256k1Scalar,
        secp256k1_ecdsa_recover,
    },
    types::Secp256k1EcdsaSignature as EcdsaSignature,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Point {
    x: String,
    y: String,
    identity: bool,
}

impl Point {
    fn checked(&self) -> Secp256k1Point {
        let x = Secp256k1Base::from_le_bytes(little_endian(&self.x)).expect("canonical x");
        let y = Secp256k1Base::from_le_bytes(little_endian(&self.y)).expect("canonical y");
        if self.identity {
            assert_eq!(x, Secp256k1Base::default());
            assert_eq!(y, Secp256k1Base::default());
            Secp256k1Point::default()
        } else {
            Secp256k1Point::from_affine(x, y).expect("on-curve point")
        }
    }
}

fn bytes<const N: usize>(hex: &str) -> FixedBytes<N> {
    FixedBytes::new(
        hex::decode(hex)
            .expect("hex bytes")
            .try_into()
            .expect("fixed width"),
    )
}

fn little_endian(hex: &str) -> [u8; 32] {
    let mut value = bytes::<32>(hex).into_array();
    value.reverse();
    value
}

fn scalar(hex: &str) -> Option<Secp256k1Scalar> {
    if hex.len() > 64 {
        return None;
    }
    Secp256k1Scalar::from_le_bytes(little_endian(hex))
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Expectation {
    Valid,
    Invalid,
    Abort,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Vector {
    tc_id: u32,
    expectation: Expectation,
    scalars_in_range: bool,
    message: String,
    digest: String,
    r: String,
    s: String,
    point: Point,
}

impl Vector {
    fn signature(&self) -> Option<(Secp256k1Scalar, Secp256k1Scalar)> {
        let r = scalar(&self.r);
        let s = scalar(&self.s);
        let signature = r.zip(s);
        assert_eq!(
            signature.is_some(),
            self.scalars_in_range,
            "tcId {}",
            self.tc_id
        );
        if signature.is_none() {
            assert_eq!(
                self.expectation,
                Expectation::Invalid,
                "tcId {}",
                self.tc_id
            );
        }
        signature
    }

    fn check(&self, result: Result<bool, CompactError>, s: Secp256k1Scalar) {
        let expectation = match result {
            Ok(true) => Expectation::Valid,
            Ok(false) => Expectation::Invalid,
            Err(CompactError::Secp256k1ScalarZeroInverse) if s == Secp256k1Scalar::default() => {
                Expectation::Abort
            }
            Err(CompactError::Secp256k1IdentityX) if s != Secp256k1Scalar::default() => {
                Expectation::Abort
            }
            Err(error) => panic!("tcId {} unexpected refusal: {error}", self.tc_id),
        };
        assert_eq!(expectation, self.expectation, "tcId {}", self.tc_id);
    }
}

#[derive(Deserialize)]
struct Corpus {
    driven: Vec<Vector>,
}

fn corpus() -> Vec<Vector> {
    let corpus: Corpus =
        serde_json::from_str(include_str!("ecdsa-vectors.json")).expect("vector file");
    assert_eq!(corpus.driven.len(), 244);
    corpus.driven
}

#[test]
fn prehashed_wycheproof_and_recovery() {
    let mut valid = 0;
    let mut checked = 0;
    for vector in corpus() {
        let Some((r, s)) = vector.signature() else {
            continue;
        };
        let digest = bytes(&vector.digest);
        let signature = EcdsaSignature { r, s };
        let verify =
            |point| ecdsa_verify::pure_circuits::verifyEcdsa(digest, signature.clone(), point);
        vector.check(verify(vector.point.checked()), s);
        checked += 1;
        if vector.expectation == Expectation::Valid {
            assert!(
                (0..=3).any(|id| {
                    secp256k1_ecdsa_recover(digest, r, s, BoundedUint::new(id).unwrap())
                        .is_ok_and(|point| matches!(verify(point), Ok(true)))
                }),
                "recovery round trip for tcId {}",
                vector.tc_id
            );
            valid += 1;
        }
    }
    assert_eq!((checked, valid), (187, 162));
    println!(
        "prehashed: {checked} circuit cases, 57 scalar refusals, {valid} recovery round trips"
    );
}

#[test]
fn bitcoin_message_widths() {
    use bitcoin_kat::{pure_circuits as pure, types::Secp256k1EcdsaSignature};

    let mut widths = std::collections::BTreeSet::new();
    let mut checked = 0;
    for vector in corpus() {
        let Some((r, s)) = vector.signature() else {
            continue;
        };
        let signature = Secp256k1EcdsaSignature { r, s };
        let point = vector.point.checked();
        let message = &vector.message;
        let width = message.len() / 2;
        widths.insert(width);
        let result = match width {
            0 => pure::proveBitcoin0(bytes(message), signature, point),
            3 => pure::proveBitcoin3(bytes(message), signature, point),
            5 => pure::proveBitcoin5(bytes(message), signature, point),
            6 => pure::proveBitcoin6(bytes(message), signature, point),
            7 => pure::proveBitcoin7(bytes(message), signature, point),
            8 => pure::proveBitcoin8(bytes(message), signature, point),
            9 => pure::proveBitcoin9(bytes(message), signature, point),
            10 => pure::proveBitcoin10(bytes(message), signature, point),
            11 => pure::proveBitcoin11(bytes(message), signature, point),
            20 => pure::proveBitcoin20(bytes(message), signature, point),
            _ => panic!(
                "tcId {} missing circuit for {width}-byte input",
                vector.tc_id
            ),
        };
        vector.check(result, s);
        checked += 1;
    }
    assert_eq!(
        widths.into_iter().collect::<Vec<_>>(),
        [0, 3, 5, 6, 7, 8, 9, 10, 11, 20]
    );
    assert_eq!(checked, 187);
    println!("Bitcoin: {checked} circuit cases across 10 widths, 57 scalar refusals");
}

#[derive(Deserialize)]
struct AddressVector {
    seed: String,
    point: Point,
    address: String,
}

#[test]
fn published_ethereum_addresses() {
    let vectors: Vec<AddressVector> =
        serde_json::from_str(include_str!("address-vectors.json")).expect("address vectors");
    assert_eq!(vectors.len(), 2);
    for vector in vectors {
        let result =
            ethereum_address::pure_circuits::ethereumAddress(vector.point.checked()).unwrap();
        assert_eq!(result, bytes(&vector.address), "seed {}", vector.seed);
    }
    println!("Ethereum: 2 published address vectors");
}
