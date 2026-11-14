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
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

const vectors = JSON.parse(readFileSync(new URL('./upstream-vectors.json', import.meta.url)));

export function registerArithmeticChecks(compile) {
  test('runs all original P-256 arithmetic assertions in generated TypeScript', async () => {
    const { build, result } = compile('arithmetic', {}, ['ts', 'rust']);
    expect(result.status, result.stderr).toBe(0);
    const code = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
    const { contract, ctx } = await createTestContract(code.Contract, {}, 0);
    expect(vectors).toHaveLength(34);
    for (const { id, circuit, inputs, result } of vectors) {
      const field = circuit.endsWith('b') ? 'base' : 'scalar';
      const call = contract.circuits[circuit](ctx, ...inputs.map(BigInt));
      expect(call.result, id).toBe(BigInt(result));
      expect(code.ledger(call.context.currentQueryContext.state)[field], id).toBe(BigInt(result));
    }
    for (const [circuit, max, type] of [
      ['addb', runtime.MAX_SECP256R1_BASE, 'Secp256r1Base'],
      ['adds', runtime.MAX_SECP256R1_SCALAR, 'Secp256r1Scalar'],
    ]) {
      for (const bad of [-1n, max + 1n]) {
        expect(() => contract.circuits[circuit](ctx, bad, 0n)).toThrow(type);
        expect(() => contract.circuits[circuit](ctx, 0n, bad)).toThrow(type);
      }
    }
  });

  for (const [fixture, left, right, operator] of [
    ['base-scalar', 'Secp256r1Base', 'Secp256r1Scalar', '+'],
    ['base-k1', 'Secp256r1Base', 'Secp256k1Base', '-'],
    ['scalar-field', 'Secp256r1Scalar', 'Field', '*'],
  ]) {
    for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
      test(`arithmetic rejects ${left} ${operator} ${right} in ${targets.join('+')}`, () => {
        const { build, result } = compile(`arithmetic/${fixture}`, {}, targets);
        expect(result.status).not.toBe(0);
        expect(result.stderr).toContain("line 18 char 10");
        expect(result.stderr.replace(/\s+/g, " ")).toContain(
          `incompatible combination of types ${left} and ${right} for binary arithmetic operator ${operator}`);
        expect(existsSync(build)).toBe(false);
      });
    }
  }
}
