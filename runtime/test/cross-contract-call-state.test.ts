// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

// Derivative internal call-state tests. The pinned original 14 invocation
// cases require the future public Ledger 9 cross-contract Runtime API.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { describe, expect, test, vi } from 'vitest';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import * as publicRuntime from '../src/index.js';
import { InternalCallTransactionRollbackError, InternalCrossContractCallState } from '../src/cross-contract-call-state.js';
import {
  advanceInternalCallTraceContext,
  createInternalCallTraceContext,
  enterInternalCallTraceContext,
  finalizeInternalCallProofData,
} from '../src/internal-call-trace-context.js';
import {
  bindQuerySourceState,
  emptyRunningCost,
  forkCalleeCircuitContext,
  inheritQuerySourceState,
} from '../src/circuit-context.js';
import { InterfaceDescriptor } from '../src/interface-descriptor.js';
import { Module } from '../src/module.js';
import { verifierKeyHashOf } from '../src/verifier-key-hash.js';
import { emptyZswapLocalState } from '../src/zswap.js';

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
  test('pins a frozen parent-block identity and keeps the public call export closed', () => {
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
    const context = publicRuntime.createCircuitContext({ ...options, parentBlockHash: BLOCK });
    expect(context.callContext.currentQueryContext.block.parentBlockHash).toBe(BLOCK);
    expect(() => publicRuntime.createCircuitContext({ ...options, crossContract: {} as never })).toThrow(
      /cross-contract providers require a pinned parent block/,
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

  test('adopts the actual generated root after its pre-call query without rewinding the transcript', async () => {
    const { calls, root, getContractState } = setup();
    const context = publicRuntime.createCircuitContext({
      circuitId: 'caller',
      contractAddress: A,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: root,
      privateState: {},
      parentBlockHash: BLOCK,
      time: 37,
    });
    const original = context.callContext.currentQueryContext;
    const queried = new ocrt.QueryContext(root.data, A);
    queried.block = { ...original.block };
    vi.spyOn(original, 'query').mockReturnValue({
      context: queried,
      gasCost: { ...emptyRunningCost(), computeTime: 4n },
      events: [],
    } as ReturnType<typeof original.query>);
    const proofData = {
      input: { value: [], alignment: [] },
      publicTranscript: [] as ocrt.Op<ocrt.AlignedValue>[],
      privateTranscriptOutputs: [],
    };
    publicRuntime.queryLedgerState(context, proofData, [{ dup: { n: 0 } }]);
    expect(proofData.publicTranscript).toEqual([{ dup: { n: 0 } }]);
    expect(calls.queryContextFor(A)).not.toBe(queried);

    calls.bindRootCircuitContext(context);
    expect(calls.queryContextFor(A)).toBe(queried);
    expect(calls.gasCostFor(A)?.computeTime).toBe(4n);
    const entered = await calls.begin(request(B));
    expect(getContractState).toHaveBeenCalledExactlyOnceWith(BLOCK, B);
    expect(calls.queryContextFor(A)).toBe(queried);
    expect(proofData.publicTranscript).toEqual([{ dup: { n: 0 } }]);
    entered.end();
  });

  test('refuses a stale or active generated root before replacing its cached query', async () => {
    const { calls, root } = setup();
    const context = publicRuntime.createCircuitContext({
      circuitId: 'caller',
      contractAddress: A,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: root,
      privateState: {},
      parentBlockHash: BLOCK,
      time: 37,
    });
    const original = calls.queryContextFor(A);
    context.callContext = { ...context.callContext, parentBlockHash: 'cd'.repeat(32) };
    expect(() => calls.bindRootCircuitContext(context)).toThrow(/pinned provenance/);
    expect(calls.queryContextFor(A)).toBe(original);
    context.callContext = { ...context.callContext, parentBlockHash: BLOCK };
    const otherState = publicRuntime.createCircuitContext({
      circuitId: 'caller',
      contractAddress: A,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: new ocrt.ContractState(),
      privateState: {},
      parentBlockHash: BLOCK,
      time: 37,
    });
    expect(() => calls.bindRootCircuitContext(otherState)).toThrow(/differs from the call-state parent-block snapshot/);
    expect(calls.queryContextFor(A)).toBe(original);
    const forgedQuery = new ocrt.QueryContext(new ocrt.ContractState().data, A);
    forgedQuery.block = { ...context.callContext.currentQueryContext.block };
    context.callContext = { ...context.callContext, currentQueryContext: forgedQuery };
    context.currentQueryContext = forgedQuery;
    context.queryContexts[A] = forgedQuery;
    expect(() => calls.bindRootCircuitContext(context)).toThrow(/different sources/);
    expect(calls.queryContextFor(A)).toBe(original);
    const entered = await calls.begin(request(B));
    expect(() => calls.bindRootCircuitContext(context)).toThrow(/callee is active/);
    expect(calls.queryContextFor(A)).toBe(original);
    entered.end();
  });

  test('forks a separately pinned generated callee without changing the caller', async () => {
    const { calls, root, getContractState, resolve } = setup();
    const context = publicRuntime.createCircuitContext({
      circuitId: 'caller',
      contractAddress: A,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: root,
      privateState: { before: 1 },
      parentBlockHash: BLOCK,
      time: 37,
      crossContract: {
        stateProvider: { getContractState },
        moduleProvider: { resolve },
      },
    });
    calls.bindRootCircuitContext(context);
    const entered = await calls.begin(request(B));
    const child = forkCalleeCircuitContext(context, 'add', B, entered.queryContext, entered.gasCost, entered.deployedState);
    expect(child.callContext.contractAddress).toBe(B);
    expect(child.callContext.circuitId).toBe('add');
    expect(child.callContext.currentPrivateState).toBeUndefined();
    expect(child.queryContexts[B]).toBe(entered.queryContext);
    expect(child.gasCosts[B]).toBe(child.callContext.currentGasCost);
    expect(child.zswapLocalStates[B]).toBe(child.callContext.currentZswapLocalState);
    expect(child.contractStates[B]).toBe(entered.deployedState);
    expect(child.activeContracts).toEqual(new Set([A, B]));
    expect(context.activeContracts).toEqual(new Set([A]));
    expect(child.callContext.currentQueryContext.block.caller).toEqual({ tag: 'contract', address: A });
    const generatedCopy = publicRuntime.copyCircuitContext(child);
    expect(generatedCopy.callContext.contractAddress).toBe(B);
    expect(() => publicRuntime.assertLiveCircuitCall(generatedCopy)).not.toThrow();
    const next = new ocrt.QueryContext(entered.deployedState.data, B);
    next.block = { ...entered.queryContext.block };
    vi.spyOn(entered.queryContext, 'query').mockReturnValue({
      context: next,
      gasCost: { ...emptyRunningCost(), readTime: 2n },
      events: [],
    } as ReturnType<typeof entered.queryContext.query>);
    const proofData = {
      input: { value: [], alignment: [] },
      output: { value: [], alignment: [] },
      publicTranscript: [] as ocrt.Op<ocrt.AlignedValue>[],
      privateTranscriptOutputs: [],
    };
    publicRuntime.queryLedgerState(generatedCopy, proofData, [{ dup: { n: 0 } }]);
    publicRuntime.finalizeCallProofData(generatedCopy, proofData);
    expect(generatedCopy.queryContexts[B]).toBe(next);
    expect(generatedCopy.gasCosts[B].readTime).toBe(2n);
    expect(generatedCopy.callProofDataTrace).toHaveLength(1);
    expect(generatedCopy.callProofDataTrace[0].contractAddress).toBe(B);
    expect(generatedCopy.callProofDataTrace[0].publicTranscript).toEqual([{ dup: { n: 0 } }]);
    expect(child.callProofDataTrace).toEqual([]);
    const decoyCaller = new ocrt.QueryContext(entered.deployedState.data, B);
    decoyCaller.block = { ...next.block, caller: { tag: 'contract', address: C } };
    vi.spyOn(next, 'query').mockReturnValue({
      context: decoyCaller,
      gasCost: emptyRunningCost(),
      events: [],
    } as ReturnType<typeof next.query>);
    expect(() => publicRuntime.queryLedgerState(generatedCopy, proofData, [])).toThrow(/pinned block or call identity/);
    expect(generatedCopy.callContext.currentQueryContext).toBe(next);
    expect(context.callContext.contractAddress).toBe(A);
    expect(context.callContext.currentPrivateState).toEqual({ before: 1 });
    expect('forkCalleeCircuitContext' in publicRuntime).toBe(false);
    entered.end();
  });

  test('refuses a callee query outside the pinned caller identity before publishing a child', async () => {
    const { calls, root, getContractState, resolve } = setup();
    const context = publicRuntime.createCircuitContext({
      circuitId: 'caller',
      contractAddress: A,
      coinPublicKeyOrZswapState: '0'.repeat(64),
      contractState: root,
      privateState: {},
      parentBlockHash: BLOCK,
      time: 37,
      crossContract: { stateProvider: { getContractState }, moduleProvider: { resolve } },
    });
    calls.bindRootCircuitContext(context);
    const entered = await calls.begin(request(B));
    const query = entered.queryContext;
    const before = context.callProofDataTrace.length;
    expect(() => forkCalleeCircuitContext(context, 'add', B, query, entered.gasCost, new ocrt.ContractState())).toThrow(
      /different sources/,
    );
    query.block = { ...query.block, parentBlockHash: 'cd'.repeat(32) };
    expect(() => forkCalleeCircuitContext(context, 'add', B, query, entered.gasCost, entered.deployedState)).toThrow(
      /differs from the caller parent-block snapshot/,
    );
    expect(context.callProofDataTrace).toHaveLength(before);
    expect(context.activeContracts).toEqual(new Set([A]));
    entered.abort();
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
    const otherState = new ocrt.ContractState();
    const forged = new ocrt.QueryContext(otherState.data, B);
    forged.block = { ...original.block };
    bindQuerySourceState(forged, otherState);
    expect(() => calls.advance(b, forged, b.gasCost)).toThrow(/different sources/);
    expect(calls.queryContextFor(B)).toBe(original);
    const valid = new ocrt.QueryContext(b.deployedState.data, B);
    valid.block = { ...original.block };
    // The real queryLedgerState replacement inherits this private source tag.
    inheritQuerySourceState(original, valid);
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

  test('abort removes a new callee cache and restores the root after an advance', async () => {
    const { calls, root, callee } = setup();
    const before = callee.serialize();
    const rootQuery = calls.queryContextFor(A);
    const b = await calls.begin(request(B));
    const next = new ocrt.QueryContext(b.deployedState.data, B);
    next.block = { ...b.queryContext.block };
    inheritQuerySourceState(b.queryContext, next);
    calls.advance(b, next, { ...emptyRunningCost(), computeTime: 7n });
    b.abort();
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
    expect(calls.queryContextFor(A)).toBe(rootQuery);
    expect(calls.queryContextFor(B)).toBeUndefined();
    expect(calls.gasCostFor(B)).toBeUndefined();
    expect(calls.deployedStateFor(B)).toBeUndefined();
    expect(calls.deployedStateFor(A)).toBe(root);
    expect(callee.serialize()).toEqual(before);
    expect(() => b.abort()).toThrow(/top of its active stack/);
  });

  test('abort restores a sequential callee’s prior query object, block and effects', async () => {
    const { calls, getContractState } = setup();
    const first = await calls.begin(request(B));
    first.end();
    const priorQuery = calls.queryContextFor(B)!;
    priorQuery.effects = { ...priorQuery.effects, claimedNullifiers: ['aa'.repeat(32)] };
    const priorBlock = priorQuery.block;
    const priorEffects = priorQuery.effects;
    const priorGas = calls.gasCostFor(B);
    const priorState = calls.deployedStateFor(B);
    const second = await calls.begin(request(B));
    expect(second.queryContext).toBe(priorQuery);
    expect(second.queryContext.effects).not.toBe(priorEffects);
    expect(second.queryContext.effects.claimedNullifiers).toEqual([]);
    second.queryContext.effects = { ...second.queryContext.effects, claimedNullifiers: ['bb'.repeat(32)] };
    second.abort();
    expect(calls.queryContextFor(B)).toBe(priorQuery);
    expect(priorQuery.block).toEqual(priorBlock);
    expect(priorQuery.effects).toEqual(priorEffects);
    expect(priorQuery.effects.claimedNullifiers).toEqual(['aa'.repeat(32)]);
    expect(calls.gasCostFor(B)).toBe(priorGas);
    expect(calls.deployedStateFor(B)).toBe(priorState);
    expect(getContractState).toHaveBeenCalledTimes(1);
  });

  test('abort restores in-place VM, gas, and provider balance writes through original aliases', async () => {
    const { calls, root, callee } = setup();
    const providerBalance = callee.balance;
    Object.defineProperty(callee, 'balance', { value: providerBalance, writable: true, configurable: true });
    const first = await calls.begin(request(B));
    first.end();
    const query = calls.queryContextFor(B)!;
    // Current wasm accessors return fresh values; emulate a generated VM that
    // exposes mutable own cells so the rollback boundary must restore aliases.
    Object.defineProperties(query, {
      effects: { value: query.effects, writable: true, configurable: true },
      block: { value: query.block, writable: true, configurable: true },
      comIndices: { value: query.comIndices, writable: true, configurable: true },
    });
    const effects = query.effects;
    const block = query.block;
    const balance = block.balance;
    const nullifiers = effects.claimedNullifiers;
    const mints = effects.shieldedMints;
    const indices = query.comIndices;
    const gas = calls.gasCostFor(B)!;
    const rootBalance = calls.queryContextFor(A)!.block.balance;
    expect(rootBalance).not.toBe(root.balance);
    expect(balance).not.toBe(callee.balance);
    nullifiers.push('aa'.repeat(32));
    mints.set('bb'.repeat(32), 2n);
    balance.set('cc'.repeat(32), 3n);
    indices.set('dd'.repeat(32), 4n);
    gas.readTime = 5n;
    const second = await calls.begin(request(B));
    // A cached callee is reset for the call, but all earlier aliases must be
    // restored if that call fails after a generated VM or provider mutation.
    second.queryContext.effects.claimedNullifiers.push('ee'.repeat(32));
    second.queryContext.effects.shieldedMints.set('ff'.repeat(32), 7n);
    second.queryContext.block.balance.set('11'.repeat(32), 8n);
    second.queryContext.comIndices.set('22'.repeat(32), 9n);
    second.gasCost.readTime = 12n;
    callee.balance.set('33'.repeat(32), 10n);
    second.abort();
    expect(query.effects).toBe(effects);
    expect(query.block).toBe(block);
    expect(effects.claimedNullifiers).toBe(nullifiers);
    expect(effects.shieldedMints).toBe(mints);
    expect(block.balance).toBe(balance);
    expect(query.comIndices).toBe(indices);
    expect(calls.gasCostFor(B)).toBe(gas);
    expect(nullifiers).toEqual(['aa'.repeat(32)]);
    expect(mints.get('bb'.repeat(32))).toBe(2n);
    expect(mints.has('ff'.repeat(32))).toBe(false);
    expect(balance.get('cc'.repeat(32))).toBe(3n);
    expect(balance.has('11'.repeat(32))).toBe(false);
    expect(indices.get('dd'.repeat(32))).toBe(4n);
    expect(indices.has('22'.repeat(32))).toBe(false);
    expect(gas.readTime).toBe(5n);
    expect(callee.balance.has('33'.repeat(32))).toBe(false);
    expect(callee.balance).toBe(providerBalance);
  });

  test('abort rebinds a replaced commitment-index map on a cached callee', async () => {
    const { calls } = setup();
    const first = await calls.begin(request(B));
    first.end();
    const query = calls.queryContextFor(B)!;
    const originalIndices = new Map(query.comIndices);
    originalIndices.set('aa'.repeat(32), 1n);
    Object.defineProperty(query, 'comIndices', { value: originalIndices, writable: true, configurable: true });
    const second = await calls.begin(request(B));
    Object.defineProperty(query, 'comIndices', {
      value: new Map([['bb'.repeat(32), 2n]]),
      writable: true,
      configurable: true,
    });
    second.abort();
    expect(calls.queryContextFor(B)).toBe(query);
    expect(query.comIndices).toBe(originalIndices);
    expect(originalIndices.get('aa'.repeat(32))).toBe(1n);
    expect(query.comIndices.has('bb'.repeat(32))).toBe(false);
  });

  test('an un-restorable frozen VM cell poisons state and releases the active frame', async () => {
    const { calls } = setup();
    const first = await calls.begin(request(B));
    first.end();
    const query = calls.queryContextFor(B)!;
    Object.defineProperty(query, 'block', { value: query.block, writable: true, configurable: true });
    const savedBlock = query.block;
    const second = await calls.begin(request(B));
    savedBlock.balance.set('aa'.repeat(32), 3n);
    Object.freeze(savedBlock);
    expect(() => second.abort()).toThrow(/rollback failed.*poisoned/);
    expect(() => second.abort()).toThrow(/top of its active stack/);
    expect(() => calls.queryContextFor(B)).toThrow(/poisoned/);
    expect(() => first.queryContext).toThrow(/poisoned/);
    expect(() => first.gasCost).toThrow(/poisoned/);
    await expect(calls.begin(request(C))).rejects.toThrow(/poisoned/);
  });

  test('transaction owner unwinds a frozen nested frame and reports rollback failure', async () => {
    const { calls } = setup();
    const first = await calls.begin(request(C));
    first.end();
    const cached = calls.queryContextFor(C)!;
    Object.defineProperty(cached, 'block', { value: cached.block, writable: true, configurable: true });
    const savedBlock = cached.block;
    const rootQuery = calls.queryContextFor(A)!;
    Object.defineProperty(rootQuery, 'block', { value: rootQuery.block, writable: true, configurable: true });
    const savedRootBlock = rootQuery.block;
    let retainedView: { readonly queryContext: ocrt.QueryContext; readonly gasCost: ocrt.RunningCost } | undefined;
    let failure: unknown;
    try {
      await calls.transact(request(B), async (view) => {
        retainedView = view;
        const nested = await calls.begin(request(C));
        savedBlock.balance.set('bb'.repeat(32), 4n);
        Object.freeze(savedBlock);
        savedRootBlock.balance.set('cc'.repeat(32), 5n);
        Object.freeze(savedRootBlock);
        expect(nested.calleeAddress).toBe(C);
        throw new Error('callback failed');
      });
    } catch (error) {
      failure = error;
    }
    expect(failure).toBeInstanceOf(InternalCallTransactionRollbackError);
    const rollbackError = failure as InternalCallTransactionRollbackError;
    expect(rollbackError.message).toMatch(/rollback failed.*poisoned/);
    expect(rollbackError.cause).toEqual(new Error('callback failed'));
    expect(rollbackError.rollbackFailures).toHaveLength(2);
    expect(rollbackError.rollbackFailures.every((error) => String(error).includes('rollback failed'))).toBe(true);
    expect(() => calls.queryContextFor(B)).toThrow(/poisoned/);
    expect(() => retainedView!.queryContext).toThrow(/poisoned/);
    expect(() => retainedView!.gasCost).toThrow(/poisoned/);
    await expect(calls.begin(request(B))).rejects.toThrow(/poisoned/);
  });

  test('aborting a parent rolls back a completed nested call and its cached effects', async () => {
    const { calls, getContractState } = setup();
    const seededC = await calls.begin(request(C));
    seededC.end();
    const priorC = calls.queryContextFor(C)!;
    const priorCState = calls.deployedStateFor(C);
    priorC.effects = { ...priorC.effects, claimedNullifiers: ['cc'.repeat(32)] };
    const priorBlock = priorC.block;
    const priorEffects = priorC.effects;
    const b = await calls.begin(request(B));
    const nestedC = await calls.begin(request(C));
    expect(nestedC.queryContext).toBe(priorC);
    expect(nestedC.queryContext.block.caller).toEqual({ tag: 'contract', address: B });
    nestedC.queryContext.effects = { ...nestedC.queryContext.effects, claimedNullifiers: ['dd'.repeat(32)] };
    nestedC.end();
    b.abort();
    expect(calls.currentAddress).toBe(A);
    expect(calls.queryContextFor(B)).toBeUndefined();
    expect(calls.deployedStateFor(B)).toBeUndefined();
    expect(calls.queryContextFor(C)).toBe(priorC);
    expect(calls.deployedStateFor(C)).toBe(priorCState);
    expect(priorC.block).toEqual(priorBlock);
    expect(priorC.effects).toEqual(priorEffects);
    expect(priorC.effects.claimedNullifiers).toEqual(['cc'.repeat(32)]);
    expect(getContractState).toHaveBeenCalledTimes(2);
  });

  test('abort enforces LIFO but can recover a corrupt callee block', async () => {
    const { calls } = setup();
    const b = await calls.begin(request(B));
    const c = await calls.begin(request(C));
    expect(() => b.abort()).toThrow(/top of its active stack/);
    c.queryContext.block = { ...c.queryContext.block, parentBlockHash: 'cd'.repeat(32) };
    c.abort();
    expect(calls.currentAddress).toBe(B);
    expect(calls.queryContextFor(C)).toBeUndefined();
    b.abort();
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
  });

  test('abort invalidates a pending preflight without letting it commit later', async () => {
    let release!: (value: ocrt.ContractState) => void;
    const pending = new Promise<ocrt.ContractState>((resolve) => {
      release = resolve;
    });
    const race = new InternalCrossContractCallState({
      entryAddress: A,
      entryState: state(),
      parentBlockHash: BLOCK,
      time: 37,
      stateProvider: { getContractState: async (_hash, address) => (address === C ? pending : state()) },
      moduleProvider: { resolve: () => async () => module() },
    });
    const c = race.begin(request(C));
    const b = await race.begin(request(B));
    b.abort();
    release(state());
    await expect(c).rejects.toThrow(/stack changed during preflight/);
    expect(race.currentAddress).toBe(A);
    expect(race.queryContextFor(B)).toBeUndefined();
    expect(race.queryContextFor(C)).toBeUndefined();
  });

  test('the call-state transaction owner unwinds nested work after a callback failure', async () => {
    const { calls, callee } = setup();
    const rootQuery = calls.queryContextFor(A);
    const before = callee.serialize();
    await expect(
      calls.transact(request(B), async (b) => {
        expect('end' in b).toBe(false);
        expect('abort' in b).toBe(false);
        expect('deployedState' in b).toBe(false);
        expect('module' in b).toBe(false);
        const c = await calls.begin(request(C));
        b.queryContext.block.balance.set('aa'.repeat(32), 7n);
        c.queryContext.effects = { ...c.queryContext.effects, claimedNullifiers: ['bb'.repeat(32)] };
        throw new Error('generated VM failed after nested entry');
      }),
    ).rejects.toThrow('generated VM failed after nested entry');
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
    expect(calls.isActive(C)).toBe(false);
    expect(calls.queryContextFor(A)).toBe(rootQuery);
    expect(calls.queryContextFor(B)).toBeUndefined();
    expect(calls.queryContextFor(C)).toBeUndefined();
    expect(callee.serialize()).toEqual(before);
  });

  test('the call-state transaction owner commits a completed leaf and releases re-entry', async () => {
    const { calls } = setup();
    const result = await calls.transact(request(B), async (call) => call.calleeAddress);
    expect(result).toBe(B);
    expect(calls.queryContextFor(B)).toBeDefined();
    expect(calls.currentAddress).toBe(A);
    expect(calls.isActive(B)).toBe(false);
  });

  test('aborting state and discarding a copied trace leave the original EntryPoint caller intact', async () => {
    const callerBytes = readFileSync(new URL('../../test-center/composable/EntryPoint/Caller.compact', import.meta.url));
    expect(createHash('sha256').update(callerBytes).digest('hex')).toBe(
      '25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6',
    );
    const { calls, root } = setup();
    const trace = createInternalCallTraceContext({
      circuitId: 'pay',
      contractAddress: A,
      queryContext: calls.queryContextFor(A)!,
      zswapLocalState: emptyZswapLocalState('0'.repeat(64)),
      privateState: 0,
      parentBlockHash: BLOCK,
      time: 37,
    });
    const b = await calls.begin(request(B));
    const child = enterInternalCallTraceContext(trace, b);
    const failed = finalizeInternalCallProofData(
      advanceInternalCallTraceContext(
        child,
        child.callContext.currentQueryContext,
        { ...emptyRunningCost(), readTime: 1n },
        { ...child.callContext.currentZswapLocalState, currentIndex: 1n },
      ),
      {
        input: { value: [], alignment: [] },
        output: { value: [], alignment: [] },
        publicTranscript: [],
        privateTranscriptOutputs: [],
      },
    );
    expect(failed.callProofDataTrace).toHaveLength(1);
    b.abort();
    expect(trace.callProofDataTrace).toEqual([]);
    expect(trace.zswapLocalStates[B]).toBeUndefined();
    expect(calls.queryContextFor(B)).toBeUndefined();
    expect(calls.deployedStateFor(A)).toBe(root);
  });
});
