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
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const paired = join(family, 'base/roundtrip/ts-rust/.build');
const extra = join(family, 'controls/ts-rust/.build');
const runtimeEntry = createRequire(join(paired, 'contract/index.js')).resolve('@midnight-ntwrk/compact-runtime');
assert.equal(createRequire(join(extra, 'contract/index.js')).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const original = await import(pathToFileURL(join(paired, 'contract/index.js')));
const controls = await import(pathToFileURL(join(extra, 'contract/index.js')));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json'), 'utf8'));
assert.equal(manifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(join(wasmPackage, manifest.exports.node)).href);
const B = 2n ** 256n - 2n ** 224n + 2n ** 192n + 2n ** 96n - 2n;
const S = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550n;
assert.equal(runtime.MAX_SECP256R1_BASE, B);
assert.equal(runtime.MAX_SECP256R1_SCALAR, S);
const A = [24, 8].map((length) => ({tag: 'atom', value: {tag: 'bytes', length}}));
const base = {name: 'Base<Secp256r1>', max: B, rt: runtime.CompactTypeSecp256r1Base};
const scalar = {name: 'Scalar<Secp256r1>', max: S, rt: runtime.CompactTypeSecp256r1Scalar};
const C = [
  {folder: paired, module: original, name: 'secp256r1_base_roundtrip', slot: 'stored', field: base,
   inputs: [0n, 1n, 12345678901234567890n, B, 1n << 192n, (1n << 192n) + 1n]},
  {folder: extra, module: controls, name: 'scalarRoundtrip', slot: 'scalarCell', field: scalar,
   inputs: [0n, 1n, 1n << 192n, (1n << 192n) + 1n, S]},
  {folder: extra, module: controls, name: 'resetFields', inputs: [null]},
];

function littleEndian(value, width) {
  return Uint8Array.from({length: width}, (_, index) => Number(value >> BigInt(index * 8) & 255n));
}
function normalized(bytes) {
  let length = bytes.length;
  while (length > 0 && bytes[length - 1] === 0) --length;
  return bytes.slice(0, length);
}
function fabric(value, field) {
  const shifted = value === 0n ? field.max : value - 1n;
  const raw = littleEndian(shifted, 32);
  const out = [normalized(raw.slice(0, 24)), normalized(raw.slice(24))];
  assert.deepEqual(field.rt.toValue(value), out);
  assert.deepEqual(field.rt.alignment(), A);
  return out;
}
const sameValue = (a, b) => a.length === b.length && a.every((atom, i) => Buffer.from(atom).equals(Buffer.from(b[i])));
const summarize = (value, field) => ({value: String(value), raw: Buffer.from(littleEndian(value, 32)).toString('hex'), fab: fabric(value, field).map((atom) => Buffer.from(atom).toString('hex'))});
const records = {};
const cases = {};
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
await mkdir(output);
const wasmEntry = join(wasmPackage, manifest.exports.node);
const tools = await Promise.all([runtimeEntry, onchainEntry, wasmEntry, join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
for (const {folder, module, name, slot, field, inputs} of C) {
  const rawPath = join(folder, 'zkir', name + '.zkir');
  const binary = join(folder, 'zkir', name + '.bzkir');
  const raw = await readFile(rawPath, 'utf8');
  const program = JSON.parse(raw);
  const types = field ? [field.name] : [base.name, scalar.name];
  assert.deepEqual(program.inputs.map((input) => input.type), field ? types : []);
  assert.deepEqual(program.outputs, types);
  assert.deepEqual(program.instructions.filter((op) => op.op === 'public_input').map((op) => op.type), types);
  if (!field) assert.deepEqual(program.instructions.filter((op) => op.op === 'from_bytes').map((op) => op.type), types);
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(binary));
  const artifacts = await Promise.all([rawPath, binary, join(folder, 'contract/index.js'),
    join(folder, 'compiler/contract-info.json')].map(pin));
  const options = {
    lookupKey: async () => ({proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir}),
    getParams: async () => { throw new Error('parameters requested during a preimage check'); },
  };
  const contract = new module.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const makeContext = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const initialLedger = module.ledger(makeContext().currentQueryContext.state);
  if (field) assert.equal(initialLedger[slot], 0n);
  else assert.deepEqual([initialLedger.baseCell, initialLedger.scalarCell], [0n, 0n]);
  records[name] = [];
  for (const [index, value] of inputs.entries()) {
    const call = field ? contract.circuits[name](makeContext(), value) : contract.circuits[name](makeContext());
    const ledger = module.ledger(call.context.currentQueryContext.state);
    const proof = call.proofData;
    if (field) {
      assert.equal(call.result, value);
      assert.equal(ledger[slot], value);
      assert.deepEqual(proof.input.alignment, A);
      assert.deepEqual(proof.input.value, fabric(value, field));
      assert.deepEqual(proof.output.alignment, A);
      assert.deepEqual(proof.output.value, fabric(value, field));
    } else {
      assert.deepEqual(call.result, [0n, 0n]);
      assert.deepEqual([ledger.baseCell, ledger.scalarCell], [0n, 0n]);
      assert.deepEqual(proof.output.value, [...fabric(0n, base), ...fabric(0n, scalar)]);
    }
    const serialize = (transcript, result = proof.output) => proofDataIntoSerializedPreimage(
      proof.input, result, transcript, proof.privateTranscriptOutputs, name);
    const preimage = serialize(proof.publicTranscript);
    await check(preimage, options);
    const id = name + '-' + index;
    await writeFile(join(output, id + '.preimage'), preimage, {flag: 'wx'});
    const bad = structuredClone(proof.publicTranscript);
    const written = fabric(value ?? 0n, field ?? base);
    const writes = bad.filter((op) => op.push?.value?.tag === 'cell'
      && sameValue(op.push.value.content.value, written));
    assert.equal(writes.length, 1);
    writes[0].push.value.content.value = fabric(value === 1n ? 0n : 1n, field ?? base);
    const changedWrite = serialize(bad);
    await assert.rejects(() => check(changedWrite, options), /transcript input mismatch/i);
    await writeFile(join(output, id + '.changed-write'), changedWrite, {flag: 'wx'});
    const changedOutput = structuredClone(proof.output);
    changedOutput.value = field
      ? fabric(value === 1n ? 0n : 1n, field)
      : [...fabric(1n, base), ...fabric(0n, scalar)];
    const changedReturn = serialize(proof.publicTranscript, changedOutput);
    await assert.rejects(() => check(changedReturn, options), /communications commitment mismatch/i);
    await writeFile(join(output, id + '.changed-return'), changedReturn, {flag: 'wx'});
    cases[id] = { name, artifacts, preimage: sha(preimage), changedWrite: sha(changedWrite), changedReturn: sha(changedReturn) };
    if (field) records[name].push(summarize(value, field));
  }
}
await writeFile(join(output, 'field-cases.json'), JSON.stringify(records, null, 2)+'\n', {flag: 'wx'});
await writeFile(join(output, 'summary.json'), JSON.stringify({ tools, cases }, null, 2) + '\n', { flag: 'wx' });
console.log('12 honest preimages, 12 changed-write refusals, 12 changed-return refusals');
