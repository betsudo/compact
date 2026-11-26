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

//! Public Lookup adapter for the intact generated eight-export Map library.
//! No model, parameter, key, proof, witness, transaction or RNG call occurs here.
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
    Nonzero,
    DefaultZero,
    Wide,
}
impl Case {
    const ALL: [Self; 3] = [Self::Nonzero, Self::DefaultZero, Self::Wide];
    fn id(self) -> &'static str {
        match self {
            Self::Nonzero => "one-to-seven",
            Self::DefaultZero => "default-one-zero",
            Self::Wide => "wide-to-wide",
        }
    }
    fn key(self) -> Field {
        match self {
            Self::Wide => wide(128, 1),
            _ => Field::from(1u64),
        }
    }
    fn value(self) -> Field {
        match self {
            Self::Nonzero => Field::from(7u64),
            Self::DefaultZero => Field::from(0u64),
            Self::Wide => wide(129, 7),
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
fn empty() -> Result<CircuitContext<Private>> {
    Ok(
        contract::initial_state(ConstructorContext::new(Private(91)))?
            .into_circuit_context(ContractAddress::default()),
    )
}
fn fresh(case: Case) -> Result<CircuitContext<Private>> {
    let c = contract::Contract::default();
    let cx = empty()?;
    let inserted = match case {
        Case::DefaultZero => c.insert_default(cx, case.key())?,
        _ => c.insert(cx, case.key(), case.value())?,
    };
    ensure!(
        inserted.context.private_state.0 == 91 && inserted.private_transcript_outputs.is_empty(),
        "actual no-witness insertion"
    );
    let view = contract::PublicStateView::from(&inserted.context);
    ensure!(
        view.c()?.try_member(case.key())? && view.c()?.lookup(case.key())? == case.value(),
        "actual inserted Map Field value including default zero"
    );
    Ok(inserted.context)
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
    runtime::proof_data::RecordedProofData<Private, Field>,
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
        "independent fresh contexts equivalent before lookup"
    );
    let replay_before = hx.query.clone();
    let addr = addresses(&hx);
    let effects = hx.query.effects.clone();
    let intents = hx.circuit_zswap().clone();
    let events = hx.events().to_vec();
    let native = n.lookup(nx, case.key())?;
    let recorded = r.recording.lookup(rx, case.key())?;
    let data = h.recording.lookup_proof_data(hx, case.key())?; // One generated helper invocation.
    ensure!(
        native.result == case.value()
            && recorded.execution.result == case.value()
            && data.execution().result == case.value(),
        "actual native/recorded/helper Field result"
    );
    ensure!(
        data.circuit_name() == "lookup"
            && data.input() == &AlignedValue::from(case.key())
            && data.output() == &AlignedValue::from(case.value()),
        "actual typed Field input and Field output envelope"
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
        "lookup has no writes/removals"
    );
    let observed = json!({"case":case.id(),"supplemental":true,"originalUniqueCaseCredit":0,"typedInput":"Field","typedOutput":"Field","input":compact_rust_proof_data::input::hex(&case.key()),"output":compact_rust_proof_data::input::hex(&case.value()),"nonClonePrivate91":true,"noWitnessOutputs":true,"nativeRecordedHelperGasEqual":true,"completeStateUnchanged":true,"contextPreserved":true,"completeRecordedHelperOpsEqual":true,"officialVerifyReplayPassed":true,"verifyOps":ops.len(),"publicTranscript":public_transcript,"idx":ops.iter().filter(|o|matches!(o,Op::Idx{..})).count(),"popeq":ops.iter().filter(|o|matches!(o,Op::Popeq{..})).count(),"routeGas":gas(&data.execution().gas_cost),"aggregateReplayGas":gas(&replay.gas_cost),"debugTraceSHA256":compact_rust_proof_data::codec::hash(format!("{ops:?}").as_bytes()),"observedCallExecuted":false});
    Ok((data, observed))
}
fn missing_lookup_refusal() -> Result<Value> {
    let c = contract::Contract::default();
    let expected =
        runtime::CompactError::LedgerQueryRejected("Execution(ExpectedCell(null))".into());
    let key = Field::from(1u64);
    let n = c
        .lookup(empty()?, key)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing native lookup accepted"))?;
    let r = c
        .recording
        .lookup(empty()?, key)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing recorded lookup accepted"))?;
    let h = c
        .recording
        .lookup_proof_data(empty()?, key)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing helper produced a carrier"))?;
    ensure!(
        n == expected && r == expected,
        "exact native/recorded expected-cell refusal"
    );
    ensure!(
        matches!(h, runtime::proof_data::ProofDataError::Execution(ref e) if e == &expected),
        "exact helper Execution(expected-cell) refusal with no carrier"
    );
    Ok(
        json!({"native":true,"recorded":true,"helper":true,"exactError":"Execution(ExpectedCell(null))","successfulCarrier":false,"observedCallExecuted":false}),
    )
}
// Immutable data conversion only; the generated recorded helper is NOT invoked again.
fn emit_lookup(
    out: &std::path::Path,
    id: &str,
    data: &runtime::proof_data::RecordedProofData<Private, Field>,
    binding: Field,
    random: Field,
) -> Result<compact_rust_proof_data::input::Case> {
    use compact_rust_proof_data::codec;
    use midnight_transient_crypto::{
        fab::ValueReprAlignedValue, proofs::ProofPreimage, repr::FieldRepr,
    };
    let bytes = data.serialize_preimage(binding, random)?;
    let decoded: ProofPreimage = codec::decode(&bytes)?;
    ensure!(
        decoded.private_transcript.is_empty()
            && decoded.inputs == ValueReprAlignedValue(data.input().clone()).field_vec(),
        "actual typed declared Field input and no private outputs"
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
    ensure!(op == "lookup", "lookup capture only");
    let mut cases = Vec::new();
    let mut observations = Vec::new();
    for case in Case::ALL {
        let (data, observation) = capture(case)?;
        cases.push(emit_lookup(out, case.id(), &data, binding, random)?);
        observations.push(observation);
    }
    let refusal = missing_lookup_refusal()?;
    compact_rust_proof_data::codec::write_json(
        out,
        "observed.json",
        &json!({"schema":"map-lookup-native-capture/v1","cases":observations,"missingLookup":refusal,"modelExecuted":false,"parametersUsed":false,"keysOrProofsProduced":false,"originalTenAssertionFixtureExecuted":false}),
    )?;
    common::finish(out, &cases)
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("PRIVATE_CAPTURE_PANIC")));
    let result = common::args().and_then(|(op, out, binding, random)| {
        let first = run(&op, &out, binding, random);
        if let Err(ref error) = first {
            if compact_rust_proof_data::codec::write_json(&out, "failure.json", &json!({"schema":"map-lookup-native-capture-failure/v1","error":format!("{error:#}"),"success":false})).is_err() {
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
