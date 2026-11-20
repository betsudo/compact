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
assert.ok(entry && name && process.argv.length === 4);
const cases = new Map([
  ['default/semantics', { expected: Array(128).fill(0), state: 'default_result' }],
  ['equality/empty', { expected: true, state: 'equality_result' }],
  ['equality/semantics', { expected: true, pure: true }],
  ['indexing/basic', { expected: [10n, 50n, 80n], state: 'indexing_result' }],
  ['pad/semantics', { expected: [97, 98, 0, 0], pure: true }],
  ['slice/basic', { expected: [5], state: 'slice_result' }],
  ['slice/empty', { expected: [], state: 'slice_result' }],
  ['spread/basic', { expected: [0, 255, 255, 255, 128, 0, 255, 255, 255, 128], state: 'spread_result' }],
  ['spread/empty', { expected: [], state: 'spread_result' }],
  ['spread/multiple', { expected: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], state: 'spread_result' }],
  ['spread/nested', { expected: [1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3], state: 'spread_result' }],
]);
assert.ok(cases.has(name), `unexpected Bytes case ${name}`);
const { expected, state, pure } = cases.get(name);
const generated = await import(pathToFileURL(entry).href);
const circuit = `bytes_${name.replace('/', '_')}`;
const check = (actual) => Array.isArray(expected)
  ? assert.deepStrictEqual(Array.from(actual), expected)
  : assert.equal(actual, expected);
if (pure) {
  check(generated.pureCircuits[circuit]());
} else {
  const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
  const runtime = await import(pathToFileURL(runtimeEntry).href);
  const contract = new generated.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = runtime.createCircuitContext(
    runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState,
  );
  const execution = contract.circuits[circuit](context);
  check(execution.result);
  check(generated.ledger(execution.context.currentQueryContext.state)[state]);
  assert.ok(execution.proofData);
}
console.log(`${name}: original assertion and state passed`);
