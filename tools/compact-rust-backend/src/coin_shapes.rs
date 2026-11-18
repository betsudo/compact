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

//! Canonical coin and recipient IR shapes shared by native and recorded emission.
//! These shapes define neither runtime codecs nor circuit admission policy.

use crate::ir::{StructField, Type};

pub(crate) fn qualified_coin_type() -> Type {
    let bytes = Type::Bytes { length: 32 };
    Type::Struct {
        name: "QualifiedShieldedCoinInfo".into(),
        fields: vec![
            StructField {
                name: "nonce".into(),
                ty: bytes.clone(),
            },
            StructField {
                name: "color".into(),
                ty: bytes,
            },
            StructField {
                name: "value".into(),
                ty: Type::Unsigned {
                    max: u128::MAX.to_string(),
                },
            },
            StructField {
                name: "mt_index".into(),
                ty: Type::Unsigned {
                    max: u64::MAX.to_string(),
                },
            },
        ],
    }
}

pub(crate) fn shielded_coin_type() -> Type {
    let Type::Struct { name, mut fields } = qualified_coin_type() else {
        unreachable!()
    };
    fields.pop();
    Type::Struct {
        name: name.replace("Qualified", ""),
        fields,
    }
}

pub(crate) fn shielded_recipient_type() -> Type {
    let bytes = Type::Bytes { length: 32 };
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: Type::Struct {
                    name: "ZswapCoinPublicKey".into(),
                    fields: vec![StructField {
                        name: "bytes".into(),
                        ty: bytes.clone(),
                    }],
                },
            },
            StructField {
                name: "right".into(),
                ty: Type::Struct {
                    name: "ContractAddress".into(),
                    fields: vec![StructField {
                        name: "bytes".into(),
                        ty: bytes,
                    }],
                },
            },
        ],
    }
}

pub(crate) fn token_type() -> Type {
    let bytes = Type::Bytes { length: 32 };
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: bytes.clone(),
            },
            StructField {
                name: "right".into(),
                ty: bytes,
            },
        ],
    }
}

pub(crate) fn unshielded_recipient_type() -> Type {
    let address = |name: &str| Type::Struct {
        name: name.into(),
        fields: vec![StructField {
            name: "bytes".into(),
            ty: Type::Bytes { length: 32 },
        }],
    };
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: address("ContractAddress"),
            },
            StructField {
                name: "right".into(),
                ty: address("UserAddress"),
            },
        ],
    }
}

/// Compact monomorphizes repeated `Either<A, B>` instantiations as
/// `EitherCompact1`, `EitherCompact2`, and so on. The Kernel operand contract
/// is the field structure of that instantiation, not the assigned suffix.
pub(crate) fn same_either_instantiation(actual: &Type, expected: &Type) -> bool {
    let (
        Type::Struct { name, fields },
        Type::Struct {
            fields: expected_fields,
            ..
        },
    ) = (actual, expected)
    else {
        return false;
    };
    let generated = name.strip_prefix("EitherCompact").is_some_and(|suffix| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    });
    (name == "Either" || generated) && fields == expected_fields
}
