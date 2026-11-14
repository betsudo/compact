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

use base::{ledger_contract, runtime};

fn main() {
    let context = || {
        ledger_contract::initial_state(runtime::context::ConstructorContext::new(()))
            .unwrap()
            .into_circuit_context(runtime::ledger::ContractAddress::default())
    };
    let _ =
        ledger_contract::secp256r1_base_roundtrip(context(), runtime::Secp256r1Scalar::default());
    let _ = ledger_contract::secp256r1_base_roundtrip(context(), runtime::Secp256k1Base::default());
}
