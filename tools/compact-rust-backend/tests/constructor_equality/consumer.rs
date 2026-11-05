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

use midnight_compact_runtime as r;
fn field(v: u64) -> r::Field {
    r::Field::from(v)
}
fn fields(a: u64, b: u64) -> r::FixedVector<r::Field, 2> {
    r::FixedVector([field(a), field(b)])
}
fn u(v: u128) -> r::BoundedUint<255> {
    r::BoundedUint::new(v).unwrap()
}
#[test]
fn constructors_match_independent_typescript_and_literal_branch_results() {
    let rows: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("COMPACT_CONSTRUCTOR_EQUALITY_TS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 16);
    for row in rows.as_array().unwrap() {
        let role = row["role"].as_str().unwrap();
        let active = row["case"].as_str().unwrap().ends_with("_true");
        let expected = if active { 7 } else { 11 };
        assert_eq!(row["initial"], expected.to_string());
        assert_eq!(row["advanced"], (expected + 4).to_string());
        macro_rules! check {($sdk:ident,$($args:expr),*)=>{{
    use $sdk::{ledger_contract as c,ledger_slots as slots};
    let constructor=|| c::initial_state(r::context::ConstructorContext::new(42u64),$($args),*).unwrap();
    let initial=constructor();
    assert_eq!(slots::stored.inspect(&initial.ledger_state).unwrap(),field(expected));
    let context=||constructor().into_circuit_context(Default::default());
    let native=c::advance(context(),field(4)).unwrap();
    let recorded=c::recorded::advance(context(),field(4)).unwrap();
    let functional=c::functional::advance::Program::new(field(4)).execute_native(context()).unwrap();
    let functional_recorded=c::functional::advance::Program::new(field(4)).execute_recorded(context()).unwrap();
    for actual in [&native,&recorded.execution,&functional,&functional_recorded.execution]{
      assert_eq!(actual.result,field(expected+4));
      assert_eq!(actual.context.query.state,native.context.query.state);
      assert_eq!(actual.context.query.effects,native.context.query.effects);
      assert_eq!(actual.context.private_state,42);
    }
    assert_eq!(recorded.public.verify_ops(),functional_recorded.public.verify_ops());
  }}}
        match role {
            "boolean_parameter" => check!(boolean_parameter, fields(1, 2), active),
            "field_literal_eq" => check!(
                field_literal_eq,
                if active { fields(1, 2) } else { fields(2, 3) }
            ),
            "field_pair_eq" => check!(
                field_pair_eq,
                if active { fields(1, 2) } else { fields(2, 3) },
                field(2)
            ),
            "field_pair_ne" => check!(
                field_pair_ne,
                if active { fields(1, 3) } else { fields(3, 2) },
                field(2)
            ),
            "bool_pair_eq" => check!(
                bool_pair_eq,
                if active {
                    r::FixedVector([false, true])
                } else {
                    r::FixedVector([true, false])
                },
                true
            ),
            "uint_pair_eq" => check!(
                uint_pair_eq,
                if active {
                    r::FixedVector([u(1), u(2)])
                } else {
                    r::FixedVector([u(2), u(3)])
                },
                u(2)
            ),
            "bytes_pair_eq" => {
                let right = r::FixedBytes([3, 4]);
                let values = if active {
                    r::FixedVector([r::FixedBytes([1, 2]), right])
                } else {
                    r::FixedVector([right, r::FixedBytes([1, 2])])
                };
                check!(bytes_pair_eq, values.clone(), right);
                assert_eq!(right.0, [3, 4]);
            }
            "nested_eq" => check!(
                nested_eq,
                if active { fields(1, 2) } else { fields(2, 3) },
                field(2),
                field(3),
                field(3)
            ),
            other => panic!("unknown case {other}"),
        }
    }
}
