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

#![cfg(feature = "ledger-transaction")]

use midnight_base_crypto::time::Timestamp;
use midnight_coin_structure::coin::PublicAddress;
use midnight_compact_runtime::Field;
use midnight_compact_runtime::ledger::{ContractAddress, ContractState, HashOutput, StateValue};
use midnight_compact_runtime::recording::RecordingFrame;
use midnight_compact_runtime::transaction::{
    CallProofBinding, CrossContractCallError, Observation, ObservedCallError,
    ObservedContractState, RecordedCall, VerifierKey, prepare_cross_contract_call,
};
use midnight_ledger_v9::structure::{ContractAction, LedgerState, Transaction};
use midnight_onchain_state::state::{ContractOperation, EntryPointBuf};
use midnight_serialize::{Deserializable, Serializable};
use rand::{SeedableRng, rngs::StdRng};

type Claim = (ContractAddress, HashOutput, Field);

struct Fixture {
    ledger: LedgerState<midnight_compact_runtime::ledger::DefaultDB>,
    caller: ObservedContractState,
    callee: ObservedContractState,
    caller_key: VerifierKey,
    callee_key: VerifierKey,
}

impl Fixture {
    fn new() -> Self {
        // Deterministic carrier keys for transaction preparation tests.
        let caller_key = carrier_key(1);
        let callee_key = carrier_key(2);
        let observed = |address, key: &VerifierKey, names: &[&str]| {
            let mut state = ContractState::new(
                StateValue::Array(Default::default()),
                Default::default(),
                Default::default(),
            );
            for name in names {
                state.operations = state.operations.insert(
                    entry_point(name),
                    ContractOperation::new(Some(key.clone()), None),
                );
            }
            ObservedContractState::new(
                address,
                state,
                Observation {
                    block_height: 12,
                    block_hash: [20; 32],
                    transaction_hash: address.0.0,
                },
            )
        };
        let caller = observed(ContractAddress(HashOutput([1; 32])), &caller_key, &["pay"]);
        // Install the same verifier for both names so substitution tests
        // isolate entrypoint binding from verifier identity.
        let callee = observed(
            ContractAddress(HashOutput([2; 32])),
            &callee_key,
            &["transfer", "decoy"],
        );
        let mut ledger = LedgerState::new("entrypoint-test");
        ledger.contract = ledger
            .contract
            .insert(caller.address(), caller.contract().clone())
            .insert(callee.address(), callee.contract().clone());
        Self {
            ledger,
            caller,
            callee,
            caller_key,
            callee_key,
        }
    }

    fn claim(&self) -> Claim {
        let commitment =
            midnight_compact_runtime::transient_commit((10_u64, 990_u64), Field::from(101));
        (
            self.callee.address(),
            entry_point("transfer").ep_hash(),
            commitment,
        )
    }

    fn caller(&self, claims: &[Claim]) -> RecordedCall<'_, (), u64> {
        let (frame, _) = RecordingFrame::new(self.caller.circuit_context(()))
            .kernel_self()
            .unwrap();
        let frame = claims
            .iter()
            .fold(frame, |frame, &(address, entry_point, commitment)| {
                frame
                    .kernel_claim_contract_call(address, entry_point, commitment)
                    .unwrap()
            });
        RecordedCall::new(&self.caller, frame.finish(990), "pay", 10_u64)
    }

    fn callee(&self, name: &'static str) -> RecordedCall<'_, (), u64> {
        let mut context = self.callee.circuit_context(());
        context.query.call_context.caller = Some(PublicAddress::Contract(self.caller.address()));
        let (frame, _) = RecordingFrame::new(context).kernel_self().unwrap();
        RecordedCall::new(&self.callee, frame.finish(990), name, 10_u64)
    }

    fn bindings(&self) -> (CallProofBinding<'_>, CallProofBinding<'_>) {
        (
            CallProofBinding {
                verifier: &self.caller_key,
                key_location: "caller/pay".into(),
                communication_commitment_rand: Field::from(102),
            },
            CallProofBinding {
                verifier: &self.callee_key,
                key_location: "callee/transfer".into(),
                communication_commitment_rand: Field::from(101),
            },
        )
    }
}

fn carrier_key(byte: u8) -> VerifierKey {
    let mut encoded = Vec::new();
    Serializable::serialize(&vec![byte], &mut encoded).unwrap();
    VerifierKey::deserialize(&mut encoded.as_slice(), 0).unwrap()
}

fn entry_point(name: &str) -> EntryPointBuf {
    EntryPointBuf(name.as_bytes().to_vec())
}

