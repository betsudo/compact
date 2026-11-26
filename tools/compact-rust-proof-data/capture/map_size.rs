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

//! Source-only adapter for fresh generated root Map cardinality proof data.
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
type Cardinality = runtime::BoundedUint<{ u64::MAX as u128 }>;
#[derive(Clone, Copy)]
enum Case {
    Empty,
    DefaultZero,
    TwoWide,
    Reset,
}
impl Case {
    const ALL: [Self; 4] = [Self::Empty, Self::DefaultZero, Self::TwoWide, Self::Reset];
    fn id(self) -> &'static str {
        match self {
            Self::Empty => "empty-zero",
            Self::DefaultZero => "default-zero-singleton-one",
            Self::TwoWide => "wide-two-entries",
            Self::Reset => "reset-zero",
        }
    }
    fn expected(self) -> u128 {
        match self {
            Self::Empty | Self::Reset => 0,
            Self::DefaultZero => 1,
            Self::TwoWide => 2,
        }
    }
}
fn wide(bits: usize, tail: u64) -> Field {
    let mut n = Field::from(1u64);
    for _ in 0..bits {
        n = n + n;
    }
    n + Field::from(tail)
}
fn uint64_scalar(value: Field) -> Result<u64> {
    let raw = value.as_le_bytes();
    ensure!(
        raw.len() == 32 && raw[8..].iter().all(|b| *b == 0),
        "Uint64 canonical LE32 range"
    );
    let result = u64::from_le_bytes(raw[..8].try_into()?);
    ensure!(Field::from(result) == value, "Uint64 field roundtrip");
    Ok(result)
}
fn carrier_boundaries() -> Result<()> {
    use midnight_transient_crypto::{
        fab::ValueReprAlignedValue,
        repr::{FieldRepr, FromFieldRepr},
    };
    ensure!(Cardinality::BYTE_LENGTH == 8, "Uint64 eight-byte alignment");
    for n in [0, 1, 2, u64::MAX] {
        let value = Cardinality::new(u128::from(n))?;
        let scalar = Field::from(n);
        ensure!(
            value.value() == u128::from(n)
                && uint64_scalar(scalar)? == n
                && Cardinality::from_field_repr(&[scalar]) == Some(value)
                && ValueReprAlignedValue(AlignedValue::from(value)).field_vec() == vec![scalar],
            "pure Uint64 carrier/LE32 boundary"
        );
    }
    let overflow = Field::from(u64::MAX) + Field::from(1u64);
    ensure!(
        Cardinality::new(u128::from(u64::MAX) + 1).is_err()
            && Cardinality::from_field_repr(&[overflow]).is_none()
            && uint64_scalar(overflow).is_err()
            && uint64_scalar(wide(128, 1)).is_err(),
        "out-of-range Uint64 refused"
    );
    ensure!(
        compact_rust_proof_data::input::scalar(&"ff".repeat(32)).is_err(),
        "noncanonical LE32 field refused"
    );
    Ok(())
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
        contract::PublicStateView::from(&cx).c()?.size()?.value() == 0,
        "actual fresh empty Map has cardinality zero"
    );
    if matches!(case, Case::Empty) {
        return Ok(cx);
    }
    let key = Field::from(1u64);
    let cx = preparation(c.insert_default(cx, key)?)?;
    let view = contract::PublicStateView::from(&cx);
    ensure!(
        view.c()?.size()?.value() == 1
            && view.c()?.try_member(key)?
            && view.c()?.lookup(key)? == Field::from(0u64),
        "actual present default-zero entry counts as one"
    );
    if matches!(case, Case::DefaultZero) {
        return Ok(cx);
    }
    let wide_key = wide(128, 1);
    let wide_value = wide(129, 7);
    let cx = preparation(c.insert(cx, wide_key, wide_value)?)?;
    let view = contract::PublicStateView::from(&cx);
    ensure!(
        view.c()?.size()?.value() == 2
            && view.c()?.try_member(wide_key)?
            && view.c()?.lookup(wide_key)? == wide_value
            && view.c()?.try_member(key)?
            && view.c()?.lookup(key)? == Field::from(0u64),
        "two distinct keys including wide Field with zero/wide values"
    );
    if matches!(case, Case::Reset) {
        let cx = preparation(c.reset_to_default(cx)?)?;
        ensure!(
            contract::PublicStateView::from(&cx).c()?.size()?.value() == 0
                && state(cx.query.state.get_ref())? == initial,
            "actual reset restores whole initial Map state and cardinality zero"
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
fn capture(
    case: Case,
) -> Result<(
    runtime::proof_data::RecordedProofData<Private, Cardinality>,
    Value,
)> {
    let n = contract::Contract::default();
    let r = contract::Contract::default();
    let h = contract::Contract::default();
    let nx = fresh(case)?;
    let rx = fresh(case)?;
    let hx = fresh(case)?;
    let before = state(hx.query.state.get_ref())?;
    ensure!(
        before == state(nx.query.state.get_ref())? && before == state(rx.query.state.get_ref())?,
        "independent fresh contexts equivalent before size"
    );
    let replay_before = hx.query.clone();
    let addr = addresses(&hx);
    let effects = hx.query.effects.clone();
    let intents = hx.circuit_zswap().clone();
    let events = hx.events().to_vec();
    let native = n.size(nx)?;
    let recorded = r.recording.size(rx)?;
    let data = h.recording.size_proof_data(hx)?; // One generated helper invocation.
    ensure!(
        native.result.value() == case.expected()
            && recorded.execution.result.value() == case.expected()
            && data.execution().result.value() == case.expected(),
        "actual native/recorded/helper Uint64 result"
    );
    ensure!(
        data.circuit_name() == "size"
            && data.input() == &AlignedValue::from(())
            && data.output() == &AlignedValue::from(Cardinality::new(case.expected())?),
        "actual typed Unit input and Uint64 output envelope"
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
        "size has no writes/removals"
    );
    let cardinality = u64::try_from(data.execution().result.value())?;
    let scalar = Field::from(cardinality);
    ensure!(
        uint64_scalar(scalar)? == cardinality,
        "actual Uint64 LE32 result"
    );
    let observed = json!({"case":case.id(),"supplemental":true,"originalUniqueCaseCredit":0,
        "typedInput":"Unit","typedOutput":"Uint64","input":[],
        "output":compact_rust_proof_data::input::hex(&scalar),"actualResult":data.execution().result.value(),
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
fn emit_size(
    out: &std::path::Path,
    id: &str,
    data: &runtime::proof_data::RecordedProofData<Private, Cardinality>,
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
    let cardinality = u64::try_from(data.execution().result.value())?;
    let scalar = Field::from(cardinality);
    ensure!(
        uint64_scalar(scalar)? == cardinality,
        "actual Uint64 LE32 result"
    );
    ensure!(
        decoded.private_transcript.is_empty()
            && decoded.inputs.is_empty()
            && output == vec![scalar]
            && decoded.public_transcript_outputs == output,
        "canonical zero-input/actual Uint64 public output with no private outputs"
    );
    ensure!(
        decoded.communications_commitment
            == Some((
                midnight_transient_crypto::hash::transient_commit(&[scalar], random),
                random
            )),
        "actual communication commits Unit input plus actual Uint64 output only"
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
    ensure!(op == "size", "size capture only");
    carrier_boundaries()?;
    let mut cases = Vec::new();
    let mut observations = Vec::new();
    for case in Case::ALL {
        let (data, observation) = capture(case)?;
        cases.push(emit_size(out, case.id(), &data, binding, random)?);
        observations.push(observation);
    }
    compact_rust_proof_data::codec::write_json(
        out,
        "observed.json",
        &json!({
        "schema":"map-size-native-capture/v1","cases":observations,
        "observedCallExecuted":false,"modelExecuted":false,"parametersUsed":false,
        "keysOrProofsProduced":false,"originalTenAssertionFixtureExecuted":false,
        "pureUint64CarrierBoundaryChecksPassed":true,"maximumCardinalityMapExecuted":false,
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
                "schema":"map-size-native-capture-failure/v1","error":format!("{error:#}"),"success":false})).is_err() {
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
