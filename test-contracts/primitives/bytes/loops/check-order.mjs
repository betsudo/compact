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

const [entry] = process.argv.slice(2);
assert.ok(entry);
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const calls = [];
const contract = new generated.Contract({
  initial_operand: ({ privateState }) => { calls.push('initial'); return [privateState + 1, 2n]; },
  empty_operand: ({ privateState }) => { calls.push('source'); return [privateState + 10, new Uint8Array()]; },
  unreachable_reducer: () => { throw new Error('zero-length reducer executed'); },
});
const initial = contract.initialState(runtime.createConstructorContext(0, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const call = contract.circuits.zero_operand_order(context);
assert.strictEqual(call.result, 0n);
assert.strictEqual(call.context.currentPrivateState, 11);
assert.deepStrictEqual(calls, ['initial', 'source']);
assert.strictEqual(call.proofData.privateTranscriptOutputs.length, 2);
console.log('zero_operand_order: initial and source evaluated in order; reducer unreachable');
