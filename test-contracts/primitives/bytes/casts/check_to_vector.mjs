// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

// Execute each unchanged upstream result assertion in generated TypeScript.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';

const [contractPath, runtimePath, name] = process.argv.slice(2);
const expected = {
  to_vector: [1n, 2n],
  to_vector_default: [0n, 0n, 0n, 0n],
  to_vector_wide_element: [1n, 2n, 3n, 255n],
};
if (!Object.hasOwn(expected, name)) throw new Error(`unknown Bytes-to-Vector fixture: ${name}`);

const { Contract } = await import(pathToFileURL(contractPath));
const { createCircuitContext, createConstructorContext, dummyContractAddress } =
  await import(pathToFileURL(`${runtimePath}/dist/index.js`));
const contract = new Contract({});
const initial = await contract.initialState(createConstructorContext(undefined, '0'.repeat(64)));
const context = createCircuitContext(
  dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState,
  initial.currentPrivateState,
);
const actual = Array.from(
  (await contract.circuits[`bytes_casts_${name}`](context)).result,
);
assert.deepStrictEqual(actual, expected[name]);
console.log(`${name}: original runtime result passed`);
