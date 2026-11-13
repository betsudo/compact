// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [plain, stateful, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 6) {
  throw new Error('Usage: node check-field.mjs <field-output> <stateful-output> <zkir-wasm-package> <new-output-dir>');
}

const require = createRequire(resolve(stateful, 'contract/index.js'));
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits } = await import(pathToFileURL(resolve(plain, 'contract/index.js')));
const { Contract, ledger } = await import(pathToFileURL(resolve(stateful, 'contract/index.js')));
const modulus = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001n;
const bytes = (value, length = 32) =>
  Uint8Array.from({ length }, (_, i) => Number(value >> BigInt(i * 8) & 255n));
for (const value of [0n, 1n, 1n << 248n, modulus - 1n]) {
  assert.deepEqual(pureCircuits.serialize_field(value), bytes(value));
  assert.equal(pureCircuits.deserialize_field(bytes(value)), value);
  assert.deepEqual(pureCircuits.pad_field(value), bytes(value, 34));
  assert.equal(pureCircuits.decode_field(Uint8Array.of(...bytes(value), 255, 123)), value);
}
assert.equal(pureCircuits.direct_short(Uint8Array.of(3, 2, 1)), 0x010203n);
assert.equal(pureCircuits.direct_long(bytes(modulus - 1n, 33)), modulus - 1n);
assert.throws(() => pureCircuits.direct_long(bytes(1n << 256n, 33)), runtime.CompactError);
assert.throws(() => pureCircuits.deserialize_field(bytes(modulus)), runtime.CompactError);
assert.throws(() => pureCircuits.deserialize_field(new Uint8Array(32).fill(255)), runtime.CompactError);

const name = 'store_field';
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));
const ir = jsonIrToBinary(await readFile(join(stateful, 'zkir', name + '.zkir'), 'utf8'));
const options = {
  lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const read = (ctx) => ledger(ctx.currentQueryContext.state).stored;
const inputType = new runtime.CompactTypeBytes(32);
const alignInput = (input) => ({ value: inputType.toValue(input), alignment: inputType.alignment() });
await mkdir(output);
await writeFile(join(output, name + '.bzkir'), ir, { flag: 'wx' });
for (const [index, expected] of [0n, modulus - 1n].entries()) {
  const input = bytes(expected);
  const result = contract.circuits.store_field(context, input);
  assert.equal(result.result, expected);
  assert.equal(read(result.context), expected);
  assert.equal(read(context), 0n);
  const data = result.proofData;
  assert.deepEqual(data.input, alignInput(input));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const preimage = (argument, outputValue) => ocrt.proofDataIntoSerializedPreimage(argument, outputValue,
    data.publicTranscript, data.privateTranscriptOutputs, name);
  const valid = preimage(data.input, data.output);
  await check(valid, options);
  await assert.rejects(check(preimage(alignInput(bytes(modulus)), data.output), options));
  await assert.rejects(check(preimage(alignInput(new Uint8Array(32).fill(255)), data.output), options));
  const wrong = structuredClone(data.output);
  wrong.value = ocrt.bigIntToValue(expected === 0n ? 1n : 0n);
  await assert.rejects(check(preimage(data.input, wrong), options));
  await writeFile(join(output, name + '-' + index + '.preimage'), valid, { flag: 'wx' });
}
assert.throws(() => contract.circuits.store_field(context, bytes(modulus)), runtime.CompactError);
assert.throws(() => contract.circuits.store_field(context, new Uint8Array(31)), /Bytes<32>/);
assert.equal(read(context), 0n);
console.log('Canonical Field, nominal padding and dynamic casts passed; the modulus, all-ones and changed output were refused.');
