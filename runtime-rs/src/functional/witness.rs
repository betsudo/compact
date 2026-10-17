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

//! Typed witness requests; callback and ledger details belong to interpreters.
use super::Executor;
use crate::CompactError;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessId(u32);
impl WitnessId {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Identity supplied by a checked generated witness declaration.
/// Handwritten construction is not a proof of witness or privacy authority.
pub struct Witness<Args, T> {
    id: WitnessId,
    types: PhantomData<fn(Args) -> T>,
}
impl<Args, T> Witness<Args, T> {
    pub const fn new(id: WitnessId) -> Self {
        Self {
            id,
            types: PhantomData,
        }
    }
    pub const fn id(self) -> WitnessId {
        self.id
    }
}
impl<Args, T> Copy for Witness<Args, T> {}
impl<Args, T> Clone for Witness<Args, T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait WitnessCall<Args, T>: Executor {
    fn witness(self, witness: Witness<Args, T>, arguments: Args)
    -> Result<(Self, T), CompactError>;
}
