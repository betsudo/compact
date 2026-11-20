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

use arithmetic::{
    ledger_contract::{addb, adds},
    runtime::{Field, Secp256k1Base, Secp256r1Base, Secp256r1Scalar, context::CircuitContext},
};

fn refuse_scalar(ctx: CircuitContext<()>, a: Secp256r1Base, b: Secp256r1Scalar) {
    let _ = addb(ctx, a, b);
}

fn refuse_other_curve(ctx: CircuitContext<()>, a: Secp256r1Base, b: Secp256k1Base) {
    let _ = addb(ctx, a, b);
}

fn refuse_native_field(ctx: CircuitContext<()>, a: Secp256r1Scalar, b: Field) {
    let _ = adds(ctx, a, b);
}

fn main() {}
