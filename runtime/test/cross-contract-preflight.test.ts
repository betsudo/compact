// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Derivative tests of the pinned pre-entry resolution order. These are not the
// original cross-contract-resolution tests: no callee is entered in Runtime 0.18.

import { describe, expect, test, vi } from 'vitest';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import * as publicRuntime from '../src/index.js';
import { preflightCrossContractCall, CrossContractPreflightOptions } from '../src/cross-contract-preflight.js';
import { InterfaceDescriptor } from '../src/interface-descriptor.js';
import { Module } from '../src/module.js';
import { ModuleResolutionError } from '../src/module-resolution.js';
import { verifierKeyHashOf } from '../src/verifier-key-hash.js';

const CALLER = `0200${'11'.repeat(30)}`;
const CALLEE = `0200${'22'.repeat(30)}`;
const BLOCK = 'ab'.repeat(32);
const CIRCUIT = 'add';
const KEY = Uint8Array.from([1, 2, 3, 4]);
const OTHER_KEY = Uint8Array.from([5, 6, 7, 8]);
const FIELD = { tag: 'Field' } as const;
const DECLARATION: InterfaceDescriptor = {
  [CIRCUIT]: { argumentTypes: [FIELD], resultType: FIELD },
};

const conformantModule = (): Module => ({
  Contract: class {
    readonly provableCircuits = {};
    constructor(_witnesses: unknown) {}
  },
  circuitSignatures: {
    [CIRCUIT]: { argumentTypes: [FIELD], resultType: FIELD, pure: false, provable: true },
  },
  expectedVk: { [CIRCUIT]: verifierKeyHashOf(KEY) },
  declaredInterfaces: {},
});

const stateWithKeys = (keys: Readonly<Record<string, Uint8Array>> = { [CIRCUIT]: KEY }): ocrt.ContractState => {
  const state = new ocrt.ContractState();
  vi.spyOn(state, 'operation').mockImplementation((name) =>
    Object.prototype.hasOwnProperty.call(keys, String(name))
      ? ({ verifierKey: keys[String(name)] } as unknown as ocrt.ContractOperation)
      : undefined,
  );
  return state;
};

const inputs = (state = stateWithKeys(), module = conformantModule()): CrossContractPreflightOptions => ({
  callerAddress: CALLER,
  calleeAddress: CALLEE,
  calleeCircuitId: CIRCUIT,
  interfaceName: 'Inner',
  declaration: DECLARATION,
  parentBlockHash: BLOCK,
  stateProvider: { getContractState: async () => state },
  moduleProvider: { resolve: () => async () => module },
});

const failure = async (options: CrossContractPreflightOptions): Promise<ModuleResolutionError> => {
  try {
    await preflightCrossContractCall(options);
  } catch (error) {
    if (ModuleResolutionError.is(error)) return error;
    throw error;
  }
  throw new Error('expected a ModuleResolutionError');
};

