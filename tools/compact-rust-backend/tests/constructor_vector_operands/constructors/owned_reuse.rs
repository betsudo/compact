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

fn initialize(_case: &Value, _log: &Log) -> Result<ConstructorResult<u64>, r::CompactError> {
    // OpaqueString is Clone and is not Copy. The constructor reuses the input
    // after iterating a helper temporary, and uses both complete output cells.
    c::initial_state(
        ConstructorContext::new(5),
        r::FixedVector([r::OpaqueString::from("first"), r::OpaqueString::from("β🚀")]),
    )
}
fn check_extra(state: &ConstructorResult<u64>) {
    let expected = r::OpaqueString::from("β🚀");
    assert_eq!(
        generated::ledger_slots::temporary
            .inspect(state.ledger_state.get_ref())
            .unwrap(),
        expected
    );
    assert_eq!(
        generated::ledger_slots::reused
            .inspect(state.ledger_state.get_ref())
            .unwrap(),
        expected
    );
}
