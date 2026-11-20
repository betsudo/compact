// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Kernel effects use their own ledger VM frame, never a contract ledger slot.
use super::{
    CoinCommitment, CoinNullifier, CompactError, ContractAddress, HashOutput, QueryContext,
    QueryResults, StateValue,
};
use crate::Field;
use midnight_base_crypto::{cost_model::RunningCost, fab::AlignedValue};
use midnight_coin_structure::coin::{PublicAddress, TokenType};
use midnight_onchain_runtime::context::ClaimedUnshieldedSpendsKey;
use midnight_onchain_state::state::AlignedValueChecked;
use midnight_onchain_vm::{
    cost_model::CostModel,
    ops::{Key, Op},
    result_mode::{ResultMode, ResultModeVerify},
};
use midnight_storage::db::DB;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelClaim {
    Nullifier(CoinNullifier),
    CoinSpend(CoinCommitment),
    CoinReceive(CoinCommitment),
}

pub fn query_kernel_claim<D: DB>(
    context: &QueryContext<D>,
    claim: KernelClaim,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    context
        .query(&kernel_claim_program(claim), gas_limit, cost_model)
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))
}
pub fn query_kernel_mint_shielded<D: DB>(
    context: &QueryContext<D>,
    domain: HashOutput,
    amount: u64,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    context
        .query(
            &kernel_mint_shielded_program(domain, amount),
            gas_limit,
            cost_model,
        )
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))
}

fn query_unshielded_effect<D: DB>(
    context: &QueryContext<D>,
    program: Vec<Op<ResultModeVerify, D>>,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    context
        .query(&program, gas_limit, cost_model)
        .map_err(|error| CompactError::LedgerQueryRejected(format!("{error:?}")))
}

pub fn query_kernel_mint_unshielded<D: DB>(
    context: &QueryContext<D>,
    domain: HashOutput,
    amount: u64,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    query_unshielded_effect(
        context,
        kernel_mint_unshielded_program(domain, amount),
        gas_limit,
        cost_model,
    )
}

pub fn query_kernel_inc_unshielded_inputs<D: DB>(
    context: &QueryContext<D>,
    token: TokenType,
    amount: u128,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    query_unshielded_effect(
        context,
        kernel_inc_unshielded_inputs_program(token, amount),
        gas_limit,
        cost_model,
    )
}

pub fn query_kernel_inc_unshielded_outputs<D: DB>(
    context: &QueryContext<D>,
    token: TokenType,
    amount: u128,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    query_unshielded_effect(
        context,
        kernel_inc_unshielded_outputs_program(token, amount),
        gas_limit,
        cost_model,
    )
}

pub fn query_kernel_claim_unshielded_coin_spend<D: DB>(
    context: &QueryContext<D>,
    token: TokenType,
    recipient: PublicAddress,
    amount: u128,
    gas_limit: Option<RunningCost>,
    cost_model: &CostModel,
) -> Result<QueryResults<ResultModeVerify, D>, CompactError> {
    query_unshielded_effect(
        context,
        kernel_claim_unshielded_coin_spend_program(token, recipient, amount),
        gas_limit,
        cost_model,
    )
}

pub(crate) fn kernel_claim_program<M: ResultMode<D>, D: DB>(claim: KernelClaim) -> Vec<Op<M, D>> {
    let (index, value) = match claim {
        KernelClaim::Nullifier(value) => (0_u8, AlignedValue::from(value)),
        KernelClaim::CoinSpend(value) => (2, AlignedValue::from(value)),
        KernelClaim::CoinReceive(value) => (1, AlignedValue::from(value)),
    };
    vec![
        Op::Swap { n: 0 },
        Op::Idx {
            cached: true,
            push_path: true,
            path: vec![Key::Value(AlignedValueChecked::from(index))].into(),
        },
        Op::Push {
            storage: false,
            value: super::vm::literal(value),
        },
        Op::Push {
            storage: false,
            value: StateValue::Null,
        },
        Op::Ins { cached: true, n: 2 },
        Op::Swap { n: 0 },
    ]
}

