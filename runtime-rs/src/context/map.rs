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

//! Native map operations, preserving context ownership and ledger query cost.

use super::{CircuitContext, CircuitResult};
use crate::ledger::{CellValue, DB};
use crate::{CompactError, ledger};

impl<Private, D: DB> CircuitContext<Private, D> {
    pub fn insert_map<K: CellValue, V: CellValue>(
        mut self,
        path: impl Into<ledger::LedgerPath>,
        key: K,
        value: V,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        key.validate_cell_value()?;
        value.validate_cell_value()?;
        let result = ledger::insert_map(
            &self.query,
            path,
            key,
            value,
            self.gas_limit,
            &self.cost_model,
        )
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: (),
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn member_map<K: CellValue>(
        mut self,
        path: impl Into<ledger::LedgerPath>,
        key: K,
    ) -> Result<CircuitResult<Private, bool, D>, CompactError> {
        let (result, present) =
            ledger::member_map(&self.query, path, key, self.gas_limit, &self.cost_model)?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: present,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn lookup_map<K: CellValue, V: CellValue>(
        mut self,
        path: impl Into<ledger::LedgerPath>,
        key: K,
    ) -> Result<CircuitResult<Private, V, D>, CompactError> {
        let (result, value) =
            ledger::lookup_map(&self.query, path, key, self.gas_limit, &self.cost_model)?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: value,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn remove_map<K: CellValue>(
        self,
        path: impl Into<ledger::LedgerPath>,
        key: K,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        self.remove_set(path, key)
    }

    pub fn size_map(
        mut self,
        path: impl Into<ledger::LedgerPath>,
    ) -> Result<CircuitResult<Private, u64, D>, CompactError> {
        let (result, size) = ledger::size_map(&self.query, path, self.gas_limit, &self.cost_model)?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: size,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn is_empty_map(
        mut self,
        path: impl Into<ledger::LedgerPath>,
    ) -> Result<CircuitResult<Private, bool, D>, CompactError> {
        let (result, empty) =
            ledger::is_empty_map(&self.query, path, self.gas_limit, &self.cost_model)?;
        self.query = result.context;
        Ok(CircuitResult {
            context: self,
            result: empty,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }

    pub fn reset_map(
        self,
        path: impl Into<ledger::LedgerPath>,
    ) -> Result<CircuitResult<Private, (), D>, CompactError> {
        self.reset_set(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ConstructorContext, ConstructorResult};
    use crate::fab::AlignedValue;
    use crate::ledger::{DefaultDB, LedgerHashMap, StateValue, constructor_cell, contract_state};
    use crate::recording::RecordingFrame;

    fn context(value: StateValue<DefaultDB>) -> CircuitContext<()> {
        ConstructorResult::new(ConstructorContext::new(()), contract_state(vec![value]))
            .into_circuit_context(Default::default())
    }

    #[test]
    fn map_metadata_rejects_array_and_keeps_valid_emptiness_cost() {
        let error = Some(CompactError::LedgerQueryRejected(
            "expected Map at metadata query path".into(),
        ));
        assert_eq!(
            context(StateValue::Array(vec![].into()))
                .is_empty_map(&[0][..])
                .err(),
            error
        );
        assert_eq!(
            RecordingFrame::new(context(StateValue::Array(vec![].into())))
                .is_empty_map(&[0][..])
                .err(),
            error
        );
        for entries in [0, 1] {
            let mut values = LedgerHashMap::new();
            if entries == 1 {
                values = values.insert(
                    AlignedValue::from(crate::Field::from(1u64)),
                    constructor_cell(true),
                );
            }
            let map = StateValue::Map(values);
            let actual = context(map.clone()).is_empty_map(&[0][..]).unwrap();
            let original = context(map.clone()).is_empty_set(&[0][..]).unwrap();
            assert_eq!(actual.result, entries == 0);
            assert_eq!(actual.result, original.result);
            assert_eq!(actual.gas_cost, original.gas_cost);
            let (_, recorded) = RecordingFrame::new(context(map))
                .is_empty_map(&[0][..])
                .unwrap();
            assert_eq!(recorded, entries == 0);
        }
    }
}
