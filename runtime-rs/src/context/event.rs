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

use super::{CircuitContext, CircuitResult};
use crate::ledger::{ContractAddress, DB, DefaultDB, shielded_burn_program, shielded_spend_program};
use midnight_onchain_vm::result_mode::ResultModeVerify;
use crate::{CompactError, FixedBytes};
use midnight_onchain_vm::ops::{Op, VersionedLogItem};

/// A VM-decoded contract event tagged with the emitting contract address.
pub struct ContractEvent<D: DB = DefaultDB> {
    pub address: ContractAddress,
    pub log: VersionedLogItem<D>,
}

impl<D: DB> Clone for ContractEvent<D> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            log: self.log.clone(),
        }
    }
}

impl<D: DB> PartialEq for ContractEvent<D> {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.log == other.log
    }
}

impl<D: DB> Eq for ContractEvent<D> {}

impl<Private, D: DB> CircuitContext<Private, D> {
    /// Events emitted so far in this execution, in VM order.
    pub fn events(&self) -> &[ContractEvent<D>] {
        &self.events
    }

    pub(crate) fn append_events(&mut self, events: Vec<VersionedLogItem<D>>) {
        let address = self.query.address;
        self.events
            .extend(events.into_iter().map(|log| ContractEvent { address, log }));
    }

    /// Emit a public ShieldedSpend event through the ledger VM.
    pub fn emit_shielded_spend(
        self,
        nullifier: FixedBytes<32>,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        self.emit_event(shielded_spend_program(nullifier)?)
    }

    /// Emit a public ShieldedBurn event with its fixed-width wire payload.
    #[doc(hidden)]
    pub fn emit_shielded_burn(
        self,
        payload: FixedBytes<49>,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        self.emit_event(shielded_burn_program(payload)?)
    }

    fn emit_event(
        mut self,
        program: Vec<Op<ResultModeVerify, D>>,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        let result = self
            .query
            .query(&program, self.gas_limit, &self.cost_model)
            .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))?;
        self.append_events(result.events);
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: (),
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }
}
