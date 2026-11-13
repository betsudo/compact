// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [entry, callbackPath] = process.argv.slice(2).map((path) => resolve(path));
assert(entry && callbackPath);
const code = await import(pathToFileURL(entry));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const callbacks = [];
const test = (_name, callback) => callbacks.push(callback);
const expect = (value) => ({ toEqual: (expected) => assert.deepStrictEqual(value, expected) });
async function startContract(contractCode) {
  const contract = new contractCode.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const circuits = new Proxy({}, { get: (_, name) => (...args) => {
    const result = contract.circuits[name](...args);
    // Mainstream's async call context encloses this branch's synchronous context.
    return { ...result, context: { callContext: result.context } };
  } });
  return [{ circuits }, context];
}
const callback = await readFile(callbackPath, 'utf8');
new Function('test', 'expect', 'startContract', 'contractCode', 'runtime', callback)(
  test, expect, startContract, code, runtime);
assert.equal(callbacks.length, 1, 'exactly one upstream callback expected');
await callbacks[0]();
console.log('original upstream callback passed');
