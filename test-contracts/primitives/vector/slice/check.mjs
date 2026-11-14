// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

// Execute the unchanged upstream runtime assertions against generated output.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
const expected = {
  computed_index: [20n, 30n],
  default: [0n, 0n],
  empty: [],
  from_parameter: [42n],
  mixed_types: [4n, true],
  nested: [1200n, 1200n],
  radix_bounds: [30n, 40n],
  semantics: [20n, 30n],
};
if (!Object.hasOwn(expected, name)) throw new Error(`unknown original vector slice fixture: ${name}`);

const { Contract } = await import(pathToFileURL(contractPath));
const { createCircuitContext, createConstructorContext, dummyContractAddress } =
  await import(pathToFileURL(`${runtimePath}/dist/index.js`));
const contract = new Contract({});
const initial = await contract.initialState(
  createConstructorContext(undefined, '0'.repeat(64)),
);
const context = createCircuitContext(
  dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState,
  initial.currentPrivateState,
);
const args = name === 'from_parameter' ? [[false, true, 42n]] : [];
const actual = Array.from(
  (await contract.circuits[`vector_slice_${name}`](context, ...args)).result,
);
assert.deepStrictEqual(actual, expected[name]);
console.log(`${name}: original runtime result passed`);
