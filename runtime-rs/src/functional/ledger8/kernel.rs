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

//! Circuit-only Ledger8 domain capabilities. No constructor interpreter is provided.
//! Handwritten implementations do not certify phase, privacy or offer authority.
use super::super::Executor;
use super::{NativeLedger8, RecordedLedger8};
use crate::{
    CompactError,
    ledger::{self, DB},
};
pub trait ExecutionIdentity: Executor {
    fn own_public_key(self) -> Result<(Self, [u8; 32]), CompactError>;
}
pub trait KernelAddress: Executor {
    fn kernel_address(self) -> Result<(Self, ledger::ContractAddress), CompactError>;
}
pub trait KernelClaims: Executor {
    fn kernel_claim(self, claim: ledger::KernelClaim) -> Result<Self, CompactError>;
}
pub trait ShieldedMint: Executor {
    fn mint_shielded(self, domain: ledger::HashOutput, amount: u64) -> Result<Self, CompactError>;
}
pub trait ZswapInput: Executor {
    fn zswap_input(self, coin: ledger::QualifiedCoinInfo) -> Result<Self, CompactError>;
}
pub trait ZswapOutput: Executor {
    fn zswap_output(
        self,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError>;
}
impl<Private, D: DB, W> ExecutionIdentity for NativeLedger8<'_, Private, D, W> {
    fn own_public_key(self) -> Result<(Self, [u8; 32]), CompactError> {
        let (frame, value) = self.frame.own_coin_public_key()?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
impl<Private, D: DB, W> KernelAddress for NativeLedger8<'_, Private, D, W> {
    fn kernel_address(self) -> Result<(Self, ledger::ContractAddress), CompactError> {
        let (frame, value) = self.frame.apply(|context| context.kernel_self())?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
impl<Private, D: DB, W> KernelClaims for NativeLedger8<'_, Private, D, W> {
    fn kernel_claim(self, claim: ledger::KernelClaim) -> Result<Self, CompactError> {
        let frame = self.frame.kernel_claim(claim)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ShieldedMint for NativeLedger8<'_, Private, D, W> {
    fn mint_shielded(self, domain: ledger::HashOutput, amount: u64) -> Result<Self, CompactError> {
        let (frame, ()) = self
            .frame
            .apply(|context| context.kernel_mint_shielded(domain, amount))?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ZswapInput for NativeLedger8<'_, Private, D, W> {
    fn zswap_input(self, coin: ledger::QualifiedCoinInfo) -> Result<Self, CompactError> {
        let frame = self.frame.create_zswap_input(coin);
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ZswapOutput for NativeLedger8<'_, Private, D, W> {
    fn zswap_output(
        self,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let frame = self.frame.create_zswap_output(coin, recipient)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ExecutionIdentity for RecordedLedger8<'_, Private, D, W> {
    fn own_public_key(self) -> Result<(Self, [u8; 32]), CompactError> {
        let (frame, value) = self.frame.own_coin_public_key()?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
impl<Private, D: DB, W> KernelAddress for RecordedLedger8<'_, Private, D, W> {
    fn kernel_address(self) -> Result<(Self, ledger::ContractAddress), CompactError> {
        let (frame, value) = self.frame.kernel_self()?;
        Ok((
            Self {
                frame,
                bindings: self.bindings,
                witnesses: self.witnesses,
            },
            value,
        ))
    }
}
impl<Private, D: DB, W> KernelClaims for RecordedLedger8<'_, Private, D, W> {
    fn kernel_claim(self, claim: ledger::KernelClaim) -> Result<Self, CompactError> {
        let frame = self.frame.kernel_claim(claim)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ShieldedMint for RecordedLedger8<'_, Private, D, W> {
    fn mint_shielded(self, domain: ledger::HashOutput, amount: u64) -> Result<Self, CompactError> {
        let frame = self.frame.kernel_mint_shielded(domain, amount)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ZswapInput for RecordedLedger8<'_, Private, D, W> {
    fn zswap_input(self, coin: ledger::QualifiedCoinInfo) -> Result<Self, CompactError> {
        let frame = self.frame.create_zswap_input(coin);
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
impl<Private, D: DB, W> ZswapOutput for RecordedLedger8<'_, Private, D, W> {
    fn zswap_output(
        self,
        coin: ledger::CoinInfo,
        recipient: ledger::CoinRecipient,
    ) -> Result<Self, CompactError> {
        let frame = self.frame.create_zswap_output(coin, recipient)?;
        Ok(Self {
            frame,
            bindings: self.bindings,
            witnesses: self.witnesses,
        })
    }
}
