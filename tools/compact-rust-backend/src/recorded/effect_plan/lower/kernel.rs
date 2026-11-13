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
    pub(super) fn shielded_spend_event(
        &mut self,
        expression: &Expr,
        scope: &Scope,
        bindings: &mut Vec<Binding>,
        path: &str,
    ) -> Result<(Operation, Type), RecordingGap> {
        let Expr::EmitShieldedSpend { value } = expression else {
            return Err(invalid(path, "expected ShieldedSpend event"));
        };
        let path = format!("{path}.value");
        let value = self.expression(value, scope, bindings, &path)?;
        if value.ty != (Type::Bytes { length: 32 }) {
            return Err(invalid(
                &path,
                "ShieldedSpend payload must have type Bytes<32>",
            ));
        }
        Ok((Operation::EmitShieldedSpend(value), Type::Unit))
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
