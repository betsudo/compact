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
  throw new Error('Usage: node check-enum.mjs <compiled-root> <zkir-wasm-package> <new-output-dir>');
}

const generated = resolve(build, 'enum-stateful/contract/index.js');
const require = createRequire(generated);
const runtimePath = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimePath));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', { paths: [runtimePath] })));
const { pureCircuits: pure } = await import(pathToFileURL(resolve(build, 'enum/contract/index.js')));
const { Contract, ledger } = await import(pathToFileURL(generated));
const manifest = JSON.parse(await readFile(join(sdk, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(resolve(sdk, manifest.exports.node)));

for (const ordinal of [0, 1, 2]) {
  assert.deepEqual(pure.serialize_enum(ordinal), Uint8Array.of(ordinal));
  assert.equal(pure.deserialize_enum(Uint8Array.of(ordinal)), ordinal);
  assert.deepEqual(pure.padded(ordinal), Uint8Array.of(ordinal, 0, 0));
  assert.equal(pure.unpadded(Uint8Array.of(ordinal, 255, 255)), ordinal);
  assert.equal(pure.to_field(ordinal), BigInt(ordinal));
  assert.equal(pure.from_field(BigInt(ordinal)), ordinal);
}
assert.equal(pure.to_small(1), 1n);
assert.throws(() => pure.to_small(2), runtime.CompactError);
assert.throws(() => pure.from_field(1n << 200n), runtime.CompactError);
assert.deepEqual(pure.zero_encode(0), new Uint8Array());
assert.equal(pure.zero_decode(new Uint8Array()), 0);
assert.deepEqual(pure.checked_zero(true), new Uint8Array());
assert.throws(() => pure.checked_zero(false), /checked singleton input/);
assert.deepEqual(pure.nested([2, true]), Uint8Array.of(2, 1));

const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
  initial.currentContractState, initial.currentPrivateState);
const stored = (ctx) => ledger(ctx.currentQueryContext.state).stored;
const name = 'store_color';
const model = JSON.parse(await readFile(join(build, 'enum-stateful/zkir', name + '.zkir'), 'utf8'));
const ir = jsonIrToBinary(JSON.stringify(model));
const options = {
  lookupKey: async () => ({ ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
const inputType = new runtime.CompactTypeBytes(1);
const aligned = (ordinal) => ({ value: inputType.toValue(Uint8Array.of(ordinal)), alignment: inputType.alignment() });
const preimage = (data, input, output) =>
  onchain.proofDataIntoSerializedPreimage(input, output, data.publicTranscript, data.privateTranscriptOutputs, name);

await mkdir(output);
await writeFile(join(output, name + '.bzkir'), ir, { flag: 'wx' });
for (const ordinal of [0, 1, 2]) {
  const result = contract.circuits.store_color(context, Uint8Array.of(ordinal));
  assert.equal(result.result, ordinal);
  assert.equal(stored(result.context), ordinal);
  assert.equal(stored(context), 0);
  const data = result.proofData;
  assert.deepEqual(data.input, aligned(ordinal));
  assert.deepEqual(data.privateTranscriptOutputs, []);
  const honest = preimage(data, data.input, data.output);
  await check(honest, options);
  await writeFile(join(output, name + '-' + ordinal + '.preimage'), honest, { flag: 'wx' });
  const changed = structuredClone(data.output);
  changed.value[0] = onchain.bigIntToValue(BigInt((ordinal + 1) % 3))[0];
  await assert.rejects(check(preimage(data, data.input, changed), options));
}
const data = contract.circuits.store_color(context, Uint8Array.of(2)).proofData;
const ordinalPreimage = (ordinal) => {
  const candidate = structuredClone(data);
  const value = onchain.bigIntToValue(BigInt(ordinal))[0];
  candidate.output.value[0] = value;
  const write = candidate.publicTranscript.filter((op) => op.push?.storage);
  assert.equal(write.length, 1);
  assert.equal(write[0].push.value.tag, 'cell');
  write[0].push.value.content.value[0] = value;
  return preimage(candidate, aligned(ordinal), candidate.output);
};
await check(ordinalPreimage(1), options);

// Establish that the forged statement agrees on input, write and return
// by removing only the ordinal assertion from an independent control model.
const assertion = model.instructions.findIndex((op) => op.op === 'assert');
assert.equal(model.instructions.filter((op) => op.op === 'assert').length, 1);
const guard = model.instructions[assertion - 1];
assert.equal(guard.op, 'less_than');
assert.equal(guard.output, model.instructions[assertion].cond);
assert.equal(guard.b, '0x03');
model.instructions.splice(assertion - 1, 2);
const unguarded = jsonIrToBinary(JSON.stringify(model));
const unguardedOptions = {
  ...options,
  lookupKey: async () => ({ ir: unguarded, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
};
for (let ordinal = 3; ordinal < 256; ordinal++) {
  const input = Uint8Array.of(ordinal);
  assert.throws(() => pure.deserialize_enum(input), runtime.CompactError);
  assert.throws(() => contract.circuits.store_color(context, input), runtime.CompactError);
  const forged = ordinalPreimage(ordinal);
  await check(forged, unguardedOptions);
  await assert.rejects(check(forged, options));
}
const direct = contract.circuits.store_ordinal(context, 1n);
assert.equal(direct.result, 1n);
assert.equal(stored(direct.context), 1);
for (const ordinal of [2n, 3n, 255n]) {
  assert.throws(() => contract.circuits.store_ordinal(context, ordinal), runtime.CompactError);
}
assert.equal(stored(context), 0);
console.log('Three valid ordinals and all 253 invalid ordinals passed the generated and SDK controls.');
