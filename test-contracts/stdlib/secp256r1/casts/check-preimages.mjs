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
const fixtures = ['decode', 'encode', 'guarded-save', 'guarded-decode', 'guarded-encode', 'constant', 'unused'];
const build = (fixture) => join(family, 'casts', fixture, 'ts-rust/.build');
const entry = (fixture) => join(build(fixture), 'contract/index.js');
const runtimeEntry = createRequire(entry('decode')).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const baseModulus = 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffffn;
const scalarModulus = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551n;
assert.equal(runtime.MAX_SECP256R1_BASE, baseModulus - 1n);
assert.equal(runtime.MAX_SECP256R1_SCALAR, scalarModulus - 1n);
const allOnes = (1n << 256n) - 1n;
const bytes = (value) => Uint8Array.from({ length: 32 }, (_, index) => Number(value >> BigInt(index * 8) & 255n));
const values = [0n, 1n, (1n << 192n) - 1n, 1n << 192n, (1n << 192n) + 1n,
  (1n << 248n) - 1n, 1n << 248n, (1n << 248n) + 1n,
  baseModulus - 1n, baseModulus, baseModulus + 1n,
  scalarModulus - 1n, scalarModulus, scalarModulus + 1n, allOnes];
const hex = (value) => Buffer.from(value).toString('hex');
const base = runtime.CompactTypeSecp256r1Base;
const scalar = runtime.CompactTypeSecp256r1Scalar;
const bytes32 = new runtime.CompactTypeBytes(32);
const nativeField = runtime.CompactTypeField;
const sameAtoms = (a, b) => a.length === b.length && a.every((atom, i) => hex(atom) === hex(b[i]));
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, wasmEntry, join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const cases = {};
const vectors = {};

await mkdir(output);
async function write(name, value) {
  await writeFile(join(output, name), value, { flag: 'wx' });
  return sha(value);
}
const contracts = new Map();
for (const fixture of fixtures) {
  const source = entry(fixture);
  assert.equal(createRequire(source).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const code = await import(pathToFileURL(source));
  const contract = new code.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  contracts.set(fixture, { contract, context, ledger: code.ledger, programs: new Map() });
}

async function loadProgram(fixture, name) {
  const { programs } = contracts.get(fixture);
  if (programs.has(name)) return programs.get(name);
  const rawPath = join(build(fixture), 'zkir', name + '.zkir');
  const binaryPath = rawPath.replace(/\.zkir$/, '.bzkir');
  const raw = await readFile(rawPath, 'utf8');
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(binaryPath));
  const result = {
    zkir: JSON.parse(raw),
    artifacts: await Promise.all([rawPath, binaryPath, entry(fixture), join(build(fixture), 'compiler/contract-info.json')].map(pin)),
    materials: {
      lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
      getParams: async () => { throw new Error('unexpected parameter request'); },
    },
  };
  programs.set(name, result);
  return result;
}

const inputsJson = (args) => args.map((value) => value instanceof Uint8Array ? { bytes: hex(value) }
  : typeof value === 'bigint' ? { field: value.toString(16).padStart(64, '0') } : value);
const ledgerJson = (writes) => Object.fromEntries(writes.map(([slot, , value]) =>
  [slot, value instanceof Uint8Array ? hex(value) : value.toString(16).padStart(64, '0')]));

