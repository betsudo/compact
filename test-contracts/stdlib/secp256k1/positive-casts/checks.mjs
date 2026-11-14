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

import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

const directory = 'positive-casts';
const baseModulus = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2fn;
const scalarModulus = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141n;
const allOnes = (1n << 256n) - 1n;
const boundaries = [
  0n,
  1n,
  (1n << 192n) - 1n,
  1n << 192n,
  (1n << 192n) + 1n,
  (1n << 248n) - 1n,
  1n << 248n,
  (1n << 248n) + 1n,
  baseModulus - 1n,
  baseModulus,
  baseModulus + 1n,
  scalarModulus - 1n,
  scalarModulus,
  scalarModulus + 1n,
  allOnes,
];

function bytes32(value) {
  return Uint8Array.from({ length: 32 }, (_, index) => Number((value >> BigInt(index * 8)) & 0xffn));
}

async function loadContract(compile, file) {
  const { build, result } = compile(directory, { options: {} }, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract', 'index.js')).href;
  return import(/* @vite-ignore */ entry);
}

async function startContract(contractCode, witnesses, privateState) {
  const { contract, ctx } = await createTestContract(contractCode.Contract, witnesses, privateState);
  return [contract, ctx];
}

export function registerPositiveCastTests(compile) {
  test('runs the secp256k1 portions of the upstream constant-cast assertions', async () => {
    const contractCode = await loadContract(compile, 'constant-k1.compact');
    const [C, Ctxt] = await startContract(contractCode, {}, 0);
    const L = (r) => contractCode.ledger(r.context.currentQueryContext.state);
    const V = 452312848583266388373324160190187140051835877600158453279131187530910662661n;
    let l = L(await C.circuits.castK1(Ctxt, false, 7n, 8n));
    expect([l.k1b, l.k1s]).toEqual([V, V]);
    l = L(await C.circuits.castK1(Ctxt, true, 7n, 8n));
    expect([l.k1b, l.k1s]).toEqual([7n, 8n]);
    l = L(await C.circuits.castZero(Ctxt));
    expect([l.k1b, l.k1s]).toEqual([0n, 0n]);
  });

  test('reduces dynamic bytes modulo the selected field and encodes canonical values', async () => {
    const contractCode = await loadContract(compile, 'dynamic-k1.compact');
    const [contract, context] = await startContract(contractCode, {}, 0);
    const { decodeBase, decodeScalar, encodeBase, encodeScalar, guardedDecode } = contract.circuits;
    const ledger = (result) => contractCode.ledger(result.context.currentQueryContext.state);

    for (const value of boundaries) {
      const base = value % baseModulus;
      const scalar = value % scalarModulus;
      const label = `input 0x${value.toString(16)}`;
      const decodedBase = decodeBase(context, bytes32(value));
      const decodedScalar = decodeScalar(context, bytes32(value));
      expect(decodedBase.result, label).toBe(base);
      expect(ledger(decodedBase).baseCell, label).toBe(base);
      expect(decodedScalar.result, label).toBe(scalar);
      expect(ledger(decodedScalar).scalarCell, label).toBe(scalar);
      const encodedBase = encodeBase(context, base);
      const encodedScalar = encodeScalar(context, scalar);
      expect(encodedBase.result, label).toEqual(bytes32(base));
      expect(ledger(encodedBase).bytesCell, label).toEqual(bytes32(base));
      expect(encodedScalar.result, label).toEqual(bytes32(scalar));
      expect(ledger(encodedScalar).bytesCell, label).toEqual(bytes32(scalar));
    }

    const seeded = guardedDecode(context, true, bytes32(7n));
    const inactive = guardedDecode(seeded.context, false, bytes32(allOnes));
    expect(inactive.result).toEqual([7n, 7n]);
    expect(ledger(inactive)).toEqual(ledger(seeded));
    const active = guardedDecode(seeded.context, true, bytes32(allOnes));
    expect(active.result).toEqual([allOnes % baseModulus, allOnes % scalarModulus]);
    expect(ledger(active).baseCell).toBe(active.result[0]);
    expect(ledger(active).scalarCell).toBe(active.result[1]);

    expect(() => decodeBase(context, new Uint8Array(31))).toThrow(/expected value of type Bytes<32>/);
    expect(() => decodeScalar(context, new Uint8Array(33))).toThrow(/expected value of type Bytes<32>/);
    expect(() => encodeBase(context, baseModulus)).toThrow(/expected value of type Secp256k1Base/);
    expect(() => encodeScalar(context, scalarModulus)).toThrow(/expected value of type Secp256k1Scalar/);
    expect(() => encodeBase(context, -1n)).toThrow(/expected value of type Secp256k1Base/);
    expect(() => encodeScalar(context, -1n)).toThrow(/expected value of type Secp256k1Scalar/);
  });

  const wrongWidths = [
    ['wrong-decode-base.compact', /cannot cast from type Bytes<31> to type Secp256k1Base/],
    ['wrong-decode-scalar.compact', /cannot cast from type Bytes<33> to type Secp256k1Scalar/],
    ['wrong-encode-base.compact', /cannot cast from type Secp256k1Base to type Bytes<31>/],
    ['wrong-encode-scalar.compact', /cannot cast from type Secp256k1Scalar to type Bytes<33>/],
  ];
  for (const [file, diagnostic] of wrongWidths) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${file} refuses compilation for ${targets.join('+')}`, () => {
        const { build, result } = compile(directory, { options: {} }, targets, file);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(diagnostic);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
