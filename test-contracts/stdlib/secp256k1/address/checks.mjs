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

import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { CompactTypeBytes, secp256k1MulGenerator } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import { five, seven, identityRepresentations, pointHashCases } from '../point/cases.mjs';
import { coordinateBytes, expected, identityError, point } from './cases.mjs';

async function load(compile, file) {
  const { build, result } = compile('address', { options: {} }, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const source = join(build, 'contract/index.js');
  const { Contract } = await import(/* @vite-ignore */ pathToFileURL(source).href);
  const runtime = createRequire(source).resolve('@midnight-ntwrk/compact-runtime');
  const noble = createRequire(runtime).resolve('@noble/hashes/sha3.js');
  const { keccak_256 } = await import(/* @vite-ignore */ pathToFileURL(noble).href);
  return { ...await createTestContract(Contract, {}, 0), keccak_256 };
}

function cell(result, slot, width) {
  const value = result.context.currentQueryContext.state.state.asArray()[slot].asCell();
  const descriptor = new CompactTypeBytes(width);
  expect(value.alignment).toEqual(descriptor.alignment());
  return descriptor.fromValue(value.value);
}

export function registerAddressTests(compile) {
  test('runs the complete upstream issue 608 address and nested hash fixture', async () => {
    const { contract, ctx, keccak_256 } = await load(compile, 'upstream-issue608.compact');
    const p0 = secp256k1MulGenerator(5n);
    const p1 = secp256k1MulGenerator(7n);
    expect(p0).toStrictEqual(five);
    expect(p1).toStrictEqual(seven);
    const writeAddress = contract.circuits.test0(ctx, p0);
    const writeHash = contract.circuits.test1(ctx, p0, p1);
    expect(writeAddress.result).toEqual([]);
    expect(writeHash.result).toEqual([]);
    const address = keccak_256(Buffer.concat([coordinateBytes(five.x), coordinateBytes(five.y)])).slice(12);
    expect(cell(writeAddress, 0, 20)).toStrictEqual(address);
    const pairedHash = pointHashCases.find(({ args }) => args[0] === five && args[1] === seven);
    expect(Buffer.from(cell(writeHash, 1, 32)).toString('hex')).toBe(pairedHash.keccak256);
  });

  test('matches the original Ethereum address byte-order vector and refuses identity', async () => {
    const { contract, ctx, keccak_256 } = await load(compile, 'upstream-byte-order.compact');
    const result = contract.circuits.test(ctx, point);
    expect(result.result).toEqual(expected);
    expect(cell(result, 0, 20)).toEqual(expected);

    const littleEndian = Buffer.concat([
      coordinateBytes(point.x).reverse(),
      coordinateBytes(point.y).reverse(),
    ]);
    expect(keccak_256(littleEndian).slice(12)).not.toEqual(expected);
    for (const identity of identityRepresentations) {
      expect(() => contract.circuits.test(ctx, identity)).toThrow(identityError);
    }
  });
}
