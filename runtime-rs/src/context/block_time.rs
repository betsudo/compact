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

//! Native, metered Kernel block-time comparisons through the ledger VM.

use super::CircuitResult;
use crate::{CompactError, ledger};

impl<Private, D: crate::ledger::DB> super::CircuitContext<Private, D> {
    fn compare_block_time(
        mut self,
        bound: u64,
        comparison: ledger::BlockTimeComparison,
    ) -> Result<CircuitResult<Private, bool, D>, CompactError> {
        let (result, value) = ledger::query_block_time(
            &self.query,
            bound,
            comparison,
            self.gas_limit,
            &self.cost_model,
        )?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: value,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn kernel_block_time_less_than(
        self,
        bound: ::core::primitive::u64,
    ) -> ::core::result::Result<
        super::CircuitResult<Private, ::core::primitive::bool, D>,
        crate::CompactError,
    > {
        self.compare_block_time(bound, ledger::BlockTimeComparison::LessThan)
    }

    pub fn kernel_block_time_greater_than(
        self,
        bound: ::core::primitive::u64,
    ) -> ::core::result::Result<
        super::CircuitResult<Private, ::core::primitive::bool, D>,
        crate::CompactError,
    > {
        self.compare_block_time(bound, ledger::BlockTimeComparison::GreaterThan)
    }
}
