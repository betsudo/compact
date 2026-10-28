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

//! Source-bound constant-valued Vector index proof qualification.
use compact_rust_constant_valued_vector_indices_source_fixture as fixture;
const SOURCE_LABEL: &str = "constant-valued-vector-indices";
const IR: &str = include_str!(
    "../../../tests-rust-backend/constant-valued-vector-indices-source/original-ir.json"
);
include!("vector_projection_protocol.rs");
