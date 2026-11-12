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
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { CompactError, CompactTypeBoolean } from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import { identityRepresentations } from '../point/cases.mjs';
import { identityError, message, originalInvalid, signatureCases, witnesses } from './cases.mjs';

async function load(compile, file) {
  const { build, result } = compile('ecdsa', { options: {} }, ['ts', 'rust'], file);
  expect(result.status, result.stderr).toBe(0);
  const source = join(build, 'contract/index.js');
  const runtime = createRequire(source).resolve('@midnight-ntwrk/compact-runtime');
  const { Contract, ledger } = await import(/* @vite-ignore */ pathToFileURL(source).href);
  return { build, Contract, ledger, cases: await signatureCases(runtime) };
}

export function registerEcdsaTests(compile) {
  test('runs the original Ethereum signature false and identity assertions and accepts both s values', async () => {
    const { Contract, cases } = await load(compile, 'upstream-ordinary.compact');
    const { contract, ctx } = await createTestContract(Contract);
    const { first, second, high, changed, generator } = cases;
    const foo = (msg, sig, pk, expected) => {
      const invocation = contract.circuits.foo(ctx, msg, sig, pk);
      expect(invocation.result).toBe(expected);
      const cell = invocation.context.currentQueryContext.state.state.asArray()[0].asCell();
      expect(cell.alignment).toEqual(CompactTypeBoolean.alignment());
      expect(CompactTypeBoolean.fromValue(cell.value)).toBe(expected);
    };
    foo(message, originalInvalid, generator, false);
    foo(message, first.sig, first.pk, true);
    foo(message, high, first.pk, true);
    foo(changed, first.sig, first.pk, false);
    foo(message, first.sig, second.pk, false);
    for (const identity of identityRepresentations) {
      expect(() => contract.circuits.foo(ctx, message, originalInvalid, identity)).toThrow(CompactError);
      expect(() => contract.circuits.foo(ctx, message, originalInvalid, identity)).toThrow(identityError);
    }
    expect(() => contract.circuits.foo(ctx, message, { ...first.sig, s: 0n }, first.pk))
      .toThrow('secp256k1 scalar field has no inverse for 0');
  });

  test('runs the original #609 two-witness-pair circuit and independently refuses each assertion', async () => {
    const { build, Contract, ledger, cases } = await load(compile, 'upstream-double-609.compact');
    const { digest, first, second } = cases;
    const ir = JSON.parse(readFileSync(join(build, 'zkir/bump.zkir'), 'utf8'));
    const privateTypes = ir.instructions.filter((instruction) => instruction.op === 'private_input')
      .map((instruction) => instruction.type);
    expect(privateTypes).toEqual([
      'Scalar<Secp256k1>', 'Scalar<Secp256k1>', 'Point<Secp256k1>',
      'Scalar<Secp256k1>', 'Scalar<Secp256k1>', 'Point<Secp256k1>',
    ]);
    const invoke = async (left, right) => {
      const { contract, ctx } = await createTestContract(Contract, witnesses(left, right));
      return contract.circuits.bump(ctx, digest);
    };
    const success = await invoke(first, second);
    expect(success.result).toEqual([]);
    expect(ledger(success.context.currentQueryContext.state).n).toBe(1n);
    await expect(invoke({ ...first, sig: second.sig }, second)).rejects.toThrow('failed assert: b0');
    await expect(invoke(first, { ...second, sig: first.sig })).rejects.toThrow('failed assert: b1');
  });
}
