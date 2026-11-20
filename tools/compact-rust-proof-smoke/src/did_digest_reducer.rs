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

use super::{
    local_v3_application as v3, positional_transaction::verify_separately, qualified_coin_funding,
};
use compact_rust_did_digest_read_reducer_fixture::{
    ledger_contract as c, ledger_slots, runtime as r,
};
use midnight_transient_crypto::curve::Fr;
use r::{
    context::{ConstructorContext, WitnessContext},
    fab::AlignedValue,
    transaction::{CallSpec, Observation, ObservedContractState, prepare_call},
};
use rand::{SeedableRng, rngs::StdRng};
use std::{error::Error, fs, path::Path};

struct Witness;
impl c::TryWitnesses<u64> for Witness {
    fn admitted(
        &self,
        context: WitnessContext<'_, u64, c::LedgerView<'_>>,
    ) -> Result<(u64, bool), r::CompactError> {
        Ok((*context.private_state + 1, true))
    }
}

pub(super) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    const NAME: &str = "verify";
    const STEM: &str = "reducer-final";
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("DID digest reducer qualification requires local proving".into());
    }
    let mut rng = StdRng::seed_from_u64(0x288);
    let point = r::ec_mul_generator(r::Field::from(5u64))?;
    let id = r::OpaqueString("reducer-renamed".to_owned());
    let initial = c::initial_state(ConstructorContext::new(0u64), id.clone(), point)?;
    let constructor_data = initial.ledger_state.get_ref().clone();
    if !ledger_slots::active.inspect(&constructor_data)?
        || ledger_slots::methods.inspect(&constructor_data)?.is_empty()
    {
        return Err("constructor did not seed active declared Map".into());
    }
    let (_, verifier, operation) = v3::operation(root, NAME, 7)?;
    let deploy = v3::deploy(constructor_data.clone(), NAME, operation.clone(), &mut rng);
    let address = deploy.address();
    let mut funded = qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 2)?;
    let resolver = qualified_coin_funding::fee_resolver(root, NAME)?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    v3::apply_funded_deployment(
        root,
        "reducer",
        &deploy,
        &mut funded,
        &mut rng,
        &executor,
        &resolver,
    )?;
    let prior = funded
        .ledger
        .contract
        .get(&address)
        .ok_or("deployed contract absent")?;
    if prior.data.get_ref() != &constructor_data {
        return Err("deployment changed constructor data".into());
    }
    let observed = ObservedContractState::new(
        address,
        prior.clone(),
        Observation {
            transaction_hash: [0; 32],
            block_hash: [0; 32],
            block_height: 1,
        },
    );
    let witness = Witness;
    let native = c::verify(observed.circuit_context(0u64), &witness, id.clone(), point)?;
    let recorded =
        c::recorded::verify(observed.circuit_context(0u64), &witness, id.clone(), point)?;
    if native.context.query.state != recorded.execution.context.query.state
        || native.gas_cost != recorded.execution.gas_cost
        || native.private_transcript_outputs != recorded.execution.private_transcript_outputs
    {
        return Err("native/recorded mismatch".into());
    }
    let input = AlignedValue::from((id.clone(), point));
    let manual = prepare_call(
        recorded,
        CallSpec::new("verify", verifier.clone(), input, Fr::from(0u64)),
    )?;
    let facade = c::Contract::from(Witness);
    let mut prepared = facade
        .recording()
        .verify_call(&observed, 0u64, id, point)?
        .prepare(verifier.clone(), Fr::from(0u64))?;
    if format!("{manual:?}") != format!("{prepared:?}") {
        return Err("observed/manual call mismatch".into());
    }
    prepared.op = operation;
    let proof = v3::prove_standalone(root, NAME, STEM, &prepared, &verifier, None)?;
    verify_separately(root, NAME, STEM)?;
    let (transaction_proof_bytes, _) = v3::apply_funded_call(
        root,
        (NAME, STEM),
        prepared,
        &mut funded,
        &mut rng,
        &proof,
        |state| {
            if state.data.get_ref() != &constructor_data {
                return Err("read-only reducer changed contract data".into());
            }
            Ok(())
        },
    )?;
    let final_state_bytes = fs::metadata(root.join("reducer-final-state.bin"))?.len();
    let receipt = serde_json::json!({
        "format": "compact-did-digest-reducer-proof/v2",
        "status": "passed", "strictness": "default",
        "constructor_execution_proved": false, "deployed_constructor_data": true,
        "proof_bytes": proof.0.len(), "transaction_proof_bytes": transaction_proof_bytes,
        "transaction_proof_version": 3, "funded_dust_spends": 1,
        "separate_verification": true, "corrupt_proof_rejected": true,
        "changed_binding_rejected": true, "applied": true, "read_only_contract_data": true,
        "replay_refusal": "IntentAlreadyExists", "replay_unchanged": true,
        "final_state_bytes": final_state_bytes
    });
    v3::write_new(
        root.join("reducer-proof-result.json"),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    println!("reducer strict proof passed: {receipt}");
    Ok(())
}
