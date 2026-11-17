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
const call = contract.circuits.bytes_loop_indexed_access(context);
assert.equal(call.result, 32n);
assert.equal(generated.ledger(call.context.currentQueryContext.state).loop_result, 32n);
assert.equal(call.proofData.publicTranscript.length, 54);
assert.equal(call.proofData.privateTranscriptOutputs.length, 0);
assert.deepStrictEqual(call.context.gasCost, {
  readTime: 2380000000n,
  computeTime: 22370692016n,
  bytesWritten: 290n,
  bytesDeleted: 288n,
});
console.log('indexed_access: original result, ledger, transcript and gas passed');
