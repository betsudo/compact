// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
assert.ok(entry && ['generic_bound', 'return_in_lambda_in_body', 'zero_iteration',
  'repeated_values', 'iterable_once'].includes(name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const circuit = ['repeated_values', 'iterable_once'].includes(name)
  ? 'for_generic_bound' : `for_${name}`;
const execution = contract.circuits[circuit](context);
assert.deepStrictEqual(execution.result,
  name === 'zero_iteration' ? 0n : name === 'return_in_lambda_in_body' ? 7n
    : name === 'repeated_values' ? 4n : 5n);
if (name !== 'return_in_lambda_in_body') assert.ok(execution.proofData);
if (['repeated_values', 'iterable_once'].includes(name)) {
  assert.deepStrictEqual(generated.ledger(execution.context.currentQueryContext.state).total,
    name === 'repeated_values' ? 4n : 5n);
  assert.equal(execution.proofData.publicTranscript.length,
    name === 'repeated_values' ? 18 : 24);
}
if (name === 'generic_bound') {
  assert.deepStrictEqual(generated.ledger(execution.context.currentQueryContext.state).total, 5n);
  assert.equal(execution.proofData.publicTranscript.length, 18);
  assert.equal(execution.proofData.privateTranscriptOutputs.length, 0);
  assert.deepStrictEqual(execution.context.gasCost, {
    readTime: 765000000n,
    computeTime: 7385878722n,
    bytesWritten: 72n,
    bytesDeleted: 72n,
  });
}
console.log(`${name}: original for result passed`);
