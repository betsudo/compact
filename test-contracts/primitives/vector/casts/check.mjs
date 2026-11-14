// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

// Run the unchanged upstream result assertions against generated TypeScript.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
const expected = {
  empty: [],
  to_bytes_default: [0, 0, 0, 0],
  to_bytes_narrow_element: [1, 2, 3, 127],
  to_tuple: [100n, 200n],
  to_vector: [1n, 2n, 3n, 4n],
};
if (!Object.hasOwn(expected, name)) throw new Error(`unknown vector cast fixture: ${name}`);

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
const actual = Array.from(
  (await contract.circuits[`vector_casts_${name}`](context)).result,
);
assert.deepStrictEqual(actual, expected[name]);
console.log(`${name}: original runtime result passed`);
