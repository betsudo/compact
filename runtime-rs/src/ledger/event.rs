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

use super::{CompactError, DB, FixedBytes, Op, StateValue, vm};
use midnight_onchain_vm::ops::LogEventType;
use midnight_onchain_vm::result_mode::ResultModeVerify;

pub(crate) fn shielded_spend_program<D: DB>(
    nullifier: FixedBytes<32>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::ShieldedSpend, nullifier)
}

pub(crate) fn shielded_burn_program<D: DB>(
    payload: FixedBytes<49>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::ShieldedBurn, payload)
}

pub(crate) fn shielded_mint_program<D: DB>(
    payload: FixedBytes<81>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::ShieldedMint, payload)
}

pub(crate) fn shielded_receive_program<D: DB>(
    payload: FixedBytes<578>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::ShieldedReceive, payload)
}

pub(crate) fn unshielded_spend_program<D: DB>(
    payload: FixedBytes<145>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::UnshieldedSpend, payload)
}

pub(crate) fn unshielded_receive_program<D: DB>(
    payload: FixedBytes<145>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    event_program(LogEventType::UnshieldedReceive, payload)
}

fn event_program<D: DB, const LENGTH: usize>(
    tag: LogEventType,
    bytes: FixedBytes<LENGTH>,
) -> Result<Vec<Op<ResultModeVerify, D>>, CompactError> {
    // Upstream lower-emit packs the version, event tag and payload in order.
    let payload = StateValue::Array(
        vec![
            vm::literal(1u32),
            vm::literal(tag as u8),
            vm::cell(bytes)?,
        ]
        .into(),
    );
    Ok(vec![
        Op::Push {
            storage: false,
            value: payload,
        },
        Op::Log,
    ])
}
