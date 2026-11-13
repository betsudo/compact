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

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import { createTestContract } from '@test/compact-test';
import { expect, test } from 'vitest';

import baseCompile from './base/roundtrip/compile.pass.test.ts';
import baseRuntime from './base/roundtrip/runtime.pass.test.ts';
import scalarCompile from './scalar/roundtrip/compile.pass.test.ts';
import scalarRuntime from './scalar/roundtrip/runtime.pass.test.ts';
import defaultCompile from './default/maybe_either/compile.pass.test.ts';
import defaultRuntime from './default/maybe_either/runtime.pass.test.ts';

const { MAX_CURVE25519_BASE, MAX_CURVE25519_SCALAR } = runtime;
const root = fileURLToPath(new URL('.', import.meta.url));
const runtimeRoot = fileURLToPath(new URL('../../..', import.meta.url));
const compiler = process.env.COMPACT_CURVE25519_TEST_COMPILER;
const output = process.env.COMPACT_CURVE25519_TEST_OUTPUT;

function compile(fixture, options, targets, feature = '--feature-zkir-v3', source) {
  if (source === undefined) {
    const directory = join(root, fixture);
    const sources = readdirSync(directory).filter((name) => name.endsWith('.compact'));
    expect(sources).toHaveLength(1);
    source = join(directory, sources[0]);
  }
  const caseDirectory = join(output, fixture, feature ? targets.join('-') : 'legacy');
  const build = join(caseDirectory, '.build');
  mkdirSync(caseDirectory, { recursive: true });
  const result = spawnSync(compiler, [
    ...targets.flatMap((target) => ['--target', target]),
    ...(targets.includes('rust') ? ['--rust-runtime-root', runtimeRoot] : []),
    ...(feature ? [feature] : []), '--skip-zk', ...(options.compilerArgs ?? []),
    source, build,
  ], { encoding: 'utf8' });
  if (result.error) throw result.error;
  expect(result.signal).toBeNull();
  writeFileSync(join(caseDirectory, 'stdout.log'), result.stdout);
  writeFileSync(join(caseDirectory, 'stderr.log'), result.stderr);
  writeFileSync(join(caseDirectory, 'exit.txt'), `${result.status}\n`);
  return { build, result };
}

const refusals = import.meta.glob('./{base,scalar}/casts/*/compile.fail.test.ts', { eager: true, import: 'default' });

test('retains all six original compile refusal declarations', () => {
  expect(Object.keys(refusals)).toHaveLength(6);
});

for (const [path, definition] of Object.entries(refusals)) {
  const fixture = dirname(path);
  for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
    test(`${fixture} refuses compilation for ${targets.join('+')}`, () => {
      expect(definition.result).toBe('fail');
      const { build, result } = compile(fixture, definition.options, targets);
      expect(result.status).not.toBe(0);
      expect(result.stderr).toMatch(definition.options.expectedError);
      expect(existsSync(build)).toBe(false);
    });
  }
}

for (const [name, definition, runtimeTest, maximum] of [
  ['base', baseCompile, baseRuntime, MAX_CURVE25519_BASE],
  ['scalar', scalarCompile, scalarRuntime, MAX_CURVE25519_SCALAR],
]) {
  test(`runs the intact ${name} roundtrip for generated TypeScript`, async () => {
    expect(definition.result).toBe('pass');
    expect(runtimeTest.result).toBe('pass');
    const { build, result } = compile(`${name}/roundtrip`, definition.options, ['ts', 'rust']);
    expect(result.status, result.stderr).toBe(0);
    const { Contract, pureCircuits } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
    await runtimeTest.run(Contract, pureCircuits);
    const { contract, ctx } = await createTestContract(Contract);
    for (const invalid of [-1n, maximum + 1n]) {
      expect(() => contract.circuits[`curve25519_${name}_roundtrip`](ctx, invalid)).toThrow(/Curve25519/);
    }
  });

  test(`refuses the ${name} roundtrip in the legacy proof profile`, () => {
    const { build, result } = compile(`${name}/roundtrip`, definition.options, ['ts'], '');
    expect(result.status).not.toBe(0);
    expect(result.stderr).toMatch(/Curve25519(?:Base|Scalar) requires ZKIR v3/);
    expect(existsSync(build)).toBe(false);
  });
}

