// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

// Derivative internal call-state tests. The pinned original 14 invocation
// cases require the future public Ledger 9 cross-contract Runtime API.
import { describe, expect, test, vi } from 'vitest';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import * as publicRuntime from '../src/index.js';
import { InternalCrossContractCallState } from '../src/cross-contract-call-state.js';
import { InterfaceDescriptor } from '../src/interface-descriptor.js';
import { Module } from '../src/module.js';
import { verifierKeyHashOf } from '../src/verifier-key-hash.js';

const A = `0200${'11'.repeat(30)}`;
const B = `0200${'22'.repeat(30)}`;
const C = `0200${'33'.repeat(30)}`;
const BLOCK = 'ab'.repeat(32);
const FIELD = { tag: 'Field' } as const;
const KEY = Uint8Array.of(1, 2, 3, 4);
const declaration: InterfaceDescriptor = { add: { argumentTypes: [FIELD], resultType: FIELD } };
const request = (calleeAddress: string) => ({ calleeAddress, calleeCircuitId: 'add', interfaceName: 'Inner', declaration });

const state = (): ocrt.ContractState => {
  const result = new ocrt.ContractState();
  vi.spyOn(result, 'operation').mockImplementation((name) =>
    name === 'add' ? ({ verifierKey: KEY } as unknown as ocrt.ContractOperation) : undefined,
  );
  return result;
};
const module = (): Module => ({
  Contract: class {
    readonly provableCircuits = {};
    constructor(_witnesses: unknown) {}
  },
  circuitSignatures: { add: { argumentTypes: [FIELD], resultType: FIELD, pure: false, provable: true } },
  expectedVk: { add: verifierKeyHashOf(KEY) },
  declaredInterfaces: {},
});
const setup = () => {
  const root = state();
  const callee = state();
  const getContractState = vi.fn(async (_block: string, _address: string) => callee);
  const resolve = vi.fn(() => async () => module());
  const calls = new InternalCrossContractCallState({
    entryAddress: A,
    entryState: root,
    parentBlockHash: BLOCK,
    time: 37,
    stateProvider: { getContractState },
    moduleProvider: { resolve },
  });
  return { calls, root, callee, getContractState, resolve };
};

