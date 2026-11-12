// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use decode::runtime::{FixedBytes, Secp256k1Point, Secp256k1Scalar, Secp256r1Base};
use p256_wycheproof::{pure_circuits::verifyEcdsa, types::Secp256r1EcdsaSignature};

fn main() {
    let _ = Secp256r1EcdsaSignature {
        r: Secp256k1Scalar::default(),
        s: Secp256r1Base::default(),
    };
    let _ = verifyEcdsa(
        FixedBytes::default(),
        Secp256r1EcdsaSignature::default(),
        Secp256k1Point::default(),
    );
}
