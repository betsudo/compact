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

use super::*;
use midnight_coin_structure::coin::PublicAddress;
use midnight_ledger_v9::construct::{
    PrePartitionContractCall, SegmentSpecifier, communication_commitment,
};
use midnight_ledger_v9::structure::StandardTransaction;
use midnight_onchain_runtime::context::ClaimedContractCallsValue;
use midnight_onchain_state::state::AlignedValueChecked;
use midnight_storage::storage::HashSet;

/// The expected installed verifier, prover lookup key and opening for one call.
pub struct CallProofBinding<'a> {
    pub verifier: &'a VerifierKey,
    pub key_location: Cow<'static, str>,
    pub communication_commitment_rand: Fr,
}

#[derive(Debug)]
pub enum CrossContractCallError {
    Caller(ObservedCallError),
    Callee(ObservedCallError),
    DifferentSnapshots,
    SameContract,
    SnapshotContractMismatch(ContractAddress),
    InitialContextMismatch(ContractAddress),
    UnsupportedEffects(ContractAddress),
    InvalidProofValues(ContractAddress),
    DuplicateKeyLocation,
    DuplicateCommunicationCommitment,
    ClaimMismatch,
    Partition(String),
}

impl fmt::Display for CrossContractCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Caller(error) => write!(f, "caller rejected: {error}"),
            Self::Callee(error) => write!(f, "callee rejected: {error}"),
            Self::DifferentSnapshots => f.write_str("the calls must share an observed block"),
            Self::SameContract => f.write_str("the caller and callee must be different contracts"),
            Self::SnapshotContractMismatch(address) => {
                write!(
                    f,
                    "observation for {address:?} differs from the ledger snapshot"
                )
            }
            Self::InitialContextMismatch(address) => {
                write!(
                    f,
                    "initial context for {address:?} does not match the call pair"
                )
            }
            Self::UnsupportedEffects(address) => {
                write!(
                    f,
                    "coin effects for {address:?} require offer-backed preparation"
                )
            }
            Self::InvalidProofValues(address) => {
                write!(f, "proof values for {address:?} have invalid FAB encodings")
            }
            Self::DuplicateKeyLocation => f.write_str("the two prover lookup keys must differ"),
            Self::DuplicateCommunicationCommitment => {
                f.write_str("the two communication commitments must differ")
            }
            Self::ClaimMismatch => {
                f.write_str("expected exactly one caller claim matching the callee")
            }
            Self::Partition(error) => write!(f, "ledger rejected the joint partition: {error}"),
        }
    }
}

impl Error for CrossContractCallError {}

/// Assemble one caller and one leaf callee against a caller-trusted ledger
/// snapshot. The callee trace records the caller's contract identity. Both
/// traces must be fully guaranteed and have no coin effects. The result
/// contains proof preimages and requires fee funding, proving and ledger
/// verification before application.
pub fn prepare_cross_contract_call<
    CallerPrivate,
    CallerOutput: Into<AlignedValue>,
    CalleePrivate,
    CalleeOutput: Into<AlignedValue>,
    D: DB,
>(
    ledger: &LedgerState<D>,
    caller: RecordedCall<'_, CallerPrivate, CallerOutput, D>,
    caller_proof: CallProofBinding<'_>,
    callee: RecordedCall<'_, CalleePrivate, CalleeOutput, D>,
    callee_proof: CallProofBinding<'_>,
    ttl: Timestamp,
    rng: &mut (impl Rng + CryptoRng),
) -> Result<
    Transaction<Signature, ProofPreimageMarker, PedersenRandomness, D>,
    CrossContractCallError,
