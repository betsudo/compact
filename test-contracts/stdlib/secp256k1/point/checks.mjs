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

import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createTestContract } from '@test/compact-test';
import {
  CompactError,
  CompactTypeBytes,
  CompactTypeMerkleTreePath,
  CompactTypeSecp256k1Point,
  CompactTypeVector,
  isValidSecp256k1Point,
} from '@midnight-ntwrk/compact-runtime';
import { expect, test } from 'vitest';

import {
  baseModulus,
  equalityCases,
  five,
  generator,
  identity,
  identityRepresentations,
  ignoredCoordinateCases,
  ledgerSteps,
  malformedPoints,
  pointHashCases,
} from './cases.mjs';

const directory = 'point';
const definitions = { options: {} };
const pointAlignment = [
  ...[24, 8, 24, 8].map((length) => ({ tag: 'atom', value: { tag: 'bytes', length } })),
  { tag: 'atom', value: { tag: 'field' } },
];

async function loadContract(compile, file) {
  const { build, result } = compile(directory, definitions, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const entry = pathToFileURL(join(build, 'contract', 'index.js')).href;
  const { Contract, ledger } = await import(/* @vite-ignore */ entry);
  return { ...await createTestContract(Contract, {}, 0), build, ledger };
}

function atom(value) {
  const bytes = [];
  for (let remaining = value; remaining !== 0n; remaining >>= 8n) {
    bytes.push(Number(remaining & 0xffn));
  }
  return Uint8Array.from(bytes);
}

function coordinateAtoms(value) {
  const shifted = (value + baseModulus - 1n) % baseModulus;
  return [atom(shifted & ((1n << 192n) - 1n)), atom(shifted >> 192n)];
}

function expectedAtoms({ x, y, identity: isIdentity }) {
  return [
    ...coordinateAtoms(isIdentity ? 0n : x),
    ...coordinateAtoms(isIdentity ? 0n : y),
    atom(isIdentity ? 1n : 0n),
  ];
}

function checkPrefixDecoding(descriptor, value) {
  const encoded = descriptor.toValue(value);
  expect(encoded).toHaveLength(descriptor.alignment().length);
  const prefixOnly = encoded.slice();
  expect(descriptor.fromValue(prefixOnly)).toStrictEqual(value);
  expect(prefixOnly).toHaveLength(0);
  const suffix = [new Uint8Array([9]), new Uint8Array([10])];
  const chained = encoded.concat(suffix);
  expect(descriptor.fromValue(chained)).toStrictEqual(value);
  expect(chained).toStrictEqual(suffix);
  expect(() => descriptor.fromValue(encoded.slice(0, -1))).toThrow(CompactError);
}

export function registerPointTests(compile) {
  test('runs the original secp256k1 point equality and default assertions', async () => {
    const { contract, ctx } = await loadContract(compile, 'upstream-equality.compact');
    const defaultPoint = contract.circuits.getDefault(ctx).result;
    expect(defaultPoint).toStrictEqual(identity);
    for (const [circuit, operands, expected] of [...equalityCases, ...ignoredCoordinateCases]) {
      const args = operands.map((point) => point === 'default' ? defaultPoint : point);
      expect(contract.circuits[circuit](ctx, ...args).result, circuit).toEqual(expected);
    }
    for (const [label, malformed] of malformedPoints) {
      expect(() => contract.circuits.pointsEqual(ctx, malformed, five), label)
        .toThrow(/expected value of type Secp256k1Point/);
      expect(() => contract.circuits.pointsNotEqual(ctx, five, malformed), label)
        .toThrow(/expected value of type Secp256k1Point/);
    }
  });

  test('runs the original #795 point identity, persistent hash and Cell assertions', async () => {
    const { contract, ctx, ledger } = await loadContract(compile, 'upstream-identity-795.compact');
    const [identity0, identity1, identity2] = identityRepresentations;
    const { identityEqual, hashPoint, storePoint } = contract.circuits;
    identityEqual(ctx, identity0, identity1);
    identityEqual(ctx, identity1, identity2);
    const hash0 = hashPoint(ctx, identity0).result;
    expect(hash0).toHaveLength(32);
    expect(hashPoint(ctx, identity1).result).toEqual(hash0);
    expect(hashPoint(ctx, identity2).result).toEqual(hash0);
    expect(hashPoint(ctx, five).result).not.toEqual(hash0);
    for (const point of identityRepresentations) {
      const result = storePoint(ctx, point);
      expect(result.result).toEqual(identity);
      expect(ledger(result.context.currentQueryContext.state).point).toEqual(identity);
    }
  });

  test('round trips, reads and resets a point Cell through generated TypeScript', async () => {
    const { contract, ctx, ledger } = await loadContract(compile, 'point-ledger.compact');
    let context = ctx;
    for (const [circuit, args, expected] of ledgerSteps) {
      const result = contract.circuits[circuit](context, ...args);
      expect(result.result, circuit).toStrictEqual(expected);
      expect(ledger(result.context.currentQueryContext.state).point, circuit).toStrictEqual(expected);
      context = result.context;
    }
    for (const [label, malformed] of malformedPoints) {
      expect(isValidSecp256k1Point(malformed), label).toBe(false);
      expect(() => contract.circuits.storePoint(context, malformed), label)
        .toThrow(/expected value of type Secp256k1Point/);
    }
    expect(contract.circuits.readPoint(context).result).toStrictEqual(identity);
  });

  const hashFixtures = [
    ['upstream-keccak-pair.compact', 'test1', 'keccak256', 32],
    ['upstream-sha512.compact', 'test', 'sha512', 64],
  ];
  for (const [file, circuit, algorithm, width] of hashFixtures) {
    test(`writes the aligned ${algorithm} of two points to the original ledger Cell`, async () => {
      const { contract, ctx, build } = await loadContract(compile, file);
      const zkir = JSON.parse(readFileSync(join(build, 'zkir', `${circuit}.zkir`), 'utf8'));
      expect(zkir.inputs.map(({ type }) => type)).toEqual(['Point<Secp256k1>', 'Point<Secp256k1>']);
      const encoded = zkir.inputs.flatMap(({ name }) => {
        const encoders = zkir.instructions.filter(({ op, input }) => op === 'encode' && input === name);
        expect(encoders).toHaveLength(1);
        expect(encoders[0].outputs).toHaveLength(5);
        return encoders[0].outputs;
      });
      const hashes = zkir.instructions.filter(({ op }) => op === algorithm);
      expect(hashes).toHaveLength(1);
      expect(hashes[0]).toMatchObject({ alignment: [...pointAlignment, ...pointAlignment], inputs: encoded });
      const descriptor = new CompactTypeBytes(width);
      for (const pointCase of pointHashCases) {
        const result = contract.circuits[circuit](ctx, ...pointCase.args);
        expect(result.result).toEqual([]);
        // The original fixture's single ledger Cell is intentionally internal.
        const state = result.context.currentQueryContext.state.state.asArray();
        expect(state).toHaveLength(1);
        const cell = state[0].asCell();
        expect(cell.alignment).toEqual(descriptor.alignment());
        const digest = descriptor.fromValue(cell.value);
        expect(Buffer.from(digest).toString('hex'), pointCase.label).toBe(pointCase[algorithm]);
      }
    });
  }

  test('the point descriptor uses five FAB atoms and consumes only its own prefix', () => {
    const descriptor = CompactTypeSecp256k1Point;
    expect(descriptor.alignment()).toStrictEqual(pointAlignment);
    for (const point of [generator, ...identityRepresentations]) {
      expect(isValidSecp256k1Point(point)).toBe(true);
      expect(descriptor.toValue(point)).toStrictEqual(expectedAtoms(point));
    }
    for (const point of [generator, identity, { x: 1n, y: 1n, identity: false }]) {
      checkPrefixDecoding(descriptor, point);
    }
    // The descriptor is a structural codec. Its five empty atoms are not a valid host point.
    const emptyAtoms = Array.from({ length: 5 }, () => new Uint8Array(0));
    expect(descriptor.toValue({ x: 1n, y: 1n, identity: false })).toStrictEqual(emptyAtoms);
    const invalidFlag = descriptor.toValue(generator);
    invalidFlag[4] = new Uint8Array([2]);
    expect(() => descriptor.fromValue(invalidFlag)).toThrow(/expected Secp256k1Point/);
  });

  test('point descriptors chain inside a vector and before the Merkle path atoms', () => {
    const point = CompactTypeSecp256k1Point;
    checkPrefixDecoding(new CompactTypeVector(2, point), [generator, identity]);
    checkPrefixDecoding(new CompactTypeMerkleTreePath(2, point), {
      leaf: generator,
      path: [
        { sibling: { field: 1n }, goes_left: true },
        { sibling: { field: 2n }, goes_left: false },
      ],
    });
  });

  const refusals = [
    ['compare-jubjub.compact', /incompatible types Secp256k1Point and JubjubPoint for equality operator/],
    ['compare-lookalike.compact', /incompatible types Secp256k1Point and struct PointLookalike<x: Secp256k1Base, y: Secp256k1Base,\s+identity: Boolean> for equality operator/],
  ];
  for (const [file, diagnostic] of refusals) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${file} refuses compilation for ${targets.join('+')}`, () => {
        const { build, result } = compile(directory, definitions, targets, file);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(diagnostic);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
