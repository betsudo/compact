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
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath] = process.argv.slice(2);
assert.ok(contractPath && runtimePath);
const { Contract } = await import(pathToFileURL(contractPath).href);
const { createCircuitContext, createConstructorContext, dummyContractAddress } =
  await import(pathToFileURL(`${runtimePath}/dist/index.js`).href);

function context(contract) {
  const initial = contract.initialState(createConstructorContext(3, '0'.repeat(64)));
  return createCircuitContext(
    dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState,
  );
}

let calls = 0;
const good = new Contract({
  vector_witness: ({ privateState }) => {
    calls++;
    return [privateState, [5n, 6n, 7n, 8n]];
  },
});
const success = good.circuits.vector_witness_return_semantics(context(good));
assert.deepStrictEqual(success.result, []);
assert.strictEqual(calls, 1);
assert.strictEqual(success.context.currentPrivateState, 3);
assert.strictEqual(success.proofData.privateTranscriptOutputs.length, 1);

const wrong = new Contract({
  vector_witness: ({ privateState }) => [privateState, [5n, 6n, 7n, 9n]],
});
assert.throws(
  () => wrong.circuits.vector_witness_return_semantics(context(wrong)),
  /Witness element 3 should be 8/,
);
console.log('original Vector witness return semantics passed');
