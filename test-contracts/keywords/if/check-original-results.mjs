// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

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
  const first = contract.circuits.if_branches_write_ledger(context(), true);
  assert.equal(first.result, true);
  const second = contract.circuits.if_branches_write_ledger(first.context, false);
  assert.equal(second.result, false);
}
console.log(`${name}: original if result passed`);
