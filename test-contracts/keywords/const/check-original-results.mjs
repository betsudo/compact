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
const expected = {
  empty_tuple_pattern: [],
  generic_type_annotation: [0n, 0n, 0n],
  initializer_side_effects: [1n, 2n],
  later_binding_sees_earlier: 2n,
  multiple_bindings: [true, 1n, 2n],
  pattern_trailing_comma: 1n,
  shadowing_in_blocks: 1n,
  struct_destructuring: [1n, 2n],
  tuple_destructuring: [1n, 2n, 10n, 20n],
  underscore_binding: 2n,
};
assert.ok(entry && (name in expected || name === 'in_braced_bodies'));
const generated = await import(pathToFileURL(entry).href);
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry).href);
const contract = new generated.Contract({});
const context = () => {
  const initial = contract.initialState(runtime.createConstructorContext(3, '0'.repeat(64)));
  return runtime.createCircuitContext(
    runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
    initial.currentContractState, initial.currentPrivateState,
  );
};
const circuit = contract.circuits[`const_${name}`];
assert.equal(typeof circuit, 'function');
if (name === 'in_braced_bodies') {
  assert.deepStrictEqual(circuit(context(), true).result, 3n);
  assert.deepStrictEqual(circuit(context(), false).result, 12n);
} else {
  const execution = circuit(context());
  assert.deepStrictEqual(execution.result, expected[name]);
  if (name === 'initializer_side_effects') {
    assert.ok(execution.proofData);
  }
}
console.log(`${name}: original const result passed`);
