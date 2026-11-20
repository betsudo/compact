// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! Private source-identity controls; root owns execution and physical binding.
use super::*;
use crate::ir::{Expr, LocalBinding, NativeWitnessBuiltin, Parameter, StateAction, StateReturn};
use crate::source_diagnostics::Slot;
use std::cell::RefCell;

mod helpers;
#[rustfmt::skip]
mod old_reference;
use helpers::*;
mod pipeline;
mod sources;
mod validation;
