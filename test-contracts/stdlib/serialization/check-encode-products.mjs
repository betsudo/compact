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

const [build, sdk, out] = process.argv.slice(2);
if (process.argv.length !== 5) {
  throw new Error('Usage: node check-encode-products.mjs <compiled-root> <zkir-wasm-package> <new-output-dir>');
}
const file = resolve(build, 'alias-roundtrip/contract/index.js');
const require = createRequire(file);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits: pure } = await import(pathToFileURL(resolve(build, 'encode-products/contract/index.js')));
const { Contract } = await import(pathToFileURL(file));
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));

assert.deepEqual(pure.serialize_vector([0x31n, 0x43n, 0x7fn, 0xffn]), Uint8Array.of(0x31, 0x43, 0x7f, 0xff));
assert.deepEqual(pure.serialize_uints([0x31n, 0x1234n, 0x12345678n]), Uint8Array.of(0x31, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12));
const field = new Uint8Array(33);
field.set([1, 3, 2, 1]);
assert.deepEqual(pure.serialize_tuple([true, 0x010203n]), field);
assert.deepEqual(pure.serialize_struct({ a: true, b: 255n }), Uint8Array.of(1, 255));
assert.deepEqual(pure.serialize_nested([[true, 49n], [false, 255n]]), Uint8Array.of(1, 49, 0, 255));
assert.deepEqual(pure.encode_message({ header: [true, 0x1234n], tail: [[false, 49n], [true, 255n]] }),
  Uint8Array.of(1, 0x34, 0x12, 0, 49, 1, 255, 0));
assert.deepEqual(pure.encode_empty(true), new Uint8Array(2));
assert.throws(() => pure.encode_empty(false), /checked empty input/);

const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const input = [[0xa7b6c5d4n, true], Uint8Array.of(0x11, 0x17, 0x1d, 0x1f, 0x25, 0x29, 0x2b, 0x2f)];
const wire = Uint8Array.of(0xd4, 0xc5, 0xb6, 0xa7, 1, 0x11, 0x17, 0x1d, 0x1f, 0x25, 0x29, 0x2b, 0x2f, 0, 0, 0);
const encoded = contract.circuits.foo(ctx, ...input);
assert.deepEqual(encoded.result, wire);
const decoded = contract.circuits.unfoo(ctx, encoded.result);
assert.deepEqual(decoded.result, input);
const padded = Uint8Array.from(wire);
padded.fill(255, 13);
assert.deepEqual(contract.circuits.unfoo(ctx, padded).result, input);
const invalid = Uint8Array.from(wire);
invalid[4] = 2;
assert.throws(() => contract.circuits.unfoo(ctx, invalid), runtime.CompactError);

await mkdir(out);
for (const [name, outcome] of [['foo', encoded], ['unfoo', decoded]]) {
  const ir = jsonIrToBinary(await readFile(join(build, 'alias-roundtrip/zkir', name + '.zkir'), 'utf8'));
  const options = {
    lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
    getParams: async () => { throw Error('unexpected parameter request'); },
  };
  const data = outcome.proofData;
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const preimage = (argument, result) =>
    onchain.proofDataIntoSerializedPreimage(argument, result, data.publicTranscript, data.privateTranscriptOutputs, name);
  const honest = preimage(data.input, data.output);
  await check(honest, options);
  const changed = structuredClone(data.output);
  changed.value[0] = onchain.bigIntToValue(0n)[0];
  await assert.rejects(check(preimage(data.input, changed), options));
  if (name === 'unfoo') {
    const inputType = new runtime.CompactTypeBytes(16);
    await assert.rejects(check(preimage({ value: inputType.toValue(invalid), alignment: inputType.alignment() }, data.output), options));
  }
  await writeFile(join(out, name + '.bzkir'), ir, { flag: 'wx' });
  await writeFile(join(out, name + '.preimage'), honest, { flag: 'wx' });
}
console.log('Five upstream encoders, original foo/unfoo vectors, padding, refusal and both SDK statements passed.');
