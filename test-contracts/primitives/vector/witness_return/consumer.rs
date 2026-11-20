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

use fixture::{ledger_contract as c, runtime as r};
use r::context::{CircuitContext, ConstructorContext, WitnessContext};
use r::{Field, FixedVector};
use std::cell::Cell;

struct Witness {
    wrong_last: bool,
    calls: Cell<usize>,
}

impl c::Witnesses<u64> for Witness {
    fn vector_witness(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> (u64, FixedVector<Field, 4>) {
        self.calls.set(self.calls.get() + 1);
        (
            *context.private_state,
            FixedVector::new([
                Field::from(5_u64),
                Field::from(6_u64),
                Field::from(7_u64),
                Field::from(if self.wrong_last { 9_u64 } else { 8_u64 }),
            ]),
        )
    }
}

fn context() -> CircuitContext<u64> {
    c::initial_state(ConstructorContext::new(3_u64))
        .unwrap()
        .into_circuit_context(Default::default())
}

#[test]
fn original_witness_slice_and_indices_are_preserved() {
    let witness = Witness {
        wrong_last: false,
        calls: Cell::new(0),
    };
    let result = c::vector_witness_return_semantics(context(), &witness).unwrap();
    assert_eq!(result.result, ());
    assert_eq!(result.context.private_state, 3);
    assert_eq!(witness.calls.get(), 1);
    assert_eq!(result.private_transcript_outputs.len(), 1);
}

#[test]
fn wrong_witness_element_is_refused() {
    let witness = Witness {
        wrong_last: true,
        calls: Cell::new(0),
    };
    let result = c::vector_witness_return_semantics(context(), &witness);
    let error = match result {
        Ok(_) => panic!("wrong witness element was accepted"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("Witness element 3 should be 8"));
    assert_eq!(witness.calls.get(), 1);
}
