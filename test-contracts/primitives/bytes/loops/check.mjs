// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
const expected = { empty_iteration: 0n, spread_iteration: 0n, iteration: 120n,
  indexed_access: 32n };
assert.ok(entry && Object.hasOwn(expected, name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const call = contract.circuits[`bytes_loop_${name}`](context);
assert.strictEqual(call.result, expected[name]);
assert.strictEqual(generated.ledger(call.context.currentQueryContext.state).loop_result, expected[name]);
if (expected[name] === 0n) {
  assert.deepStrictEqual(call.context.currentQueryContext.state.state.encode(),
    context.currentQueryContext.state.state.encode());
}
assert.ok(call.proofData);
console.log(`${name}: original Bytes loop result and ledger passed`);
