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

//! Replay exact balance reads from the immutable execution-start snapshot.
use super::*;

impl<Private, D: DB> RecordingFrame<Private, D> {
    fn observe_balance(
        mut self,
        token: ledger::TokenType,
        mode: ledger::BalanceMode,
    ) -> Result<(Self, ledger::BalanceValue), CompactError> {
        let (result, value) = ledger::query_balance(
            &self.state.context.query,
            token.clone(),
            mode,
            self.state.context.gas_limit,
            &self.state.context.cost_model,
        )?;
        let Some(GatherEvent::Read(observed)) = result.events.last() else {
            return Err(CompactError::InvalidLedgerCell(
                "missing Kernel balance read event".into(),
            ));
        };
        let program = ledger::balance_program::<ResultModeVerify, D>(token, mode, observed.clone());
        self.state.context.query = result.context;
        self.state.observed_gas += result.gas_cost;
        self.state.verify_ops.extend(program);
        Ok((self, value))
    }
    pub fn kernel_balance(self, token: ledger::TokenType) -> Result<(Self, u128), CompactError> {
        let (frame, value) = self.observe_balance(token, ledger::BalanceMode::Read)?;
        let ledger::BalanceValue::Amount(value) = value else {
            unreachable!()
        };
        Ok((frame, value))
    }
    pub fn kernel_balance_less_than(
        self,
        token: ledger::TokenType,
        amount: u128,
    ) -> Result<(Self, bool), CompactError> {
        let (frame, value) = self.observe_balance(token, ledger::BalanceMode::LessThan(amount))?;
        let ledger::BalanceValue::Comparison(value) = value else {
            unreachable!()
        };
        Ok((frame, value))
    }
    pub fn kernel_balance_greater_than(
        self,
        token: ledger::TokenType,
        amount: u128,
    ) -> Result<(Self, bool), CompactError> {
        let (frame, value) =
            self.observe_balance(token, ledger::BalanceMode::GreaterThan(amount))?;
        let ledger::BalanceValue::Comparison(value) = value else {
            unreachable!()
        };
        Ok((frame, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ConstructorContext, ConstructorResult};
    use crate::ledger::{
        ChargedState, ContractAddress, DefaultDB, HashOutput, StateValue, UnshieldedTokenType,
    };

    #[test]
    fn balance_read_records_same_start_snapshot_after_multiple_queries() {
        let token = ledger::TokenType::Unshielded(UnshieldedTokenType(HashOutput([3; 32])));
        let mut context = ConstructorResult::new(
            ConstructorContext::new(()),
            ChargedState::new(StateValue::<DefaultDB>::Array(
                vec![ledger::constructor_cell(false)].into(),
            )),
        )
        .into_circuit_context(ContractAddress::default());
        context.query.call_context.balance = context.query.call_context.balance.insert(token, 100);
        let (frame, read) = RecordingFrame::new(context).kernel_balance(token).unwrap();
        let (frame, gt) = frame.kernel_balance_greater_than(token, 50).unwrap();
        let (frame, lt) = frame.kernel_balance_less_than(token, 150).unwrap();
        assert_eq!(read, 100);
        assert!(gt && lt);
        let result = frame.finish(());
        assert!(!result.public.verify_ops().is_empty());
        assert_eq!(
            *result
                .execution
                .context
                .query
                .call_context
                .balance
                .get(&token)
                .unwrap(),
            100
        );
    }
}
