// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_contract_return_lambda_default::{pure_circuits, runtime};
#[test]
fn original_lambda_scope() {
    let input = runtime::BoundedUint::<255>::new(5u128).unwrap();
    assert_eq!(
        pure_circuits::return_lambda_default(input).unwrap(),
        runtime::Field::from(0u128)
    );
}
