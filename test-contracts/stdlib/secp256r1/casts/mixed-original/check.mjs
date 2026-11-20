// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [constantPath, unusedPath] = process.argv.slice(2).map((path) => resolve(path));
assert(constantPath && unusedPath);
const runtimeEntry = createRequire(constantPath).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const constant = await import(pathToFileURL(constantPath));
const unused = await import(pathToFileURL(unusedPath));
const v = (1n << 248n) + 5n;

function open(code) {
  const contract = new code.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const ledger = (result) => code.ledger(result.context.currentQueryContext.state);
  return { contract, context, ledger };
}

{
  const { contract, context, ledger } = open(constant);
  let l = ledger(contract.circuits.castK1(context(), false, 7n, 8n));
  assert.deepEqual([l.k1b, l.k1s], [v, v]);
  l = ledger(contract.circuits.castK1(context(), true, 7n, 8n));
  assert.deepEqual([l.k1b, l.k1s], [7n, 8n]);
  l = ledger(contract.circuits.castR1(context(), false, 7n, 8n));
  assert.deepEqual([l.r1b, l.r1s], [v, v]);
  l = ledger(contract.circuits.castR1(context(), true, 7n, 8n));
  assert.deepEqual([l.r1b, l.r1s], [7n, 8n]);
  l = ledger(contract.circuits.castC25519(context(), false, 7n, 8n));
  assert.deepEqual([l.c25519b, l.c25519s], [v, v]);
  l = ledger(contract.circuits.castC25519(context(), true, 7n, 8n));
  assert.deepEqual([l.c25519b, l.c25519s], [7n, 8n]);
  l = ledger(contract.circuits.castZero(context()));
  assert.deepEqual([l.k1b, l.k1s, l.r1b, l.r1s, l.c25519b, l.c25519s],
    [0n, 0n, 0n, 0n, 0n, 0n]);
  assert.equal(ledger(contract.circuits.castWide(context())).c25519s, 5n);
  assert.equal(ledger(contract.circuits.castNative(context(), new Uint8Array(32), 1n, 2n)).f, v);
}

{
  const { contract, context, ledger } = open(unused);
  const n = (name, ...args) => ledger(contract.circuits[name](context(), ...args)).n;
  const bytes = new Uint8Array(32).fill(7);
  assert.equal(n('unusedK1', bytes), 1n);
  assert.equal(n('unusedR1', bytes), 2n);
  assert.equal(n('unusedC25519', bytes, new Uint8Array(64).fill(9)), 3n);
  assert.equal(n('unusedFieldK1', bytes, 4n), 4n);
  assert.equal(n('unusedFieldR1', bytes, 5n), 5n);
}

console.log('Both original mixed-curve callbacks passed (14 ledger assertions)');
