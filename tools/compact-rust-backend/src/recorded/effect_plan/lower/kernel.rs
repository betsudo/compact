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

//! Ordered kernel and event effects use the exact Ledger VM operand types.
use super::*;

impl Lowerer<'_> {
    pub(super) fn emit_event(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let (value, length, operation): (_, _, fn(Value) -> Operation) = match expression {
            Expr::EmitShieldedSpend { value } => (value, 32, Operation::EmitShieldedSpend),
            Expr::EmitShieldedBurn { value } => (value, 49, Operation::EmitShieldedBurn),
            Expr::EmitShieldedMint { value } => (value, 81, Operation::EmitShieldedMint),
            Expr::EmitShieldedReceive { value } => (value, 578, Operation::EmitShieldedReceive),
            Expr::EmitUnshieldedSpend { value } => (value, 145, Operation::EmitUnshieldedSpend),
            Expr::EmitUnshieldedReceive { value } => (value, 145, Operation::EmitUnshieldedReceive),
            Expr::EmitUnshieldedMint { value } => (value, 80, Operation::EmitUnshieldedMint),
            Expr::EmitUnshieldedBurn { value } => (value, 113, Operation::EmitUnshieldedBurn),
            Expr::EmitPaused { value } => (value, 0, Operation::EmitPaused),
            Expr::EmitUnpaused { value } => (value, 0, Operation::EmitUnpaused),
            _ => return Err(invalid(path, "expected an event expression")),
        };
        let path = format!("{path}.value");
        let value = self.expression(value, scope, bindings, &path)?;
        if value.ty != (Type::Bytes { length }) {
            return Err(invalid(&path, format!("event payload must have type Bytes<{length}>")));
        }
        Ok((operation(value), Type::Unit))
    }

    pub(super) fn kernel(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let mut operand = |expression, name, expected| {
            let path = format!("{path}.{name}");
            let value = self.expression(expression, scope, bindings, &path)?;
            if value.ty != expected {
                return Err(invalid(
                    &path,
                    format!("expected {expected:?} kernel operand"),
                ));
            }
            Ok(value)
        };
        let operation = match expression {
            Expr::KernelClaim { claim, value } => Operation::KernelClaim {
                kind: *claim,
                value: operand(value, "value", Type::Bytes { length: 32 })?,
            },
            Expr::KernelMintShielded { domain, amount } => {
                let domain = operand(domain, "domain", Type::Bytes { length: 32 })?;
                let amount = operand(amount, "amount", uint("18446744073709551615"))?;
                Operation::KernelMintShielded { domain, amount }
            }
            _ => {
                return Err(invalid(
                    path,
                    "kernel expression dispatch requires a kernel operation",
                ));
            }
        };
        Ok((operation, Type::Unit))
    }
}
