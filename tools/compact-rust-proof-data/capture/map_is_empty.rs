// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Source-only adapter for fresh generated root Map emptiness proof data.
//! No witness/RNG/check/model/parameter/key/proof/transaction call is made.
use anyhow::{Result, ensure};
use generated::{ledger_contract as contract, runtime};
use midnight_onchain_vm::ops::Op;
use runtime::{
    Field,
    context::{CircuitContext, ConstructorContext},
    fab::AlignedValue,
    ledger::ContractAddress,
};
use serde_json::{Value, json};
mod common;
use common::{gas, state};

pub struct Private(pub u8); // Intentionally not Clone or Copy.
#[derive(Clone, Copy)]
enum Case {
    Empty,
    DefaultZero,
    Nonzero,
    Reset,
}
impl Case {
    const ALL: [Self; 4] = [Self::Empty, Self::DefaultZero, Self::Nonzero, Self::Reset];
    fn id(self) -> &'static str {
        match self {
            Self::Empty => "empty-true",
            Self::DefaultZero => "default-zero-false",
            Self::Nonzero => "nonzero-false",
            Self::Reset => "reset-true",
        }
    }
    fn expected(self) -> bool {
        matches!(self, Self::Empty | Self::Reset)
    }
}
fn preparation(
    step: runtime::context::CircuitResult<Private, ()>,
) -> Result<CircuitContext<Private>> {
    ensure!(
        step.context.private_state.0 == 91 && step.private_transcript_outputs.is_empty(),
        "actual no-witness preparation retains nonClone private91"
    );
    Ok(step.context)
}
fn fresh(case: Case) -> Result<CircuitContext<Private>> {
    let c = contract::Contract::default();
    let cx = contract::initial_state(ConstructorContext::new(Private(91)))?
        .into_circuit_context(ContractAddress::default());
    let initial = state(cx.query.state.get_ref())?;
    ensure!(
        contract::PublicStateView::from(&cx).c()?.is_empty(),
        "actual fresh empty Map"
    );
    if matches!(case, Case::Empty) {
        return Ok(cx);
    }
    let key = Field::from(1u64);
    let value = Field::from(if matches!(case, Case::DefaultZero) {
        0u64
    } else {
        7u64
    });
    let cx = preparation(if matches!(case, Case::DefaultZero) {
        c.insert_default(cx, key)?
    } else {
        c.insert(cx, key, value)?
    })?;
    let view = contract::PublicStateView::from(&cx);
    ensure!(
        !view.c()?.is_empty() && view.c()?.try_member(key)? && view.c()?.lookup(key)? == value,
        "actual inserted Map including a present default-zero value"
    );
    if matches!(case, Case::Reset) {
        let cx = preparation(c.reset_to_default(cx)?)?;
        ensure!(
            contract::PublicStateView::from(&cx).c()?.is_empty()
                && state(cx.query.state.get_ref())? == initial,
            "actual reset restores whole initial Map state"
        );
        Ok(cx)
    } else {
        Ok(cx)
    }
}
fn addresses(c: &CircuitContext<Private>) -> (ContractAddress, ContractAddress) {
    (
        c.query.address.clone(),
        c.query.call_context.own_address.clone(),
    )
}
fn capture(case: Case) -> Result<(runtime::proof_data::RecordedProofData<Private, bool>, Value)> {
    let n = contract::Contract::default();
    let r = contract::Contract::default();
    let h = contract::Contract::default();
    let nx = fresh(case)?;
    let rx = fresh(case)?;
    let hx = fresh(case)?;
    let before = state(hx.query.state.get_ref())?;
    ensure!(
        before == state(nx.query.state.get_ref())? && before == state(rx.query.state.get_ref())?,
        "independent fresh contexts equivalent before is_empty"
    );
    let replay_before = hx.query.clone();
    let addr = addresses(&hx);
    let effects = hx.query.effects.clone();
    let intents = hx.circuit_zswap().clone();
    let events = hx.events().to_vec();
    let native = n.is_empty(nx)?;
    let recorded = r.recording.is_empty(rx)?;
    let data = h.recording.is_empty_proof_data(hx)?; // One generated helper invocation.
    ensure!(
        native.result == case.expected()
            && recorded.execution.result == case.expected()
            && data.execution().result == case.expected(),
        "actual native/recorded/helper Boolean result"
    );
    ensure!(
        data.circuit_name() == "is_empty"
            && data.input() == &AlignedValue::from(())
            && data.output() == &AlignedValue::from(case.expected()),
        "actual typed Unit input and Boolean output envelope"
    );
    ensure!(
        data.private_transcript_outputs().is_empty()
            && native.private_transcript_outputs.is_empty()
            && recorded.execution.private_transcript_outputs.is_empty(),
        "no witness/private transcript outputs"
    );
    for cx in [
        &native.context,
        &recorded.execution.context,
        &data.execution().context,
    ] {
        ensure!(
            cx.private_state.0 == 91 && state(cx.query.state.get_ref())? == before,
            "nonClone91 and complete read-only state"
        );
        ensure!(
            addresses(cx) == addr
                && cx.query.effects == effects
                && cx.circuit_zswap() == &intents
                && cx.events() == events.as_slice(),
            "full address/effects/intents/events preserved"
        );
    }
    ensure!(
        data.public_transcript() == recorded.public.verify_ops(),
        "complete actual ordered recorded/helper ops equality"
    );
    ensure!(
        gas(&native.gas_cost) == gas(&recorded.execution.gas_cost)
            && gas(&native.gas_cost) == gas(&data.execution().gas_cost),
        "three-route exact gas equality"
    );
    let replay = replay_before.query(
        data.public_transcript(),
        None,
        &data.execution().context.cost_model,
    )?;
    ensure!(
        state(replay.context.state.get_ref())? == before && replay.context.effects == effects,
        "official VM Verify replay complete state/effects"
    );
    let ops = data.public_transcript();
    let public_transcript = serde_json::to_value(ops)?;
    ensure!(
        !ops.iter()
            .any(|o| matches!(o, Op::Ins { .. } | Op::Rem { .. })),
        "is_empty has no writes/removals"
    );
    let bit = Field::from(u64::from(data.execution().result));
    let observed = json!({"case":case.id(),"supplemental":true,"originalUniqueCaseCredit":0,
        "typedInput":"Unit","typedOutput":"Boolean","input":[],
        "output":compact_rust_proof_data::input::hex(&bit),"actualResult":data.execution().result,
        "nonClonePrivate91":true,"noWitnessOutputs":true,"nativeRecordedHelperGasEqual":true,
        "completeStateUnchanged":true,"addressEffectsIntentsEventsPreserved":true,
        "completeRecordedHelperOpsEqual":true,"officialVerifyReplayPassed":true,
        "verifyOps":ops.len(),"publicTranscript":public_transcript,
        "idx":ops.iter().filter(|o|matches!(o,Op::Idx{..})).count(),
        "popeq":ops.iter().filter(|o|matches!(o,Op::Popeq{..})).count(),
        "routeGas":gas(&data.execution().gas_cost),"aggregateReplayGas":gas(&replay.gas_cost),
        "observedCallExecuted":false,"TSParityClaim":false,"modelExecuted":false});
    Ok((data, observed))
}
// Pure conversion may repeat serialization, never the generated helper call.
fn emit_empty(
    out: &std::path::Path,
    id: &str,
    data: &runtime::proof_data::RecordedProofData<Private, bool>,
    binding: Field,
    random: Field,
) -> Result<compact_rust_proof_data::input::Case> {
    use compact_rust_proof_data::codec;
    use midnight_transient_crypto::{
        fab::ValueReprAlignedValue, proofs::ProofPreimage, repr::FieldRepr,
    };
    let bytes = data.serialize_preimage(binding, random)?;
    let decoded: ProofPreimage = codec::decode(&bytes)?;
    let output = ValueReprAlignedValue(data.output().clone()).field_vec();
    let bit = Field::from(u64::from(data.execution().result));
    ensure!(
        decoded.private_transcript.is_empty()
            && decoded.inputs.is_empty()
            && output == vec![bit]
            && decoded.public_transcript_outputs == output,
        "canonical zero-input/actual Boolean public output with no private outputs"
    );
    ensure!(
        decoded.communications_commitment
            == Some((
                midnight_transient_crypto::hash::transient_commit(&[bit], random),
                random
            )),
        "actual communication commits Unit input plus actual Bool output only"
    );
    ensure!(
        codec::decode::<ProofPreimage>(&bytes[..bytes.len() - 1]).is_err(),
        "truncated tagged object refused"
    );
    let mut appended = bytes.clone();
    appended.push(0);
    ensure!(
        codec::decode::<ProofPreimage>(&appended).is_err(),
        "appended tagged object refused"
    );
    let mut wrong_tag = bytes;
    wrong_tag[0] ^= 0x80;
    ensure!(
        codec::decode::<ProofPreimage>(&wrong_tag).is_err(),
        "wrong tagged object refused"
    );
    common::emit(out, id, true, data, binding, random)
}
fn run(op: &str, out: &std::path::Path, binding: Field, random: Field) -> Result<()> {
    ensure!(op == "is_empty", "is_empty capture only");
    let mut cases = Vec::new();
    let mut observations = Vec::new();
    for case in Case::ALL {
        let (data, observation) = capture(case)?;
        cases.push(emit_empty(out, case.id(), &data, binding, random)?);
        observations.push(observation);
    }
    compact_rust_proof_data::codec::write_json(
        out,
        "observed.json",
        &json!({
        "schema":"map-is-empty-native-capture/v1","cases":observations,
        "observedCallExecuted":false,"modelExecuted":false,"parametersUsed":false,
        "keysOrProofsProduced":false,"originalTenAssertionFixtureExecuted":false,
        "originalUniqueCaseCredit":0,"TSParityClaim":false}),
    )?;
    common::finish(out, &cases)
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("PRIVATE_CAPTURE_PANIC")));
    let result = common::args().and_then(|(op, out, binding, random)| {
        let first = run(&op, &out, binding, random);
        if let Err(ref error) = first {
            if compact_rust_proof_data::codec::write_json(&out, "failure.json", &json!({
                "schema":"map-is-empty-native-capture-failure/v1","error":format!("{error:#}"),"success":false})).is_err() {
                eprintln!("PRIVATE_FAILURE_RECEIPT_UNAVAILABLE");
            }
        }
        first
    });
    if result.is_err() {
        eprintln!("GENERATED_CAPTURE_FAILED");
        std::process::exit(1);
    }
}
