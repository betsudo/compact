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
import { identity, pointAtoms } from '../point/cases.mjs';
import { hashCases, pointTupleDigest, witnessCases } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const build = (fixture) => join(family, 'transport', fixture, 'ts-rust/.build');
const entry = (fixture) => join(build(fixture), 'contract/index.js');
const witnessFixture = 'upstream-witness';
const hashFixture = 'upstream-hash608';
const runtimeEntry = createRequire(entry(witnessFixture)).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const requireRuntime = createRequire(runtimeEntry);
const onchainEntry = requireRuntime.resolve('@midnightntwrk/onchain-runtime-v4');
const nobleEntry = requireRuntime.resolve('@noble/curves/nist.js');
const keccakEntry = requireRuntime.resolve('@noble/hashes/sha3.js');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const { p256 } = await import(pathToFileURL(nobleEntry));
const { keccak_256 } = await import(pathToFileURL(keccakEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const point = runtime.CompactTypeSecp256r1Point;
const bytes = new runtime.CompactTypeBytes(32);
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, nobleEntry, keccakEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const hex = (value) => Buffer.from(value).toString('hex');
const encodePoint = ({ x, y, identity }) => ({
  x: x.toString(16).padStart(64, '0'), y: y.toString(16).padStart(64, '0'), identity,
});
const malformed = { ...identity, identity: false };
const invalidPoint = /Failed to decode .*Secp256r1Point|encoded value of type Secp256r1Point.*not in canonical form/i;

const cases = {};
const vectors = {};
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}

async function execute({ fixture, circuit, id, args, witness, digest, selected }) {
  const source = entry(fixture);
  assert.equal(createRequire(source).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const module = await import(pathToFileURL(source));
  const infoPath = join(build(fixture), 'compiler/contract-info.json');
  const info = JSON.parse(await readFile(infoPath));
  assert.equal(info.circuits.find(({ name }) => name === circuit).proof, true);
  const calls = [];
  const callbacks = {};
  if (witness) {
    for (const name of ['point0', 'point1']) {
      callbacks[name] = ({ privateState }) => {
        calls.push(name);
        assert.equal(name, witness.callback, id);
        return [privateState + witness.step, witness.value];
      };
    }
  }
  const contract = new module.Contract(callbacks);
  const initial = contract.initialState(runtime.createConstructorContext(witness?.initial, '0'.repeat(64)));
  const context = runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const call = contract.circuits[circuit](context, ...args);
  const proof = call.proofData;
  assert.deepEqual(proof.input.alignment, args.flatMap(() => point.alignment()), id);
  assert.deepEqual(proof.input.value, args.flatMap(pointAtoms), id);
  assert.deepEqual(call.result, witness ? witness.value : [], id);
  const descriptor = witness ? point : bytes;
  const writeValue = witness ? witness.value : digest;
  const writeAtoms = witness ? pointAtoms(writeValue) : bytes.toValue(writeValue);
  const stateCell = call.context.currentQueryContext.state.state.asArray()[0].asCell();
  assert.deepEqual(stateCell, { alignment: descriptor.alignment(), value: writeAtoms }, id);
  if (witness) {
    assert.deepEqual(calls, [witness.callback], id);
    assert.equal(context.currentPrivateState, witness.initial, id);
    assert.equal(call.context.currentPrivateState, witness.initial + witness.step, id);
    assert.deepEqual(proof.output, stateCell, id);
    assert.deepEqual(proof.privateTranscriptOutputs, [stateCell], id);
  } else {
    assert.deepEqual(proof.output, { value: [], alignment: [] }, id);
    assert.deepEqual(proof.privateTranscriptOutputs, [], id);
  }

  const rawPath = join(build(fixture), 'zkir', circuit + '.zkir');
  const binary = rawPath.replace(/\.zkir$/, '.bzkir');
  const raw = await readFile(rawPath, 'utf8');
  const program = JSON.parse(raw);
  assert(program.do_communications_commitment, id);
  const inputTypes = args.map(() => 'Point<Secp256r1>');
  const outputTypes = witness ? ['Point<Secp256r1>'] : [];
  assert.deepEqual(program.inputs.map(({ type }) => type), inputTypes, id);
  assert.deepEqual(program.outputs, outputTypes, id);
  const privateInputs = program.instructions.filter(({ op }) => op === 'private_input');
  assert.deepEqual(privateInputs.map(({ type }) => type), outputTypes, id);
  const encodings = program.instructions.filter(({ op }) => op === 'encode');
  if (witness) {
    const privateEncoding = encodings.find(({ input }) => input === privateInputs[0].output);
    assert.equal(privateEncoding.outputs.length, 5, id);
  } else {
    const hash = program.instructions.find(({ op }) => op === 'keccak256');
    assert.deepEqual(hash.alignment, proof.input.alignment, id);
    assert.deepEqual(hash.inputs, program.inputs.flatMap(({ name }) =>
      encodings.find(({ input }) => input === name).outputs), id);
  }
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(binary), id);
  const materials = {
    lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
    getParams: async () => { throw new Error('unexpected parameter request'); },
  };
  const artifacts = await Promise.all([rawPath, binary, infoPath, source, join(build(fixture), 'contract/lib.rs')].map(pin));
  const serialize = ({ input = proof.input, result = proof.output,
    transcript = proof.publicTranscript, privateOutputs = proof.privateTranscriptOutputs } = {}) =>
    proofDataIntoSerializedPreimage(input, result, transcript, privateOutputs, circuit);
  const honest = serialize();
  await check(honest, materials);
  const row = { fixture, folder: 'transport/' + fixture, circuit, selected, artifacts,
    privateFields: witness ? 5 : 0, preimage: await save(id + '.preimage', honest), refusals: {} };
  const refuse = async (kind, mutation, reason) => {
    const invalid = serialize(mutation);
    await assert.rejects(() => check(invalid, materials), reason, id);
    row.refusals[kind] = await save(id + '.' + kind, invalid);
  };

  const transcript = structuredClone(proof.publicTranscript);
  const cells = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(cells.length, 1, id);
  assert.deepEqual(cells[0].push.value.content, stateCell, id);
  const alternate = witness ? pointAtoms(identity)
    : bytes.toValue(Uint8Array.from(digest, (byte, i) => i === 0 ? byte ^ 1 : byte));
  cells[0].push.value.content.value = alternate;
  await refuse('changed-write', { transcript }, /transcript input mismatch/i);
  if (witness) {
    await refuse('changed-return', { result: { ...proof.output, value: alternate } },
      /communications commitment mismatch/i);
    await refuse('substituted-witness', { privateOutputs: [{ ...stateCell, value: alternate }] },
      /transcript input mismatch/i);
    await refuse('off-curve-witness', { privateOutputs: [{ ...stateCell, value: pointAtoms(malformed) }] },
      invalidPoint);
  } else {
    const input = (points) => ({ ...proof.input, value: points.flatMap(pointAtoms) });
    await refuse('reordered-input', { input: input(args.slice().reverse()) }, /transcript input mismatch/i);
    await refuse('off-curve-input', { input: input([malformed, args[1]]) }, invalidPoint);
  }
  cases[id] = row;
  vectors[id] = { fixture, circuit, selected, inputs: args.map(encodePoint),
    expected: witness ? encodePoint(witness.value) : hex(digest) };
  if (witness) {
    vectors[id].initial = witness.initial;
    vectors[id].step = witness.step;
  }
}

for (const witness of witnessCases) {
  await execute({ fixture: witnessFixture, circuit: witness.circuit, id: witnessFixture + '-' + witness.circuit,
    args: [], witness, selected: true });
}
for (const [index, { args, selected }] of hashCases(p256).entries()) {
  await execute({ fixture: hashFixture, circuit: 'test', id: hashFixture + '-test-' + index,
    args, digest: pointTupleDigest(keccak_256, args), selected });
}
assert.equal(Object.keys(cases).length, 5);
assert.equal(Object.values(cases).filter(({ selected }) => selected).length, 3);
assert.equal(Object.values(cases).reduce((n, { refusals }) => n + Object.keys(refusals).length, 0), 17);
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases }, null, 2) + '\n');
console.log('5 SDK preimages admitted, 17 mutations refused');
