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


use did_contract::{ledger_contract as did, runtime as r, types};
#[allow(dead_code)]
fn wrong<W: did::TryWitnesses<()>>(ctx: r::context::CircuitContext<()>, w: &W, sig: types::SchnorrSignature) {
    let _ = did::deactivate(ctx, w, sig, r::BoundedUint::<255>::new(0).unwrap());
}
fn main() {}
