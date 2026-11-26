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

//! Real generated Tiny clear/set; constructor, witness and errors are unchanged.
use anyhow::{Result, ensure};
use generated::{ledger_contract as contract, runtime, types};
use midnight_onchain_vm::ops::Op;
use runtime::{
    CompactError, Field, FixedBytes,
    context::{CircuitContext, ConstructorContext, WitnessContext},
    fab::AlignedValue,
    ledger::{ContractAddress, DefaultDB, StateValue},
};
use std::{cell::Cell, rc::Rc};
mod common;
const _: [(); 54] = [(); runtime::RUST_RUNTIME_ABI as usize];
const _: [(); 1] = [(); runtime::proof_data::API_VERSION as usize];
struct Private(u64); // Deliberately not Clone or Copy.
#[derive(Default)]
struct Audit {
    calls: Cell<usize>,
    secret: Cell<[u8; 32]>,
    deny: Cell<bool>,
}
#[derive(Clone)]
struct Witness(Rc<Audit>);
impl contract::Witnesses<Private> for Witness {
    fn private_secret_key(
        &self,
        cx: WitnessContext<'_, Private, contract::LedgerView<'_>>,
    ) -> (Private, FixedBytes<32>) {
        assert!(!self.0.deny.get());
        assert_eq!(cx.private_state.0, 0);
        self.0.calls.set(self.0.calls.get() + 1);
        (
            Private(cx.private_state.0),
            FixedBytes::new(self.0.secret.get()),
        )
    }
}
fn fresh(
    secret: [u8; 32],
    set: bool,
) -> Result<(
    contract::Contract<Witness>,
    CircuitContext<Private>,
    Rc<Audit>,
)> {
    let a = Rc::new(Audit::default());
    a.secret.set(secret);
    let w = Witness(a.clone());
    let c = contract::Contract::from(w.clone());
    let mut cx =
        contract::initial_state(ConstructorContext::new(Private(0)), &w, Field::from(64u64))?
            .into_circuit_context(ContractAddress::default());
    ensure!(a.calls.get() == 1, "constructor witness once");
    if set {
        cx = c.clear(cx)?.context;
        ensure!(a.calls.get() == 2, "proper clear setup witness");
    }
    Ok((c, cx, a))
}
fn cells(secret: [u8; 32], set: bool) -> Result<Vec<StateValue<DefaultDB>>> {
    Ok(vec![
        runtime::ledger::try_constructor_cell::<_, DefaultDB>(if set {
            generated::pure_circuits::public_key(FixedBytes::new(secret))?
        } else {
            FixedBytes::new([0; 32])
        })?,
        runtime::ledger::try_constructor_cell::<_, DefaultDB>(Field::from(if set {
            5u64
        } else {
            0u64
        }))?,
        runtime::ledger::try_constructor_cell::<_, DefaultDB>(if set {
            types::STATE::set
        } else {
            types::STATE::unset
        })?,
    ])
}
fn writes(
    ops: &[Op<midnight_onchain_vm::result_mode::ResultModeVerify>],
    expected: &[StateValue<DefaultDB>],
) -> Result<()> {
    ensure!(
        !ops.iter().any(|op| matches!(
            op,
            Op::Idx {
                push_path: true,
                ..
            }
        )),
        "no nested-path Tiny writes"
    );
    let mut count = 0;
    for (i, op) in ops.iter().enumerate() {
        if matches!(op, Op::Ins { .. }) {
            ensure!(i >= 2 && count < 3, "three root writes");
            match (&ops[i - 2], &ops[i - 1], op) {
                (
                    Op::Push {
                        storage: false,
                        value: key,
                    },
                    Op::Push {
                        storage: true,
                        value: cell,
                    },
                    Op::Ins {
                        cached: false,
                        n: 1,
                    },
                ) => {
                    let key: StateValue<DefaultDB> = key.clone().into();
                    let cell: StateValue<DefaultDB> = cell.clone().into();
                    ensure!(
                        runtime::ledger::read_cell::<u8, DefaultDB>(&key)? == count as u8
                            && cell == expected[count],
                        "exact authority/value/state write order/values"
                    );
                }
                _ => anyhow::bail!("unexpected root write grammar"),
            };
            count += 1;
        }
    }
    ensure!(count == 3, "exact three actual writes");
    Ok(())
}
fn capture(
    secret: [u8; 32],
    set: bool,
) -> Result<runtime::proof_data::RecordedProofData<Private, ()>> {
    let (n, nx, na) = fresh(secret, set)?;
    let (r, rx, ra) = fresh(secret, set)?;
    let (h, hx, ha) = fresh(secret, set)?;
    let before = hx.query.clone();
    let address = (
        hx.query.address.clone(),
        hx.query.call_context.own_address.clone(),
    );
    let intents = hx.circuit_zswap().clone();
    let events = hx.events().to_vec();
    let counts = [na.calls.get(), ra.calls.get(), ha.calls.get()];
    let native = if set {
        n.set(nx, Field::from(5u64))?
    } else {
        n.clear(nx)?
    };
    let recorded = if set {
        r.recording().set(rx, Field::from(5u64))?
    } else {
        r.recording().clear(rx)?
    };
    let data = if set {
        h.recording().set_proof_data(hx, Field::from(5u64))?
    } else {
        h.recording().clear_proof_data(hx)?
    };
    ensure!(
        na.calls.get() == counts[0] + 1
            && ra.calls.get() == counts[1] + 1
            && ha.calls.get() == counts[2] + 1,
        "one real witness per actual route"
    );
    let expected_cells = cells(secret, set)?;
    let expected = runtime::ledger::contract_state(expected_cells.clone());
    for cx in [
        &native.context,
        &recorded.execution.context,
        &data.execution().context,
    ] {
        ensure!(
            cx.private_state.0 == 0
                && common::state(cx.query.state.get_ref())? == common::state(expected.get_ref())?,
            "nonClone/private and exact final state"
        );
        ensure!(
            (
                cx.query.address.clone(),
                cx.query.call_context.own_address.clone()
            ) == address
                && cx.circuit_zswap() == &intents
                && cx.events() == events.as_slice(),
            "context addresses/intents/events retained"
        );
    }
    ensure!(
        native.context.query.effects == recorded.execution.context.query.effects
            && native.context.query.effects == data.execution().context.query.effects,
        "three-route effects equality"
    );
    ensure!(
        data.public_transcript() == recorded.public.verify_ops()
            && common::gas(&native.gas_cost) == common::gas(&recorded.execution.gas_cost)
            && common::gas(&native.gas_cost) == common::gas(&data.execution().gas_cost),
        "actual operations and route gas equality"
    );
    ensure!(
        data.input()
            == &if set {
                AlignedValue::from(Field::from(5u64))
            } else {
                AlignedValue::from(())
            }
            && data.output() == &AlignedValue::from(()),
        "typed IO"
    );
    ensure!(
        data.private_transcript_outputs() == [AlignedValue::from(FixedBytes::new(secret))],
        "actual Bytes32 witness output"
    );
    writes(data.public_transcript(), &expected_cells)?;
    let replay = before.query(
        data.public_transcript(),
        None,
        &data.execution().context.cost_model,
    )?;
    ensure!(
        common::state(replay.context.state.get_ref())? == common::state(expected.get_ref())?
            && replay.context.effects == data.execution().context.query.effects,
        "official Verify state/effects replay"
    );
    Ok(data)
}
fn refusal<T>(
    r: std::result::Result<T, runtime::proof_data::ProofDataError>,
    s: &str,
) -> Result<()> {
    match r {
        Err(runtime::proof_data::ProofDataError::Execution(CompactError::AssertionFailed(got))) => {
            ensure!(got == s, "exact original refusal")
        }
        _ => anyhow::bail!("wrong refusal result/layer"),
    };
    Ok(())
}
fn refusals(set: bool) -> Result<()> {
    if set {
        let (c, cx, a) = fresh([0; 32], true)?;
        let cx = c.set(cx, Field::from(5u64))?.context;
        a.deny.set(true);
        let before = a.calls.get();
        refusal(
            c.recording().set_proof_data(cx, Field::from(7u64)),
            "set: attempted to overwrite recorded value",
        )?;
        ensure!(a.calls.get() == before, "overwrite before witness");
        let (c, cx, _) = fresh([0; 32], true)?;
        let big = Field::from(1_000_000_000_000u64) * Field::from(1_000_000_000_000u64);
        let result = c.get(c.set(cx, big)?.context)?;
        ensure!(
            result.result.is_some && result.result.value == big,
            "original large Field result"
        );
    } else {
        let (c, cx, a) = fresh([0; 32], true)?;
        a.deny.set(true);
        let before = a.calls.get();
        refusal(
            c.recording().clear_proof_data(cx),
            "clear: no value is currently recorded",
        )?;
        ensure!(a.calls.get() == before, "state refusal before witness");
        let (c, cx, a) = fresh([0; 32], false)?;
        let before = a.calls.get();
        let mut wrong = [0; 32];
        wrong[0] = 1;
        a.secret.set(wrong);
        refusal(
            c.recording().clear_proof_data(cx),
            "clear: attempted clear without proper authorization",
        )?;
        ensure!(a.calls.get() == before + 1, "authority refusal one witness");
    }
    Ok(())
}
fn run() -> Result<()> {
    let (op, out, binding, random) = common::args()?;
    ensure!(op == "clear" || op == "set", "Tiny operation");
    let set = op == "set";
    let mut cases = Vec::new();
    for (nonzero, secret) in [(false, [0; 32]), (true, std::array::from_fn(|i| i as u8))] {
        let data = capture(secret, set)?;
        cases.push(common::emit(
            &out,
            if nonzero {
                "supplemental-nonzero"
            } else {
                "original-zero"
            },
            nonzero,
            &data,
            binding,
            random,
        )?);
    }
    refusals(set)?;
    common::finish(&out, &cases)
}
fn main() {
    std::panic::set_hook(Box::new(|_| eprintln!("PRIVATE_CAPTURE_PANIC")));
    if run().is_err() {
        eprintln!("GENERATED_CAPTURE_FAILED");
        std::process::exit(1);
    }
}
