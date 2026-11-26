// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Standalone proof data from a generated recorded call.
//!
//! This is data conversion, not transaction preparation, source authentication,
//! satisfiability checking or proving. The generated method supplies the circuit
//! name and captures its declared input before executing the circuit once.

use crate::context::CircuitResult;
use crate::ledger::{DB, DefaultDB};
use crate::recording::RecordedCircuitResult;
use crate::{CompactError, Field};
use midnight_base_crypto::fab::AlignedValue;
use midnight_onchain_vm::ops::Op;
use midnight_onchain_vm::result_mode::ResultModeVerify;
use midnight_transient_crypto::fab::{AlignedValueExt, ValueReprAlignedValue};
use midnight_transient_crypto::proofs::{KeyLocation, ProofPreimage};
use midnight_transient_crypto::repr::FieldRepr;
use std::{error::Error, fmt};

/// Version of the explicitly selected standalone proof-data domain.
pub const API_VERSION: u32 = 1;

#[derive(Debug)]
pub enum ProofDataError {
    Execution(CompactError),
    InvalidCircuitName,
    InvalidAlignedValue(&'static str),
    InvalidProgram(&'static str),
    Serialization(std::io::Error),
}
impl fmt::Display for ProofDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(e) => write!(f, "proof-data execution failed: {e}"),
            Self::InvalidCircuitName => {
                f.write_str("proof data requires a nonempty circuit name without NUL")
            }
            Self::InvalidAlignedValue(part) => {
                write!(f, "invalid or noncanonical proof-data {part}")
            }
            Self::InvalidProgram(reason) => {
                write!(f, "invalid proof-data program encoding: {reason}")
            }
            Self::Serialization(e) => write!(f, "proof-data serialization failed: {e}"),
        }
    }
}
impl Error for ProofDataError {}
impl From<CompactError> for ProofDataError {
    fn from(error: CompactError) -> Self {
        Self::Execution(error)
    }
}

/// A captured call and immutable aligned proof-data components.
///
/// Construction is ordinary public data, not evidence of compiler identity or
/// transaction validity. Generated helpers hardcode their actual circuit name;
/// callers cannot change that name through a getter or during conversion.
pub struct RecordedProofData<Private, Output, D: DB = DefaultDB> {
    recorded: RecordedCircuitResult<Private, Output, D>,
    circuit: &'static str,
    input: AlignedValue,
    output: AlignedValue,
    private_outputs: Vec<AlignedValue>,
    public_transcript: Vec<Op<ResultModeVerify, D>>,
}

fn validate(value: &AlignedValue, part: &'static str) -> Result<(), ProofDataError> {
    if !value
        .alignment
        .fits_field_check(&value.value, |atom| Field::from_le_bytes(&atom.0).is_some())
        || !value.value.0.iter().all(|atom| atom.is_in_normal_form())
    {
        return Err(ProofDataError::InvalidAlignedValue(part));
    }
    Ok(())
}

// Mirror the SDK's encoding invariant, not a VM/satisfiability validator.
fn validate_program<D: DB>(ops: &[Op<ResultModeVerify, D>]) -> Result<(), ProofDataError> {
    for op in ops {
        match op {
            Op::Dup { n } | Op::Swap { n } | Op::Ins { n, .. } if *n >= 16 => {
                return Err(ProofDataError::InvalidProgram(
                    "stack operand exceeds four bits",
                ));
            }
            Op::Idx { path, .. } if !(1..=16).contains(&path.len()) => {
                return Err(ProofDataError::InvalidProgram(
                    "idx path length is not 1..=16",
                ));
            }
            Op::Popeq { result, .. } => validate(result, "public read")?,
            _ => {}
        }
    }
    Ok(())
}

