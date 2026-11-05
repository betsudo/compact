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

// Compile this complete unedited SDK at its own crate root: generated crate::
// paths must resolve inside this package, independently of every other SDK.
#[cfg(compact_component_product_vector_family_sdk)]
include!(concat!(
    env!("COMPACT_COMPONENT_PRODUCT_VECTOR_GENERATED"),
    "/family/contract/lib.rs"
));
