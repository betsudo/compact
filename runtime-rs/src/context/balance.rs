// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Kernel balance reads use the call's input snapshot and metered VM queries.
use super::{CircuitContext, CircuitResult};
use crate::ledger::DB;
use crate::{CompactError, ledger};

impl<Private, D: DB> CircuitContext<Private, D> {
    pub fn kernel_balance(
        mut self,
        token: ledger::TokenType,
    ) -> Result<CircuitResult<Private, u128, D>, CompactError> {
        let (result, value) = ledger::query_balance(
            &self.query,
            token,
            ledger::BalanceMode::Read,
            self.gas_limit,
            &self.cost_model,
        )?;
        self.query = result.context;
        let ledger::BalanceValue::Amount(value) = value else {
            unreachable!()
        };
        Ok(CircuitResult {
            context: self,
            result: value,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }
    pub fn kernel_balance_less_than(
        mut self,
        token: ledger::TokenType,
        amount: u128,
    ) -> Result<CircuitResult<Private, bool, D>, CompactError> {
        let (result, value) = ledger::query_balance(
            &self.query,
            token,
            ledger::BalanceMode::LessThan(amount),
            self.gas_limit,
            &self.cost_model,
        )?;
        self.query = result.context;
        let ledger::BalanceValue::Comparison(value) = value else {
            unreachable!()
        };
        Ok(CircuitResult {
            context: self,
            result: value,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }
    pub fn kernel_balance_greater_than(
        mut self,
        token: ledger::TokenType,
        amount: u128,
    ) -> Result<CircuitResult<Private, bool, D>, CompactError> {
        let (result, value) = ledger::query_balance(
            &self.query,
            token,
            ledger::BalanceMode::GreaterThan(amount),
            self.gas_limit,
            &self.cost_model,
        )?;
        self.query = result.context;
        let ledger::BalanceValue::Comparison(value) = value else {
            unreachable!()
        };
        Ok(CircuitResult {
            context: self,
            result: value,
            gas_cost: result.gas_cost,
            private_transcript_outputs: Vec::new(),
        })
    }
}