> {
    let caller_address = caller.observed.address;
    if caller_address == callee.observed.address {
        return Err(CrossContractCallError::SameContract);
    }
    let caller_block = caller.observed.observation;
    let callee_block = callee.observed.observation;
    let caller_context = &caller.recorded.public.initial().call_context;
    let callee_context = &callee.recorded.public.initial().call_context;
    if caller_block.block_hash != callee_block.block_hash
        || caller_block.block_height != callee_block.block_height
        || caller_context.tblock != callee_context.tblock
        || caller_context.tblock_err != callee_context.tblock_err
        || caller_context.parent_block_hash != callee_context.parent_block_hash
        || caller_context.last_block_time != callee_context.last_block_time
    {
        return Err(CrossContractCallError::DifferentSnapshots);
    }
    if caller_proof.key_location == callee_proof.key_location {
        return Err(CrossContractCallError::DuplicateKeyLocation);
    }

    let caller_operation = caller
        .checked_operation(caller_proof.verifier)
        .map_err(CrossContractCallError::Caller)?;
    let callee_operation = callee
        .checked_operation(callee_proof.verifier)
        .map_err(CrossContractCallError::Callee)?;
    let (caller, caller_claims) =
        prepare_unpartitioned(ledger, caller, caller_operation, caller_proof, None)?;
    let (callee, callee_claims) = prepare_unpartitioned(
        ledger,
        callee,
        callee_operation,
        callee_proof,
        Some(PublicAddress::Contract(caller_address)),
    )?;
    let callee_commitment = call_commitment(&callee);
    let expected_claim = (
        0,
        callee.address,
        callee.entry_point.ep_hash(),
        callee_commitment,
    );
    let mut claims = caller_claims.iter();
    if !callee_claims.is_empty()
        || claims.next().map(|claim| claim.into_inner()) != Some(expected_claim)
        || claims.next().is_some()
    {
        return Err(CrossContractCallError::ClaimMismatch);
    }
    if call_commitment(&caller) == callee_commitment {
        return Err(CrossContractCallError::DuplicateCommunicationCommitment);
    }
    let tx = StandardTransaction::new(
        ledger.network_id.clone(),
        HashMap::new(),
        None,
        HashMap::new(),
    )
    .add_calls::<ProofPreimage>(
        rng,
        SegmentSpecifier::GuaranteedOnly,
        &[caller, callee],
        &ledger.parameters,
        ttl,
        &[],
        &[],
        &[],
    )
    .map_err(|error| CrossContractCallError::Partition(format!("{error:?}")))?;
    Ok(Transaction::Standard(tx))
}

fn call_commitment<D: DB>(call: &PrePartitionContractCall<D>) -> Fr {
    communication_commitment(
        call.input.clone(),
        call.output.clone(),
        call.communication_commitment_rand,
    )
}

fn prepare_unpartitioned<Private, Output: Into<AlignedValue>, D: DB>(
    ledger: &LedgerState<D>,
    call: RecordedCall<'_, Private, Output, D>,
    operation: ContractOperation,
    proof: CallProofBinding<'_>,
    expected_caller: Option<PublicAddress>,
) -> Result<
    (
        PrePartitionContractCall<D>,
        HashSet<ClaimedContractCallsValue, D>,
    ),
    CrossContractCallError,
> {
    let address = call.observed.address;
    let installed = ledger.contract.get(&address);
    if installed != Some(&call.observed.contract) {
        return Err(CrossContractCallError::SnapshotContractMismatch(address));
    }
    let initial = call.recorded.public.initial();
    if initial.effects != Effects::default()
        || !initial.call_context.com_indices.is_empty()
        || initial.call_context.balance != call.observed.contract.balance
        || initial.call_context.caller != expected_caller
    {
        return Err(CrossContractCallError::InitialContextMismatch(address));
    }
    let (pre_transcript, mut effects) =
        validated_pre_transcript(call.recorded.public, &call.recorded.execution, false).map_err(
            |error| {
                let error = ObservedCallError::Prepare(error);
                if expected_caller.is_some() {
                    CrossContractCallError::Callee(error)
                } else {
                    CrossContractCallError::Caller(error)
                }
            },
        )?;
    let claims = std::mem::take(&mut effects.claimed_contract_calls);
    if effects != Effects::default() {
        return Err(CrossContractCallError::UnsupportedEffects(address));
    }
    let output = call.recorded.execution.result.into();
    if AlignedValueChecked::try_from(call.input.clone()).is_err()
        || AlignedValueChecked::try_from(output.clone()).is_err()
    {
        return Err(CrossContractCallError::InvalidProofValues(address));
    }
    Ok((
        PrePartitionContractCall {
            address,
            entry_point: EntryPointBuf(call.entry_point.as_bytes().to_vec()),
            op: operation,
            pre_transcript,
            private_transcript_outputs: call.recorded.execution.private_transcript_outputs,
            input: call.input,
            output,
            communication_commitment_rand: proof.communication_commitment_rand,
            key_location: KeyLocation(proof.key_location),
        },
        claims,
    ))
}
