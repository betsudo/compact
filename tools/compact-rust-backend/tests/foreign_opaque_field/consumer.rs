// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use foreign::runtime::{Curve25519Scalar, Secp256k1Scalar, Secp256r1Base};
use foreign::runtime::context::ConstructorContext;
use foreign::runtime::ledger::ContractAddress;
use foreign::runtime::public_state::PublicStateSource;

fn bytes(value: u64) -> [u8; 32] {
    let mut result = [0; 32];
    result[..8].copy_from_slice(&value.to_le_bytes());
    result
}

#[test]
fn original_opaque_result_is_bytes() {
    let result: opaque::runtime::OpaqueBytes = opaque::pure_circuits::opaque_casts_to_opaque().unwrap();
    assert_eq!(result.0, Vec::<u8>::new());
}

#[test]
fn five_original_foreign_zero_results_ordinary_and_recorded() {
    let k = Secp256k1Scalar::from_le_bytes(bytes(12345)).unwrap();
    let z = Secp256k1Scalar::from_le_bytes(bytes(678)).unwrap();
    let r = Secp256r1Base::from_le_bytes(bytes(12345)).unwrap();
    let c = Curve25519Scalar::from_le_bytes(bytes(12345)).unwrap();
    let zero_k = Secp256k1Scalar::default();
    let zero_r = Secp256r1Base::default();
    let zero_c = Curve25519Scalar::default();
    let context = || foreign::ledger_contract::initial_state(ConstructorContext::new(()))
        .unwrap().into_circuit_context(ContractAddress::default());

    let ordinary = foreign::ledger_contract::self_subtract(context(), k).unwrap();
    let recorded = foreign::ledger_contract::recorded::self_subtract(context(), k).unwrap();
    assert_eq!(ordinary.result, zero_k);
    assert_eq!(recorded.execution.result, zero_k);
    assert_eq!(ordinary.public_state(), recorded.execution.public_state());

    let ordinary = foreign::ledger_contract::inlined_self_subtract(context(), k).unwrap();
    let recorded = foreign::ledger_contract::recorded::inlined_self_subtract(context(), k).unwrap();
    assert_eq!(ordinary.result, zero_k);
    assert_eq!(recorded.execution.result, zero_k);
    assert_eq!(ordinary.public_state(), recorded.execution.public_state());

    let ordinary = foreign::ledger_contract::multiply_self_subtract(context(), k, z).unwrap();
    let recorded = foreign::ledger_contract::recorded::multiply_self_subtract(context(), k, z).unwrap();
    assert_eq!(ordinary.result, zero_k);
    assert_eq!(recorded.execution.result, zero_k);
    assert_eq!(ordinary.public_state(), recorded.execution.public_state());

    let ordinary = foreign::ledger_contract::secp256r1_self_subtract(context(), r).unwrap();
    let recorded = foreign::ledger_contract::recorded::secp256r1_self_subtract(context(), r).unwrap();
    assert_eq!(ordinary.result, zero_r);
    assert_eq!(recorded.execution.result, zero_r);
    assert_eq!(ordinary.public_state(), recorded.execution.public_state());

    let ordinary = foreign::ledger_contract::curve25519_self_subtract(context(), c).unwrap();
    let recorded = foreign::ledger_contract::recorded::curve25519_self_subtract(context(), c).unwrap();
    assert_eq!(ordinary.result, zero_c);
    assert_eq!(recorded.execution.result, zero_c);
    assert_eq!(ordinary.public_state(), recorded.execution.public_state());
}
