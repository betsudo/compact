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

//! Strict block-time comparisons against QueryContext slot 2 (`tblock`).
//! These are the five VM operations in the pinned Compact Ledger 9 compiler.

use super::{CompactError, QueryContext, QueryResults, decode_last_read};
use midnight_base_crypto::cost_model::RunningCost;
use midnight_onchain_state::state::AlignedValueChecked;
use midnight_onchain_vm::{
    cost_model::CostModel,
    ops::{Key, Op},
    result_mode::{ResultMode, ResultModeGather},
};
use midnight_storage::db::DB;

#[derive(Clone, Copy, Debug)]
pub(crate) enum BlockTimeComparison {
    LessThan,
    GreaterThan,
}

pub(crate) fn block_time_program<M: ResultMode<D>, D: DB>(
    bound: u64,
    comparison: BlockTimeComparison,
    read_result: M::ReadResult,
) -> Vec<Op<M, D>> {
    let read_time = Op::Idx {
        cached: true,
        push_path: false,
        path: vec![Key::Value(AlignedValueChecked::from(2u8))].into(),
    };
    let push_bound = Op::Push {
        storage: false,
        value: super::vm::literal(bound),
    };
    match comparison {
        BlockTimeComparison::LessThan => vec![
            Op::Dup { n: 2 },
            read_time,
            push_bound,
            Op::Lt,
            Op::Popeq {
                cached: true,
                result: read_result,
            },
        ],
        BlockTimeComparison::GreaterThan => vec![
            push_bound,
            Op::Dup { n: 3 },
            read_time,
            Op::Lt,
            Op::Popeq {
                cached: true,
                result: read_result,
            },
        ],
    }
}

pub(crate) fn query_block_time<D: DB>(
    context: &QueryContext<D>,
    bound: u64,
    comparison: BlockTimeComparison,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<(QueryResults<ResultModeGather, D>, bool), CompactError> {
    let result = context
        .query(
            &block_time_program::<ResultModeGather, D>(bound, comparison, ()),
            gas_limit,
            cost_model,
        )
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))?;
    let value = decode_last_read::<bool, D>(&result)?;
    Ok((result, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{DefaultDB, empty_query_context};
    use midnight_base_crypto::cost_model::RunningCost;
    use midnight_base_crypto::time::Timestamp;
    use midnight_onchain_vm::{
        cost_model::INITIAL_COST_MODEL,
        result_mode::{GatherEvent, ResultModeVerify},
    };

    #[test]
    fn original_twelve_boundaries_read_tblock_not_last_block_time() {
        let mut context = empty_query_context();
        context.call_context.tblock = Timestamp::from_secs(1);
        context.call_context.tblock_err = 0;
        context.call_context.last_block_time = Timestamp::from_secs(900);
        for (bound, lt, lte, gt, gte) in [
            (0, false, false, true, true),
            (1, false, true, false, true),
            (5, true, true, false, false),
        ] {
            let (_, actual_lt) = query_block_time(
                &context,
                bound,
                BlockTimeComparison::LessThan,
                None,
                &INITIAL_COST_MODEL,
            )
            .unwrap();
            let (_, actual_gt) = query_block_time(
                &context,
                bound,
                BlockTimeComparison::GreaterThan,
                None,
                &INITIAL_COST_MODEL,
            )
            .unwrap();
            assert_eq!(actual_lt, lt);
            assert_eq!(actual_gt, gt);
            assert_eq!(!actual_gt, lte); // Standard Lte wrapper is Not(Gt).
            assert_eq!(!actual_lt, gte); // Standard Gte wrapper is Not(Lt).
        }
        assert!(
            !query_block_time(
                &context,
                1,
                BlockTimeComparison::LessThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
        assert!(
            !query_block_time(
                &context,
                1,
                BlockTimeComparison::GreaterThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
        // Explicit high-bit input must stay u64, not a signed or floating carrier.
        assert!(
            query_block_time(
                &context,
                1u64 << 63,
                BlockTimeComparison::LessThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
        assert!(
            !query_block_time(
                &context,
                1u64 << 63,
                BlockTimeComparison::GreaterThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
        assert!(
            query_block_time(
                &context,
                u64::MAX,
                BlockTimeComparison::LessThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
        assert!(
            !query_block_time(
                &context,
                u64::MAX,
                BlockTimeComparison::GreaterThan,
                None,
                &INITIAL_COST_MODEL
            )
            .unwrap()
            .1
        );
    }

    #[test]
    fn gathered_read_replays_as_metered_verify_and_refuses_changed_context() {
        let mut context = empty_query_context();
        context.call_context.tblock = Timestamp::from_secs(1);
        let (result, value) = query_block_time(
            &context,
            5,
            BlockTimeComparison::LessThan,
            None,
            &INITIAL_COST_MODEL,
        )
        .unwrap();
        assert!(value);
        assert_ne!(result.gas_cost, RunningCost::ZERO);
        let Some(GatherEvent::Read(observed)) = result.events.last() else {
            panic!("missing comparison read")
        };
        let program = block_time_program::<ResultModeVerify, DefaultDB>(
            5,
            BlockTimeComparison::LessThan,
            observed.clone(),
        );
        assert_eq!(program.len(), 5);
        let replay = context.query(&program, None, &INITIAL_COST_MODEL).unwrap();
        assert_eq!(replay.gas_cost, result.gas_cost);
        let wrong_bound = block_time_program::<ResultModeVerify, DefaultDB>(
            0,
            BlockTimeComparison::LessThan,
            observed.clone(),
        );
        assert!(
            context
                .query(&wrong_bound, None, &INITIAL_COST_MODEL)
                .is_err()
        );
        assert!(
            query_block_time(
                &context,
                5,
                BlockTimeComparison::LessThan,
                Some(RunningCost::ZERO),
                &INITIAL_COST_MODEL,
            )
            .is_err()
        );
        // Verify commits the observed aligned Boolean, not every distinct timestamp.
        let mut same_result = context.clone();
        same_result.call_context.tblock = Timestamp::from_secs(2);
        assert!(
            same_result
                .query(&program, None, &INITIAL_COST_MODEL)
                .is_ok()
        );
        let mut changed = context;
        changed.call_context.tblock = Timestamp::from_secs(6);
        assert!(changed.query(&program, None, &INITIAL_COST_MODEL).is_err());
    }
}
