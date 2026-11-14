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

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const build = join(family, 'arithmetic/ts-rust/.build');
const entry = join(build, 'contract/index.js');
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const code = await import(pathToFileURL(entry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const fields = {
  b: { slot: 'base', type: 'Base<Secp256r1>', max: runtime.MAX_SECP256R1_BASE, rt: runtime.CompactTypeSecp256r1Base },
  s: { slot: 'scalar', type: 'Scalar<Secp256r1>', max: runtime.MAX_SECP256R1_SCALAR, rt: runtime.CompactTypeSecp256r1Scalar },
};
const alignment = [24, 8].map((length) => ({ tag: 'atom', value: { tag: 'bytes', length } }));
const upstream = JSON.parse(await readFile(new URL('./upstream-vectors.json', import.meta.url)));
assert.equal(upstream.length, 34);
const hex = (value) => value.toString(16).padStart(64, '0');
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));

function atoms(value, field) {
  const shifted = value === 0n ? field.max : value - 1n;
  const bytes = Uint8Array.from({ length: 32 }, (_, i) => Number(shifted >> BigInt(8 * i) & 255n));
  const normalize = (bytes) => bytes.slice(0, bytes.findLastIndex((value) => value !== 0) + 1);
  const result = [normalize(bytes.slice(0, 24)), normalize(bytes.slice(24))];
  assert.deepEqual(field.rt.toValue(value), result);
  return result;
}

const contract = new code.Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
  initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
assert.deepEqual(Object.values(fields).map(({ slot }) => code.ledger(context().currentQueryContext.state)[slot]), [0n, 0n]);

const cases = {};
const vectors = {};
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}

for (const circuit of new Set(upstream.map(({ circuit }) => circuit))) {
  const field = fields[circuit.at(-1)];
  const rawPath = join(build, 'zkir', circuit + '.zkir');
  const binaryPath = rawPath.replace(/\.zkir$/, '.bzkir');
  const raw = await readFile(rawPath, 'utf8');
  const program = JSON.parse(raw);
  assert.deepEqual(program.inputs.map(({ type }) => type), [field.type, field.type]);
  assert.deepEqual(program.outputs, [field.type]);
  assert.deepEqual(program.instructions.filter(({ op }) => op === 'public_input').map(({ type }) => type),
    [field.type]);
  const arithmeticOps = program.instructions.map(({ op }) => op).filter((op) => ['add', 'neg', 'mul'].includes(op));
  assert.deepEqual(arithmeticOps, { add: ['add'], sub: ['neg', 'add'], mul: ['mul'] }[circuit.slice(0, -1)]);
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(binaryPath));
  const artifacts = await Promise.all([rawPath, binaryPath, entry, join(build, 'contract/lib.rs'),
    join(build, 'compiler/contract-info.json')].map(pin));
  const materials = {
    lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
    getParams: async () => { throw new Error('unexpected parameter request'); },
  };
  const rows = upstream.filter((row) => row.circuit === circuit);
  for (const [index, row] of rows.entries()) {
    const { id } = row;
    const inputs = row.inputs.map(BigInt);
    const result = BigInt(row.result);
    const call = contract.circuits[circuit](context(), ...inputs);
    assert.equal(call.result, result, id);
    const state = code.ledger(call.context.currentQueryContext.state);
    assert.equal(state[field.slot], result, id);
    assert.equal(state[field.slot === 'base' ? 'scalar' : 'base'], 0n, id);
    const proof = call.proofData;
    assert.deepEqual(proof.input.alignment, [...alignment, ...alignment], id);
    assert.deepEqual(proof.input.value, inputs.flatMap((value) => atoms(value, field)), id);
    assert.deepEqual(proof.output.alignment, alignment, id);
    assert.deepEqual(proof.output.value, atoms(result, field), id);
    assert.deepEqual(proof.privateTranscriptOutputs, [], id);
    const serialize = (transcript, output = proof.output) => proofDataIntoSerializedPreimage(
      proof.input, output, transcript, proof.privateTranscriptOutputs, circuit);
    const preimage = serialize(proof.publicTranscript);
    await check(preimage, materials);
    const changedTranscript = structuredClone(proof.publicTranscript);
    const cells = changedTranscript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
    assert.equal(cells.length, 1, id);
    const cell = cells[0].push.value.content;
    assert.deepEqual(cell.alignment, alignment, id);
    assert.deepEqual(cell.value, atoms(result, field), id);
    const changedAtoms = atoms(result === 0n ? 1n : 0n, field);
    cell.value = changedAtoms;
    const changedWrite = serialize(changedTranscript);
    await assert.rejects(() => check(changedWrite, materials), /transcript input mismatch/i);
    const changedReturn = serialize(proof.publicTranscript, { ...proof.output, value: changedAtoms });
    await assert.rejects(() => check(changedReturn, materials), /communications commitment mismatch/i);
    const selected = index === rows.length - 1;
    cases[id] = {
      fixture: 'arithmetic', circuit, selected, artifacts,
      preimage: await save(id + '.preimage', preimage),
      changedWrite: await save(id + '.changed-write', changedWrite),
      changedReturn: await save(id + '.changed-return', changedReturn),
    };
    vectors[id] = {
      fixture: 'arithmetic', circuit, selected, inputs: inputs.map((value) => ({ field: hex(value) })),
      expected: { [field.slot]: hex(result) },
    };
  }
}
await save('summary.json', JSON.stringify({ tools, cases }, null, 2) + '\n');
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
console.log('34 honest preimages, 34 changed writes refused, 34 changed returns refused, 6 selected');
