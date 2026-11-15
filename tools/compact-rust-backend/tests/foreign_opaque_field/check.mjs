// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const [entry, kind] = process.argv.slice(2);
assert.ok(entry && ['opaque', 'foreign'].includes(kind));
const generated = await import(pathToFileURL(entry).href);
if (kind === 'opaque') {
  const result = generated.pureCircuits.opaque_casts_to_opaque();
  assert.ok(result instanceof Uint8Array);
  console.log('opaque: original Uint8Array assertion passed');
} else {
  const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
  const runtime = await import(pathToFileURL(runtimeEntry).href);
  const contract = new generated.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
  const context = runtime.createCircuitContext(
    runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState,
  );
  const cases = [
    ['self_subtract', [12345n]],
    ['inlined_self_subtract', [12345n]],
    ['multiply_self_subtract', [12345n, 678n]],
    ['secp256r1_self_subtract', [12345n]],
    ['curve25519_self_subtract', [12345n]],
  ];
  let current = context;
  for (const [name, args] of cases) {
    const execution = contract.circuits[name](current, ...args);
    assert.strictEqual(execution.result, 0n, name);
    assert.ok(execution.proofData, name);
    current = execution.context;
  }
  console.log('foreign: five original zero-result assertions passed');
}