impl<Private, Output, D: DB> RecordedProofData<Private, Output, D> {
    /// Capture actual recorded execution. Input comes from the generated typed
    /// arguments, not a fixture, decoder, callback or a returned Boolean alone.
    pub fn from_recorded(
        circuit: &'static str,
        input: AlignedValue,
        recorded: RecordedCircuitResult<Private, Output, D>,
    ) -> Result<Self, ProofDataError>
    where
        Output: Clone + Into<AlignedValue>,
    {
        if circuit.is_empty() || circuit.contains('\0') {
            return Err(ProofDataError::InvalidCircuitName);
        }
        validate(&input, "input")?;
        let output = recorded.execution.result.clone().into();
        validate(&output, "output")?;
        let private_outputs = recorded.execution.private_transcript_outputs.clone();
        for value in &private_outputs {
            validate(value, "private output")?;
        }
        let public_transcript = recorded.public.verify_ops().to_vec();
        validate_program(&public_transcript)?;
        Ok(Self {
            recorded,
            circuit,
            input,
            output,
            private_outputs,
            public_transcript,
        })
    }
    pub fn circuit_name(&self) -> &'static str {
        self.circuit
    }
    pub fn input(&self) -> &AlignedValue {
        &self.input
    }
    pub fn output(&self) -> &AlignedValue {
        &self.output
    }
    pub fn private_transcript_outputs(&self) -> &[AlignedValue] {
        &self.private_outputs
    }
    pub fn public_transcript(&self) -> &[Op<ResultModeVerify, D>] {
        &self.public_transcript
    }
    pub fn execution(&self) -> &CircuitResult<Private, Output, D> {
        &self.recorded.execution
    }
    pub fn into_recorded(self) -> RecordedCircuitResult<Private, Output, D> {
        self.recorded
    }

    /// Convert with the official SDK field representation and commitment.
    /// Binding and communication randomness are explicit caller inputs. There
    /// are no implicit zeros, key lookup, parameter reads, RNG or proof calls.
    pub fn to_preimage(&self, binding: Field, communications_randomness: Field) -> ProofPreimage {
        let mut private_transcript = Vec::new();
        for value in &self.private_outputs {
            value.value_only_field_repr(&mut private_transcript);
        }
        let mut public_transcript_inputs = Vec::new();
        let mut public_transcript_outputs = Vec::new();
        for op in &self.public_transcript {
            op.field_repr(&mut public_transcript_inputs);
            if let Op::Popeq { result, .. } = op {
                result.value_only_field_repr(&mut public_transcript_outputs);
            }
        }
        let io = ValueReprAlignedValue(AlignedValue::concat([&self.input, &self.output]));
        let commitment =
            midnight_transient_crypto::hash::transient_commit(&io, communications_randomness);
        ProofPreimage {
            inputs: ValueReprAlignedValue(self.input.clone()).field_vec(),
            private_transcript,
            public_transcript_inputs,
            public_transcript_outputs,
            binding_input: binding,
            communications_commitment: Some((commitment, communications_randomness)),
            key_location: KeyLocation(self.circuit.into()),
        }
    }
    pub fn serialize_preimage(
        &self,
        binding: Field,
        communications_randomness: Field,
    ) -> Result<Vec<u8>, ProofDataError> {
        let mut bytes = Vec::new();
        midnight_serialize::tagged_serialize(
            &self.to_preimage(binding, communications_randomness),
            &mut bytes,
        )
        .map_err(ProofDataError::Serialization)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdk_encoding_invariant_refuses_invalid_stack_operands() {
        for op in [
            Op::<ResultModeVerify, DefaultDB>::Dup { n: 16 },
            Op::Swap { n: 255 },
            Op::Ins {
                cached: false,
                n: 16,
            },
        ] {
            assert!(matches!(
                validate_program(&[op]),
                Err(ProofDataError::InvalidProgram(_))
            ));
        }
        for op in [
            Op::<ResultModeVerify, DefaultDB>::Dup { n: 15 },
            Op::Swap { n: 15 },
            Op::Ins {
                cached: true,
                n: 15,
            },
        ] {
            assert!(validate_program(&[op]).is_ok());
        }
        assert!(validate_program(&[Op::<ResultModeVerify, DefaultDB>::Dup { n: 2 }]).is_ok());
    }
}
