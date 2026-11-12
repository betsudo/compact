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
use midnight_compact_runtime as r;
use midnight_ledger::structure::ContractDeploy;
use midnight_onchain_state::state::{ContractMaintenanceAuthority, ContractState, EntryPointBuf};
use midnight_storage::storage::HashMap;
use midnight_transient_crypto::curve::Fr;
use r::transaction::{CallSpec, Observation, ObservedContractState, prepare_call};
use rand::{SeedableRng, rngs::StdRng};
use std::{error::Error, fs, path::Path};

#[path = "did_primitive_reducers/cases.rs"]
mod cases;

pub(crate) fn run(root: &Path, kind: &str) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("MIDNIGHT_PROOF_SERVER").is_some() {
        return Err("DID reducer qualification requires local proving".into());
    }
    let fixture = cases::fixture(kind)?;
    let mut rng = StdRng::seed_from_u64(0x294);
    let constructor_data = (fixture.initial)()?.ledger_state.get_ref().clone();
    let mut operations = HashMap::new();
    for name in fixture.operations {
        let (_, _, operation) = v3::load_operation(root, name)?;
        operations = operations.insert(EntryPointBuf(name.as_bytes().to_vec()), operation);
    }
    let deploy = ContractDeploy::new(
        &mut rng,
        ContractState::new(
            constructor_data.clone(),
            operations,
            ContractMaintenanceAuthority::default(),
        ),
    );
    let address = deploy.address();
    let mut funded =
        qualified_coin_funding::fee_funded_state_with_utxos(&mut rng, 1 + fixture.cases.len())?;
    let resolver = qualified_coin_funding::fee_resolver(root, fixture.operations[0])?;
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
    if funded
        .ledger
        .contract
        .get(&address)
        .ok_or("missing deployment")?
        .data
        .get_ref()
        != &constructor_data
    {
        return Err("constructor data mismatch".into());
    }
    let mut results = vec![];
    for selected in fixture.cases {
        let case = selected.id;
        let operation = selected.operation;
        let (_, verifier, operation_artifact) = v3::load_operation(root, operation)?;
        let prior = funded
            .ledger
            .contract
            .get(&address)
            .ok_or("missing contract")?;
        let observed = ObservedContractState::new(
            address,
            prior.clone(),
            Observation {
                transaction_hash: [0; 32],
                block_hash: [0; 32],
                block_height: 1,
            },
        );
        let (recorded, input) = (fixture.call)(selected, &observed)?;
        let expected = recorded.execution.context.query.state.clone();
        let mut prepared = prepare_call(
            recorded,
            CallSpec::new(operation, verifier.clone(), input, Fr::from(0u64)),
        )?;
        prepared.op = operation_artifact;
        let proof = v3::prove_standalone(root, operation, case, &prepared, &verifier, None)?;
        verify_separately(root, operation, case)?;
        let (transaction_proof_bytes, _) = v3::apply_funded_call(
            root,
            (operation, case),
            prepared,
            &mut funded,
            &mut rng,
            &proof,
            |state| {
                if state.data != expected {
                    return Err("applied state does not match recorded state".into());
                }
                Ok(())
            },
        )?;
        let state_bytes = fs::metadata(root.join(format!("{case}-state.bin")))?.len();
        results.push(serde_json::json!({
            "case": case, "operation": operation,
            "proof_bytes": proof.0.len(), "transaction_proof_bytes": transaction_proof_bytes,
            "transaction_proof_version": 3, "funded_dust_spends": 1,
            "separate_verification": true, "corrupt_proof_rejected": true,
            "changed_binding_rejected": true, "applied": true, "recorded_state_matches": true,
            "replay_refusal": "IntentAlreadyExists", "replay_unchanged": true,
            "state_bytes": state_bytes
        }));
    }
    let result = serde_json::json!({
        "format": "compact-did-primitive-reducer-proof/v2", "kind": kind,
        "strictness": "default", "constructor_data_deployed": true,
        "constructor_execution_proved": false, "status": "passed", "calls": results
    });
    v3::write_new(
        root.join("proof-result.json"),
        &serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
