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

//! Native Compact runtime surface for the typed Rust backend.
//!
//! Generated code uses this facade rather than naming the underlying
//! ledger or proof crates. Each facade operation must preserve Compact's
//! semantics and have a direct test against the corresponding ledger API.

#![forbid(unsafe_code)]

pub mod context;
mod foreign_fields;
#[cfg(feature = "experimental-functional")]
pub mod functional;
mod hash_input;
pub mod ledger;
mod natives;
mod opaque;
mod primitives;
pub mod public_state;
pub mod recording;
mod secp256k1_point;
mod secp256k1_recovery;
pub mod slots;
mod zswap;
pub use zswap::{CircuitZswapOutput, CircuitZswapPlan};
#[cfg(feature = "ledger-transaction")]
pub mod transaction;

pub use foreign_fields::{Secp256k1Base, Secp256k1Scalar, Secp256r1Base, Secp256r1Scalar};
pub use hash_input::{ExactHashInput, HashInputValue, exact_hash_input, exact_private_output};
pub use midnight_compact_runtime_macros::{
    CompactCellValue, CompactEnum, CompactHashInput, CompactMerklePath, CompactMerklePathEntry,
    CompactMerkleTreeDigest, CompactStructRepr, compact_witness_bridge,
};
pub use natives::{
    construct_jubjub_point, degrade_to_transient, ec_add, ec_mul, ec_mul_generator, ec_neg,
    hash_to_curve, jubjub_point_x, jubjub_point_y, jubjub_scalar_from_native, keccak256,
    keccak256_aligned, persistent_commit, persistent_hash, sha512, transient_commit,
    transient_hash, upgrade_from_transient,
};
pub use opaque::{OpaqueBytes, OpaqueString};
pub use primitives::{
    BoundedUint, FixedBytes, FixedVector, JubjubPoint, UnsignedOperand, WideUint, add_unsigned,
    add_wide_unsigned, cast_unsigned, multiply_unsigned, narrow_wide_uint, subtract_unsigned,
};
pub use secp256k1_point::Secp256k1Point;
pub use secp256k1_recovery::secp256k1_ecdsa_recover;

/// Ledger FAB types used by the generated user-type derive.
pub mod fab {
    pub use midnight_base_crypto::fab::{
        Aligned, AlignedValue, Alignment, AlignmentAtom, Value, ValueSlice,
    };
}

/// Increment when generated Rust and the runtime's public contract change.
pub const RUST_RUNTIME_ABI: u32 = 54;
/// Ledger version used by the native Rust runtime dependencies.
pub const LEDGER_VERSION: &str = "ledger-9.1.0.1";

/// Compact `Field` is the scalar field used by the Midnight circuit runtime.
pub use midnight_transient_crypto::curve::Fr;
pub use midnight_transient_crypto::curve::Fr as Field;

pub use midnight_base_crypto::repr::BinaryHashRepr;
/// Required by the ledger derives in their generated implementation.
pub use midnight_base_crypto::repr::MemWrite;
/// Reuse the ledger's field representation traits and derives for user structs.
pub use midnight_transient_crypto::repr::{FieldRepr, FromFieldRepr};

/// Errors raised while evaluating a Compact circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactError {
    AssertionFailed(String),
    InvalidLedgerCell(String),
    /// An experimental generated adapter has no callback for this checked identity.
    #[cfg(feature = "experimental-functional")]
    UnknownWitnessIdentity {
        id: u32,
    },
    LedgerQueryRejected(String),
    InvalidUnsignedValue,
    InvalidJubjubScalar,
    InvalidJubjubPoint,
    Secp256k1BaseZeroInverse,
    Secp256k1ScalarZeroInverse,
    Secp256k1IdentityX,
    Secp256k1IdentityY,
    InvalidSecp256k1Signature,
    Secp256k1RecoveryFailed,
    InvalidHashInput,
    MissingCoinPublicKey,
    ZswapAllocationLocked,
    ZswapOfferOutputMismatch,
    ZswapCursorAlreadyUsed,
    ZswapCursorOverflow,
    UnsignedOutOfRange {
        value: u128,
        max: u128,
    },
    UnsignedOverflow,
    UnsignedUnderflow,
}

impl std::fmt::Display for CompactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssertionFailed(message) => write!(f, "failed assert: {message}"),
            Self::InvalidLedgerCell(message) => write!(f, "invalid Compact ledger cell: {message}"),
            #[cfg(feature = "experimental-functional")]
            Self::UnknownWitnessIdentity { id } => {
                write!(f, "unknown checked witness identity {id}")
            }
            Self::LedgerQueryRejected(message) => write!(f, "ledger query rejected: {message}"),
            Self::InvalidUnsignedValue => write!(f, "invalid Compact unsigned value"),
            Self::InvalidJubjubScalar => write!(f, "invalid Jubjub scalar"),
            Self::InvalidJubjubPoint => write!(f, "invalid Jubjub point"),
            Self::Secp256k1BaseZeroInverse => {
                write!(f, "secp256k1 base field has no inverse for 0")
            }
            Self::Secp256k1ScalarZeroInverse => {
                write!(f, "secp256k1 scalar field has no inverse for 0")
            }
            Self::Secp256k1IdentityX => {
                write!(
                    f,
                    "cannot extract the x-coordinate of the secp256k1 identity point"
                )
            }
            Self::Secp256k1IdentityY => {
                write!(
                    f,
                    "cannot extract the y-coordinate of the secp256k1 identity point"
                )
            }
            Self::InvalidSecp256k1Signature => {
                write!(f, "expected nonzero secp256k1 signature scalars")
            }
            Self::Secp256k1RecoveryFailed => write!(
                f,
                "no finite secp256k1 key for this signature and recovery id"
            ),
            Self::InvalidHashInput => write!(f, "invalid Compact hash input"),
            Self::ZswapOfferOutputMismatch => write!(
                f,
                "Zswap output intent differs from authoritative offer allocation"
            ),
            Self::ZswapAllocationLocked => write!(
                f,
                "provisional Zswap allocation is unavailable for an observed or offer-backed context"
            ),
            Self::ZswapCursorAlreadyUsed => write!(
                f,
                "cannot reset the provisional Zswap cursor after producing an output"
            ),
            Self::ZswapCursorOverflow => {
                write!(f, "provisional Zswap cursor exceeds its u64 bound")
            }
            Self::MissingCoinPublicKey => write!(
                f,
                "native ownPublicKey requires a coin public key in the circuit context"
            ),
            Self::UnsignedOutOfRange { value, max } => {
                write!(f, "unsigned value {value} exceeds Compact maximum {max}")
            }
            Self::UnsignedOverflow => write!(f, "Compact unsigned arithmetic overflow"),
            Self::UnsignedUnderflow => write!(f, "Compact unsigned arithmetic underflow"),
        }
    }
}

impl std::error::Error for CompactError {}
