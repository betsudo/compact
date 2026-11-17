// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

//! Read the execution-start balance map with Compact's exact Kernel VM path.
//! Effects from mint, send, and receive do not change this snapshot.

use super::{CompactError, QueryContext, QueryResults, decode_last_read};
use midnight_base_crypto::{cost_model::RunningCost, fab::AlignedValue};
use midnight_coin_structure::coin::TokenType;
use midnight_onchain_state::state::AlignedValueChecked;
use midnight_onchain_vm::{
    cost_model::CostModel,
    ops::{Key, Op},
    result_mode::{ResultMode, ResultModeGather},
};
use midnight_storage::db::DB;

#[derive(Clone, Copy, Debug)]
pub(crate) enum BalanceMode {
    Read,
    LessThan(u128),
    GreaterThan(u128),
}

pub(crate) fn balance_program<M: ResultMode<D>, D: DB>(
    token: TokenType,
    mode: BalanceMode,
    read_result: M::ReadResult,
) -> Vec<Op<M, D>> {
    // The SDK's typed FAB conversion supplies the three-field TokenType key.
    let key = AlignedValue::from(token);
    let mut ops = Vec::new();
    if let BalanceMode::GreaterThan(amount) = mode {
        ops.push(Op::Push {
            storage: false,
            value: super::vm::literal(amount),
        });
    }
    ops.extend([
        Op::Dup {
            n: if matches!(mode, BalanceMode::GreaterThan(_)) {
                3
            } else {
                2
            },
        },
        Op::Idx {
            cached: true,
            push_path: false,
            path: vec![Key::Value(AlignedValueChecked::from(5u8))].into(),
        },
        Op::Dup { n: 0 },
        Op::Push {
            storage: false,
            value: super::vm::literal(key.clone()),
        },
        Op::Member,
        Op::Branch { skip: 3 },
        Op::Pop,
        Op::Push {
            storage: false,
            value: super::vm::literal(0u128),
        },
        Op::Jmp { skip: 1 },
        Op::Idx {
            cached: true,
            push_path: false,
            path: vec![Key::Value(
                super::vm::key(key).expect("SDK TokenType has canonical FAB encoding"),
            )]
            .into(),
        },
    ]);
    match mode {
        BalanceMode::Read => {}
        BalanceMode::LessThan(amount) => {
            ops.push(Op::Push {
                storage: false,
                value: super::vm::literal(amount),
            });
            ops.push(Op::Lt);
        }
        BalanceMode::GreaterThan(_) => ops.push(Op::Lt),
    }
    ops.push(Op::Popeq {
        cached: true,
        result: read_result,
    });
    ops
}

pub(crate) fn query_balance<D: DB>(
    context: &QueryContext<D>,
    token: TokenType,
    mode: BalanceMode,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<(QueryResults<ResultModeGather, D>, BalanceValue), CompactError> {
    let result = context
        .query(
            &balance_program::<ResultModeGather, D>(token, mode, ()),
            gas_limit,
            cost_model,
        )
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))?;
    let value = match mode {
        BalanceMode::Read => BalanceValue::Amount(decode_last_read::<u128, D>(&result)?),
        BalanceMode::LessThan(_) | BalanceMode::GreaterThan(_) => {
            BalanceValue::Comparison(decode_last_read::<bool, D>(&result)?)
        }
    };
    Ok((result, value))
}

pub(crate) enum BalanceValue {
    Amount(u128),
    Comparison(bool),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, DefaultDB, HashOutput, ShieldedTokenType, UnshieldedTokenType};
    use midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;

    fn observe(
        context: &QueryContext<DefaultDB>,
        token: TokenType,
        mode: BalanceMode,
    ) -> BalanceValue {
        query_balance(context, token, mode, None, &INITIAL_COST_MODEL)
            .unwrap()
            .1
    }

    #[test]
    fn original_unshielded_balance_assertions_use_execution_start_snapshot() {
        // Values and assertions are the five cases in pinned upstream
        // test-center/ts/unshielded-tokens.ts:getUnshieldedBalanceTest.
        let token = TokenType::Unshielded(UnshieldedTokenType(HashOutput([1; 32])));
        let other = TokenType::Unshielded(UnshieldedTokenType(HashOutput([2; 32])));
        let shielded = TokenType::Shielded(ShieldedTokenType(HashOutput([1; 32])));
        let mut context = ledger::empty_query_context();
        context.call_context.balance = context.call_context.balance.insert(token, 100);
        assert!(matches!(
            observe(&context, token, BalanceMode::Read),
            BalanceValue::Amount(100)
        ));
        assert!(matches!(
            observe(&context, other, BalanceMode::Read),
            BalanceValue::Amount(0)
        ));
        assert!(matches!(
            observe(&context, shielded, BalanceMode::Read),
            BalanceValue::Amount(0)
        ));
        assert!(matches!(
            observe(&context, token, BalanceMode::GreaterThan(50)),
            BalanceValue::Comparison(true)
        ));
        assert!(matches!(
            observe(&context, token, BalanceMode::GreaterThan(150)),
            BalanceValue::Comparison(false)
        ));
        assert!(matches!(
            observe(&context, token, BalanceMode::LessThan(50)),
            BalanceValue::Comparison(false)
        ));
        assert!(matches!(
            observe(&context, token, BalanceMode::LessThan(150)),
            BalanceValue::Comparison(true)
        ));
        assert_eq!(*context.call_context.balance.get(&token).unwrap(), 100);
    }
}