pub(crate) fn kernel_claim_contract_call_program<M: ResultMode<D>, D: DB>(
    address: ContractAddress,
    entry_point_hash: HashOutput,
    communication_commitment: Field,
) -> Vec<Op<M, D>> {
    let claim = AlignedValue::concat(
        [
            AlignedValue::from(address),
            AlignedValue::from(entry_point_hash),
            AlignedValue::from(communication_commitment),
        ]
        .iter(),
    );
    // Match Kernel.claimContractCall. The current list size is the claim's
    // position, so repeated claims retain their ordering in the effects.
    vec![
        Op::Swap { n: 0 },
        Op::Idx {
            cached: true,
            push_path: true,
            path: vec![Key::Value(AlignedValueChecked::from(3_u8))].into(),
        },
        Op::Dup { n: 0 },
        Op::Size,
        Op::Push {
            storage: false,
            value: super::vm::literal(claim),
        },
        Op::Concat {
            cached: true,
            n: 160,
        },
        Op::Push {
            storage: false,
            value: StateValue::Null,
        },
        Op::Ins { cached: true, n: 2 },
        Op::Swap { n: 0 },
    ]
}

pub(crate) fn kernel_mint_shielded_program<M: ResultMode<D>, D: DB>(
    domain: HashOutput,
    amount: u64,
) -> Vec<Op<M, D>> {
    vec![
        Op::Swap { n: 0 },
        Op::Idx {
            cached: true,
            push_path: true,
            path: vec![Key::Value(AlignedValueChecked::from(4_u8))].into(),
        },
        Op::Push {
            storage: false,
            value: super::vm::literal(domain),
        },
        Op::Dup { n: 1 },
        Op::Dup { n: 1 },
        Op::Member,
        Op::Push {
            storage: false,
            value: super::vm::literal(amount),
        },
        Op::Swap { n: 0 },
        Op::Neg,
        Op::Branch { skip: 4 },
        Op::Dup { n: 2 },
        Op::Dup { n: 2 },
        Op::Idx {
            cached: true,
            push_path: false,
            path: vec![Key::Stack].into(),
        },
        Op::Add,
        Op::Ins { cached: true, n: 2 },
        Op::Swap { n: 0 },
    ]
}

// The four unshielded effects share the exact additive map update emitted by
// Compact 0.36.0. Keep the VM program intact: it charges gas, checks overflow,
// and records the public transcript used by the ledger verifier.
fn unshielded_effect_program<M: ResultMode<D>, D: DB>(
    index: u8,
    key: AlignedValue,
    amount: AlignedValue,
) -> Vec<Op<M, D>> {
    vec![
        Op::Swap { n: 0 },
        Op::Idx {
            cached: true,
            push_path: true,
            path: vec![Key::Value(AlignedValueChecked::from(index))].into(),
        },
        Op::Push {
            storage: false,
            value: super::vm::literal(key),
        },
        Op::Dup { n: 1 },
        Op::Dup { n: 1 },
        Op::Member,
        Op::Push {
            storage: false,
            value: super::vm::literal(amount),
        },
        Op::Swap { n: 0 },
        Op::Neg,
        Op::Branch { skip: 4 },
        Op::Dup { n: 2 },
        Op::Dup { n: 2 },
        Op::Idx {
            cached: true,
            push_path: false,
            path: vec![Key::Stack].into(),
        },
        Op::Add,
        Op::Ins { cached: true, n: 2 },
        Op::Swap { n: 0 },
    ]
}

pub(crate) fn kernel_mint_unshielded_program<M: ResultMode<D>, D: DB>(
    domain: HashOutput,
    amount: u64,
) -> Vec<Op<M, D>> {
    unshielded_effect_program(5, domain.into(), amount.into())
}

