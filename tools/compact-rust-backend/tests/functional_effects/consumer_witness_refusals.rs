// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn unknown_aligned_request<
    E: runtime::functional::Executor + runtime::functional::WitnessCall<bool, bool>,
>(
    executor: E,
    later: &std::cell::Cell<usize>,
) -> Result<E::Outcome<()>, CompactError> {
    runtime::functional::Circuit::begin(executor)
        .witness(
            runtime::functional::Witness::<bool, bool>::new(runtime::functional::WitnessId::new(
                u32::MAX,
            )),
            true,
        )
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                later.set(later.get() + 1);
                Ok(())
            })
        })
        .finish()
}
fn unknown_exact_request<
    E: runtime::functional::Executor
        + runtime::functional::WitnessCall<
            runtime::OpaqueString,
            runtime::OpaqueString,
            runtime::functional::ledger8::ExactWitnessOutput,
        >,
>(
    executor: E,
    later: &std::cell::Cell<usize>,
) -> Result<E::Outcome<()>, CompactError> {
    runtime::functional::Circuit::begin(executor)
        .witness_in(
            runtime::functional::Witness::<
                runtime::OpaqueString,
                runtime::OpaqueString,
                runtime::functional::ledger8::ExactWitnessOutput,
            >::with_mode(runtime::functional::WitnessId::new(u32::MAX)),
            runtime::OpaqueString::from("exact\0"),
        )
        .bind(|_, circuit| {
            circuit.try_returning(|| {
                later.set(later.get() + 1);
                Ok(())
            })
        })
        .finish()
}
#[test]
fn generated_aligned_adapter_refuses_unknown_identity_without_callbacks() {
    use assert_fixture::ledger_contract as c;
    use c::functional::checked_value as program;
    let context = || {
        c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let witnesses: [Echo; 2] = std::array::from_fn(|_| Echo::default());
    let later = std::cell::Cell::new(0);
    let native = NativeLedger8::new(context(), program::ledger8_bindings().unwrap())
        .with_witnesses(program::WitnessAdapter(&witnesses[0]));
    let recorded = RecordedLedger8::new(context(), program::ledger8_bindings().unwrap())
        .with_witnesses(program::WitnessAdapter(&witnesses[1]));
    let first = unknown_aligned_request(native, &later)
        .err()
        .expect("unknown ID must refuse");
    let second = unknown_aligned_request(recorded, &later)
        .err()
        .expect("unknown ID must refuse");
    assert_eq!(first, CompactError::UnknownWitnessIdentity { id: u32::MAX });
    assert_eq!(first, second);
    assert_eq!(
        first.to_string(),
        "unknown checked witness identity 4294967295"
    );
    assert_eq!(later.get(), 0);
    for witness in &witnesses {
        assert!(witness.0.borrow().is_empty());
    }
}
#[test]
fn generated_exact_adapter_refuses_unknown_identity_without_callbacks() {
    use c::functional::exact as program;
    use exact_fixture::ledger_contract as c;
    let context = || {
        c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let witnesses: [ExactWitnesses; 2] = std::array::from_fn(|_| ExactWitnesses::default());
    let later = std::cell::Cell::new(0);
    let native = NativeLedger8::new(context(), program::ledger8_bindings().unwrap())
        .with_witnesses(program::WitnessAdapter(&witnesses[0]));
    let recorded = RecordedLedger8::new(context(), program::ledger8_bindings().unwrap())
        .with_witnesses(program::WitnessAdapter(&witnesses[1]));
    let first = unknown_exact_request(native, &later)
        .err()
        .expect("unknown ID must refuse");
    let second = unknown_exact_request(recorded, &later)
        .err()
        .expect("unknown ID must refuse");
    assert_eq!(first, CompactError::UnknownWitnessIdentity { id: u32::MAX });
    assert_eq!(first, second);
    assert_eq!(later.get(), 0);
    for witness in &witnesses {
        assert!(witness.0.borrow().is_empty());
        assert!(witness.1.borrow().is_empty());
        assert!(witness.2.borrow().is_empty());
    }
}
struct CustomRefusal;
impl<Args, T> runtime::functional::ledger8::WitnessHandler<u64, runtime::ledger::DefaultDB, Args, T>
    for CustomRefusal
{
    fn call(
        &self,
        _id: runtime::functional::WitnessId,
        _arguments: Args,
        _context: &runtime::context::CircuitContext<u64>,
        _meter: &runtime::context::WitnessReadMeter<'_>,
    ) -> Result<(u64, T), CompactError> {
        Err(CompactError::LedgerQueryRejected(
            "handwritten host refusal".into(),
        ))
    }
}
#[test]
fn handwritten_witness_errors_pass_through_both_existing_frame_transports() {
    use exact_fixture::ledger_contract as c;
    let context = || {
        c::initial_state(ConstructorContext::new(7_u64))
            .unwrap()
            .into_circuit_context(ContractAddress::default())
    };
    let bindings = runtime::functional::ledger8::Ledger8Bindings::new(&[]).unwrap();
    let later = std::cell::Cell::new(0);
    let a = unknown_aligned_request(
        NativeLedger8::new(context(), bindings).with_witnesses(CustomRefusal),
        &later,
    )
    .err()
    .unwrap();
    let b = unknown_aligned_request(
        RecordedLedger8::new(context(), bindings).with_witnesses(CustomRefusal),
        &later,
    )
    .err()
    .unwrap();
    let d = unknown_exact_request(
        NativeLedger8::new(context(), bindings).with_witnesses(CustomRefusal),
        &later,
    )
    .err()
    .unwrap();
    let e = unknown_exact_request(
        RecordedLedger8::new(context(), bindings).with_witnesses(CustomRefusal),
        &later,
    )
    .err()
    .unwrap();
    for error in [a, b, d, e] {
        assert_eq!(
            error,
            CompactError::LedgerQueryRejected("handwritten host refusal".into())
        );
    }
    assert_eq!(later.get(), 0);
}
