// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, name] = process.argv.slice(2);
assert.ok(entry && ['if_else_circuit_call', 'if_else_either', 'lambda_default'].includes(name));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(
  runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState,
);
const call = (value) => contract.circuits[`return_${name}`](context(), value).result;
if (name === 'if_else_circuit_call') {
  assert.equal(call(2n), 1n);
  assert.equal(call(0n), 1n);
} else if (name === 'if_else_either') {
  assert.deepEqual(call(2n), { is_left: true, left: 1n, right: false });
  assert.deepEqual(call(0n), { is_left: false, left: 0n, right: true });
} else {
  assert.equal(call(5n), 0n);
}
console.log(`${name}: original return result passed`);
