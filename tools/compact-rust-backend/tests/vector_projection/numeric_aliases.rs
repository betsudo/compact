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

//! External consumers of unedited, freshly generated source APIs.

use numeric_fixture::{pure_circuits, runtime};
use runtime::{BoundedUint, CompactError};

fn ts() -> Option<serde_json::Value> {
    std::env::var("COMPACT_VECTOR_NUMERIC_TS_REFERENCE")
        .ok()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
}

#[test]
fn nested_alias_zero_preserves_nonzero_operands() {
    let reference = ts();
    for (index, value) in [0, 1, 5, 255].into_iter().enumerate() {
        let actual = pure_circuits::zero(BoundedUint::<255>::new(value).unwrap()).unwrap();
        assert_eq!(actual.value(), 0);
        if let Some(reference) = &reference {
            assert_eq!(reference["zero"][index], actual.value().to_string());
        }
    }
}

#[test]
fn checked_source_cast_is_not_an_arithmetic_annotation() {
    assert_eq!(
        pure_circuits::checked(BoundedUint::<255>::new(0).unwrap())
            .unwrap()
            .value(),
        0
    );
    let reference = ts();
    for value in [1, 5, 255] {
        assert!(matches!(
            pure_circuits::checked(BoundedUint::<255>::new(value).unwrap()),
            Err(CompactError::UnsignedOutOfRange { value: actual, max: 0 }) if actual == value
        ));
        if let Some(reference) = &reference {
            assert!(reference["checked"][value.to_string()]["error"].is_string());
        }
    }
}

#[test]
fn checked_cast_failure_precedes_the_later_assertion() {
    assert!(matches!(
        pure_circuits::ordered(BoundedUint::<255>::new(1).unwrap()),
        Err(CompactError::UnsignedOutOfRange { value: 1, max: 0 })
    ));
    let later = pure_circuits::ordered(BoundedUint::<255>::new(0).unwrap()).unwrap_err();
    assert!(format!("{later:?}").contains("second failure"));
    if let Some(reference) = ts() {
        assert!(
            !reference["ordered"]["1"]["error"]
                .as_str()
                .unwrap()
                .contains("second failure")
        );
        assert!(
            reference["ordered"]["0"]["error"]
                .as_str()
                .unwrap()
                .contains("second failure")
        );
    }
}