test('compiles both targets and checks field arithmetic, casts and persistent hashing in TypeScript', async () => {
  const { build, result } = compile('operations', {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const { Contract, ledger } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  const { contract, ctx } = await createTestContract(Contract);
  expect(contract.circuits.arithmetic(ctx, 3n, 5n).result).toEqual([5n, 7n]);
  const wrapped = contract.circuits.arithmetic(ctx, MAX_CURVE25519_BASE, MAX_CURVE25519_SCALAR);
  expect(wrapped.result).toEqual([1n, 1n]);
  const wrappedLedger = ledger(wrapped.context.currentQueryContext.state);
  expect([wrappedLedger.base, wrappedLedger.scalar]).toEqual([1n, 1n]);
  expect(() => contract.circuits.arithmetic(ctx, 0n, 1n)).toThrow(/Curve25519 base field has no inverse/);
  expect(() => contract.circuits.arithmetic(ctx, 1n, 0n)).toThrow(/Curve25519 scalar field has no inverse/);
  const littleEndian = value => Uint8Array.from(
    {length: 32}, (_, index) => Number(value >> BigInt(8 * index) & 255n),
  );
  const wide = (1n << 256n) - 1n;
  const scalar = wide % (MAX_CURVE25519_SCALAR + 1n);
  const cast = contract.circuits.casts(ctx, new Uint8Array(32).fill(255));
  expect(cast.result).toEqual([littleEndian(37n), littleEndian(scalar)]);
  const castLedger = ledger(cast.context.currentQueryContext.state);
  expect([castLedger.base, castLedger.scalar]).toEqual([37n, scalar]);
  for (const [value, offset] of [[1n, -1], [(1n << 204n) + 1n, 26]]) {
    const binary = Buffer.alloc(33);
    if (offset >= 0) binary[offset] = 1;
    const digest = new Uint8Array(createHash('sha256').update(binary).digest());
    const hash = contract.circuits.hashScalar(ctx, value);
    expect(hash.result).toEqual(digest);
    expect(ledger(hash.context.currentQueryContext.state).digest).toEqual(digest);
  }
});

test('runs the original wide scalar cast with the mainstream ZKIR limb order', async () => {
  const { build, result } = compile('wide-cast', {}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const ir = JSON.parse(readFileSync(join(build, 'zkir/test.zkir'), 'utf8'));
  expect(ir.inputs.map(({type}) => type)).toEqual(Array(3).fill('Scalar<BLS12-381>'));
  expect(ir.instructions.slice(0, 3).map(({op, val, bits}) => [op, val, bits])).toEqual(
    [16, 248, 248].map((bits, index) => ['constrain_bits', ir.inputs[index].name, bits]),
  );
  const assemble = ir.instructions[3];
  expect(assemble).toMatchObject({
    op: 'bytes_from_natives',
    len: 64,
    inputs: ir.inputs.map(({name}) => name).reverse(),
  });
  expect(ir.instructions[4]).toMatchObject({
    op: 'from_bytes', type: 'Scalar<Curve25519>', bytes: assemble.output,
  });
  // The existing emitter opens the ledger slot before encoding the value.
  // Mainstream opens it immediately after encoding. Check both operations
  // and the complete write and return sequence.
  const scalar = ir.instructions[4].output;
  const [open, encode, write, close, resultInstruction] = ir.instructions.slice(5);
  expect(ir.instructions).toHaveLength(10);
  expect(ir.outputs).toEqual(['Scalar<Curve25519>']);
  expect(ir.do_communications_commitment).toBe(true);
  expect(open).toEqual({op: 'impact', guard: '0x01', inputs: ['0x10', '0x01', '0x01', '0x01', '0x00']});
  expect(encode).toMatchObject({op: 'encode', input: scalar});
  expect(encode.outputs).toHaveLength(2);
  expect(write).toEqual({
    op: 'impact', guard: '0x01',
    inputs: ['0x11', '0x01', '0x02', '0x1a', '0x07', ...encode.outputs],
  });
  expect(close).toEqual({op: 'impact', guard: '0x01', inputs: ['0x91']});
  expect(resultInstruction).toEqual({op: 'output', vals: [scalar]});
  const original = readFileSync(join(root, 'wide-cast/upstream.ss.txt'), 'utf8');
  const callback = [...original.slice(original.indexOf('(stage-javascript'))
    .matchAll(/^\s*(?:'\()?("(?:[^"\\]|\\.)*")$/gm)]
    .map(match => JSON.parse(match[1])).join('\n');
  expect(callback).toContain('10^42');
  expect(callback.match(/\bexpect\(/g)).toHaveLength(10);
  expect(callback.endsWith('});')).toBe(true);
  const code = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  const startContract = async (_, witnesses) => {
    const { contract, ctx } = await createTestContract(code.Contract, witnesses);
    const call = contract.circuits.test;
    const adapted = (...args) => {
      const output = call(...args);
      return {...output, context: {callContext: output.context}};
    };
    return [{circuits: {test: adapted}}, ctx];
  };
  const run = new Function('test', 'expect', 'contractCode', 'startContract', 'runtime', `return ${callback}`);
  await run((_, check) => check(), expect, code, startContract, runtime);
});

for (const [fixture, message] of [
  ['wrong-width', 'cannot cast from type Bytes<63> to type Curve25519Scalar'],
  ['wrong-field', 'cannot cast from type Bytes<64> to type Curve25519Base'],
]) {
  test(`refuses wide scalar casts with the ${fixture}`, () => {
    const { build, result } = compile(`wide-cast/${fixture}`, {}, ['ts', 'rust']);
    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(message);
    expect(existsSync(build)).toBe(false);
  });
}

async function runPointFixture(name, assertions) {
  const { build, result } = compile(`point/${name}`, {
    compilerArgs: ['--rust-require-recording'],
  }, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const original = readFileSync(join(root, 'point', name, 'upstream.ss.txt'), 'utf8');
  const callback = [...original.slice(original.indexOf('(stage-javascript'))
    .matchAll(/^\s*(?:'\()?("(?:[^"\\]|\\.)*")$/gm)]
    .map(match => JSON.parse(match[1])).join('\n');
  expect(callback.match(/\bexpect\(/g)).toHaveLength(assertions);
  const code = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  const startContract = async (_, witnesses) => {
    const { contract, ctx } = await createTestContract(code.Contract, witnesses);
    const adapt = call => (context, ...args) => {
      const output = call(context.callContext ?? context, ...args);
      return {...output, context: {callContext: output.context}};
    };
    const circuits = Object.fromEntries(Object.entries(contract.circuits)
      .map(([entry, call]) => [entry, adapt(call)]));
    return [{circuits}, ctx];
  };
  const run = new Function('test', 'expect', 'contractCode', 'startContract', 'runtime', `return ${callback}`);
  await run((_, check) => check(), expect, code, startContract, runtime);
  return {build, code};
}

test('runs the original Curve25519 point ledger roundtrip and coordinate projection', async () => {
  const {build, code} = await runPointFixture('roundtrip', 6);
  const capabilities = JSON.parse(readFileSync(join(build, 'contract/rust-capabilities.json'), 'utf8'));
  expect(capabilities.circuits.map(({name, recorded, observed_call}) => [name, recorded, observed_call]))
    .toEqual([['test0', true, true], ['test1', true, true]]);
  const { contract, ctx } = await createTestContract(code.Contract);
  for (const point of [{x: 0n, y: 0n}, {x: 0n, y: MAX_CURVE25519_BASE}, {x: -1n, y: 1n}]) {
    expect(() => contract.circuits.test0(ctx, point)).toThrow(/Curve25519Point/);
  }
});

test('runs the original Curve25519 point equality and identity assertions', async () => {
  await runPointFixture('equality', 8);
});

test('generates both targets for Curve25519 addition and scalar multiplication', async () => {
  const {build, result} = compile('point/group', {compilerArgs: ['--rust-require-recording']}, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const ir = JSON.parse(readFileSync(join(build, 'zkir/group.zkir'), 'utf8'));
  expect(ir.instructions.map(({op}) => op)).toEqual(expect.arrayContaining(['ec_mul_generator', 'ec_mul', 'add']));
  const {Contract, ledger} = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  const {contract, ctx} = await createTestContract(Contract);
  const generator = runtime.curve25519MulGenerator(1n);
  const doubled = runtime.curve25519MulGenerator(2n);
  const identity = {x: 0n, y: 1n};
  for (const [point, scalar, expected] of [
    [generator, 1n, doubled],
    [generator, 0n, identity],
    [identity, 1n, generator],
  ]) {
    const call = contract.circuits.group(ctx, point, scalar);
    expect(call.result).toEqual(expected);
    expect(ledger(call.context.currentQueryContext.state).point).toEqual(expected);
  }
});

test('runs the original Curve25519 Maybe and Either defaults', async () => {
  expect(defaultCompile.result).toBe('pass');
  expect(defaultRuntime.result).toBe('pass');
  const { build, result } = compile('default/maybe_either', defaultCompile.options, ['ts', 'rust']);
  expect(result.status, result.stderr).toBe(0);
  const { Contract, pureCircuits } = await import(/* @vite-ignore */ pathToFileURL(join(build, 'contract/index.js')).href);
  await defaultRuntime.run(Contract, pureCircuits);
});

for (const name of ['serialize', 'deserialize']) {
  for (const targets of [['ts'], ['rust'], ['ts', 'rust']]) {
    test(`refuses the original Curve25519 point ${name} for ${targets.join('+')}`, () => {
      const { build, result } = compile(`point/${name}`, {}, targets);
      expect(result.status).not.toBe(0);
      expect(result.stderr).toMatch(new RegExp(
        `unbound identifier ${name}|(?:de)?serialization is not yet supported for curve type Curve25519Point`,
      ));
      expect(existsSync(build)).toBe(false);
    });
  }
}
