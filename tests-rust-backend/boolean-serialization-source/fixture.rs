// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

include!(concat!(env!("OUT_DIR"), "/contract.rs"));

#[cfg(test)]
mod tests {
    use super::{ledger_contract, runtime};
    use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
    use runtime::context::ConstructorContext;
    use runtime::ledger::ContractAddress;
    use runtime::{CompactError, Field, FixedBytes};

    #[test]
    fn boolean_decoder_checks_every_byte_in_native_and_recorded_rust() {
        let context = || {
            ledger_contract::initial_state(ConstructorContext::new(()))
                .unwrap()
                .into_circuit_context(ContractAddress::default())
        };
        for byte in 0..=255 {
            let input = FixedBytes::new([byte]);
            let native = ledger_contract::d1(context(), input);
            let recorded = ledger_contract::recorded::d1(context(), input);
            if byte > 1 {
                let expected = u128::from(byte);
                assert!(
                    matches!(native, Err(CompactError::UnsignedOutOfRange { value, max: 1 }) if value == expected),
                    "native did not reject Boolean byte {byte} as out of range",
                );
                assert!(
                    matches!(recorded, Err(CompactError::UnsignedOutOfRange { value, max: 1 }) if value == expected),
                    "recording did not reject Boolean byte {byte} as out of range",
                );
                continue;
            }
            let native = native.unwrap();
            let recorded = recorded.unwrap();
            assert_eq!(native.result, byte == 1);
            assert_eq!(recorded.execution.result, native.result);
            assert_eq!(recorded.execution.gas_cost, native.gas_cost);
            assert_eq!(
                ledger_contract::PublicStateView::from(&native)
                    .forceField()
                    .unwrap(),
                Field::from(7u128),
            );
            let replay = recorded
                .public
                .initial()
                .query(recorded.public.verify_ops(), None, &INITIAL_COST_MODEL)
                .unwrap();
            assert_eq!(replay.context.state, native.context.query.state);
        }
    }
}
