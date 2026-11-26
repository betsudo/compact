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

//! Record the exact VM comparison and observed Boolean for Verify replay.

use super::*;

impl<Private, D: crate::ledger::DB> super::RecordingFrame<Private, D> {
    fn compare_block_time(
        mut self,
        bound: u64,
        comparison: ledger::BlockTimeComparison,
    ) -> Result<(Self, bool), CompactError> {
        let (result, value) = ledger::query_block_time(
            &self.state.context.query,
            bound,
            comparison,
            self.state.context.gas_limit,
            &self.state.context.cost_model,
        )?;
        let Some(GatherEvent::Read(observed)) = result.events.last() else {
            return Err(CompactError::InvalidLedgerCell(
                "missing Kernel block-time comparison read event".into(),
            ));
        };
        let program =
            ledger::block_time_program::<ResultModeVerify, D>(bound, comparison, observed.clone());
        self.state.context.query = result.context;
        self.state.observed_gas += result.gas_cost;
        self.state.verify_ops.extend(program);
        Ok((self, value))
    }

    pub fn kernel_block_time_less_than(
        self,
        bound: ::core::primitive::u64,
    ) -> ::core::result::Result<(Self, ::core::primitive::bool), crate::CompactError> {
        self.compare_block_time(bound, ledger::BlockTimeComparison::LessThan)
    }

    pub fn kernel_block_time_greater_than(
        self,
        bound: ::core::primitive::u64,
    ) -> ::core::result::Result<(Self, ::core::primitive::bool), crate::CompactError> {
        self.compare_block_time(bound, ledger::BlockTimeComparison::GreaterThan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ConstructorContext, ConstructorResult};
    use crate::ledger::{self, ContractAddress, DefaultDB};
    use midnight_base_crypto::time::Timestamp;

    fn context() -> CircuitContext<(), DefaultDB> {
        let mut context =
            ConstructorResult::new(ConstructorContext::new(()), ledger::empty_contract_state())
                .into_circuit_context(ContractAddress::default());
        context.query.call_context.tblock = Timestamp::from_secs(1);
        context.query.call_context.last_block_time = Timestamp::from_secs(900);
        context
    }

    #[test]
    fn recording_replays_exact_time_query_and_matches_native_gas() {
        let native = context().kernel_block_time_less_than(5).unwrap();
        let (frame, recorded_value) = RecordingFrame::new(context())
            .kernel_block_time_less_than(5)
            .unwrap();
        assert_eq!(recorded_value, native.result);
        let recorded = frame.finish(recorded_value);
        assert_eq!(recorded.execution.gas_cost, native.gas_cost);
        assert_eq!(recorded.execution.private_transcript_outputs.len(), 0);
        assert_eq!(recorded.public.verify_ops().len(), 5);
        let (initial, verify) = recorded.public.into_parts();
        assert!(
            initial
                .query(&verify, None, &native.context.cost_model)
                .is_ok()
        );
    }
}
