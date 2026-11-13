// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");

/** Capture proof preimages for the three remaining original Bytes map exports. */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const CASES = new Map([
  ['reordering', [[3, 2], [5, 4], [7, 6]]],
  ['slice_composition', [[20], [40], [60]]],
  ['to_bytes', [[0], [1], [2], [3], [4], [5], [6], [7]]],
]);
const [caseName, entryArg, rawArg, wasmArg, outputArg] = process.argv.slice(2);
assert.ok(CASES.has(caseName) && entryArg && rawArg && wasmArg && outputArg);
const circuit = `bytes_map_${caseName}`;
const entry = resolve(entryArg);
const rawPath = resolve(rawArg);
const wasmPath = resolve(wasmArg);
const output = resolve(outputArg);
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');

const wasmManifest = JSON.parse(await readFile(join(wasmPath, 'package.json')));
assert.equal(wasmManifest.version, '3.2.0');
const { check, jsonIrToBinary } = await import(pathToFileURL(join(wasmPath, wasmManifest.exports.node)));
const runtimeEntry = createRequire(entry).resolve('@midnight-ntwrk/compact-runtime');
const runtimeManifest = JSON.parse(await readFile(createRequire(entry).resolve('@midnight-ntwrk/compact-runtime/package.json')));
assert.equal(runtimeManifest.version, '0.18.0');
const runtime = await import(pathToFileURL(runtimeEntry));
const onchainEntry = createRequire(runtimeEntry).resolve('@midnightntwrk/onchain-runtime-v4');
const onchainManifest = JSON.parse(await readFile(join(dirname(onchainEntry), 'package.json')));
assert.equal(onchainManifest.version, '4.0.1');
const { proofDataIntoSerializedPreimage } = await import(pathToFileURL(onchainEntry));
const { Contract } = await import(pathToFileURL(entry));
const contract = new Contract({});
const initial = contract.initialState(runtime.createConstructorContext(undefined, '0'.repeat(64)));
const context = runtime.createCircuitContext(runtime.dummyContractAddress(), initial.currentZswapLocalState.coinPublicKey, initial.currentContractState, initial.currentPrivateState);
const execution = contract.circuits[circuit](context);
assert.deepEqual(execution.result, CASES.get(caseName).map((row) => Uint8Array.from(row)));
const proof = execution.proofData;
assert.ok(proof);
assert.deepEqual(proof.privateTranscriptOutputs, []);
const serialize = (transcript) => proofDataIntoSerializedPreimage(proof.input, proof.output, transcript, proof.privateTranscriptOutputs, circuit);
const honest = serialize(proof.publicTranscript);
const ir = jsonIrToBinary(await readFile(rawPath, 'utf8'));
const materials = {
  lookupKey: async () => ({ proverKey: new Uint8Array(), verifierKey: new Uint8Array(), ir }),
  getParams: async () => { throw Error('unexpected parameter request'); },
};
await check(honest, materials);

const changedValue = structuredClone(proof.publicTranscript);
const stored = changedValue.find((op) => op.push?.storage === true);
assert.ok(stored);
const changedStoredRowIndex = CASES.get(caseName).findIndex((row) => row[0] !== 0);
assert.ok(changedStoredRowIndex >= 0);
const bytes = stored.push.value.content.value[changedStoredRowIndex];
assert.deepEqual(Array.from(bytes), CASES.get(caseName)[changedStoredRowIndex]);
bytes[0] = (bytes[0] + 1) % 256;
const altered = serialize(changedValue);
await assert.rejects(() => check(altered, materials), /public transcript input mismatch/i);

const changedInsert = structuredClone(proof.publicTranscript);
const insert = changedInsert.find((op) => op.ins?.n === 1);
assert.ok(insert);
insert.ins.n = 2;
const alteredInsert = serialize(changedInsert);
await assert.rejects(() => check(alteredInsert, materials), /public transcript input mismatch/i);

await mkdir(output);
await writeFile(join(output, 'honest.preimage'), honest, { flag: 'wx' });
await writeFile(join(output, 'changed-value.preimage'), altered, { flag: 'wx' });
await writeFile(join(output, 'changed-insert.preimage'), alteredInsert, { flag: 'wx' });
const receipt = {
  source: `test-contracts/primitives/bytes/map/${caseName}/${circuit}.compact`,
  circuit,
  result: CASES.get(caseName),
  runtimeVersion: runtimeManifest.version,
  onchainRuntimeVersion: onchainManifest.version,
  wasmVersion: wasmManifest.version,
  rawZkirSha256: sha(await readFile(rawPath)),
  generatedTypeScriptSha256: sha(await readFile(entry)),
  honestPreimageSha256: sha(honest),
  changedValuePreimageSha256: sha(altered),
  changedInsertPreimageSha256: sha(alteredInsert),
  changedStoredRowIndex,
  checks: ['honest accepted', 'changed stored byte refused', 'changed insert count refused'],
};
await writeFile(join(output, 'capture.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify(receipt));
