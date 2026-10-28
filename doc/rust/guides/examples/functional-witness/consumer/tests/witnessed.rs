// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use contract::{ledger_contract as api, runtime as rt};
use midnight_compact_testkit::{ArtifactIdentity, CallReport, ContractLab, Environment, LabError};
use rt::{
    CompactError, Field,
    context::{ConstructorContext, WitnessContext},
    fab::AlignedValue,
};
use std::cell::RefCell;

#[derive(Default)]
struct Answers(RefCell<Vec<(u64, Field, Field)>>);
impl api::TryWitnesses<u64> for Answers {
    fn private_offset(
        &self,
        context: WitnessContext<'_, u64, api::LedgerView<'_>>,
        value: Field,
    ) -> Result<(u64, Field), CompactError> {
        let previous = context.ledger.cell()?;
        self.0
            .borrow_mut()
            .push((*context.private_state, value, previous));
        if value == Field::from(99_u64) {
            return Err(CompactError::AssertionFailed(
                "fixture witness denied".into(),
            ));
        }
        Ok((*context.private_state + 1, value))
    }
}
#[derive(Clone, Copy, Debug)]
enum Route {
    OrdinaryNative,
    OrdinaryRecorded,
    FunctionalNative,
    FunctionalRecorded,
}
const ROUTES: [Route; 4] = [
    Route::OrdinaryNative,
    Route::OrdinaryRecorded,
    Route::FunctionalNative,
    Route::FunctionalRecorded,
];
fn identity() -> ArtifactIdentity {
    use sha2::{Digest, Sha256};
    // These hashes identify local source bytes, not deployment or chain provenance.
    ArtifactIdentity {
        source_sha256: Sha256::digest(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source/field_witness_proof.compact"
        )))
        .into(),
        generated_sha256: Sha256::digest(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../generated/contract/lib.rs"
        )))
        .into(),
    }
}
fn lab() -> ContractLab<u64> {
    ContractLab::from_constructor(
        identity(),
        Environment::new(Default::default(), Default::default(), [42; 32]),
        api::initial_state(ConstructorContext::new(7_u64)).unwrap(),
    )
    .unwrap()
}
fn invoke(
    lab: &mut ContractLab<u64>,
    answers: &Answers,
    value: Field,
    route: Route,
) -> Result<CallReport<Field>, LabError> {
    match route {
        Route::OrdinaryNative => lab.native(|context| api::apply_offset(context, answers, value)),
        Route::OrdinaryRecorded => {
            lab.recorded(|context| api::recorded::apply_offset(context, answers, value))
        }
        Route::FunctionalNative => lab.native(|context| {
            api::functional::apply_offset::Program::new(value).execute_native(context, answers)
        }),
        Route::FunctionalRecorded => lab.recorded(|context| {
            api::functional::apply_offset::Program::new(value).execute_recorded(context, answers)
        }),
    }
}
fn cell(lab: &ContractLab<u64>) -> Field {
    contract::ledger_slots::cell
        .inspect(lab.snapshot().public_state().get_ref())
        .unwrap()
}
#[test]
fn all_four_routes_preserve_typed_witness_state_and_public_replay() {
    let mut reference: Option<Vec<CallReport<Field>>> = None;
    let mut recorded_reference: Option<Vec<CallReport<Field>>> = None;
    for route in ROUTES {
        let mut lab = lab();
        let answers = Answers::default();
        let mut reports = Vec::new();
        for (i, value) in [42_u64, 17].into_iter().enumerate() {
            let value = Field::from(value);
            let report = invoke(&mut lab, &answers, value, route).unwrap();
            assert_eq!(*report.output(), value);
            assert_eq!(report.private_outputs(), &[AlignedValue::from(value)]);
            assert_eq!(cell(&lab), value);
            assert_eq!(*lab.private_state(), 8 + i as u64);
            assert_eq!(
                report.replay().is_some(),
                matches!(route, Route::OrdinaryRecorded | Route::FunctionalRecorded)
            );
            reports.push(report);
        }
        assert_eq!(
            *answers.0.borrow(),
            [
                (7, Field::from(42_u64), Field::from(0_u64)),
                (8, Field::from(17_u64), Field::from(42_u64))
            ]
        );
        if let Some(reference) = &reference {
            for (actual, expected) in reports.iter().zip(reference) {
                assert_eq!(actual.output(), expected.output());
                assert_eq!(actual.public_state(), expected.public_state());
                assert_eq!(actual.effects(), expected.effects());
                assert_eq!(actual.private_outputs(), expected.private_outputs());
                assert_eq!(actual.execution_gas(), expected.execution_gas());
            }
        }
        if matches!(route, Route::FunctionalRecorded) {
            for (actual, expected) in reports.iter().zip(recorded_reference.as_ref().unwrap()) {
                assert_eq!(
                    actual.replay().unwrap().program(),
                    expected.replay().unwrap().program()
                );
                assert_eq!(
                    actual.replay().unwrap().gas(),
                    expected.replay().unwrap().gas()
                );
            }
        }
        if matches!(route, Route::OrdinaryRecorded) {
            recorded_reference = Some(reports);
        } else if matches!(route, Route::OrdinaryNative) {
            reference = Some(reports);
        }
    }
}
#[test]
fn witness_failure_preserves_committed_checkpoint_and_keeps_external_effects() {
    for route in ROUTES {
        let mut lab = lab();
        let answers = Answers::default();
        invoke(&mut lab, &answers, Field::from(42_u64), route).unwrap();
        let before = lab.snapshot();
        let error = invoke(&mut lab, &answers, Field::from(99_u64), route).unwrap_err();
        assert!(
            matches!(error, LabError::Execution(CompactError::AssertionFailed(message)) if message == "fixture witness denied")
        );
        assert_eq!(lab.snapshot().public_state(), before.public_state());
        assert_eq!(lab.private_state(), before.private_state());
        assert_eq!(
            *answers.0.borrow(),
            [
                (7, Field::from(42_u64), Field::from(0_u64)),
                (8, Field::from(99_u64), Field::from(42_u64))
            ]
        );
    }
}
#[test]
fn snapshots_restore_owned_state_and_reject_changed_identity() {
    let mut lab = lab();
    let answers = Answers::default();
    invoke(
        &mut lab,
        &answers,
        Field::from(42_u64),
        Route::FunctionalRecorded,
    )
    .unwrap();
    let snapshot = lab.snapshot();
    invoke(
        &mut lab,
        &answers,
        Field::from(17_u64),
        Route::FunctionalRecorded,
    )
    .unwrap();
    let mut changed = snapshot.clone();
    changed.metadata.runtime_abi += 1;
    assert!(matches!(
        lab.restore(&changed),
        Err(LabError::SnapshotMismatch("runtime ABI"))
    ));
    assert_eq!(cell(&lab), Field::from(17_u64));
    assert_eq!(*lab.private_state(), 9);
    lab.restore(&snapshot).unwrap();
    assert_eq!(cell(&lab), Field::from(42_u64));
    assert_eq!(*lab.private_state(), 8);
    invoke(
        &mut lab,
        &answers,
        Field::from(5_u64),
        Route::FunctionalRecorded,
    )
    .unwrap();
    assert_eq!(cell(&lab), Field::from(5_u64));
    assert_eq!(*lab.private_state(), 9);
    assert_eq!(answers.0.borrow().len(), 3);
}
