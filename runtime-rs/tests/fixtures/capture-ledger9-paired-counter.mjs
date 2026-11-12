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

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [contractPath] = process.argv.slice(2);
assert.ok(contractPath, 'expected the generated contract entry');
const entry = pathToFileURL(resolve(contractPath));
const fromContract = createRequire(entry);
const runtimeEntry = fromContract.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const fromRuntime = createRequire(runtimeEntry);
const onchainEntry = fromRuntime.resolve('@midnightntwrk/onchain-runtime-v4');
const onchain = await import(pathToFileURL(onchainEntry));
const packageVersion = (path) => JSON.parse(readFileSync(path, 'utf8')).version;
assert.equal(packageVersion(fromContract.resolve('@midnight-ntwrk/compact-runtime/package.json')), '0.17.0');
assert.equal(packageVersion(resolve(dirname(onchainEntry), 'package.json')), '4.0.1');
assert.equal(runtime.ContractState, onchain.ContractState);
assert.equal(runtime.QueryContext, onchain.QueryContext);

const { Contract, ledger } = await import(entry);
const contract = new Contract({});
const coinPublicKey = { bytes: new Uint8Array(32) };
const initial = contract.initialState({
  initialPrivateState: null,
  initialZswapLocalState: runtime.emptyZswapLocalState(coinPublicKey),
});
const state = initial.currentContractState;
assert.ok(state instanceof onchain.ContractState);
const serialize = () => Buffer.from(state.serialize()).toString('hex');
const before = serialize();
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), coinPublicKey, state.data, null);
const outcome = contract.circuits.increment(context);
assert.ok(outcome.context.currentQueryContext instanceof onchain.QueryContext);
state.data = new runtime.ChargedState(outcome.context.currentQueryContext.state.state);
const value = ledger(state.data).round;
assert.equal(value, 1n);
assert.deepEqual(outcome.result, []);

const actual = { before, after: serialize(), value: value.toString(), result: outcome.result };
const official = JSON.parse(readFileSync(new URL('ledger9-counter-oracle.json', import.meta.url), 'utf8'));
assert.deepEqual(actual, official);
const gas = Object.fromEntries(Object.entries(outcome.gasCost).map(([field, cost]) => [field, cost.toString()]));
console.log(JSON.stringify({ ...actual, gas }, null, 2));
