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

use unary_base::runtime::{Secp256k1Base, Secp256r1Base, Secp256r1Scalar, context::CircuitContext};

fn wrong_curve(ctx: CircuitContext<()>, value: Secp256k1Base) {
    let _ = unary_base::ledger_contract::test(ctx, value);
}

fn wrong_base(ctx: CircuitContext<()>, value: Secp256r1Scalar) {
    let _ = unary_base::ledger_contract::test(ctx, value);
}

fn wrong_scalar(ctx: CircuitContext<()>, value: Secp256r1Base) {
    let _ = unary_scalar::ledger_contract::test(ctx, value);
}

fn main() {}
