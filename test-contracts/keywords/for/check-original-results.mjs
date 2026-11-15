// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
assert.ok(entry && ['generic_bound', 'return_in_lambda_in_body', 'zero_iteration'].includes(name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
const context = runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const execution = contract.circuits[`for_${name}`](context);
assert.deepStrictEqual(execution.result,
  name === 'generic_bound' ? 5n : name === 'zero_iteration' ? 0n : 7n);
if (name !== 'return_in_lambda_in_body') assert.ok(execution.proofData);
if (name === 'generic_bound') {
  assert.deepStrictEqual(generated.ledger(execution.context.currentQueryContext.state).total, 5n);
}
console.log(`${name}: original for result passed`);
