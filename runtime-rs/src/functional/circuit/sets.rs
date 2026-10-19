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

//! Sets operations keep the carrier state private to its parent module.
use super::super::{Set, SetRead, SetWrite};
use super::{Circuit, Executor};
impl<E: Executor> Circuit<E, ()> {
    /// Set operands share the module-level eager argument contract.
    pub fn set_member<T>(self, set: Set<T>, value: T) -> Circuit<E, bool>
    where
        E: SetRead<T>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.set_member(set, value)),
        }
    }
    pub fn set_size<T>(self, set: Set<T>) -> Circuit<E, u64>
    where
        E: SetRead<T>,
    {
        Circuit {
            state: self.state.and_then(|(executor, ())| executor.set_size(set)),
        }
    }
    pub fn set_is_empty<T>(self, set: Set<T>) -> Circuit<E, bool>
    where
        E: SetRead<T>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.set_is_empty(set)),
        }
    }
    pub fn set_insert<T>(self, set: Set<T>, value: T) -> Self
    where
        E: SetWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.set_insert(set, value).map(|next| (next, ()))),
        }
    }
    pub fn set_remove<T>(self, set: Set<T>, value: T) -> Self
    where
        E: SetWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.set_remove(set, value).map(|next| (next, ()))),
        }
    }
    pub fn set_reset<T>(self, set: Set<T>) -> Self
    where
        E: SetWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.set_reset(set).map(|next| (next, ()))),
        }
    }
}
