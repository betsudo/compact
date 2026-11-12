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

//! Existing native value ownership policy, independent of Rust syntax.
//! Callers establish borrowed annotation bounds before this recursive predicate.

use crate::ir::Type;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Ownership {
    Copy,
    Owned,
}

pub(crate) fn copy_type(ty: &Type) -> bool {
    match ty {
        Type::Struct { .. }
        | Type::Vector { .. }
        | Type::LedgerMap { .. }
        | Type::OpaqueString
        | Type::OpaqueBytes => false,
        Type::Tuple { elements } => elements.iter().all(copy_type),
        Type::Unit
        | Type::Boolean
        | Type::Field
        | Type::JubjubPoint
        | Type::Secp256k1Base
        | Type::Secp256r1Base
        | Type::Secp256k1Scalar
        | Type::Secp256r1Scalar
        | Type::Secp256k1Point
        | Type::Bytes { .. }
        | Type::Enum { .. }
        | Type::Unsigned { .. } => true,
    }
}