pub(crate) fn kernel_inc_unshielded_inputs_program<M: ResultMode<D>, D: DB>(
    token: TokenType,
    amount: u128,
) -> Vec<Op<M, D>> {
    unshielded_effect_program(6, token.into(), amount.into())
}

pub(crate) fn kernel_inc_unshielded_outputs_program<M: ResultMode<D>, D: DB>(
    token: TokenType,
    amount: u128,
) -> Vec<Op<M, D>> {
    unshielded_effect_program(7, token.into(), amount.into())
}

pub(crate) fn kernel_claim_unshielded_coin_spend_program<M: ResultMode<D>, D: DB>(
    token: TokenType,
    recipient: PublicAddress,
    amount: u128,
) -> Vec<Op<M, D>> {
    let key = ClaimedUnshieldedSpendsKey(token, recipient);
    unshielded_effect_program(8, key.into(), amount.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::constructor_cell;
    use crate::ledger::{
        ChargedState, ContractAddress, DefaultDB, UnshieldedTokenType, UserAddress,
    };

    fn empty_query() -> QueryContext<DefaultDB> {
        QueryContext::new(
            ChargedState::new(StateValue::<DefaultDB>::Array(
                vec![constructor_cell(false)].into(),
            )),
            ContractAddress::default(),
        )
    }

    fn token() -> TokenType {
        TokenType::Unshielded(UnshieldedTokenType(HashOutput([2; 32])))
    }

    #[test]
    fn compact_either_conversion_selects_only_the_active_sdk_variant() {
        use crate::FixedBytes;
        use crate::ledger::{
            ShieldedTokenType, token_type_from_compact, unshielded_recipient_from_compact,
        };
        assert_eq!(
            token_type_from_compact(true, FixedBytes([2; 32]), FixedBytes([9; 32])),
            token()
        );
        assert_eq!(
            token_type_from_compact(false, FixedBytes([9; 32]), FixedBytes([2; 32])),
            TokenType::Shielded(ShieldedTokenType(HashOutput([2; 32])))
        );
        assert_eq!(
            unshielded_recipient_from_compact(true, FixedBytes([3; 32]), FixedBytes([9; 32]),),
            PublicAddress::Contract(ContractAddress(HashOutput([3; 32])))
        );
        assert_eq!(
            unshielded_recipient_from_compact(false, FixedBytes([9; 32]), FixedBytes([3; 32]),),
            PublicAddress::User(UserAddress(HashOutput([3; 32])))
        );
    }

    #[test]
    fn unshielded_effect_programs_use_the_canonical_effect_slots_and_aligned_keys() {
        let token = token();
        let contract = PublicAddress::Contract(ContractAddress(HashOutput([3; 32])));
        let programs = [
            kernel_mint_unshielded_program::<ResultModeVerify, DefaultDB>(HashOutput([1; 32]), 7),
            kernel_inc_unshielded_inputs_program::<ResultModeVerify, DefaultDB>(token, 7),
            kernel_inc_unshielded_outputs_program::<ResultModeVerify, DefaultDB>(token, 7),
            kernel_claim_unshielded_coin_spend_program::<ResultModeVerify, DefaultDB>(
                token, contract, 7,
            ),
        ];
        for (slot, program) in (5_u8..=8).zip(programs) {
            assert_eq!(program.len(), 16);
            assert!(matches!(program[0], Op::Swap { n: 0 }));
            let Op::Idx {
                cached: true,
                push_path: true,
                ref path,
            } = program[1]
            else {
                panic!("effect update must select a cached Effects map");
            };
            assert_eq!(
                Vec::<Key>::from(path),
                vec![Key::Value(AlignedValueChecked::from(slot))]
            );
            assert!(matches!(program[5], Op::Member));
            assert!(matches!(program[9], Op::Branch { skip: 4 }));
            assert!(matches!(program[13], Op::Add));
            assert!(matches!(program[14], Op::Ins { cached: true, n: 2 }));
            assert!(matches!(program[15], Op::Swap { n: 0 }));
        }
        let claim = kernel_claim_unshielded_coin_spend_program::<ResultModeVerify, DefaultDB>(
            token, contract, 7,
        );
        let Op::Push { value, .. } = &claim[2] else {
            panic!("claim must push the aligned composite key");
        };
        assert_eq!(
            value,
            &super::super::vm::literal::<_, DefaultDB>(AlignedValue::from(
                ClaimedUnshieldedSpendsKey(token, contract)
            ))
        );
    }

    #[test]
    fn unshielded_effects_accumulate_per_key_without_changing_contract_state() {
        let query = empty_query();
        let model = &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
        let domain = HashOutput([1; 32]);
        let token = token();
        let contract = PublicAddress::Contract(ContractAddress(HashOutput([3; 32])));
        let user = PublicAddress::User(UserAddress(HashOutput([4; 32])));
        let query = query_kernel_mint_unshielded(&query, domain, 7, None, model)
            .unwrap()
            .context;
        let query = query_kernel_mint_unshielded(&query, domain, 5, None, model)
            .unwrap()
            .context;
        let query = query_kernel_inc_unshielded_inputs(&query, token, 3, None, model)
            .unwrap()
            .context;
        let query = query_kernel_inc_unshielded_inputs(&query, token, 4, None, model)
            .unwrap()
            .context;
        let query = query_kernel_inc_unshielded_outputs(&query, token, 2, None, model)
            .unwrap()
            .context;
        let query =
            query_kernel_claim_unshielded_coin_spend(&query, token, contract, 5, None, model)
                .unwrap()
                .context;
        let query =
            query_kernel_claim_unshielded_coin_spend(&query, token, contract, 6, None, model)
                .unwrap()
                .context;
        let query = query_kernel_claim_unshielded_coin_spend(&query, token, user, 1, None, model)
            .unwrap()
            .context;
        assert_eq!(*query.effects.unshielded_mints.get(&domain).unwrap(), 12);
        assert_eq!(*query.effects.unshielded_inputs.get(&token).unwrap(), 7);
        assert_eq!(*query.effects.unshielded_outputs.get(&token).unwrap(), 2);
        assert_eq!(
            *query
                .effects
                .claimed_unshielded_spends
                .get(&ClaimedUnshieldedSpendsKey(token, contract))
                .unwrap(),
            11
        );
        assert_eq!(
            *query
                .effects
                .claimed_unshielded_spends
                .get(&ClaimedUnshieldedSpendsKey(token, user))
                .unwrap(),
            1
        );
        assert_eq!(query.effects.claimed_unshielded_spends.size(), 2);
        assert_eq!(query.state, empty_query().state);
        assert!(query.effects.shielded_mints.is_empty());
    }

    #[test]
    fn unshielded_overflow_refuses_each_map_without_mutating_prior_context() {
        let model = &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
        let domain = HashOutput([1; 32]);
        let token = token();
        let recipient = PublicAddress::Contract(ContractAddress(HashOutput([3; 32])));
        let original = empty_query();
        let minted = query_kernel_mint_unshielded(&original, domain, u64::MAX, None, model)
            .unwrap()
            .context;
        assert!(
            query_kernel_mint_unshielded(&minted, domain, 1, None, model)
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("overflow")
        );
        assert_eq!(
            *minted.effects.unshielded_mints.get(&domain).unwrap(),
            u64::MAX
        );
        assert!(original.effects.unshielded_mints.is_empty());

        let inputs = query_kernel_inc_unshielded_inputs(&original, token, u128::MAX, None, model)
            .unwrap()
            .context;
        assert!(query_kernel_inc_unshielded_inputs(&inputs, token, 1, None, model).is_err());
        assert_eq!(
            *inputs.effects.unshielded_inputs.get(&token).unwrap(),
            u128::MAX
        );

        let outputs = query_kernel_inc_unshielded_outputs(&original, token, u128::MAX, None, model)
            .unwrap()
            .context;
        assert!(query_kernel_inc_unshielded_outputs(&outputs, token, 1, None, model).is_err());
        assert_eq!(
            *outputs.effects.unshielded_outputs.get(&token).unwrap(),
            u128::MAX
        );

        let claims = query_kernel_claim_unshielded_coin_spend(
            &original,
            token,
            recipient,
            u128::MAX,
            None,
            model,
        )
        .unwrap()
        .context;
        assert!(
            query_kernel_claim_unshielded_coin_spend(&claims, token, recipient, 1, None, model,)
                .is_err()
        );
        assert_eq!(
            *claims
                .effects
                .claimed_unshielded_spends
                .get(&ClaimedUnshieldedSpendsKey(token, recipient))
                .unwrap(),
            u128::MAX
        );
    }
    #[test]
    fn canonical_kernel_programs_match_independent_typescript() {
        let rows: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/kernel-shielded-effects-oracle.json"
        ))
        .unwrap();
        for row in rows.as_array().unwrap().iter().filter(|r| {
            ["zero", "seven", "max", "nullifier", "spend", "receive"]
                .contains(&r["label"].as_str().unwrap())
        }) {
            let bytes: [u8; 32] = serde_json::from_value(row["args"][0].clone()).unwrap();
            let value = HashOutput(bytes);
            let program = match row["name"].as_str().unwrap() {
                "mint" => kernel_mint_shielded_program::<ResultModeVerify, DefaultDB>(
                    value,
                    row["args"][1].as_str().unwrap().parse().unwrap(),
                ),
                "nullifier" => kernel_claim_program(KernelClaim::Nullifier(CoinNullifier(value))),
                "spend" => kernel_claim_program(KernelClaim::CoinSpend(CoinCommitment(value))),
                "claim_receive" => {
                    kernel_claim_program(KernelClaim::CoinReceive(CoinCommitment(value)))
                }
                _ => unreachable!(),
            };
            assert_eq!(
                serde_json::to_value(program).unwrap(),
                row["queries"][0]["ops"],
                "{}",
                row["label"]
            );
        }
    }
    #[test]
    fn mint_overflow_rejects_the_second_query_without_replacing_prior_effects() {
        let query = QueryContext::new(
            ChargedState::new(StateValue::<DefaultDB>::Array(
                vec![constructor_cell(false)].into(),
            )),
            ContractAddress::default(),
        );
        let domain = HashOutput([1; 32]);
        let model = &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL;
        let first = query_kernel_mint_shielded(&query, domain, u64::MAX, None, model).unwrap();
        let error = query_kernel_mint_shielded(&first.context, domain, 1, None, model)
            .err()
            .unwrap();
        assert!(matches!(error, CompactError::LedgerQueryRejected(_)));
        assert!(error.to_string().to_lowercase().contains("overflow"));
        assert_eq!(
            *first.context.effects.shielded_mints.get(&domain).unwrap(),
            u64::MAX
        );
        assert_eq!(first.context.state, query.state);
        assert!(query.effects.shielded_mints.is_empty());
    }

    #[test]
    fn upstream_query_rejects_a_malformed_effects_frame() {
        let query = QueryContext::new(
            ChargedState::new(StateValue::<DefaultDB>::Array(
                vec![constructor_cell(false)].into(),
            )),
            ContractAddress::default(),
        );
        let corrupt_effects: Vec<Op<ResultModeVerify, DefaultDB>> = vec![
            Op::Swap { n: 0 },
            Op::Pop,
            Op::Push {
                storage: false,
                value: super::super::vm::literal(false),
            },
            Op::Swap { n: 0 },
        ];
        assert!(matches!(
            query.query(
                &corrupt_effects,
                None,
                &midnight_onchain_vm::cost_model::INITIAL_COST_MODEL
            ),
            Err(super::super::TranscriptRejected::EffectDecodeError)
        ));
    }
}
