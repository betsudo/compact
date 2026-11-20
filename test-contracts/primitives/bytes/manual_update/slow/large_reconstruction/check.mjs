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
import { writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

const entry = process.argv[2];
assert.ok(entry);
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const call = contract.circuits.bytes_manual_update_large_reconstruction(context);
const expected = Uint8Array.from([1, 4, 9, 16, 25, 36, 49, 14, 31, 50]);
assert.deepStrictEqual(call.result, expected);
assert.deepStrictEqual(
  generated.ledger(call.context.currentQueryContext.state).manual_update_large_result,
  expected,
);
assert.ok(call.proofData);
assert.strictEqual(call.proofData.publicTranscript.length, 315);
if (process.argv[3]) {
  await writeFile(process.argv[3], JSON.stringify({
    result: [...call.result],
    ledger: [...generated.ledger(call.context.currentQueryContext.state).manual_update_large_result],
    public_operations: call.proofData.publicTranscript.length,
    private_operations: call.proofData.privateTranscriptOutputs.length,
    gas: Object.fromEntries(Object.entries(call.context.gasCost)
      .map(([key, value]) => [key, value.toString()])),
  }, null, 2) + '\n');
}
if (process.argv[4]) {
  await writeFile(process.argv[4], JSON.stringify(call.proofData.publicTranscript,
    (_, value) => {
      if (value instanceof Uint8Array) return [...value];
      if (value instanceof Map || typeof value === 'bigint') {
        throw new Error('Uncertified public transcript value shape');
      }
      return value;
    }) + '\n');
}
console.log('large_reconstruction: original result, ledger, and 315 operations passed');
