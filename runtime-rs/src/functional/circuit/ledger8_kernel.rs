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

//! Explicit Ledger8 extensions preserve carrier ownership and eager operands.
use super::super::ledger8::{
    ExecutionIdentity, KernelAddress, KernelClaims, ShieldedMint, ZswapInput, ZswapOutput,
};
use super::{Circuit, Executor};
use crate::ledger;
impl<E: Executor> Circuit<E, ()> {
    pub fn own_public_key(self) -> Circuit<E, [u8; 32]>
    where
        E: ExecutionIdentity,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.own_public_key()),
        }
    }
    pub fn kernel_address(self) -> Circuit<E, ledger::ContractAddress>
    where
        E: KernelAddress,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.kernel_address()),
        }
    }
    pub fn kernel_claim(self, claim: ledger::KernelClaim) -> Self
    where
        E: KernelClaims,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.kernel_claim(claim).map(|next| (next, ()))),
        }
    }
    pub fn mint_shielded(self, domain: ledger::HashOutput, amount: u64) -> Self
    where
        E: ShieldedMint,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .mint_shielded(domain, amount)
                    .map(|next| (next, ()))
            }),
        }
    }
    pub fn zswap_input(self, coin: ledger::QualifiedCoinInfo) -> Self
    where
        E: ZswapInput,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.zswap_input(coin).map(|next| (next, ()))),
        }
    }
    pub fn zswap_output(self, coin: ledger::CoinInfo, recipient: ledger::CoinRecipient) -> Self
    where
        E: ZswapOutput,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor
                    .zswap_output(coin, recipient)
                    .map(|next| (next, ()))
            }),
        }
    }
}
