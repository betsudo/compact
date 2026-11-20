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

use point_identity::runtime::{Secp256k1Point, Secp256r1Base, context::CircuitContext};

fn wrong_curve(ctx: CircuitContext<()>, point: Secp256k1Point) {
    let _ = point_identity::ledger_contract::storePoint(ctx, point);
}

fn field_is_not_point(ctx: CircuitContext<()>, x: Secp256r1Base) {
    let _ = point_identity::ledger_contract::storePoint(ctx, x);
}

fn main() {}
