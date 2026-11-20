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
import { generator, xZero } from '../accessors/cases.mjs';
import { identity } from '../point/cases.mjs';
import { group, maxScalar, twoG } from './cases.mjs';

export function registerGroupChecks(compile) {
  for (const [fixture, operation, args] of [
    ['mixed-add', 'ecAdd', '(Secp256r1Point, Secp256k1Point)'],
    ['mixed-scalar', 'ecMul', '(Secp256r1Point, Secp256k1Scalar)'],
    ['base-generator', 'ecMulGenerator', '(Secp256r1Base)'],
  ]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`P-256 group refuses ${fixture} for ${targets.join('+')}`, () => {
        const { build, result } = compile(`group/${fixture}`, {}, targets);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toContain(`no compatible function named ${operation}`);
        expect(result.stderr).toContain(`supplied argument types:
        ${args}`);
        expect(existsSync(build)).toBe(false);
      });
    }
  }

  test('P-256 group results preserve guarded identity and exact finite coordinates', async () => {
    const fixture = group[1];
    const { build, result } = compile(fixture.folder, {}, ['ts', 'rust']);
    expect(result.status, result.stderr).toBe(0);
    const { Contract, ledger, pureCircuits } =
      await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
    const { contract, ctx } = await createTestContract(Contract);
    for (const { circuit, args, expected } of fixture.calls) {
      const call = contract.circuits[circuit](ctx, ...args);
      expect(call.result).toEqual(expected);
      expect(ledger(call.context.currentQueryContext.state).pt).toEqual(expected);
    }
    expect(pureCircuits.pureGroup(generator, 0n)).toEqual(identity);
    expect(pureCircuits.pureGroup(generator, 1n)).toEqual(twoG);
    expect(pureCircuits.pureGroup(xZero, 0n)).toEqual(identity);
    for (const scalar of [-1n, maxScalar + 1n]) {
      expect(() => pureCircuits.pureGroup(generator, scalar)).toThrow(/Secp256r1Scalar/);
    }
  });
}
