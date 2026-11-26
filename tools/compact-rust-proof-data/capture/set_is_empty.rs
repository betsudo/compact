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

//! Source-only adapter for fresh generated root Set<Field> emptiness proof data.
//! No witness/RNG/check/model/parameter/key/proof/transaction call is made.
use anyhow::{Result, ensure};
use generated::{ledger_contract as contract, runtime};
use midnight_onchain_vm::ops::Op;
use midnight_transient_crypto::repr::FieldRepr;
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
    ZeroPresent,
    ZeroAndWide,
    Reset,
}
impl Case {
    const ALL: [Self; 4] = [
        Self::Empty,
        Self::ZeroPresent,
        Self::ZeroAndWide,
        Self::Reset,
    ];
    fn id(self) -> &'static str {
        match self {
            Self::Empty => "empty-true",
            Self::ZeroPresent => "zero-present-false",
            Self::ZeroAndWide => "zero-and-wide-false",
            Self::Reset => "reset-true",
        }
    }
    fn expected(self) -> bool {
        matches!(self, Self::Empty | Self::Reset)
    }
}
type Cardinality = runtime::BoundedUint<{ u64::MAX as u128 }>;
fn wide() -> Result<Field> {
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    bytes[16] = 1; // Exactly 2^128 + 1, no caller/witness/RNG substitution.
    Field::from_le_bytes(&bytes).ok_or_else(|| anyhow::anyhow!("canonical wide Field"))
}
fn checked<T: PartialEq>(
    step: runtime::context::CircuitResult<Private, T>,
    expected: T,
) -> Result<CircuitContext<Private>> {
    ensure!(
        step.result == expected
            && step.context.private_state.0 == 91
            && step.private_transcript_outputs.is_empty(),
        "actual setup result/no-witness/private91"
    );
    Ok(step.context)
}
fn fresh(case: Case) -> Result<CircuitContext<Private>> {
    let c = contract::Contract::default();
    let mut cx = contract::initial_state(ConstructorContext::new(Private(91)))?
        .into_circuit_context(ContractAddress::default());
    let initial = state(cx.query.state.get_ref())?;
    let zero = Field::from(0u64);
    let large = wide()?;
    // Genuine original remove on an absent value is a no-op, not a refusal.
    cx = checked(c.remove(cx, zero)?, ())?;
    ensure!(
        state(cx.query.state.get_ref())? == initial,
        "absent original Set remove preserves state"
    );
    if !matches!(case, Case::Empty) {
        cx = checked(c.insert(cx, zero)?, ())?;
    }
    if matches!(case, Case::ZeroAndWide | Case::Reset) {
        cx = checked(c.insert(cx, large)?, ())?;
    }
    if matches!(case, Case::Reset) {
        cx = checked(c.reset_to_default(cx)?, ())?;
        ensure!(
            state(cx.query.state.get_ref())? == initial,
            "actual original reset restores whole initial Set state"
        );
    }
    let (has_zero, has_wide) = match case {
        Case::Empty | Case::Reset => (false, false),
        Case::ZeroPresent => (true, false),
        Case::ZeroAndWide => (true, true),
    };
    let count = u128::from(has_zero) + u128::from(has_wide);
    let before_reads = state(cx.query.state.get_ref())?;
    // Real original public size/member APIs validate the prepared state.
    cx = checked(c.size(cx)?, Cardinality::new(count)?)?;
    cx = checked(c.member(cx, zero)?, has_zero)?;
    cx = checked(c.member(cx, large)?, has_wide)?;
    ensure!(
        state(cx.query.state.get_ref())? == before_reads,
        "setup reads preserve whole Set state"
    );
    let view = contract::PublicStateView::from(&cx);
    let set = view.c()?;
    ensure!(
        set.size()?.value() == count
            && set.is_empty() == case.expected()
            && set.try_member(zero)? == has_zero
            && set.try_member(large)? == has_wide,
        "complete finite Set contents including present zero and wide element"
    );
    Ok(cx)
}
fn addresses(c: &CircuitContext<Private>) -> (ContractAddress, ContractAddress) {
    (
        c.query.address.clone(),
        c.query.call_context.own_address.clone(),
    )
}
fn capture(case: Case) -> Result<(runtime::proof_data::RecordedProofData<Private, bool>, Value)> {
    let n = contract::Contract::default();
    let r = contract::recorded::Contract;
    let h = contract::recorded::Contract;
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
    let recorded = r.is_empty(rx)?;
    ensure!(
        state(recorded.public.initial().state.get_ref())? == before
            && recorded.public.initial().address == addr.0
            && recorded.public.initial().call_context.own_address == addr.1
            && recorded.public.initial().effects == effects,
        "actual recorded frame retains complete prepared public query before-state"
    );
    let data = h.is_empty_proof_data(hx)?; // One generated helper invocation.
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
        "beforeStateEncoding":before,"recordedInitialStateMatchesPreparedState":true,
        "nativeQueryExecutionCompared":true,
        "completeRecordedHelperOpsEqual":true,"officialVerifyReplayPassed":true,
        "supportedContextScope":"wholeState/private91/queryAddress/ownAddress/effects/circuitZswap/events",
        "independentCostModelGasLimitFullCallContextCoinKeyClaim":false,
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
    use midnight_transient_crypto::{fab::ValueReprAlignedValue, proofs::ProofPreimage};
    let bytes = data.serialize_preimage(binding, random)?;
    ensure!(
        bytes.len() > 1 && bytes.len() <= 4 * 1024 * 1024,
        "bounded nonempty canonical tagged preimage"
    );
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
        "schema":"set-is-empty-native-capture/v1","cases":observations,
        "observedCallExecuted":false,"modelExecuted":false,"parametersUsed":false,
        "profile":"set-is-empty","setupOriginalPublicMethods":["remove","insert","reset_to_default","size","member"],
        "queryMethod":"is_empty","caseCount":4,"helperCallsPerCase":1,
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
                "schema":"set-is-empty-native-capture-failure/v1","error":format!("{error:#}"),"success":false})).is_err() {
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
