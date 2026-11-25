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

//! Runtime conversion tests only; generated consumer qualification is separate.
use midnight_base_crypto::time::Timestamp;
use midnight_compact_runtime::context::{CircuitContext, ConstructorContext, ConstructorResult};
use midnight_compact_runtime::fab::AlignedValue;
use midnight_compact_runtime::ledger::{ContractAddress, empty_contract_state};
use midnight_compact_runtime::proof_data::{ProofDataError, RecordedProofData};
use midnight_compact_runtime::recording::{RecordedCircuitResult, RecordingFrame};
use midnight_compact_runtime::{BoundedUint, Field};
use midnight_onchain_vm::ops::Op;
use midnight_serialize::{tagged_deserialize, tagged_serialize};
use midnight_transient_crypto::fab::{AlignedValueExt, ValueReprAlignedValue};
use midnight_transient_crypto::proofs::ProofPreimage;
use midnight_transient_crypto::repr::FieldRepr;
use std::io::Cursor;

type Bound = BoundedUint<{ u64::MAX as u128 }>;
struct PrivateToken(u64); // Deliberately neither Clone nor Copy.
fn context() -> CircuitContext<PrivateToken> {
    let mut ctx = ConstructorResult::new(
        ConstructorContext::new(PrivateToken(77)),
        empty_contract_state(),
    )
    .into_circuit_context(ContractAddress::default());
    ctx.query.call_context.tblock = Timestamp::from_secs(1);
    ctx.query.call_context.last_block_time = Timestamp::from_secs(900);
    ctx
}
fn trace(greater: bool, negate: bool, bound: u64) -> RecordedCircuitResult<PrivateToken, bool> {
    let frame = RecordingFrame::new(context());
    let (frame, value) = if greater {
        frame.kernel_block_time_greater_than(bound)
    } else {
        frame.kernel_block_time_less_than(bound)
    }
    .unwrap();
    frame.finish(if negate { !value } else { value })
}
fn data(
    name: &'static str,
    greater: bool,
    negate: bool,
    bound: u64,
) -> RecordedProofData<PrivateToken, bool> {
    RecordedProofData::from_recorded(
        name,
        AlignedValue::from(Bound::new(bound.into()).unwrap()),
        trace(greater, negate, bound),
    )
    .unwrap()
}
#[test]
fn twelve_runtime_components_and_nonzero_scalars_use_official_sdk_encoding() {
    for (name, greater, negate) in [
        ("lt", false, false),
        ("gte", false, true),
        ("gt", true, false),
        ("lte", true, true),
    ] {
        for bound in [5_u64, 0, 1] {
            let data = data(name, greater, negate, bound);
            let output = if greater { 1 > bound } else { 1 < bound } ^ negate;
            assert_eq!(data.circuit_name(), name);
            assert_eq!(
                data.input(),
                &AlignedValue::from(Bound::new(bound.into()).unwrap())
            );
            assert_eq!(data.output(), &AlignedValue::from(output));
            assert_eq!(data.execution().result, output);
            assert_eq!(data.execution().context.private_state.0, 77);
            assert!(data.private_transcript_outputs().is_empty());
            if negate {
                let read = data
                    .public_transcript()
                    .iter()
                    .find_map(|op| {
                        if let Op::Popeq { result, .. } = op {
                            Some(result)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_ne!(
                    read,
                    data.output(),
                    "raw SDK comparison must not be replaced by the negated typed result"
                );
            }
            let binding = Field::from(17_u64);
            let randomness = Field::from(29_u64);
            let preimage = data.to_preimage(binding, randomness);
            assert_eq!(preimage.binding_input, binding);
            assert_eq!(
                preimage.inputs,
                ValueReprAlignedValue(data.input().clone()).field_vec()
            );
            assert!(preimage.private_transcript.is_empty());
            let mut public_inputs = Vec::new();
            let mut public_outputs = Vec::new();
            for op in data.public_transcript() {
                op.field_repr(&mut public_inputs);
                if let Op::Popeq { result, .. } = op {
                    result.value_only_field_repr(&mut public_outputs);
                }
            }
            assert_eq!(preimage.public_transcript_inputs, public_inputs);
            assert_eq!(preimage.public_transcript_outputs, public_outputs);
            assert_eq!(preimage.key_location.0, name);
            let commitment = midnight_transient_crypto::hash::transient_commit(
                &ValueReprAlignedValue(AlignedValue::concat([data.input(), data.output()])),
                randomness,
            );
            assert_eq!(
                preimage.communications_commitment,
                Some((commitment, randomness))
            );
            let bytes = data.serialize_preimage(binding, randomness).unwrap();
            let mut cursor = Cursor::new(bytes.as_slice());
            let decoded: ProofPreimage = tagged_deserialize(&mut cursor).unwrap();
            assert_eq!(cursor.position(), bytes.len() as u64);
            assert_eq!(decoded, preimage);
            let mut canonical = Vec::new();
            tagged_serialize(&decoded, &mut canonical).unwrap();
            assert_eq!(canonical, bytes);
            let binding_only = data.to_preimage(binding + Field::from(1_u64), randomness);
            assert_ne!(binding_only.binding_input, preimage.binding_input);
            assert_eq!(
                binding_only.communications_commitment,
                preimage.communications_commitment
            );
            assert_eq!(
                binding_only.public_transcript_inputs,
                preimage.public_transcript_inputs
            );
            let randomness_only = data.to_preimage(binding, randomness + Field::from(1_u64));
            assert_eq!(randomness_only.binding_input, preimage.binding_input);
            assert_ne!(
                randomness_only.communications_commitment,
                preimage.communications_commitment
            );
            // Explicit zero is allowed for the separate historical TS comparison;
            // the API never substitutes it for either caller-supplied scalar.
            assert_ne!(
                data.serialize_preimage(Field::from(0_u64), Field::from(0_u64))
                    .unwrap(),
                bytes
            );
        }
    }
}
#[test]
fn malformed_input_alignment_normal_form_and_field_are_data_refusals() {
    let mut alignment = AlignedValue::from(true);
    alignment.alignment = AlignedValue::from(()).alignment;
    let mut trailing_zero = AlignedValue::from(Bound::new(5).unwrap());
    trailing_zero.value.0[0].0.push(0);
    let mut field = AlignedValue::from(Field::from(1_u64));
    field.value.0[0].0 = vec![255; 32];
    for invalid in [alignment, trailing_zero, field] {
        assert!(matches!(
            RecordedProofData::from_recorded("test", invalid, trace(false, false, 5)),
            Err(ProofDataError::InvalidAlignedValue("input"))
        ));
    }
    for name in ["", "bad\0name"] {
        assert!(matches!(
            RecordedProofData::from_recorded(
                name,
                AlignedValue::from(Bound::new(5).unwrap()),
                trace(false, false, 5)
            ),
            Err(ProofDataError::InvalidCircuitName)
        ));
    }
}
#[test]
fn malformed_private_output_refuses_and_consuming_access_preserves_execution() {
    let mut recorded = trace(false, false, 5);
    let mut invalid = AlignedValue::from(true);
    invalid.alignment = AlignedValue::from(()).alignment;
    recorded.execution.private_transcript_outputs.push(invalid);
    assert!(matches!(
        RecordedProofData::from_recorded(
            "test",
            AlignedValue::from(Bound::new(5).unwrap()),
            recorded
        ),
        Err(ProofDataError::InvalidAlignedValue("private output"))
    ));
    let original = data("test", false, false, 5).into_recorded();
    assert!(original.execution.result);
    assert_eq!(original.execution.context.private_state.0, 77);
    assert!(!original.public.verify_ops().is_empty());
}

/// Supplemental boundary, not an additional original upstream case.
#[test]
fn full_width_u64_input_roundtrips_without_narrowing() {
    let data = data("wide_bound", false, false, u64::MAX);
    assert!(data.execution().result);
    let expected = AlignedValue::from(Bound::new(u64::MAX as u128).unwrap());
    assert_eq!(data.input(), &expected);
    let bytes = data
        .serialize_preimage(Field::from(17_u64), Field::from(29_u64))
        .unwrap();
    let mut cursor = Cursor::new(bytes.as_slice());
    let decoded: ProofPreimage = tagged_deserialize(&mut cursor).unwrap();
    assert_eq!(cursor.position(), bytes.len() as u64);
    assert_eq!(decoded.inputs, ValueReprAlignedValue(expected).field_vec());
    let mut canonical = Vec::new();
    tagged_serialize(&decoded, &mut canonical).unwrap();
    assert_eq!(canonical, bytes);
}

/// Supplemental codec refusals, without a model, parameter provider or proof.
#[test]
fn official_single_preimage_codec_refuses_wrong_tag_truncation_and_trailing_bytes() {
    let bytes = data("codec", false, false, 5)
        .serialize_preimage(Field::from(17_u64), Field::from(29_u64))
        .unwrap();
    assert!(tagged_deserialize::<ProofPreimage>(bytes.as_slice()).is_ok());

    let mut wrong_tag = bytes.clone();
    wrong_tag[0] ^= 1;
    assert!(tagged_deserialize::<ProofPreimage>(wrong_tag.as_slice()).is_err());
    assert!(tagged_deserialize::<ProofPreimage>(&bytes[..bytes.len() - 1]).is_err());

    let mut trailing = bytes;
    trailing.push(0);
    assert!(tagged_deserialize::<ProofPreimage>(trailing.as_slice()).is_err());
}

/// Supplemental witness data exercises a nonempty private vector and Unit I/O.
#[test]
fn private_output_conversion_preserves_actual_vector_and_nonclone_private_ownership() {
    let (frame, witness_value) = RecordingFrame::new(context()).witness(|context| {
        (
            PrivateToken(context.private_state.0 + 1),
            Bound::new(7).unwrap(),
        )
    });
    let recorded = frame.finish(());
    let expected = AlignedValue::from(witness_value);
    assert_eq!(
        recorded.execution.private_transcript_outputs,
        vec![expected.clone()]
    );
    let data =
        RecordedProofData::from_recorded("private_unit", AlignedValue::from(()), recorded).unwrap();
    assert_eq!(data.execution().context.private_state.0, 78);
    assert_eq!(data.output(), &AlignedValue::from(()));
    assert_eq!(
        data.private_transcript_outputs(),
        std::slice::from_ref(&expected)
    );
    let preimage = data.to_preimage(Field::from(17_u64), Field::from(29_u64));
    let mut private_fields = Vec::new();
    expected.value_only_field_repr(&mut private_fields);
    assert_eq!(preimage.private_transcript, private_fields);
    assert!(!preimage.private_transcript.is_empty());
    assert!(preimage.inputs.is_empty());
    assert!(preimage.public_transcript_inputs.is_empty());
    assert!(preimage.public_transcript_outputs.is_empty());
    let bytes = data
        .serialize_preimage(Field::from(17_u64), Field::from(29_u64))
        .unwrap();
    let decoded: ProofPreimage = tagged_deserialize(bytes.as_slice()).unwrap();
    assert_eq!(decoded, preimage);
    let recorded = data.into_recorded();
    assert_eq!(recorded.execution.context.private_state.0, 78);
    assert_eq!(
        recorded.execution.private_transcript_outputs,
        vec![expected]
    );
}