#[test]
fn authentic_pair_produces_one_intent_with_ordered_linked_calls() {
    let fixture = Fixture::new();
    let (caller_proof, callee_proof) = fixture.bindings();
    let tx = prepare_cross_contract_call(
        &fixture.ledger,
        fixture.caller(&[fixture.claim()]),
        caller_proof,
        fixture.callee("transfer"),
        callee_proof,
        Timestamp::from_secs(100),
        &mut StdRng::seed_from_u64(1),
    )
    .unwrap();
    let Transaction::Standard(tx) = tx else {
        panic!("expected a standard transaction");
    };
    assert_eq!(tx.network_id, "entrypoint-test");
    let mut intents = tx.intents.iter();
    let entry = intents.next().unwrap();
    assert!(intents.next().is_none());
    let mut actions = entry.1.actions.iter_deref();
    let (Some(ContractAction::Call(caller)), Some(ContractAction::Call(callee))) =
        (actions.next(), actions.next())
    else {
        panic!("expected a caller followed by a callee");
    };
    assert!(actions.next().is_none());
    assert_eq!(caller.address, fixture.caller.address());
    assert_eq!(callee.address, fixture.callee.address());
    assert_eq!(callee.entry_point, entry_point("transfer"));
    assert_eq!(callee.communication_commitment, fixture.claim().2);
    assert_eq!(caller.calls_with_seq(callee), Some((true, 0)));
    for call in [caller, callee] {
        assert!(call.guaranteed_transcript.is_some());
        assert!(call.fallible_transcript.is_none());
    }
}

#[test]
fn rejects_missing_extra_or_mismatched_claims_before_partitioning() {
    let fixture = Fixture::new();
    let (address, hash, commitment) = fixture.claim();
    for (claims, name) in [
        (vec![], "transfer"),
        (vec![fixture.claim(), fixture.claim()], "transfer"),
        (vec![fixture.claim()], "decoy"),
        (
            vec![(fixture.caller.address(), hash, commitment)],
            "transfer",
        ),
        (
            vec![(address, hash, commitment + Field::from(1))],
            "transfer",
        ),
    ] {
        let (caller_proof, callee_proof) = fixture.bindings();
        assert!(matches!(
            prepare_cross_contract_call(
                &fixture.ledger,
                fixture.caller(&claims),
                caller_proof,
                fixture.callee(name),
                callee_proof,
                Timestamp::from_secs(100),
                &mut StdRng::seed_from_u64(1),
            ),
            Err(CrossContractCallError::ClaimMismatch)
        ));
    }
}

#[test]
fn rejects_missing_snapshot_contract_and_wrong_verifier() {
    let fixture = Fixture::new();
    let (caller_proof, callee_proof) = fixture.bindings();
    let mut missing = LedgerState::new("entrypoint-test");
    missing.contract = missing
        .contract
        .insert(fixture.caller.address(), fixture.caller.contract().clone());
    assert!(matches!(
        prepare_cross_contract_call(
            &missing,
            fixture.caller(&[fixture.claim()]),
            caller_proof,
            fixture.callee("transfer"),
            callee_proof,
            Timestamp::from_secs(100),
            &mut StdRng::seed_from_u64(1),
        ),
        Err(CrossContractCallError::SnapshotContractMismatch(address))
            if address == fixture.callee.address()
    ));
    let (caller_proof, mut callee_proof) = fixture.bindings();
    callee_proof.verifier = &fixture.caller_key;
    let result = prepare_cross_contract_call(
        &fixture.ledger,
        fixture.caller(&[fixture.claim()]),
        caller_proof,
        fixture.callee("transfer"),
        callee_proof,
        Timestamp::from_secs(100),
        &mut StdRng::seed_from_u64(1),
    );
    assert!(
        matches!(
            result,
            Err(CrossContractCallError::Callee(
                ObservedCallError::VerifierMismatch(_)
            ))
        ),
        "{result:?}"
    );
}

#[test]
fn rejects_callee_with_wrong_caller_context() {
    let fixture = Fixture::new();
    let (caller_proof, callee_proof) = fixture.bindings();
    let (frame, _) = RecordingFrame::new(fixture.callee.circuit_context(()))
        .kernel_self()
        .unwrap();
    let callee = RecordedCall::new(&fixture.callee, frame.finish(990_u64), "transfer", 10_u64);
    assert!(matches!(
        prepare_cross_contract_call(
            &fixture.ledger,
            fixture.caller(&[fixture.claim()]),
            caller_proof,
            callee,
            callee_proof,
            Timestamp::from_secs(100),
            &mut StdRng::seed_from_u64(1),
        ),
        Err(CrossContractCallError::InitialContextMismatch(address))
            if address == fixture.callee.address()
    ));
}

#[test]
fn rejects_colliding_commitments_before_ledger_partitions_the_graph() {
    let fixture = Fixture::new();
    let (mut caller_proof, callee_proof) = fixture.bindings();
    caller_proof.communication_commitment_rand = callee_proof.communication_commitment_rand;
    assert!(matches!(
        prepare_cross_contract_call(
            &fixture.ledger,
            fixture.caller(&[fixture.claim()]),
            caller_proof,
            fixture.callee("transfer"),
            callee_proof,
            Timestamp::from_secs(100),
            &mut StdRng::seed_from_u64(1),
        ),
        Err(CrossContractCallError::DuplicateCommunicationCommitment)
    ));
}
