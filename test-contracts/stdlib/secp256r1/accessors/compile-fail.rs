// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use accessor_direct::runtime::{Secp256k1Point, Secp256r1Base};

fn wrong_curve(point: Secp256k1Point) {
    let _ = accessor_direct::pure_circuits::test0(point);
}

fn field_is_not_point(value: Secp256r1Base) {
    let _ = accessor_direct::pure_circuits::test1(value);
}

fn main() {}
