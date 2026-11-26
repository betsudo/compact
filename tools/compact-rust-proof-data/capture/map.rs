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

//! Actual current no-witness Map API. All calls are supplemental export oracles.
use anyhow::{Result, ensure};
use generated::{ledger_contract as contract, runtime};
use runtime::{
    Field,
    context::{CircuitContext, ConstructorContext},
    fab::AlignedValue,
    ledger::ContractAddress,
};
mod common;
use common::{gas, state};
use midnight_onchain_vm::ops::Op;
use serde_json::{Value, json};

pub struct Private(pub u8); // Deliberately not Clone or Copy.
#[derive(Clone, Copy)]
pub enum Case {
    Present,
    Empty,
    Wide,
}
impl Case {
    pub const ALL: [Self; 3] = [Self::Present, Self::Empty, Self::Wide];
    pub fn id(self) -> &'static str {
        match self {
            Self::Present => "present-one",
            Self::Empty => "empty-one",
            Self::Wide => "wide-present",
        }
    }
    pub fn expected(self) -> bool {
        !matches!(self, Self::Empty)
    }
    pub fn key(self) -> Field {
        if matches!(self, Self::Wide) {
            let mut value = Field::from(1u64);
            for _ in 0..128 {
                value = value + value;
            }
            value + Field::from(1u64)
        } else {
            Field::from(1u64)
        }
    }
}
fn fresh(case: Case) -> Result<CircuitContext<Private>> {
    let c = contract::initial_state(ConstructorContext::new(Private(91)))?
        .into_circuit_context(ContractAddress::default());
    if case.expected() {
        Ok(contract::Contract::default()
            .insert_default(c, case.key())?
            .context)
    } else {
        Ok(c)
    }
}
fn addresses(c: &CircuitContext<Private>) -> (ContractAddress, ContractAddress) {
    (
        c.query.address.clone(),
        c.query.call_context.own_address.clone(),
    )
}
pub fn capture(
    case: Case,
) -> Result<(runtime::proof_data::RecordedProofData<Private, bool>, Value)> {
    let n = contract::Contract::default();
    let r = contract::Contract::default();
    let h = contract::Contract::default();
    let nx = fresh(case)?;
    let rx = fresh(case)?;
    let hx = fresh(case)?;
    let key = case.key();
    let before = state(hx.query.state.get_ref())?;
    ensure!(
        before == state(nx.query.state.get_ref())? && before == state(rx.query.state.get_ref())?,
        "independent fresh contexts equivalent before member"
    );
    let replay_before = hx.query.clone();
    let addr = addresses(&hx);
    let effects = hx.query.effects.clone();
    let intents = hx.circuit_zswap().clone();
    let events = hx.events().to_vec();
    let native = n.member(nx, key)?;
    let recorded = r.recording.member(rx, key)?;
    let captured = h.recording.member_proof_data(hx, key)?; // Exactly one actual helper call.
    ensure!(
        native.result == case.expected()
            && recorded.execution.result == case.expected()
            && captured.execution().result == case.expected(),
        "real native/recorded/helper Boolean result"
    );
    ensure!(
        captured.circuit_name() == "member"
            && captured.input() == &AlignedValue::from(key)
            && captured.output() == &AlignedValue::from(case.expected()),
        "actual typed Field/Boolean envelope"
    );
    ensure!(
        captured.private_transcript_outputs().is_empty()
            && native.private_transcript_outputs.is_empty()
            && recorded.execution.private_transcript_outputs.is_empty(),
        "actual no-witness private outputs empty"
    );
    for cx in [
        &native.context,
        &recorded.execution.context,
        &captured.execution().context,
    ] {
        ensure!(
            cx.private_state.0 == 91 && state(cx.query.state.get_ref())? == before,
            "nonClone private91/read-only whole state"
        );
        ensure!(
            addresses(cx) == addr
                && cx.query.effects == effects
                && cx.circuit_zswap() == &intents
                && cx.events() == events.as_slice(),
            "addresses/effects/intents/events unchanged"
        );
    }
    ensure!(
        captured.public_transcript() == recorded.public.verify_ops(),
        "complete actual ordered public operations"
    );
    ensure!(
        gas(&native.gas_cost) == gas(&recorded.execution.gas_cost)
            && gas(&native.gas_cost) == gas(&captured.execution().gas_cost),
        "three-route gas equality"
    );
    let replay = replay_before.query(
        captured.public_transcript(),
        None,
        &captured.execution().context.cost_model,
    )?;
    ensure!(
        state(replay.context.state.get_ref())? == before && replay.context.effects == effects,
        "official Verify replay retains complete state/effects"
    );
    let ops = captured.public_transcript();
    // A membership read has no write instruction; the actual read result remains
    // in the real captured Popeq. We never synthesize that program or result.
    ensure!(
        !ops.iter()
            .any(|o| matches!(o, Op::Ins { .. } | Op::Rem { .. })),
        "no member writes/removals"
    );
    let observed = json!({"case":case.id(),"supplemental":true,"originalUniqueCaseCredit":0,"referenceOriginalSemanticMemberOne":matches!(case,Case::Present),"privateState":91,"noWitnessOutputs":true,"typedInput":"Field","typedOutput":"Boolean","actualResult":captured.execution().result,"nativeRecordedHelperGasEqual":true,"completeStateUnchanged":true,"contextPreserved":true,"completeRecordedHelperOpsEqual":true,"officialVerifyReplayPassed":true,"verifyOps":ops.len(),"idx":ops.iter().filter(|o|matches!(o,Op::Idx{..})).count(),"cachedIdx":ops.iter().filter(|o|matches!(o,Op::Idx{cached:true,..})).count(),"popeq":ops.iter().filter(|o|matches!(o,Op::Popeq{..})).count(),"routeGas":gas(&captured.execution().gas_cost),"aggregateReplayGas":gas(&replay.gas_cost),"debugTraceSHA256":compact_rust_proof_data::codec::hash(format!("{ops:?}").as_bytes()),"encodingOrWholePreimageParityClaim":false});
    Ok((captured, observed))
}
pub fn missing_lookup_refusal() -> Result<()> {
    let expected =
        runtime::CompactError::LedgerQueryRejected("Execution(ExpectedCell(null))".into());
    let c = contract::Contract::default();
    let n = c
        .lookup(fresh(Case::Empty)?, Field::from(1u64))
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing native lookup accepted"))?;
    let r = c
        .recording
        .lookup(fresh(Case::Empty)?, Field::from(1u64))
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing recorded lookup accepted"))?;
    ensure!(
        n == expected && r == expected,
        "exact existing expected-cell refusal; no insertion/default normalization"
    );
    Ok(())
}

fn run() -> Result<()> {
    let (op, out, binding, random) = common::args()?;
    ensure!(op == "member", "member profile");
    let mut cases = Vec::new();
    for case in Case::ALL {
        let (data, _) = capture(case)?;
        cases.push(common::emit(&out, case.id(), true, &data, binding, random)?);
    }
    missing_lookup_refusal()?;
    common::finish(&out, &cases)
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("PRIVATE_CAPTURE_PANIC")));
    if run().is_err() {
        eprintln!("GENERATED_CAPTURE_FAILED");
        std::process::exit(1);
    }
}
