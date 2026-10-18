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

//! Maps operations keep the carrier state private to its parent module.
use super::super::{Map, MapRead, MapWrite};
use super::{Circuit, Executor};
impl<E: Executor> Circuit<E, ()> {
    /// Map operands follow the same eager-argument contract as Cell operations.
    pub fn map_member<K, V>(self, map: Map<K, V>, key: K) -> Circuit<E, bool>
    where
        E: MapRead<K, V>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.map_member(map, key)),
        }
    }
    pub fn map_lookup<K, V>(self, map: Map<K, V>, key: K) -> Circuit<E, V>
    where
        E: MapRead<K, V>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.map_lookup(map, key)),
        }
    }
    pub fn map_size<K, V>(self, map: Map<K, V>) -> Circuit<E, u64>
    where
        E: MapRead<K, V>,
    {
        Circuit {
            state: self.state.and_then(|(executor, ())| executor.map_size(map)),
        }
    }
    pub fn map_is_empty<K, V>(self, map: Map<K, V>) -> Circuit<E, bool>
    where
        E: MapRead<K, V>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.map_is_empty(map)),
        }
    }
    pub fn map_insert<K, V>(self, map: Map<K, V>, key: K, value: V) -> Self
    where
        E: MapWrite<K, V>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.map_insert(map, key, value).map(|next| (next, ()))
            }),
        }
    }
    pub fn map_remove<K, V>(self, map: Map<K, V>, key: K) -> Self
    where
        E: MapWrite<K, V>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.map_remove(map, key).map(|next| (next, ()))),
        }
    }
    pub fn map_reset<K, V>(self, map: Map<K, V>) -> Self
    where
        E: MapWrite<K, V>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.map_reset(map).map(|next| (next, ()))),
        }
    }
    pub fn map_insert_default<K, V>(self, map: Map<K, V>, key: K) -> Self
    where
        E: MapWrite<K, V>,
        V: Default,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.map_insert_default(map, key).map(|next| (next, ()))
            }),
        }
    }
}
