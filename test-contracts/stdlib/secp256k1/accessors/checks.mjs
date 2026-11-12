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
import { CompactError } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';
import { generator, five, seven, identity } from '../point/cases.mjs';

const identities = [identity, { x: 3n, y: 4n, identity: true }, { ...seven, identity: true }];
const finite = [generator, five, seven];

async function load(compile, file) {
  const { build, result } = compile('accessors', { options: {} }, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const { Contract, ledger } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  return { ...await createTestContract(Contract, {}, 0), ledger };
}

function expectIdentityError(call, axis) {
  expect(call).toThrow(CompactError);
  expect(call).toThrow(`cannot extract the ${axis}-coordinate of the secp256k1 identity point`);
}

export function registerAccessorTests(compile) {
  test('preserves the original secp256k1 identity accessor refusals and guarded results', async () => {
    const { contract, ctx } = await load(compile, 'upstream-pure.compact');
    const { test0, test1, test2 } = contract.circuits;
    for (const pt of identities) {
      expectIdentityError(() => test0(ctx, pt), 'x');
      expectIdentityError(() => test1(ctx, pt), 'y');
      expect(test2(ctx, pt).result).toEqual([0n, 0n]);
    }
    for (const pt of finite) {
      expect(test0(ctx, pt).result).toEqual(pt.x);
      expect(test1(ctx, pt).result).toEqual(pt.y);
      expect(test2(ctx, pt).result).toEqual([pt.x, pt.y]);
    }
  });

  test('preserves the original stateful secp256k1 Cell coordinate assertions', async () => {
    const { contract, ctx, ledger } = await load(compile, 'upstream-ledger.compact');
    const { storePoint, storeX, storeY } = contract.circuits;
    const read = (context) => ledger(context.currentQueryContext.state);
    expect(read(ctx).pt).toEqual(identity);
    expectIdentityError(() => storeX(ctx), 'x');
    expectIdentityError(() => storeY(ctx), 'y');
    let current = ctx;
    for (const pt of finite) {
      const stored = storePoint(current, pt);
      expect(read(stored.context).pt).toEqual(pt);
      const withX = storeX(stored.context);
      expect(read(withX.context).x).toBe(pt.x);
      const withY = storeY(withX.context);
      expect(read(withY.context).y).toBe(pt.y);
      current = withY.context;
    }
    const back = storePoint(current, identity);
    expectIdentityError(() => storeX(back.context), 'x');
    expectIdentityError(() => storeY(back.context), 'y');
  });

  test('checks active, inactive and discarded stateful secp256k1 accesses', async () => {
    const { contract, ctx, ledger } = await load(compile, 'guarded-accessors.compact');
    const { writeX, writeY, discardX } = contract.circuits;
    const read = (result) => ledger(result.context.currentQueryContext.state);
    for (const pt of identities) {
      expect(read(writeX(ctx, pt, false)).x).toBe(0n);
      expect(read(writeY(ctx, pt, false)).y).toBe(0n);
      expectIdentityError(() => writeX(ctx, pt, true), 'x');
      expectIdentityError(() => writeY(ctx, pt, true), 'y');
      expectIdentityError(() => discardX(ctx, pt), 'x');
    }
    for (const pt of finite) {
      expect(read(writeX(ctx, pt, true)).x).toBe(pt.x);
      expect(read(writeY(ctx, pt, true)).y).toBe(pt.y);
      expect(read(discardX(ctx, pt)).seen).toBe(true);
    }
  });

  test('preserves the upstream assertion before a secp256k1 accessor', async () => {
    const { contract, ctx, ledger } = await load(compile, 'upstream-assertion.compact');
    expect(() => contract.circuits.storeXChecked(ctx)).toThrow(/the identity has no coordinates/);
    const stored = contract.circuits.storePoint(ctx, generator);
    const checked = contract.circuits.storeXChecked(stored.context);
    expect(ledger(checked.context.currentQueryContext.state).x).toBe(generator.x);
  });

  test('preserves discarded identity access in the constructor', async () => {
    const { build, result } = compile('accessors', { options: {} }, ['ts', 'rust'], 'constructor-accessors.compact');
    expect(result.status, result.stderr).toBe(0);
    const { Contract, ledger } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
    for (const pt of finite) {
      const { ctx } = await createTestContract(Contract, {}, 0, [pt]);
      expect(ledger(ctx.currentQueryContext.state).x).toBe(pt.x);
    }
    for (const pt of identities) {
      await expect(createTestContract(Contract, {}, 0, [pt]))
        .rejects.toThrow('cannot extract the y-coordinate of the secp256k1 identity point');
    }
  });

  for (const [file, name] of [['refuse-jubjub.compact', 'secp256k1PointX'], ['refuse-field.compact', 'secp256k1PointY']]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`${name} refuses the wrong carrier for ${targets.join('+')}`, () => {
        const { build, result } = compile('accessors', { options: {} }, targets, file);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toMatch(new RegExp(`no compatible function named ${name}`));
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
