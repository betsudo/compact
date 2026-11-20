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

const [build, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 5) {
  throw new Error('Usage: node check-aggregate.mjs <compiled-output> <zkir-wasm-package> <new-output-dir>');
}

const generated = resolve(build, 'contract/index.js');
const require = createRequire(generated);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const ocrt = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { Contract } = await import(pathToFileURL(generated));
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));
const ir = jsonIrToBinary(await readFile(join(build, 'zkir/d2.zkir'), 'utf8'));
const options = {
  lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const field = (ctx) => runtime.CompactTypeField.fromValue(ctx.currentQueryContext.state.state.asArray()[0].asCell().value);
assert.equal(contract.circuits.d1(context, Uint8Array.of(0)).result, false);
assert.equal(contract.circuits.d1(context, Uint8Array.of(1)).result, true);
for (const badByte of [2, 3, 32, 255]) {
  assert.throws(() => contract.circuits.d1(context, Uint8Array.of(badByte)), runtime.CompactError);
}

// Keep the upstream #772 vectors as well as the SDK controls below.
const original = Uint8Array.of(0x01, 0x07, 0xff, 0x00, 0xc1, 0x01);
assert.deepEqual(contract.circuits.d2(context, original).result,
  { a: true, b: 0xff07n, c: false, d: 0xc1n, e: true });
for (const invalid of [
  Uint8Array.of(0x03, 0x07, 0xff, 0x00, 0xc1, 0x01),
  Uint8Array.of(0x01, 0x07, 0xff, 0x32, 0xc1, 0x01),
  Uint8Array.of(0x01, 0x07, 0xff, 0x00, 0xc1, 0xff),
]) {
  assert.throws(() => contract.circuits.d2(context, invalid), runtime.CompactError);
}

const type = new runtime.CompactTypeBytes(6);
const encode = (input) => ({ value: type.toValue(input), alignment: type.alignment() });
const cases = [
  [Uint8Array.of(0, 0, 0, 0, 0, 0), { a: false, b: 0n, c: false, d: 0n, e: false }],
  [Uint8Array.of(1, 0x34, 0x12, 0, 255, 1), { a: true, b: 0x1234n, c: false, d: 255n, e: true }],
  [Uint8Array.of(0, 255, 255, 1, 1, 0), { a: false, b: 65535n, c: true, d: 1n, e: false }],
];
await mkdir(output);
await writeFile(join(output, 'd2.bzkir'), ir, { flag: 'wx' });
for (const [index, [input, expected]] of cases.entries()) {
  const outcome = contract.circuits.d2(context, input);
  assert.deepEqual(outcome.result, expected);
  assert.equal(field(outcome.context), 7n);
  assert.equal(field(context), 0n);
  const data = outcome.proofData;
  assert.deepEqual(data.input, encode(input));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const preimage = (argument, result) =>
    ocrt.proofDataIntoSerializedPreimage(argument, result, data.publicTranscript, data.privateTranscriptOutputs, 'd2');
  const valid = preimage(data.input, data.output);
  await check(valid, options);
  await writeFile(join(output, 'd2-' + index + '.preimage'), valid, { flag: 'wx' });
  for (const offset of [0, 3, 5]) {
    for (const badByte of [2, 255]) {
      const invalid = Uint8Array.from(input);
      invalid[offset] = badByte;
      assert.throws(() => contract.circuits.d2(context, invalid), runtime.CompactError);
      assert.equal(field(context), 0n);
      await assert.rejects(check(preimage(encode(invalid), data.output), options));
    }
  }
  const altered = structuredClone(data.output);
  altered.value[1] = ocrt.bigIntToValue(expected.b ^ 1n)[0];
  await assert.rejects(check(preimage(data.input, altered), options));
}
assert.throws(() => contract.circuits.d2(context, new Uint8Array(5)), /Bytes<6>/);
console.log('Original d1/d2 exports, little-endian fields and Boolean refusals at offsets 0, 3 and 5 passed.');
