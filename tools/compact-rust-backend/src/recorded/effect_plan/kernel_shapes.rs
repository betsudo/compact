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

//! Exact Compact standard-library shapes for Ledger 9 unshielded kernel operands.
use crate::ir::{StructField, Type};

fn bytes32() -> Type {
    Type::Bytes { length: 32 }
}

fn address(name: &str) -> Type {
    Type::Struct {
        name: name.into(),
        fields: vec![StructField {
            name: "bytes".into(),
            ty: bytes32(),
        }],
    }
}

pub(super) fn token_type() -> Type {
    Type::Struct {
        name: "Either".into(),
        fields: vec![
            StructField {
                name: "is_left".into(),
                ty: Type::Boolean,
            },
            StructField {
                name: "left".into(),
                ty: bytes32(),
            },
            StructField {
                name: "right".into(),
                ty: bytes32(),
            },
        ],
    }
}

pub(super) fn recipient_type() -> Type {
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

fn generated_name(actual: &str, base: &str) -> bool {
    actual == base
        || actual
            .strip_prefix(&format!("{base}Compact"))
            .is_some_and(|suffix| suffix.parse::<u32>().is_ok_and(|n| n > 0))
}

fn bytes_address(ty: &Type, base: &str) -> bool {
    matches!(ty, Type::Struct { name, fields }
        if generated_name(name, base)
            && fields.len() == 1
            && fields[0].name == "bytes"
            && fields[0].ty == bytes32())
}

pub(super) fn is_token_type(ty: &Type) -> bool {
    matches!(ty, Type::Struct { name, fields }
        if generated_name(name, "Either")
            && fields.len() == 3
            && fields[0].name == "is_left"
            && fields[0].ty == Type::Boolean
            && fields[1].name == "left"
            && fields[1].ty == bytes32()
            && fields[2].name == "right"
            && fields[2].ty == bytes32())
}

pub(super) fn is_recipient_type(ty: &Type) -> bool {
    matches!(ty, Type::Struct { name, fields }
        if generated_name(name, "Either")
            && fields.len() == 3
            && fields[0].name == "is_left"
            && fields[0].ty == Type::Boolean
            && fields[1].name == "left"
            && bytes_address(&fields[1].ty, "ContractAddress")
            && fields[2].name == "right"
            && bytes_address(&fields[2].ty, "UserAddress"))
}
