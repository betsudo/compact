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
import { bytesHex, coordinateHex, encodePoint } from '../ecdsa/cases.mjs';
import { sequentialCases, witnessInputs, witnessOrder, witnesses } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const folder = 'signature-suites/sequential';
const build = join(family, folder, 'ts-rust/.build');
const source = join(build, 'contract/index.js');
const runtimeEntry = createRequire(source).resolve('@midnight-ntwrk/compact-runtime');
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
const { Contract, ledger } = await import(pathToFileURL(source));

const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, nobleEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const point = runtime.CompactTypeSecp256r1Point;
const scalar = runtime.CompactTypeSecp256r1Scalar;
const bytes = new runtime.CompactTypeBytes(32);
const u64 = new runtime.CompactTypeUnsignedInteger((1n << 64n) - 1n, 8);
const signatureAtoms = ({ r, s }) => [...scalar.toValue(r), ...scalar.toValue(s)];
const signatureAlignment = [...scalar.alignment(), ...scalar.alignment()];
const values = sequentialCases(p256);
const cases = {};
const vectors = {};
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}

function call(circuit, count, inputs, expected) {
  const calls = [];
  const contract = new Contract(witnesses(inputs, calls));
  const initial = contract.initialState(runtime.createConstructorContext(0, '0'.repeat(64)));
  const context = runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const invocation = contract.circuits[circuit](context, values.digest);
  assert.deepEqual(calls, witnessOrder(count));
  assert.equal(invocation.context.currentPrivateState, 2 * count);
  assert.deepEqual(invocation.result, []);
  assert.equal(ledger(invocation.context.currentQueryContext.state).n, BigInt(expected));
  const proof = invocation.proofData;
  assert.deepEqual(proof.input, { alignment: bytes.alignment(), value: bytes.toValue(values.digest) });
  assert.deepEqual(proof.output, { alignment: [], value: [] });
  assert.deepEqual(proof.privateTranscriptOutputs, inputs.signatures.flatMap((signature, slot) => [
    { alignment: signatureAlignment, value: signatureAtoms(signature) },
    { alignment: point.alignment(), value: pointAtoms(inputs.points[slot]) },
  ]));
  return proof;
}

for (const [circuit, count] of [['two', 2], ['three', 3]]) {
  const rawPath = join(build, 'zkir', circuit + '.zkir');
  const binaryPath = rawPath.replace(/\.zkir$/, '.bzkir');
  const raw = await readFile(rawPath, 'utf8');
  const program = JSON.parse(raw);
  const infoPath = join(build, 'compiler/contract-info.json');
  const info = JSON.parse(await readFile(infoPath, 'utf8'));
  assert.equal(info.circuits.find(({ name }) => name === circuit).proof, true);
  assert(program.do_communications_commitment);
  assert.deepEqual(program.inputs.map(({ type }) => type), ['Scalar<BLS12-381>', 'Scalar<BLS12-381>']);
  assert.deepEqual(program.outputs, []);
  assert.deepEqual(program.instructions.filter(({ op }) => op === 'private_input').map(({ type }) => type),
    Array.from({ length: count }, () => ['Scalar<Secp256r1>', 'Scalar<Secp256r1>', 'Point<Secp256r1>']).flat());
  const ir = jsonIrToBinary(raw);
  assert.deepEqual(Buffer.from(ir), await readFile(binaryPath));
  const materials = {
    lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
    getParams: async () => { throw new Error('unexpected parameter request'); },
  };
  const artifacts = await Promise.all([rawPath, binaryPath, infoPath, source, join(build, 'contract/lib.rs')].map(pin));
  const serialize = (proof, { transcript = proof.publicTranscript,
    privateOutputs = proof.privateTranscriptOutputs } = {}) =>
    proofDataIntoSerializedPreimage(proof.input, proof.output, transcript, privateOutputs, circuit);
  const admit = async (id, inputs, expected) => {
    const proof = call(circuit, count, inputs, expected);
    const serialized = serialize(proof);
    await check(serialized, materials);
    const row = { fixture: 'verify-sequential', folder, circuit, selected: expected === 1,
      privateFields: 9 * count, artifacts, preimage: await save(id + '.preimage', serialized), refusals: {} };
    cases[id] = row;
    vectors[id] = { fixture: row.fixture, circuit, selected: row.selected, expected,
      digest: bytesHex(values.digest), points: inputs.points.map(encodePoint),
      signatures: inputs.signatures.map(({ r, s }) => ({ r: coordinateHex(r), s: coordinateHex(s) })) };
    return { proof, row };
  };
  const { proof, row } = await admit(circuit, witnessInputs(values, count), 1);
  const refuse = async (kind, mutation, diagnostic) => {
    const invalid = serialize(proof, mutation);
    await assert.rejects(() => check(invalid, materials), diagnostic, kind);
    row.refusals[kind] = await save(circuit + '.' + kind, invalid);
  };
  const transcript = structuredClone(proof.publicTranscript);
  const writes = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
  assert.equal(writes.length, 1);
  assert.deepEqual(writes[0].push.value.content, { alignment: u64.alignment(), value: u64.toValue(1n) });
  writes[0].push.value.content.value = u64.toValue(0n);
  await refuse('changed-write', { transcript }, /transcript input mismatch/i);
  for (let slot = 0; slot < count; slot++) {
    const other = (slot + 1) % values.points.length;
    for (const [kind, offset, key, atoms] of [
      ['signature', 0, 'signatures', signatureAtoms], ['key', 1, 'points', pointAtoms],
    ]) {
      const inputs = witnessInputs(values, count);
      inputs[key][slot] = values[key][other];
      await admit(`${circuit}-${kind}-${slot}`, inputs, 0);
      const privateOutputs = structuredClone(proof.privateTranscriptOutputs);
      privateOutputs[2 * slot + offset].value = atoms(inputs[key][slot]);
      await refuse(`changed-${kind}-${slot}`, { privateOutputs }, /transcript input mismatch/i);
    }
  }
  for (const [kind, slot, value, diagnostic] of [
    ['zero-s', 0, signatureAtoms({ ...values.signatures[0], s: 0n }), /invert|inverse/i],
    ['identity-key', 1, pointAtoms(identity), /Failed direct assertion/i],
    ['off-curve-key', 1, pointAtoms({ ...identity, identity: false }),
      /Failed to decode .*Secp256r1Point|encoded value of type Secp256r1Point.*not in canonical form/i],
  ]) {
    const privateOutputs = structuredClone(proof.privateTranscriptOutputs);
    privateOutputs[slot].value = value;
    await refuse(kind, { privateOutputs }, diagnostic);
  }
}
assert.equal(Object.keys(cases).length, 12);
assert.equal(Object.values(cases).filter(({ selected }) => selected).length, 2);
assert.equal(Object.values(cases).reduce((n, { refusals }) => n + Object.keys(refusals).length, 0), 18);
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases }, null, 2) + '\n');
console.log('12 SDK sequential preimages admitted, 18 mutations refused');
