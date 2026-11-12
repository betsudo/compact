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
import { expect, test } from 'vitest';

import { unaryFixtures } from './cases.mjs';

const directory = 'unary';
const definitions = { options: {} };

function readZkir(build, circuit) {
  return JSON.parse(readFileSync(join(build, 'zkir', `${circuit}.zkir`), 'utf8'));
}

function singleInstruction(zkir, operator) {
  const instructions = zkir.instructions.filter(({ op }) => op === operator);
  expect(instructions, operator).toHaveLength(1);
  return instructions[0];
}

function checkUpstreamOperations(build, field) {
  const zkir = readZkir(build, 'test');
  const type = `${field}<Secp256k1>`;
  expect(zkir.version).toEqual({ major: 3, minor: 0 });
  expect(zkir.do_communications_commitment).toBe(true);
  expect(zkir.inputs.map((input) => input.type)).toEqual([type]);
  expect(zkir.outputs).toEqual([type, type]);
  const input = zkir.inputs[0].name;
  const negation = singleInstruction(zkir, 'neg');
  const inversion = singleInstruction(zkir, 'inv');
  expect(negation.a).toBe(input);
  expect(inversion.a).toBe(input);
  expect(singleInstruction(zkir, 'output').vals).toEqual([negation.output, inversion.output]);
}

export function registerUnaryTests(compile) {
  for (const { file, positives, zeroRefusals, purePositives = [], pureZeroRefusals = [] } of unaryFixtures) {
    test(`${file} runs generated unary operations and refuses zero inverses`, async () => {
      const { build, result } = compile(directory, definitions, ['ts', 'rust'], file);
      expect(result.status, result.stderr).toBe(0);
      const entry = pathToFileURL(join(build, 'contract', 'index.js')).href;
      const { Contract, ledger, pureCircuits } = await import(/* @vite-ignore */ entry);
      const { contract, ctx } = await createTestContract(Contract, {}, 0);
      const isControls = file === 'unary-controls.compact';

      for (const [circuit, args, expected] of positives) {
        const value = contract.circuits[circuit](ctx, ...args);
        expect(value.result, `${circuit}(${args.join(', ')})`).toEqual(expected);
        if (isControls) {
          expect(ledger(value.context.currentQueryContext.state).wantProof).toBe(true);
        }
      }
      for (const [circuit, args, diagnostic] of zeroRefusals) {
        expect(() => contract.circuits[circuit](ctx, ...args)).toThrow(new RegExp(`^${diagnostic}$`));
      }

      for (const [circuit, args, expected] of purePositives) {
        expect(pureCircuits[circuit](...args)).toEqual(expected);
      }
      for (const [circuit, args, diagnostic] of pureZeroRefusals) {
        expect(() => pureCircuits[circuit](...args)).toThrow(new RegExp(`^${diagnostic}$`));
      }

      if (isControls) {
        expect(ledger(ctx.currentQueryContext.state).wantProof).toBe(false);
        for (const field of ['Base', 'Scalar']) {
          const discarded = readZkir(build, `discardInv${field}`);
          expect(discarded.inputs.map((input) => input.type)).toEqual([`${field}<Secp256k1>`]);
          expect(singleInstruction(discarded, 'inv').a).toBe(discarded.inputs[0].name);
        }
      } else {
        checkUpstreamOperations(build, file === 'upstream-base.compact' ? 'Base' : 'Scalar');
      }
    });
  }

  const refusals = [
    ['inv-bool.compact', /no compatible function named inv is in scope at this call[\s\S]*supplied argument types:\s+\(Boolean\)/],
    ['neg-bytes.compact', /no compatible function named neg is in scope at this call[\s\S]*supplied argument types:\s+\(Bytes<32>\)/],
    ['neg-base-as-scalar.compact', /mismatch between actual return type Secp256k1Base and declared return type Secp256k1Scalar/],
    ['inv-scalar-as-base.compact', /mismatch between actual return type Secp256k1Scalar and declared return type Secp256k1Base/],
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
