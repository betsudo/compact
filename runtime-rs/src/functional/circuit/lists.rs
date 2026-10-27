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

//! List steps preserve the parent's private consuming success/error state.
use super::super::{List, ListHead, ListRead, ListWrite};
use super::{Circuit, Executor};
impl<E: Executor> Circuit<E, ()> {
    pub fn list_length<T>(self, list: List<T>) -> Circuit<E, u64>
    where
        E: ListRead<T>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.list_length(list)),
        }
    }
    pub fn list_is_empty<T>(self, list: List<T>) -> Circuit<E, bool>
    where
        E: ListRead<T>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.list_is_empty(list)),
        }
    }
    pub fn list_head<T, M>(self, list: List<T>) -> Circuit<E, M>
    where
        E: ListHead<T, M>,
    {
        Circuit {
            state: self
                .state
                .and_then(|(executor, ())| executor.list_head(list)),
        }
    }
    pub fn list_push_front<T>(self, list: List<T>, value: T) -> Self
    where
        E: ListWrite<T>,
    {
        Self {
            state: self.state.and_then(|(executor, ())| {
                executor.list_push_front(list, value).map(|next| (next, ()))
            }),
        }
    }
    pub fn list_pop_front<T>(self, list: List<T>) -> Self
    where
        E: ListWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.list_pop_front(list).map(|next| (next, ()))),
        }
    }
    pub fn list_reset<T>(self, list: List<T>) -> Self
    where
        E: ListWrite<T>,
    {
        Self {
            state: self
                .state
                .and_then(|(executor, ())| executor.list_reset(list).map(|next| (next, ()))),
        }
    }
}
