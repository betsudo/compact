// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

use compact_contract_return_if_else_circuit_call::{pure_circuits, runtime};
#[test]
fn original_branches() {
    for value in [2u128, 0] {
        let input = runtime::BoundedUint::<255>::new(value).unwrap();
        assert_eq!(
            pure_circuits::return_if_else_circuit_call(input)
                .unwrap()
                .value(),
            1
        );
    }
}
