// This file is part of Compact.
// Copyright (C) 2025 Midnight Foundation
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
import { pathToFileURL } from 'node:url';
import { join, resolve } from 'node:path';

const [build, sdk, output] = process.argv.slice(2);
if (process.argv.length !== 5) {
  throw new Error('Usage: node check-wide-uint.mjs <compiled-root> <zkir-wasm-package> <new-output-dir>');
}
const generated = pathToFileURL(resolve(build, 'wide-uint-stateful/contract/index.js'));
const require = createRequire(generated);
const path = require.resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(path));
const onchain = await import(pathToFileURL(require.resolve('@midnightntwrk/onchain-runtime-v4', {paths: [path]})));
const {Contract, ledger} = await import(generated);
const manifest = JSON.parse(await readFile(sdk + '/package.json', 'utf8'));
assert.equal(manifest.version, '3.2.0');
const {check, jsonIrToBinary} = await import(pathToFileURL(sdk + '/' + manifest.exports.node));
const raw = join(build, 'wide-uint-stateful/zkir/store_word.zkir');
const model = JSON.parse(await readFile(raw, 'utf8'));
const ir = jsonIrToBinary(JSON.stringify(model));
const options = {
 lookupKey: async () => ({ir, proverKey: new Uint8Array(), verifierKey: new Uint8Array()}),
 getParams: async () => { throw Error('unexpected params'); },
};
const { pureCircuits: p } = await import(pathToFileURL(resolve(build, 'wide-uint/contract/index.js')));
const wire = (value, length) => Uint8Array.from({length}, (_, index) => Number((value >> BigInt(index * 8)) & 255n));
const max = (1n << 248n) - 1n;
const word = (1n << 129n) - 1n;
for (const value of [0n, 1n << 128n, word]) {
  assert.deepEqual(p.encode_word(value), wire(value, 20));
  const input = wire(value, 20);
  input.fill(255, 17);
  assert.equal(p.decode_word(input), value);
  assert.equal(p.direct(wire(value, 33)), value);
}
assert.deepEqual(p.encode_max(max), wire(max, 31));
assert.equal(p.decode_max(wire(max, 31)), max);
assert.throws(() => p.decode_word(wire(1n << 129n, 20)));
assert.throws(() => p.direct(wire(1n << 256n, 33)));
assert.deepEqual(p.narrow((1n << 128n) - 1n), new Uint8Array(16).fill(255));
assert.throws(() => p.narrow(1n << 128n));
const nested = new Uint8Array(19);
nested[0] = 1;
nested.set(wire(word, 17), 1);
assert.deepEqual(p.nested([true, word]), nested);

const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const ctx = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey,
 initial.currentContractState, initial.currentPrivateState);
const toWire = (n) => Uint8Array.from({length: 17}, (_, i) => Number((n >> BigInt(8*i)) & 255n));
const t = new runtime.CompactTypeBytes(17);
const input = (bytes) => ({value: t.toValue(bytes), alignment: t.alignment()});
const preimage = (data, arg, result) => onchain.proofDataIntoSerializedPreimage(arg, result,
 data.publicTranscript, data.privateTranscriptOutputs, 'store_word');
await mkdir(output);
await writeFile(join(output, 'store_word.bzkir'), ir, {flag: 'wx'});
for (const [i, value] of [0n, 1n << 128n, (1n << 129n) - 1n].entries()) {
 const bytes = toWire(value);
 const outcome = contract.circuits.store_word(ctx, bytes);
 assert.equal(outcome.result, value);
 assert.equal(ledger(outcome.context.currentQueryContext.state).stored, value);
 const data = outcome.proofData;
 assert.deepEqual(data.input, input(bytes));
 assert.deepEqual(data.privateTranscriptOutputs, []);
 const honest = preimage(data, data.input, data.output);
 await check(honest, options);
 await writeFile(join(output, 'store_word-' + i + '.preimage'), honest, {flag: 'wx'});
 const bad = toWire(1n << 129n);
 assert.throws(() => contract.circuits.store_word(ctx, bad), runtime.CompactError);
 const changed = structuredClone(data.output);
 changed.value[0] = onchain.bigIntToValue(value ^ 1n)[0];
 await assert.rejects(check(preimage(data, data.input, changed), options));
}
// Keep all three surfaces of the forged statement consistent. Removing
// only the Uint bound shows that the exact circuit fails on that guard.
const data = contract.circuits.store_word(ctx, toWire(0n)).proofData;
const matchingStatement = (value) => {
  const candidate = structuredClone(data);
  const atom = onchain.bigIntToValue(value)[0];
  candidate.output.value[0] = atom;
  const write = candidate.publicTranscript.filter((operation) => operation.push?.storage);
  assert.equal(write.length, 1);
  assert.equal(write[0].push.value.tag, 'cell');
  write[0].push.value.content.value[0] = atom;
  return preimage(candidate, input(toWire(value)), candidate.output);
};
await check(matchingStatement(1n << 128n), options);
const bound = model.instructions.findIndex((op) => op.op === 'constrain_bits' && op.bits === 129);
assert.notEqual(bound, -1);
assert.equal(model.instructions[bound].val, model.inputs[0].name);
model.instructions.splice(bound, 1);
const unbounded = jsonIrToBinary(JSON.stringify(model));
const control = {
  ...options,
  lookupKey: async () => ({ ir: unbounded, proverKey: new Uint8Array(), verifierKey: new Uint8Array() }),
};
for (const invalid of [1n << 129n, (1n << 136n) - 1n]) {
  const candidate = matchingStatement(invalid);
  await check(candidate, control);
  await assert.rejects(check(candidate, options));
}
assert.equal(ledger(ctx.currentQueryContext.state).stored, 0n);
console.log('Wide Uint wire vectors passed. The exact circuit refused both matched overflow statements.');
