// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[derive(Default)]
struct KernelStamps {
    calls: RefCell<Vec<(u8, u64, Field)>>,
    fail: Option<u8>,
}
impl kernel_fixture::ledger_contract::TryWitnesses<u64> for KernelStamps {
    fn stamp(
        &self,
        context: WitnessContext<'_, u64, kernel_fixture::ledger_contract::LedgerView<'_>>,
        step: runtime::BoundedUint<255>,
    ) -> Result<(u64, ()), CompactError> {
        let step = step.value() as u8;
        self.calls
            .borrow_mut()
            .push((step, *context.private_state, context.ledger.marker()?));
        if self.fail == Some(step) {
            return Err(CompactError::AssertionFailed(format!(
                "stamp {step} refused"
            )));
        }
        Ok((*context.private_state + 1, ()))
    }
}
macro_rules! kernel_case {
    ($identity:expr,$selected:expr,$permit:expr,$amount:expr,$fail:expr,$error:expr) => {{
        use kernel_fixture::ledger_contract as c;
        let identity: Option<[u8; 32]> = $identity;
        let address = runtime::ledger::ContractAddress(runtime::ledger::HashOutput([23; 32]));
        let context = || {
            let ctx = c::initial_state(ConstructorContext::new(7_u64))
                .unwrap()
                .into_circuit_context(address);
            match identity {
                Some(key) => ctx.with_coin_public_key_bytes(key),
                None => ctx,
            }
        };
        let witnesses: [KernelStamps; 4] = std::array::from_fn(|_| KernelStamps {
            fail: $fail,
            ..Default::default()
        });
        let value = runtime::FixedBytes::new([11; 32]);
        let amount = runtime::BoundedUint::<18446744073709551615>::new($amount).unwrap();
        let result = outcomes(
            c::exercise(context(), &witnesses[0], $selected, $permit, value, amount),
            c::recorded::exercise(context(), &witnesses[1], $selected, $permit, value, amount),
            c::functional::exercise::Program::new($selected, $permit, value, amount)
                .execute_native(context(), &witnesses[2]),
            c::functional::exercise::Program::new($selected, $permit, value, amount)
                .execute_recorded(context(), &witnesses[3]),
            $error,
        );
        (result, witnesses)
    }};
}
#[test]
fn kernel_identity_claims_mint_and_unit_witness_outputs_keep_official_domains() {
    for (identity, selected, amount) in [
        (None, false, 13_u128),
        (Some([0; 32]), true, 0),
        (Some([19; 32]), true, u64::MAX as u128),
        (Some([19; 32]), false, 13),
    ] {
        let (result, witnesses) = kernel_case!(identity, selected, true, amount, None, None);
        let result = result.unwrap();
        assert_eq!(result.result.address.bytes.0, [23; 32]);
        assert_eq!(
            result.result.key.bytes.0,
            if selected { identity.unwrap() } else { [0; 32] }
        );
        assert_eq!(
            kernel_fixture::ledger_slots::marker
                .inspect(result.context.query.state.get_ref())
                .unwrap(),
            Field::from(9_u64)
        );
        assert_eq!(result.context.private_state, 10);
        let mut private = vec![runtime::fab::AlignedValue::from(())];
        if selected {
            private.push(runtime::fab::AlignedValue::from(identity.unwrap()));
        }
        private.extend([
            runtime::fab::AlignedValue::from(()),
            runtime::fab::AlignedValue::from(()),
        ]);
        assert_eq!(result.private_transcript_outputs, private);
        for witness in witnesses {
            assert_eq!(
                *witness.calls.borrow(),
                vec![
                    (1, 7, Field::from(1_u64)),
                    (2, 8, Field::from(1_u64)),
                    (3, 9, Field::from(1_u64))
                ]
            );
        }
        let effects = &result.context.query.effects;
        let hash = runtime::ledger::HashOutput([11; 32]);
        assert_eq!(effects.claimed_nullifiers.size(), 1);
        assert!(
            effects
                .claimed_nullifiers
                .member(&runtime::ledger::CoinNullifier(hash))
        );
        assert_eq!(effects.claimed_shielded_spends.size(), 1);
        assert!(
            effects
                .claimed_shielded_spends
                .member(&runtime::ledger::CoinCommitment(hash))
        );
        assert_eq!(effects.claimed_shielded_receives.size(), 1);
        assert!(
            effects
                .claimed_shielded_receives
                .member(&runtime::ledger::CoinCommitment(hash))
        );
        assert_eq!(effects.shielded_mints.size(), 1);
        assert_eq!(*effects.shielded_mints.get(&hash).unwrap(), amount as u64);
        assert!(effects.claimed_contract_calls.is_empty());
        assert!(effects.unshielded_mints.is_empty());
    }
}
#[test]
fn kernel_guard_missing_identity_and_callback_errors_keep_exact_prefix() {
    for (identity, permit, fail, prefix, error) in [
        (
            Some([19; 32]),
            false,
            None,
            0,
            "failed assert: kernel denied",
        ),
        (
            None,
            true,
            None,
            1,
            "native ownPublicKey requires a coin public key in the circuit context",
        ),
        (
            Some([19; 32]),
            true,
            Some(2),
            2,
            "failed assert: stamp 2 refused",
        ),
        (
            Some([19; 32]),
            true,
            Some(3),
            3,
            "failed assert: stamp 3 refused",
        ),
    ] {
        let (result, witnesses) = kernel_case!(identity, true, permit, 13, fail, Some(error));
        assert!(result.is_none());
        let expected: Vec<_> = (1..=prefix)
            .map(|step| (step, 6 + u64::from(step), Field::from(1_u64)))
            .collect();
        for witness in witnesses {
            assert_eq!(*witness.calls.borrow(), expected);
        }
        // Err carries no partial public frame: prefix writes/effects are NOT claimed observable.
    }
}