async function run(id, fixture, name, args, expected, selected = false, writes = true) {
  const { contract, context, ledger } = contracts.get(fixture);
  const program = await loadProgram(fixture, name);
  const execution = contract.circuits[name](context(), ...args);
  assert.deepEqual(execution.result, []);
  const state = ledger(execution.context.currentQueryContext.state);
  for (const [slot, , value] of expected) assert.deepEqual(state[slot], value, `${id} ${slot}`);
  const proof = execution.proofData;
  assert.deepEqual(proof.privateTranscriptOutputs, []);
  const serialize = (transcript) =>
    proofDataIntoSerializedPreimage(proof.input, proof.output, transcript, proof.privateTranscriptOutputs, name);
  const preimage = serialize(proof.publicTranscript);
  await check(preimage, program.materials);
  const record = {
    fixture, circuit: name, selected, artifacts: program.artifacts, preimage: await write(id + '.preimage', preimage),
    refusals: [],
  };
  const cellWrites = (transcript) => transcript.filter((op) =>
    op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(cellWrites(proof.publicTranscript).length, writes ? expected.length : 0, id);
  if (writes) {
    for (const [i, [slot, descriptor, value]] of expected.entries()) {
      const transcript = structuredClone(proof.publicTranscript);
      const cell = cellWrites(transcript)[i].push.value.content;
      assert.deepEqual(cell.alignment, descriptor.alignment(), `${id} ${slot}`);
      assert(sameAtoms(cell.value, descriptor.toValue(value)), `${id} ${slot}`);
      const zero = value instanceof Uint8Array ? bytes(0n) : 0n;
      const one = value instanceof Uint8Array ? bytes(1n) : 1n;
      cell.value = descriptor.toValue(sameAtoms(cell.value, descriptor.toValue(zero)) ? one : zero);
      const altered = serialize(transcript);
      await assert.rejects(() => check(altered, program.materials), /transcript input mismatch/i);
      record.refusals.push(await write(id + '.changed-' + slot, altered));
    }
  }
  cases[id] = record;
  vectors[id] = { fixture, circuit: name, inputs: inputsJson(args), expected: ledgerJson(expected), selected };
}

for (const [i, value] of values.entries()) {
  const b = value % baseModulus;
  const s = value % scalarModulus;
  await run(`decode-${i}`, 'decode', 'test', [bytes(value), bytes(value)], [['base', base, b], ['scalar', scalar, s]], value === allOnes);
  await run(`encode-${i}`, 'encode', 'test', [b, s], [['base', bytes32, bytes(b)], ['scalar', bytes32, bytes(s)]], value === allOnes);
}

const b = 0x6c8a8c3f071f4d6dbe937aaf1a1d457dc5ab2e9e188e6a5c3190ea1072df9a97n;
const s = 0x78db8cdf5ff2bc906d349cd5b11384283b39a67a9ed2739a2428dfd814c6e43dn;
await run('guarded-save', 'guarded-save', 'save', [true, b], [['out', bytes32, bytes(b)]], true);
await run('guarded-save-inactive', 'guarded-save', 'save', [false, b], [['out', bytes32, bytes(0n)]], false, false);
await run('guarded-decode', 'guarded-decode', 'test', [true, bytes(b), bytes(s)], [['base', base, b], ['scalar', scalar, s]], true);
await run('guarded-decode-inactive', 'guarded-decode', 'test', [false, bytes(allOnes), bytes(allOnes)], [['base', base, 0n], ['scalar', scalar, 0n]], false, false);
await run('guarded-encode', 'guarded-encode', 'test', [true, b, s], [['base', bytes32, bytes(b)], ['scalar', bytes32, bytes(s)]], true);
await run('guarded-encode-inactive', 'guarded-encode', 'test', [false, baseModulus - 1n, scalarModulus - 1n], [['base', bytes32, bytes(0n)], ['scalar', bytes32, bytes(0n)]], false, false);

const v = (1n << 248n) + 5n;
await run('constant-false', 'constant', 'castR1', [false, 7n, 8n], [['r1b', base, v], ['r1s', scalar, v]], true);
await run('constant-true', 'constant', 'castR1', [true, 7n, 8n], [['r1b', base, 7n], ['r1s', scalar, 8n]]);
await run('constant-zero', 'constant', 'castZero', [], [['r1b', base, 0n], ['r1s', scalar, 0n]], true);
await run('unused', 'unused', 'unusedR1', [new Uint8Array(32).fill(7)], [['n', nativeField, 2n]], true);
await run('unused-field', 'unused', 'unusedFieldR1', [new Uint8Array(32).fill(7), 5n], [['n', nativeField, 5n]], true);

const decodeProgram = (await loadProgram('decode', 'test')).zkir;
const encodeProgram = (await loadProgram('encode', 'test')).zkir;
assert.deepEqual(decodeProgram.instructions.filter(({ op }) => op === 'from_bytes').map(({ type }) => type),
  ['Base<Secp256r1>', 'Scalar<Secp256r1>']);
assert.deepEqual(encodeProgram.inputs.map(({ type }) => type), ['Base<Secp256r1>', 'Scalar<Secp256r1>']);
assert.equal(encodeProgram.instructions.filter(({ op }) => op === 'to_bytes').length, 2);
for (const name of ['unusedR1', 'unusedFieldR1']) {
  assert(!(await loadProgram('unused', name)).zkir.instructions.some(({ op }) => op === 'from_bytes'));
}
await write('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await write('summary.json', JSON.stringify({ tools, cases }, null, 2) + '\n');
console.log(JSON.stringify({ honest: Object.keys(cases).length,
  refused: Object.values(cases).reduce((sum, row) => sum + row.refusals.length, 0),
  selected: Object.values(cases).filter((row) => row.selected).length }));
