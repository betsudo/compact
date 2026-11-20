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
import { unaryFixtures } from './cases.mjs';

assert.equal(process.argv.length, 5, 'expected family output, SDK WASM package and new output directory');
const [family, wasmPackage, output] = process.argv.slice(2).map((arg) => resolve(arg));
const build = (fixture) => join(family, 'unary', fixture, 'ts-rust/.build');
const entry = (fixture) => join(build(fixture), 'contract/index.js');
const runtimeEntry = createRequire(entry('upstream-base')).resolve('@midnight-ntwrk/compact-runtime');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const manifest = JSON.parse(await readFile(join(wasmPackage, 'package.json')));
assert.equal(manifest.version, '3.2.0');
const wasmEntry = join(wasmPackage, manifest.exports.node);
const { check, jsonIrToBinary } = await import(pathToFileURL(wasmEntry));

const boolean = runtime.CompactTypeBoolean;
const fields = {
  Base: runtime.CompactTypeSecp256r1Base,
  Scalar: runtime.CompactTypeSecp256r1Scalar,
};
const sha = (data) => createHash('sha256').update(data).digest('hex');
const pin = async (path) => ({ path, sha256: sha(await readFile(path)) });
const tools = await Promise.all([runtimeEntry, onchainEntry, wasmEntry,
  join(wasmPackage, 'midnight_zkir_wasm_bg.wasm')].map(pin));
const cases = {};
const vectors = {};
const zeroRefusals = {};
const selected = new Set(['upstream-base-test-3', 'upstream-scalar-test-3',
  'unary-controls-guardedInvBase-0', 'unary-controls-guardedInvScalar-0']);
await mkdir(output);
async function save(name, data) {
  await writeFile(join(output, name), data, { flag: 'wx' });
  return sha(data);
}

function encode(value) {
  return typeof value === 'bigint' ? value.toString(16).padStart(64, '0') : value;
}

