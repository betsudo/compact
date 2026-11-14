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

//! Rules over already checked children; no traversal or syntax dependency.
use super::{Binary, CheckedCrypto, Unary};
use crate::{
    RenderError,
    ir::Type,
    native_representation::{Operation, check_tuple_traits},
    pure_semantics::{
        hash_encoding::DynamicInputEncoding,
        jubjub::{Axis, CurveOperands, check_curve},
        sink::Typed,
    },
};

fn curve<V>(input: CurveOperands<V>) -> Result<Typed<CheckedCrypto<V>>, RenderError> {
    let checked = check_curve(input)?;
    Ok(Typed {
        output: CheckedCrypto::Curve(checked.output),
        ty: checked.ty,
    })
}
fn expect(actual: Type, expected: Type) -> Result<(), RenderError> {
    if actual != expected {
        return Err(RenderError::TypeMismatch { expected, actual });
    }
    Ok(())
}

pub(super) fn unary<V>(
    kind: Unary,
    value: Typed<V>,
) -> Result<Typed<CheckedCrypto<V>>, RenderError> {
    let (output, ty) = match kind {
        Unary::TransientHash => {
            check_tuple_traits(Operation::TransientHash, &value.ty)?;
            let encoding = DynamicInputEncoding::for_type(&value.ty);
            (
                CheckedCrypto::TransientHash {
                    value: value.output,
                    encoding,
                },
                Type::Field,
            )
        }
        Unary::PersistentHash => {
            check_tuple_traits(Operation::PersistentHash, &value.ty)?;
            (
                CheckedCrypto::PersistentHash(value.output),
                Type::Bytes { length: 32 },
            )
        }
        Unary::Keccak256(encoding) => {
            check_tuple_traits(Operation::Keccak256(encoding), &value.ty)?;
            (
                CheckedCrypto::Keccak256(value.output, encoding),
                Type::Bytes { length: 32 },
            )
        }
        Unary::Sha512 => {
            check_tuple_traits(Operation::Sha512, &value.ty)?;
            (
                CheckedCrypto::Sha512(value.output),
                Type::Bytes { length: 64 },
            )
        }
        Unary::Degrade => {
            expect(value.ty, Type::Bytes { length: 32 })?;
            (CheckedCrypto::DegradeToTransient(value.output), Type::Field)
        }
        Unary::Upgrade => {
            expect(value.ty, Type::Field)?;
            (
                CheckedCrypto::UpgradeFromTransient(value.output),
                Type::Bytes { length: 32 },
            )
        }
        Unary::HashToCurve => {
            check_tuple_traits(Operation::HashToCurve, &value.ty)?;
            let encoding = DynamicInputEncoding::for_type(&value.ty);
            (
                CheckedCrypto::HashToCurve {
                    value: value.output,
                    encoding,
                },
                Type::JubjubPoint,
            )
        }
        Unary::PointX => return curve(CurveOperands::Coordinate(Axis::X, value)),
        Unary::PointY => return curve(CurveOperands::Coordinate(Axis::Y, value)),
        Unary::Negate => return curve(CurveOperands::Negate(value)),
        Unary::Secp256k1(kind) => {
            let ty = kind.result_type(&value.ty)?;
            (CheckedCrypto::Secp256k1Unary(kind, value.output), ty)
        }
        Unary::Generator => return curve(CurveOperands::Generator(value)),
        Unary::Reduce => return curve(CurveOperands::Reduce(value)),
    };
    Ok(Typed { output, ty })
}

pub(super) fn binary<V>(
    kind: Binary,
    left: Typed<V>,
    right: Typed<V>,
) -> Result<Typed<CheckedCrypto<V>>, RenderError> {
    let (output, ty) = match kind {
        Binary::TransientCommit => {
            expect(right.ty, Type::Field)?;
            check_tuple_traits(Operation::TransientCommit, &left.ty)?;
            let encoding = DynamicInputEncoding::for_type(&left.ty);
            (
                CheckedCrypto::TransientCommit {
                    value: left.output,
                    opening: right.output,
                    encoding,
                },
                Type::Field,
            )
        }
        Binary::PersistentCommit => {
            expect(right.ty, Type::Bytes { length: 32 })?;
            check_tuple_traits(Operation::PersistentCommit, &left.ty)?;
            (
                CheckedCrypto::PersistentCommit {
                    value: left.output,
                    opening: right.output,
                },
                Type::Bytes { length: 32 },
            )
        }
        Binary::Construct => return curve(CurveOperands::Construct(left, right)),
        Binary::Add => return curve(CurveOperands::Add(left, right)),
        Binary::Multiply => return curve(CurveOperands::Multiply(left, right)),
    };
    Ok(Typed { output, ty })
}
