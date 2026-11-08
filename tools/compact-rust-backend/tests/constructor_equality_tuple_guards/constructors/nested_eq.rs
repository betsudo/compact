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

fn small(v: &Value) -> r::BoundedUint<255> {
    r::BoundedUint::new(v.as_u64().unwrap().into()).unwrap()
}
fn large(v: &Value) -> r::BoundedUint<65535> {
    r::BoundedUint::new(v.as_u64().unwrap().into()).unwrap()
}
fn small_vector(v: &Value) -> r::FixedVector<r::BoundedUint<255>, 2> {
    r::FixedVector([small(&v[0]), small(&v[1])])
}
fn initialize(case: &Value, _: &Log) -> Result<ConstructorResult<u64>, Box<dyn Error>> {
    let a = case["args"].as_array().ok_or("constructor args required")?;
    Ok(c::initial_state(
        ConstructorContext::new(42u64),
        small_vector(&a[0]),
        (large(&a[1][0]), (large(&a[1][1][0]), large(&a[1][1][1]))),
    )?)
}
fn check_extra(initial: &ConstructorResult<u64>) {
    let r::ledger::StateValue::Array(fields) = initial.ledger_state.get_ref() else {
        panic!("one Cell root")
    };
    assert_eq!(fields.len(), 1);
}