for (const { fixture, positives, zeroRefusals: refusals, purePositives = [], pureZeroRefusals = [] } of unaryFixtures) {
  assert.equal(createRequire(entry(fixture)).resolve('@midnight-ntwrk/compact-runtime'), runtimeEntry);
  const { Contract, pureCircuits } = await import(pathToFileURL(entry(fixture)));
  const contract = new Contract({});
  const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
  const context = () => runtime.createCircuitContext(runtime.dummyContractAddress(),
    initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
  const programs = new Map();
  const grouped = new Map();

  async function program(circuit) {
    if (!programs.has(circuit)) {
      const raw = join(build(fixture), 'zkir', circuit + '.zkir');
      const binary = raw.replace(/\.zkir$/, '.bzkir');
      const ir = jsonIrToBinary(await readFile(raw, 'utf8'));
      assert.deepEqual(Buffer.from(ir), await readFile(binary));
      programs.set(circuit, {
        artifacts: await Promise.all([raw, binary, entry(fixture), join(build(fixture), 'contract/lib.rs'),
          join(build(fixture), 'compiler/contract-info.json')].map(pin)),
        materials: {
          lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
          getParams: async () => { throw new Error('unexpected parameter request'); },
        },
      });
    }
    return programs.get(circuit);
  }

  function serialize(circuit, proof, transcript = proof.publicTranscript, output = proof.output, input = proof.input) {
    return proofDataIntoSerializedPreimage(input, output, transcript, proof.privateTranscriptOutputs, circuit);
  }
  for (const [circuit, args, expected] of positives) {
    const history = grouped.get(circuit) ?? [];
    const id = fixture + '-' + circuit + '-' + history.length;
    const base = fixture === 'upstream-base' || circuit.endsWith('Base');
    const field = fields[base ? 'Base' : 'Scalar'];
    const call = contract.circuits[circuit](context(), ...args);
    assert.deepEqual(call.result, expected, id);
    const cell = call.context.currentQueryContext.state.state.asArray()[0].asCell();
    assert.deepEqual(cell.alignment, boolean.alignment());
    assert.deepEqual(cell.value, boolean.toValue(true));
    const proof = call.proofData;
    const outputs = Array.isArray(expected) ? expected : [expected];
    const descriptor = (value) => typeof value === 'boolean' ? boolean : field;
    const atoms = (values) => values.flatMap((value) => descriptor(value).toValue(value));
    assert.deepEqual(proof.input.value, atoms(args), id);
    assert.deepEqual(proof.output.value, atoms(outputs), id);
    assert.deepEqual(proof.privateTranscriptOutputs, [], id);
    const { artifacts, materials } = await program(circuit);
    const honest = serialize(circuit, proof);
    await check(honest, materials);
    const transcript = structuredClone(proof.publicTranscript);
    const cells = transcript.filter((op) => op.push?.storage === true && op.push.value?.tag === 'cell');
    assert.equal(cells.length, 1, id);
    assert.deepEqual(cells[0].push.value.content.value, boolean.toValue(true));
    cells[0].push.value.content.value = boolean.toValue(false);
    const changedWrite = serialize(circuit, proof, transcript);
    await assert.rejects(() => check(changedWrite, materials), /transcript input mismatch/i);
    const changed = [...outputs];
    changed[0] = typeof changed[0] === 'boolean' ? !changed[0] : changed[0] === 0n ? 1n : 0n;
    const changedReturn = serialize(circuit, proof, proof.publicTranscript, { ...proof.output, value: atoms(changed) });
    await assert.rejects(() => check(changedReturn, materials), /communications commitment mismatch/i);
    cases[id] = {
      fixture, circuit, artifacts, selected: selected.has(id),
      preimage: await save(id + '.preimage', honest),
      changedWrite: await save(id + '.changed-write', changedWrite),
      changedReturn: await save(id + '.changed-return', changedReturn),
    };
    vectors[id] = { fixture, circuit, selected: selected.has(id),
      inputs: args.map(encode), expected: Array.isArray(expected) ? expected.map(encode) : encode(expected) };
    history.push({ args, proof });
    grouped.set(circuit, history);
  }

  for (const [circuit, args, error] of refusals) {
    const initialContext = context();
    assert.throws(() => contract.circuits[circuit](initialContext, ...args), (err) => err.message === error);
    assert.deepEqual(initialContext.currentQueryContext.state.state.asArray()[0].asCell().value, boolean.toValue(false));
    const { proof } = grouped.get(circuit).find(({ args }) => args.length === 1 || args[0] === true);
    const base = fixture === 'upstream-base' || circuit.endsWith('Base');
    const field = fields[base ? 'Base' : 'Scalar'];
    const input = structuredClone(proof.input);
    input.value.splice(-2, 2, ...field.toValue(0n));
    const invalid = serialize(circuit, proof, proof.publicTranscript, proof.output, input);
    const { materials } = await program(circuit);
    await assert.rejects(() => check(invalid, materials),
      { message: `cannot invert zero of type Secp256r1${base ? 'Base' : 'Scalar'}` });
    const id = fixture + '-' + circuit + '.zero';
    zeroRefusals[id] = await save(id, invalid);
  }
  for (const [name, args, expected] of purePositives) assert.deepEqual(pureCircuits[name](...args), expected);
  for (const [name, args, error] of pureZeroRefusals) {
    assert.throws(() => pureCircuits[name](...args), (err) => err.message === error);
  }
}
assert.deepEqual(Object.keys(cases).filter((id) => cases[id].selected).sort(), [...selected].sort());
assert.equal(Object.keys(cases).length, 30);
assert.equal(Object.keys(zeroRefusals).length, 6);
await save('vectors.json', JSON.stringify(vectors, null, 2) + '\n');
await save('summary.json', JSON.stringify({ tools, cases, zeroRefusals }, null, 2) + '\n');
console.log('30 honest preimages, 30 changed writes refused, 30 changed returns refused, 6 active-zero refusals, 4 selected');
