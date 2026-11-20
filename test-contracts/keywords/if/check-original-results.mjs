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
assert.ok(entry && ['empty_else_branch', 'branches_write_ledger'].includes(name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);

if (name === 'empty_else_branch') {
  assert.equal(contract.circuits.if_empty_else_branch(context(), true).result, 1n);
  assert.equal(contract.circuits.if_empty_else_branch(context(), false).result, 0n);
} else {
  const initialView = generated.ledger(context().currentQueryContext.state);
  assert.equal(initialView.y, false);
  assert.equal(initialView.set.member(1n), true);
  assert.equal(initialView.set.size(), 1n);
  assert.equal(initialView.caroline.isEmpty(), true);
  const first = contract.circuits.if_branches_write_ledger(context(), true);
  assert.equal(first.result, true);
  const firstView = generated.ledger(first.context.currentQueryContext.state);
  assert.equal(firstView.y, true);
  assert.equal(firstView.caroline.head().value, true);
  assert.equal(first.proofData.publicTranscript.length, 34);
  assert.equal(first.proofData.privateTranscriptOutputs.length, 0);
  assert.deepStrictEqual(first.context.gasCost, {
    readTime: 935000000n,
    computeTime: 5434381661n,
    bytesWritten: 1416n,
    bytesDeleted: 1220n,
  });
  const second = contract.circuits.if_branches_write_ledger(first.context, false);
  assert.equal(second.result, false);
  const secondView = generated.ledger(second.context.currentQueryContext.state);
  assert.equal(secondView.y, false);
  assert.equal(secondView.set.member(1n), true);
  assert.equal(secondView.set.size(), 1n);
  assert.equal(secondView.caroline.head().value, false);
  assert.equal(second.proofData.publicTranscript.length, 34);
  assert.equal(second.proofData.privateTranscriptOutputs.length, 0);
  assert.deepStrictEqual(second.context.gasCost, {
    readTime: 935000000n,
    computeTime: 5434381661n,
    bytesWritten: 1808n,
    bytesDeleted: 1612n,
  });
}
console.log(`${name}: original if result passed`);
