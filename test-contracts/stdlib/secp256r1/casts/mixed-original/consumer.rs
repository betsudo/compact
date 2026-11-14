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

use constant::{ledger_contract as c, runtime as r};
use r::{
    BoundedUint, Curve25519Base, Curve25519Scalar, Field, FixedBytes, Secp256k1Base,
    Secp256k1Scalar, Secp256r1Base, Secp256r1Scalar, context::ConstructorContext,
    ledger::ContractAddress,
};
use unused::ledger_contract as u;

fn constant_context() -> r::context::CircuitContext<()> {
    c::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}

fn unused_context() -> r::context::CircuitContext<()> {
    u::initial_state(ConstructorContext::new(()))
        .unwrap()
        .into_circuit_context(ContractAddress::default())
}

fn value_bytes() -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[0] = 5;
    bytes[31] = 1;
    bytes
}

#[test]
fn original_mixed_constant_casts() {
    let v = value_bytes();
    let k1b = |x| Secp256k1Base::from_le_bytes(x).unwrap();
    let k1s = |x| Secp256k1Scalar::from_le_bytes(x).unwrap();
    let r1b = |x| Secp256r1Base::from_le_bytes(x).unwrap();
    let r1s = |x| Secp256r1Scalar::from_le_bytes(x).unwrap();
    let c25519b = |x| Curve25519Base::from_le_bytes(x).unwrap();
    let c25519s = |x| Curve25519Scalar::from_le_bytes(x).unwrap();
    let small = |x| {
        let mut bytes = [0; 32];
        bytes[0] = x;
        bytes
    };

    let result = c::castK1(constant_context(), false, k1b(small(7)), k1s(small(8))).unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(
        [
            view.k1b().unwrap().to_le_bytes(),
            view.k1s().unwrap().to_le_bytes()
        ],
        [v, v]
    );
    let result = c::castK1(constant_context(), true, k1b(small(7)), k1s(small(8))).unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.k1b().unwrap(), k1b(small(7)));
    assert_eq!(view.k1s().unwrap(), k1s(small(8)));

    let result = c::castR1(constant_context(), false, r1b(small(7)), r1s(small(8))).unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.r1b().unwrap(), r1b(v));
    assert_eq!(view.r1s().unwrap(), r1s(v));
    let result = c::castR1(constant_context(), true, r1b(small(7)), r1s(small(8))).unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.r1b().unwrap(), r1b(small(7)));
    assert_eq!(view.r1s().unwrap(), r1s(small(8)));

    let result = c::castC25519(
        constant_context(),
        false,
        c25519b(small(7)),
        c25519s(small(8)),
    )
    .unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.c25519b().unwrap(), c25519b(v));
    assert_eq!(view.c25519s().unwrap(), c25519s(v));
    let result = c::castC25519(
        constant_context(),
        true,
        c25519b(small(7)),
        c25519s(small(8)),
    )
    .unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.c25519b().unwrap(), c25519b(small(7)));
    assert_eq!(view.c25519s().unwrap(), c25519s(small(8)));

    let result = c::castZero(constant_context()).unwrap();
    let view = c::PublicStateView::from(&result);
    assert_eq!(view.k1b().unwrap(), k1b([0; 32]));
    assert_eq!(view.k1s().unwrap(), k1s([0; 32]));
    assert_eq!(view.r1b().unwrap(), r1b([0; 32]));
    assert_eq!(view.r1s().unwrap(), r1s([0; 32]));
    assert_eq!(view.c25519b().unwrap(), c25519b([0; 32]));
    assert_eq!(view.c25519s().unwrap(), c25519s([0; 32]));

    let result = c::castWide(constant_context()).unwrap();
    assert_eq!(
        c::PublicStateView::from(&result).c25519s().unwrap(),
        c25519s(small(5))
    );
    let result = c::castNative(
        constant_context(),
        FixedBytes::new([0; 32]),
        BoundedUint::<255>::new(1).unwrap(),
        BoundedUint::<255>::new(2).unwrap(),
    )
    .unwrap();
    assert_eq!(
        c::PublicStateView::from(&result).f().unwrap(),
        Field::from_le_bytes(&v).unwrap()
    );
}

#[test]
fn original_mixed_unused_casts() {
    let bytes = FixedBytes::<32>::new([7; 32]);
    let wide = FixedBytes::<64>::new([9; 64]);
    let result = u::unusedK1(unused_context(), bytes).unwrap();
    assert_eq!(
        u::PublicStateView::from(&result).n().unwrap(),
        Field::from(1u128)
    );
    let result = u::unusedR1(unused_context(), bytes).unwrap();
    assert_eq!(
        u::PublicStateView::from(&result).n().unwrap(),
        Field::from(2u128)
    );
    let result = u::unusedC25519(unused_context(), bytes, wide).unwrap();
    assert_eq!(
        u::PublicStateView::from(&result).n().unwrap(),
        Field::from(3u128)
    );
    let result = u::unusedFieldK1(unused_context(), bytes, Field::from(4u128)).unwrap();
    assert_eq!(
        u::PublicStateView::from(&result).n().unwrap(),
        Field::from(4u128)
    );
    let result = u::unusedFieldR1(unused_context(), bytes, Field::from(5u128)).unwrap();
    assert_eq!(
        u::PublicStateView::from(&result).n().unwrap(),
        Field::from(5u128)
    );
}