describe('internal cross-contract preflight', () => {
  test('resolves a conformant module against the state at the explicit parent block', async () => {
    const state = stateWithKeys();
    const module = conformantModule();
    const stateProvider = { getContractState: vi.fn(async () => state) };
    const moduleProvider = { resolve: vi.fn(() => async () => module) };
    const options = { ...inputs(state, module), stateProvider, moduleProvider };
    const before = state.serialize();
    const resolved = await preflightCrossContractCall(options);
    expect(resolved).toEqual({ deployedState: state, module });
    expect(stateProvider.getContractState).toHaveBeenCalledExactlyOnceWith(BLOCK, CALLEE);
    expect(moduleProvider.resolve).toHaveBeenCalledExactlyOnceWith(CALLEE);
    expect(state.serialize()).toEqual(before);
    expect(options.parentBlockHash).toEqual(BLOCK);
  });

  test('refuses an undeclared circuit and invalid or default address before providers', async () => {
    const stateProvider = { getContractState: vi.fn(async () => stateWithKeys()) };
    const moduleProvider = { resolve: vi.fn(() => async () => conformantModule()) };
    const options = { ...inputs(), stateProvider, moduleProvider };
    await expect(preflightCrossContractCall({ ...options, declaration: {} })).rejects.toThrow(/declaration of circuit/);
    await expect(preflightCrossContractCall({ ...options, calleeAddress: 'invalid' })).rejects.toThrow(/not a contract address/);
    await expect(preflightCrossContractCall({ ...options, calleeAddress: ocrt.dummyContractAddress() })).rejects.toThrow(
      /default contract address/,
    );
    expect(stateProvider.getContractState).not.toHaveBeenCalled();
    expect(moduleProvider.resolve).not.toHaveBeenCalled();
  });

  test('classifies a missing module provider before the state read', async () => {
    const stateProvider = { getContractState: vi.fn(async () => stateWithKeys()) };
    const result = await failure({ ...inputs(), stateProvider, moduleProvider: undefined });
    expect(result.failure.kind).toEqual('ModuleProviderAbsent');
    expect(stateProvider.getContractState).not.toHaveBeenCalled();
  });

  test('pins the state read before module resolution and refuses missing state', async () => {
    const order: string[] = [];
    const options = inputs();
    const stateProvider = {
      getContractState: vi.fn(async (block: string) => {
        order.push(`state:${block}`);
        return undefined;
      }),
    };
    const moduleProvider = {
      resolve: vi.fn(() => {
        order.push('module');
        return async () => conformantModule();
      }),
    };
    await expect(preflightCrossContractCall({ ...options, stateProvider, moduleProvider })).rejects.toThrow(
      /contract state for callee/,
    );
    expect(order).toEqual([`state:${BLOCK}`]);
    expect(moduleProvider.resolve).not.toHaveBeenCalled();
    await expect(preflightCrossContractCall({ ...options, parentBlockHash: '' })).rejects.toThrow(/parent block hash/);
  });

  test('classifies no address binding, provider throw and non-thunk return', async () => {
    expect((await failure({ ...inputs(), moduleProvider: { resolve: () => undefined } })).failure.kind).toEqual(
      'UnsupportedImplementation',
    );
    const cause = new Error('table unavailable');
    const thrown = await failure({
      ...inputs(),
      moduleProvider: {
        resolve: () => {
          throw cause;
        },
      },
    });
    expect(thrown.failure).toEqual({ kind: 'ProviderThrew', cause });
    const nonThunk = { Contract: class {} };
    const invalid = await failure({
      ...inputs(),
      moduleProvider: { resolve: () => nonThunk as unknown as () => Promise<Module> },
    });
    expect(invalid.failure).toEqual({ kind: 'ProviderThrew', cause: nonThunk });
  });

  test('preserves thunk rejection causes and classifies nullish modules', async () => {
    const cause = Object.assign(new Error('outer'), { cause: new Error('inner') });
    const rejected = await failure({ ...inputs(), moduleProvider: { resolve: () => () => Promise.reject(cause) } });
    expect(rejected.failure).toEqual({ kind: 'ModuleLoadRejected', cause });
    const nullish = await failure({
      ...inputs(),
      moduleProvider: { resolve: () => async () => undefined as unknown as Module },
    });
    expect(nullish.failure).toEqual({ kind: 'ProviderThrew', cause: undefined });
  });

  test('names incomplete exports on stale, hollow and primitive modules', async () => {
    for (const [module, missing] of [
      [{ Contract: class {} }, ['circuitSignatures', 'expectedVk']],
      [{ Contract: class {}, circuitSignatures: {}, expectedVk: undefined }, ['expectedVk']],
      [42, ['Contract', 'circuitSignatures', 'expectedVk']],
    ] as const) {
      const result = await failure({
        ...inputs(),
        moduleProvider: { resolve: () => async () => module as unknown as Module },
      });
      expect(result.failure).toEqual({ kind: 'IncompleteModule', missing });
    }
  });

  test('checks interface conformance before any operation or verifier key', async () => {
    const state = stateWithKeys({});
    const operation = vi.mocked(state.operation);
    const module = { ...conformantModule(), circuitSignatures: {} };
    const result = await failure(inputs(state, module));
    expect(result.failure).toMatchObject({
      kind: 'NonconformantImplementation',
      circuitId: CIRCUIT,
      check: 'Existence',
    });
    expect(operation).not.toHaveBeenCalled();
  });

  test('classifies an unreadable signature separately from a mismatch', async () => {
    const module = {
      ...conformantModule(),
      circuitSignatures: {
        [CIRCUIT]: {
          argumentTypes: [{ tag: 'UnknownTag' }],
          resultType: FIELD,
          pure: false,
          provable: true,
        },
      },
    } as unknown as Module;
    const result = await failure(inputs(stateWithKeys(), module));
    expect(result.failure).toMatchObject({ kind: 'UnreadableModule', unreadableTag: 'UnknownTag' });
  });

  test('classifies absent operation and malformed or missing module fingerprint', async () => {
    const absent = await failure(inputs(stateWithKeys({})));
    expect(absent.failure.kind).toEqual('OperationAbsent');
    const malformed = await failure(
      inputs(stateWithKeys(), {
        ...conformantModule(),
        expectedVk: { [CIRCUIT]: 'INVALID' },
      }),
    );
    expect(malformed.failure).toMatchObject({
      kind: 'MalformedVerifierKeyHash',
      circuitId: CIRCUIT,
      recorded: 'INVALID',
    });
    const missing = await failure(inputs(stateWithKeys(), { ...conformantModule(), expectedVk: {} }));
    expect(missing.failure).toMatchObject({
      kind: 'ImplementationMismatch',
      circuitId: CIRCUIT,
      actual: verifierKeyHashOf(KEY),
    });
  });

  test('classifies called and overlapping verifier key mismatches', async () => {
    const wrongCalled = await failure(
      inputs(stateWithKeys(), {
        ...conformantModule(),
        expectedVk: { [CIRCUIT]: verifierKeyHashOf(OTHER_KEY) },
      }),
    );
    expect(wrongCalled.failure).toEqual({
      kind: 'ImplementationMismatch',
      circuitId: CIRCUIT,
      expected: verifierKeyHashOf(OTHER_KEY),
      actual: verifierKeyHashOf(KEY),
    });
    const wrongOther = await failure(
      inputs(stateWithKeys({ [CIRCUIT]: KEY, other: OTHER_KEY }), {
        ...conformantModule(),
        expectedVk: { [CIRCUIT]: verifierKeyHashOf(KEY), other: verifierKeyHashOf(KEY) },
      }),
    );
    expect(wrongOther.failure).toEqual({
      kind: 'ImplementationMismatch',
      circuitId: 'other',
      expected: verifierKeyHashOf(KEY),
      actual: verifierKeyHashOf(OTHER_KEY),
    });
  });

  test('retains the call identity and never enters or mutates the callee on failure', async () => {
    const state = stateWithKeys();
    const before = state.serialize();
    const setOperation = vi.spyOn(state, 'setOperation');
    const constructor = vi.fn();
    const module = { ...conformantModule(), Contract: constructor as unknown as Module['Contract'], expectedVk: {} };
    const result = await failure(inputs(state, module));
    expect(result.context).toEqual({
      callerAddress: CALLER,
      calleeAddress: CALLEE,
      calleeCircuitId: CIRCUIT,
      interfaceName: 'Inner',
    });
    expect(state.serialize()).toEqual(before);
    expect(setOperation).not.toHaveBeenCalled();
    expect(constructor).not.toHaveBeenCalled();
  });

  test('leaves the Runtime 0.18 public call boundary refused', () => {
    expect('crossContractCall' in publicRuntime).toEqual(false);
    const options = {
      circuitId: 'caller',
      contractAddress: CALLER,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: new ocrt.ContractState(),
      privateState: 0,
    };
    expect(() => publicRuntime.createCircuitContext({ ...options, parentBlockHash: BLOCK })).toThrow(
      /cross-contract context requires Ledger 9 runtime/,
    );
    expect(() => publicRuntime.createCircuitContext({ ...options, crossContract: {} })).toThrow(
      /cross-contract context requires Ledger 9 runtime/,
    );
  });
});
