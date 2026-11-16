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
assert.ok(entry && ['manual_update', 'witness_return'].includes(name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const context = (contract) => {
  const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
  return runtime.createCircuitContext(
    runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState,
  );
};

if (name === 'manual_update') {
  const contract = new generated.Contract({});
  const execution = contract.circuits.bytes_manual_update_semantics(context(contract));
  assert.deepStrictEqual(Array.from(execution.result), [1, 4, 14, 50]);
  assert.deepStrictEqual(
    Array.from(generated.ledger(execution.context.currentQueryContext.state).manual_update_result),
    [1, 4, 14, 50],
  );
  assert.ok(execution.proofData);
} else {
  let calls = 0;
  const good = new generated.Contract({
    bytes_witness: ({ privateState }) => {
      calls++;
      return [privateState, Uint8Array.from([10, 20, 30, 40])];
    },
  });
  const execution = good.circuits.bytes_witness_return_semantics(context(good));
  assert.deepStrictEqual(execution.result, []);
  assert.equal(execution.context.currentPrivateState, 3);
  assert.equal(execution.proofData.privateTranscriptOutputs.length, 1);
  assert.equal(calls, 1);
  for (const [index, value, message] of [
    [0, 11, /Witness byte 0 should be 10/],
    [1, 21, /Witness byte 1 should be 20/],
    [3, 41, /Witness byte 3 should be 40/],
  ]) {
    const bytes = Uint8Array.from([10, 20, 30, 40]);
    bytes[index] = value;
    const bad = new generated.Contract({
      bytes_witness: ({ privateState }) => [privateState, bytes],
    });
    assert.throws(() => bad.circuits.bytes_witness_return_semantics(context(bad)), message);
  }
}
console.log(`${name}: original Bytes semantics passed`);