describe('internal Ledger 9 call state', () => {
  test('pins a frozen parent-block identity and keeps the Runtime 0.18 public boundary closed', () => {
    const { calls, root } = setup();
    expect(calls.snapshot).toEqual({ parentBlockHash: BLOCK, time: 37 });
    expect(Object.isFrozen(calls.snapshot)).toBe(true);
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(A)).toBe(true);
    expect(calls.queryContextFor(A)?.block).toMatchObject({
      ownAddress: A,
      parentBlockHash: BLOCK,
      secondsSinceEpoch: 37n,
    });
    expect(calls.deployedStateFor(A)).toBe(root);
    expect('crossContractCall' in publicRuntime).toBe(false);
    expect('InternalCrossContractCallState' in publicRuntime).toBe(false);
    const options = {
      circuitId: 'caller',
      contractAddress: A,
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

  test('enters a separately mapped callee only after exact parent-block preflight', async () => {
    const { calls, callee, getContractState, resolve } = setup();
    const rootQuery = calls.queryContextFor(A);
    const before = callee.serialize();
    const entered = await calls.begin(request(B));
    expect(getContractState).toHaveBeenCalledExactlyOnceWith(BLOCK, B);
    expect(resolve).toHaveBeenCalledExactlyOnceWith(B);
    expect(entered.deployedState).toBe(callee);
    expect(entered.queryContext).toBe(calls.queryContextFor(B));
    expect(entered.queryContext).not.toBe(rootQuery);
    expect(entered.queryContext.block).toMatchObject({
      ownAddress: B,
      parentBlockHash: BLOCK,
      secondsSinceEpoch: 37n,
      caller: { tag: 'contract', address: A },
    });
    expect(calls.gasCostFor(B)).toBeDefined();
    expect(calls.deployedStateFor(B)).toBe(callee);
    expect(calls.currentAddress).toBe(B);
    expect(rootQuery).toBe(calls.queryContextFor(A));
    expect(callee.serialize()).toEqual(before);
    entered.end();
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
  });

  test('failed resolution cannot enter, cache, or mutate the callee', async () => {
    const { calls, callee, getContractState } = setup();
    const before = callee.serialize();
    vi.mocked(callee.operation).mockReturnValue(undefined);
    await expect(calls.begin(request(B))).rejects.toThrow(/operation|verifier/i);
    expect(getContractState).toHaveBeenCalledExactlyOnceWith(BLOCK, B);
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
    expect(calls.queryContextFor(B)).toBeUndefined();
    expect(calls.gasCostFor(B)).toBeUndefined();
    expect(calls.deployedStateFor(B)).toBeUndefined();
    expect(callee.serialize()).toEqual(before);
  });

  test('refuses a caller context whose parent block was changed before lookup', async () => {
    const { calls, getContractState, resolve } = setup();
    const caller = calls.queryContextFor(A)!;
    caller.block = { ...caller.block, parentBlockHash: 'cd'.repeat(32) };
    await expect(calls.begin(request(B))).rejects.toThrow(/pinned parent block/);
    expect(getContractState).not.toHaveBeenCalled();
    expect(resolve).not.toHaveBeenCalled();
    expect(calls.queryContextFor(B)).toBeUndefined();
  });

  test('rejects direct and nested reentry before provider access', async () => {
    const { calls, getContractState, resolve } = setup();
    await expect(calls.begin(request(A))).rejects.toThrow(/re-entrancy/);
    expect(getContractState).not.toHaveBeenCalled();
    expect(resolve).not.toHaveBeenCalled();
    const b = await calls.begin(request(B));
    await expect(calls.begin(request(A))).rejects.toThrow(/re-entrancy/);
    expect(getContractState).toHaveBeenCalledTimes(1);
    b.end();
  });

  test('restores nested callers in LIFO order and refuses stale handles', async () => {
    const { calls } = setup();
    const b = await calls.begin(request(B));
    const c = await calls.begin(request(C));
    expect(c.callerAddress).toBe(B);
    expect(calls.currentAddress).toBe(C);
    expect(() => b.end()).toThrow(/top of its active stack/);
    c.end();
    expect(calls.currentAddress).toBe(B);
    b.end();
    expect(calls.currentAddress).toBe(A);
    expect(() => c.end()).toThrow(/top of its active stack/);
  });

  test('reuses a deployed state on a sequential call and resets prior effects', async () => {
    const { calls, getContractState } = setup();
    const first = await calls.begin(request(B));
    const query = first.queryContext;
    query.effects.claimedNullifiers.push('prior-nullifier');
    first.end();
    const second = await calls.begin(request(B));
    expect(getContractState).toHaveBeenCalledTimes(1);
    expect(second.queryContext).toBe(query);
    expect(second.queryContext.effects.claimedNullifiers).toEqual([]);
    expect(second.deployedState).toBe(first.deployedState);
    second.end();
  });

  test('advance refuses a changed block, time, or address without changing mapped state', async () => {
    const { calls } = setup();
    const b = await calls.begin(request(B));
    const original = b.queryContext;
    for (const changed of [{ parentBlockHash: 'cd'.repeat(32) }, { secondsSinceEpoch: 38n }, { ownAddress: C }]) {
      const next = new ocrt.QueryContext(b.deployedState.data, B);
      next.block = { ...original.block, ...changed };
      expect(() => calls.advance(b, next, b.gasCost)).toThrow(/pinned parent block or call identity/);
      expect(calls.queryContextFor(B)).toBe(original);
    }
    const valid = new ocrt.QueryContext(b.deployedState.data, B);
    valid.block = { ...original.block };
    calls.advance(b, valid, b.gasCost);
    expect(calls.queryContextFor(B)).toBe(valid);
    b.end();
    expect(() => calls.advance(b, original, b.gasCost)).toThrow(/top of its active stack/);
  });

  test('a preflight stale after stack movement cannot commit a new callee', async () => {
    let release!: (value: ocrt.ContractState) => void;
    const pending = new Promise<ocrt.ContractState>((resolve) => {
      release = resolve;
    });
    const stateProvider = {
      getContractState: vi.fn(async (_hash: string, address: string) => (address === C ? pending : state())),
    };
    const race = new InternalCrossContractCallState({
      entryAddress: A,
      entryState: state(),
      parentBlockHash: BLOCK,
      time: 37,
      stateProvider,
      moduleProvider: { resolve: () => async () => module() },
    });
    const c = race.begin(request(C));
    // Movement in the same root tree invalidates the pending call.
    const racedB = await race.begin(request(B));
    racedB.end();
    release(state());
    await expect(c).rejects.toThrow(/stack changed during preflight/);
    expect(race.queryContextFor(C)).toBeUndefined();
  });
});
