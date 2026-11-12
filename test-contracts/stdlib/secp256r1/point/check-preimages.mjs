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
import { identity, modulus, pointCases } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const build = (fixture) => join(family, 'point', fixture, 'ts-rust/.build');
const entry = (fixture) => join(build(fixture), 'contract/index.js');
const runtimeEntry = createRequire(entry('upstream-equality')).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const requireRuntime = createRequire(runtimeEntry);
const onchainEntry = requireRuntime.resolve('@midnightntwrk/onchain-runtime-v4');
const nobleEntry = requireRuntime.resolve('@noble/curves/nist.js');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const { p256 } = await import(pathToFileURL(nobleEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const { five, seven, variedIdentities, equality } = pointCases(p256);
const xZero = {
  x: 0n, y: 0x66485c780e2f83d72433bd5d84a06bb6541c2af31dae871728bf856a174f93f4n, identity: false,
};
const point = runtime.CompactTypeSecp256r1Point;
const bool = runtime.CompactTypeBoolean;
const bytes32 = new runtime.CompactTypeBytes(32);
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, nobleEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const sameAtoms = (a, b) => a.length === b.length
  && a.every((atom, i) => Buffer.from(atom).equals(Buffer.from(b[i])));
const hex = (value) => typeof value === 'bigint' ? value.toString(16).padStart(64, '0')
  : Buffer.from(value).toString('hex');
const encodePoint = ({ x, y, identity }) => ({ x: hex(x), y: hex(y), identity });

function coordinateBytes(value) {
  const shifted = (value + modulus - 1n) % modulus;
  return Uint8Array.from({ length: 32 }, (_, i) => Number(shifted >> BigInt(i * 8) & 255n));
}
function pointBytes(value) {
  const canonical = value.identity ? identity : value;
  return [coordinateBytes(canonical.x), coordinateBytes(canonical.y)];
}
function pointAtoms(value) {
  const [x, y] = pointBytes(value);
  const normalize = (bytes) => bytes.slice(0, bytes.findLastIndex((byte) => byte !== 0) + 1);
  return [x.slice(0, 24), x.slice(24), y.slice(0, 24), y.slice(24),
    Uint8Array.of(Number(value.identity))].map(normalize);
}
function referenceHash(value) {
  const data = Buffer.alloc(96);
  const [x, y] = pointBytes(value);
  data.set(x, 0);
  data.set(y, 32);
  data[64] = Number(value.identity);
  return Uint8Array.from(Buffer.from(sha(data), 'hex'));
}

const suites = new Map();
for (const fixture of ['upstream-equality', 'upstream-identity']) {
  const info = join(build(fixture), 'compiler/contract-info.json');
  const source = entry(fixture);
  assert.equal(createRequire(source).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const module = await import(pathToFileURL(source));
  const contract = new module.Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const circuits = JSON.parse(await readFile(info)).circuits;
  const programs = new Map();
  for (const { name, proof } of circuits) {
    if (!proof) continue;
    const rawPath = join(build(fixture), 'zkir', name + '.zkir');
    const binaryPath = rawPath.replace(/\.zkir$/, '.bzkir');
    const raw = await readFile(rawPath, 'utf8');
    const program = JSON.parse(raw);
    assert(program.inputs.every(({ type }) => type === 'Point<Secp256r1>'));
    assert(program.do_communications_commitment);
    const ir = jsonIrToBinary(raw);
    assert.deepEqual(Buffer.from(ir), await readFile(binaryPath));
    programs.set(name, {
      artifacts: await Promise.all([rawPath, binaryPath, info, source, join(build(fixture), 'contract/lib.rs')].map(pin)),
      materials: {
        lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
        getParams: async () => { throw new Error('unexpected parameter request'); },
      },
    });
  }
  suites.set(fixture, { module, contract, context, programs });
}

const cases = {};
const vectors = {};
const offCurve = {};
const calls = new Map();
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}
async function run(fixture, circuit, args, expected, descriptor, changed, selected = false) {
  const count = calls.get(circuit) ?? 0;
  calls.set(circuit, count + 1);
  const id = circuit + '-' + count;
  const { module, contract, context, programs } = suites.get(fixture);
  const call = contract.circuits[circuit](context(), ...args);
  assert.deepEqual(call.result, expected, id);
  if (circuit === 'storePoint' || circuit === 'hashPoint') {
    const ledger = module.ledger(call.context.currentQueryContext.state);
    assert.deepEqual(ledger[circuit === 'storePoint' ? 'point' : 'hash'], expected, id);
  }
  const { artifacts, materials } = programs.get(circuit);
  const proof = call.proofData;
  assert.deepEqual(proof.input.alignment, args.flatMap(() => point.alignment()), id);
  assert(sameAtoms(proof.input.value, args.flatMap(pointAtoms)), id);
  assert.deepEqual(proof.output.alignment, descriptor.alignment(), id);
  assert(sameAtoms(proof.output.value, descriptor.toValue(expected)), id);
  assert.deepEqual(proof.privateTranscriptOutputs, [], id);
  const serialize = (transcript = proof.publicTranscript, result = proof.output, input = proof.input) =>
    proofDataIntoSerializedPreimage(input, result, transcript, proof.privateTranscriptOutputs, circuit);
  const honest = serialize();
  await check(honest, materials);
  const transcript = structuredClone(proof.publicTranscript);
  const cells = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(cells.length, 1, id);
  const cell = cells[0].push.value.content;
  assert.deepEqual(cell.alignment, descriptor.alignment(), id);
  assert(sameAtoms(cell.value, descriptor.toValue(expected)), id);
  const changedAtoms = descriptor.toValue(changed);
  cell.value = changedAtoms;
  const changedWrite = serialize(transcript);
  await assert.rejects(() => check(changedWrite, materials), /transcript input mismatch/i);
  const changedReturn = serialize(proof.publicTranscript, { ...proof.output, value: changedAtoms });
  await assert.rejects(() => check(changedReturn, materials), /communications commitment mismatch/i);
  if (count === 0) {
    const bad = { ...identity, identity: false };
    assert.throws(() => contract.circuits[circuit](context(), bad, ...args.slice(1)), /expected value of type Secp256r1Point/);
    const input = { ...proof.input, value: [...pointAtoms(bad), ...proof.input.value.slice(5)] };
    const invalid = serialize(proof.publicTranscript, proof.output, input);
    await assert.rejects(() => check(invalid, materials), /Failed to decode .*Secp256r1Point|encoded value of type Secp256r1Point.*not in canonical form/i);
    offCurve[id + '.off-curve'] = await save(id + '.off-curve', invalid);
  }
  cases[id] = {
    fixture, circuit, selected, artifacts,
    preimage: await save(id + '.preimage', honest),
    changedWrite: await save(id + '.changed-write', changedWrite),
    changedReturn: await save(id + '.changed-return', changedReturn),
  };
  vectors[id] = {
    fixture, circuit, selected, inputs: args.map(encodePoint),
    expected: descriptor === point ? encodePoint(expected) : descriptor === bytes32 ? hex(expected) : expected,
  };
}

const eq = suites.get('upstream-equality');
const defaultPoint = eq.contract.circuits.getDefault(eq.context());
assert.deepEqual(defaultPoint.result, identity);
assert.equal(defaultPoint.proofData, undefined);
for (const [circuit, args, expected] of equality) {
  await run('upstream-equality', circuit, args, expected, bool, !expected, !calls.has(circuit));
}
const id = suites.get('upstream-identity');
for (let i = 0; i < 2; ++i) {
  const call = id.contract.circuits.identityEqual(id.context(), variedIdentities[i], variedIdentities[i + 1]);
  assert.deepEqual(call.result, []);
  assert.equal(call.proofData, undefined);
}
for (const value of [...variedIdentities, five, xZero]) {
  const expected = value.identity ? identity : value;
  await run('upstream-identity', 'storePoint', [value], expected, point, seven,
    value === variedIdentities[1] || value === five);
}
for (const value of [...variedIdentities, five]) {
  const expected = referenceHash(value);
  const changed = Uint8Array.from(expected);
  changed[0] ^= 1;
  await run('upstream-identity', 'hashPoint', [value], expected, bytes32, changed, value === variedIdentities[2]);
}
assert.equal(Object.keys(cases).length, 17);
assert.equal(Object.keys(offCurve).length, 4);
assert.equal(Object.values(cases).filter(({ selected }) => selected).length, 5);
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases, offCurve }, null, 2) + '\n');
console.log('17 honest preimages, 17 changed writes refused, 17 changed returns refused, 4 off-curve refusals, 5 selected');
