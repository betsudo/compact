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

//! Crypto operations with arity-safe checked output.
//! Generic child traversal stays outside support policy and Rust formatting.
use super::{
    arithmetic::Secp256k1Unary,
    children::Children,
    hash_encoding::DynamicInputEncoding,
    jubjub::CheckedCurve,
    sink::{CryptoSink, Typed},
};
use crate::{
    RenderError,
    ir::{Expr, KeccakEncoding},
};
mod check;

pub(crate) enum CryptoSource<'a> {
    TransientHash(&'a Expr),
    TransientCommit(&'a Expr, &'a Expr),
    PersistentHash(&'a Expr),
    PersistentCommit(&'a Expr, &'a Expr),
    Keccak256(&'a Expr, KeccakEncoding),
    Sha512(&'a Expr),
    DegradeToTransient(&'a Expr),
    UpgradeFromTransient(&'a Expr),
    HashToCurve(&'a Expr),
    JubjubPointX(&'a Expr),
    JubjubPointY(&'a Expr),
    ConstructJubjubPoint(&'a Expr, &'a Expr),
    EcAdd(&'a Expr, &'a Expr),
    EcNeg(&'a Expr),
    Secp256k1Unary(Secp256k1Unary, &'a Expr),
    EcMul(&'a Expr, &'a Expr),
    EcMulGenerator(&'a Expr),
    JubjubScalarFromNative(&'a Expr),
}

pub(crate) enum CheckedCrypto<V> {
    TransientHash {
        value: V,
        encoding: DynamicInputEncoding,
    },
    TransientCommit {
        value: V,
        opening: V,
        encoding: DynamicInputEncoding,
    },
    PersistentHash(V),
    PersistentCommit {
        value: V,
        opening: V,
    },
    Keccak256(V, KeccakEncoding),
    Sha512(V),
    DegradeToTransient(V),
    UpgradeFromTransient(V),
    HashToCurve {
        value: V,
        encoding: DynamicInputEncoding,
    },
    Curve(CheckedCurve<V>),
    Secp256k1Unary(Secp256k1Unary, V),
}

pub(super) enum Unary {
    TransientHash,
    PersistentHash,
    Keccak256(KeccakEncoding),
    Sha512,
    Degrade,
    Upgrade,
    HashToCurve,
    PointX,
    PointY,
    Negate,
    Secp256k1(Secp256k1Unary),
    Generator,
    Reduce,
}
pub(super) enum Binary {
    TransientCommit,
    PersistentCommit,
    Construct,
    Add,
    Multiply,
}
enum Inputs<'a> {
    Unary(Unary, &'a Expr),
    Binary(Binary, &'a Expr, &'a Expr),
}
impl<'a> CryptoSource<'a> {
    fn inputs(self) -> Inputs<'a> {
        match self {
            Self::TransientHash(value) => Inputs::Unary(Unary::TransientHash, value),
            Self::TransientCommit(value, opening) => {
                Inputs::Binary(Binary::TransientCommit, value, opening)
            }
            Self::PersistentHash(value) => Inputs::Unary(Unary::PersistentHash, value),
            Self::PersistentCommit(value, opening) => {
                Inputs::Binary(Binary::PersistentCommit, value, opening)
            }
            Self::Keccak256(value, encoding) => Inputs::Unary(Unary::Keccak256(encoding), value),
            Self::Sha512(value) => Inputs::Unary(Unary::Sha512, value),
            Self::DegradeToTransient(value) => Inputs::Unary(Unary::Degrade, value),
            Self::UpgradeFromTransient(value) => Inputs::Unary(Unary::Upgrade, value),
            Self::HashToCurve(value) => Inputs::Unary(Unary::HashToCurve, value),
            Self::JubjubPointX(value) => Inputs::Unary(Unary::PointX, value),
            Self::JubjubPointY(value) => Inputs::Unary(Unary::PointY, value),
            Self::ConstructJubjubPoint(x, y) => Inputs::Binary(Binary::Construct, x, y),
            Self::EcAdd(left, right) => Inputs::Binary(Binary::Add, left, right),
            Self::EcNeg(value) => Inputs::Unary(Unary::Negate, value),
            Self::Secp256k1Unary(kind, value) => Inputs::Unary(Unary::Secp256k1(kind), value),
            Self::EcMul(point, scalar) => Inputs::Binary(Binary::Multiply, point, scalar),
            Self::EcMulGenerator(value) => Inputs::Unary(Unary::Generator, value),
            Self::JubjubScalarFromNative(value) => Inputs::Unary(Unary::Reduce, value),
        }
    }
}

pub(crate) fn fold_crypto<S: CryptoSink, C: Children<S>>(
    source: CryptoSource<'_>,
    children: &mut C,
    sink: &mut S,
) -> Result<Typed<S::Value>, RenderError> {
    // Only two traversal shapes share child-result slots. Semantic helpers run
    // after visits finish, so paired errors keep the original source order.
    let checked = match source.inputs() {
        Inputs::Unary(kind, source) => {
            let value = children.visit(source, sink)?;
            check::unary(kind, value)?
        }
        Inputs::Binary(kind, left, right) => {
            let left = children.visit(left, sink)?;
            let right = children.visit(right, sink)?;
            check::binary(kind, left, right)?
        }
    };
    Ok(Typed {
        output: sink.crypto(checked.output),
        ty: checked.ty,
    })
}

#[cfg(test)]
mod tests;
