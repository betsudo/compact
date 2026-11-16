// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { describe, expect, test, vi } from 'vitest';
import { InternalCrossContractCallState } from '../src/cross-contract-call-state.js';
import { emptyRunningCost } from '../src/circuit-context.js';
import { CompactTypeUnsignedInteger } from '../src/compact-types.js';
import { InterfaceDescriptor } from '../src/interface-descriptor.js';
import {
  advanceInternalCallTraceContext,
  copyInternalCallTraceContext,
  createInternalCallTraceContext,
  enterInternalCallTraceContext,
  finalizeInternalCallProofData,
  queryInternalCallTraceContext,
  restoreInternalCallerTraceContext,
} from '../src/internal-call-trace-context.js';
import { Module } from '../src/module.js';
import { emptyZswapLocalState } from '../src/zswap.js';
import { verifierKeyHashOf } from '../src/verifier-key-hash.js';
import * as publicRuntime from '../src/index.js';

const A = `0200${'11'.repeat(30)}`;
const B = `0200${'22'.repeat(30)}`;
const C = `0200${'33'.repeat(30)}`;
const BLOCK = 'ab'.repeat(32);
const KEY = Uint8Array.of(1, 2, 3, 4);
const FIELD = { tag: 'Field' } as const;
const declaration: InterfaceDescriptor = { add: { argumentTypes: [FIELD], resultType: FIELD } };
const request = (calleeAddress: string) => ({ calleeAddress, calleeCircuitId: 'add', interfaceName: 'Inner', declaration });
const proofData = () => ({
  input: { value: [], alignment: [] },
  output: { value: [], alignment: [] },
  publicTranscript: [],
  privateTranscriptOutputs: [],
});
const digest = (path: string) =>
  createHash('sha256')
    .update(readFileSync(new URL(path, import.meta.url)))
    .digest('hex');

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
  const rootState = state();
  const calls = new InternalCrossContractCallState({
    entryAddress: A,
    entryState: rootState,
    parentBlockHash: BLOCK,
    time: 37,
    stateProvider: { getContractState: async () => state() },
    moduleProvider: { resolve: () => async () => module() },
  });
  const root = createInternalCallTraceContext({
    circuitId: 'pay',
    contractAddress: A,
    queryContext: calls.queryContextFor(A)!,
    zswapLocalState: emptyZswapLocalState('0'.repeat(64)),
    privateState: 7,
    parentBlockHash: BLOCK,
    time: 37,
  });
  return { calls, root };
};
const nextQuery = (previous: ocrt.QueryContext): ocrt.QueryContext => {
  const next = new ocrt.QueryContext(previous.state, previous.address);
  next.block = { ...previous.block };
  return next;
};

const callerFirstRead = (indexValue = 0n): ocrt.Op<null>[] => {
  const index = new CompactTypeUnsignedInteger(255n, 1);
  return [
    { dup: { n: 0 } },
    {
      idx: {
        cached: false,
        pushPath: false,
        path: [{ tag: 'value', value: { value: index.toValue(indexValue), alignment: index.alignment() } }],
      },
    },
    // The original generated Caller emits `undefined` before VM read insertion.
    { popeq: { cached: false, result: undefined } } as unknown as ocrt.Op<null>,
  ];
};

const entryPointReadContext = () => {
  const state = new ocrt.ContractState();
  state.data = new ocrt.ChargedState(
    ocrt.StateValue.newArray().arrayPush(
      ocrt.StateValue.newCell({
        value: [ocrt.encodeContractAddress(B)],
        alignment: [{ tag: 'atom', value: { tag: 'bytes', length: 32 } }],
      }),
    ),
  );
  const queryContext = new ocrt.QueryContext(state.data, A);
  queryContext.block = {
    ...queryContext.block,
    ownAddress: A,
    parentBlockHash: BLOCK,
    secondsSinceEpoch: 37n,
  };
  return createInternalCallTraceContext({
    circuitId: 'pay',
    contractAddress: A,
    queryContext,
    zswapLocalState: emptyZswapLocalState('0'.repeat(64)),
    privateState: 7,
    parentBlockHash: BLOCK,
    time: 37,
  });
};

describe('internal pinned call trace cells', () => {
  test('routes the original Caller first ledger read through replacement query and gas cells', () => {
    const root = entryPointReadContext();
    const originalState = root.callContext.currentQueryContext.state.toString();
    const inputProof = proofData();
    const result = queryInternalCallTraceContext(root, inputProof, callerFirstRead(), ocrt.CostModel.initialCostModel());
    expect(result.read?.value[0]).toEqual(ocrt.encodeContractAddress(B));
    expect(result.context).not.toBe(root);
    expect(result.context.callContext.currentQueryContext).toBe(result.context.queryContexts[A]);
    expect(result.context.callContext.currentGasCost).toBe(result.context.gasCosts[A]);
    expect(result.context.callContext.currentZswapLocalState).toBe(result.context.zswapLocalStates[A]);
    expect(result.context.callContext.currentQueryContext).not.toBe(root.callContext.currentQueryContext);
    expect(result.context.callContext.currentGasCost.readTime).toBeGreaterThan(0n);
    expect(result.context.callContext.currentQueryContext.state.toString()).toBe(originalState);
    expect(root.callContext.currentQueryContext.state.toString()).toBe(originalState);
    expect(root.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(inputProof.publicTranscript).toEqual([]);
    expect(result.partialProofData.publicTranscript).toHaveLength(3);
    expect(result.partialProofData.publicTranscript[2]).toEqual({
      popeq: { cached: false, result: result.read },
    });
  });

  test('query errors and changed block or address discard copied context and transcript', () => {
    for (const changed of ['block', 'address'] as const) {
      const root = entryPointReadContext();
      const inputProof = proofData();
      const firstRead = callerFirstRead();
      const valid = root.callContext.currentQueryContext.query(firstRead, ocrt.CostModel.initialCostModel());
      let changedContext: ocrt.QueryContext;
      if (changed === 'block') {
        changedContext = new ocrt.QueryContext(valid.context.state, A);
        changedContext.block = { ...valid.context.block, parentBlockHash: 'cd'.repeat(32) };
      } else {
        changedContext = new ocrt.QueryContext(valid.context.state, B);
        changedContext.block = { ...valid.context.block, ownAddress: B };
      }
      expect(changedContext.address).toBe(changed === 'address' ? B : A);
      expect(changedContext.block.parentBlockHash).toBe(changed === 'block' ? 'cd'.repeat(32) : BLOCK);
      const result = { context: changedContext, gasCost: valid.gasCost, events: valid.events } as ReturnType<
        ocrt.QueryContext['query']
      >;
      const querySpy = vi.spyOn(root.callContext.currentQueryContext, 'query').mockReturnValue(result);
      expect(() => queryInternalCallTraceContext(root, inputProof, firstRead, ocrt.CostModel.initialCostModel())).toThrow(
        'pinned parent block or call identity',
      );
      expect(querySpy).toHaveBeenCalledOnce();
      expect(root.callContext.currentQueryContext).toBe(root.queryContexts[A]);
      expect(root.callContext.currentGasCost).toEqual(emptyRunningCost());
      expect(root.events).toEqual([]);
      expect(inputProof.publicTranscript).toEqual([]);
    }
    const root = entryPointReadContext();
    const inputProof = proofData();
    expect(() =>
      queryInternalCallTraceContext(root, inputProof, callerFirstRead(9n), ocrt.CostModel.initialCostModel()),
    ).toThrow();
    expect(root.callContext.currentQueryContext).toBe(root.queryContexts[A]);
    expect(inputProof.publicTranscript).toEqual([]);
  });

  test('tags VM logs with the live address and preserves unrelated address cells', () => {
    const root = entryPointReadContext();
    const other = new ocrt.QueryContext(new ocrt.ContractState().data, B);
    root.queryContexts[B] = other;
    root.gasCosts[B] = { ...emptyRunningCost(), computeTime: 19n };
    const inputProof = proofData();
    const cell = (value: ocrt.Value, length: number): ocrt.EncodedStateValue => ({
      tag: 'cell',
      content: { value, alignment: [{ tag: 'atom', value: { tag: 'bytes', length } }] },
    });
    const logProgram: ocrt.Op<null>[] = [
      {
        push: {
          storage: false,
          value: {
            tag: 'array',
            content: [cell(ocrt.bigIntToValue(1n), 4), cell(ocrt.bigIntToValue(8n), 1), cell([new Uint8Array()], 0)],
          },
        },
      },
      'log',
    ];
    const result = queryInternalCallTraceContext(root, inputProof, logProgram, ocrt.CostModel.initialCostModel());
    expect(result.read).toBeUndefined();
    expect(result.context.events).toHaveLength(1);
    expect(result.context.events[0]).toMatchObject({ address: A, version: 1, eventType: 'paused' });
    expect(result.context.queryContexts[B]).toBe(other);
    expect(result.context.gasCosts[B]).toEqual({ ...emptyRunningCost(), computeTime: 19n });
    expect(root.events).toEqual([]);
    expect(root.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(inputProof.publicTranscript).toEqual([]);
    expect(result.partialProofData.publicTranscript).toEqual(logProgram);
  });
  test('binds original EntryPoint fixture bytes and keeps the public host context closed', () => {
    expect(digest('../../test-center/composable/EntryPoint/Callee.compact')).toBe(
      '89263a08b98928b7b9c4b750588c6bc2226784ceaf62a660511042c2126a51c7',
    );
    expect(digest('../../test-center/composable/EntryPoint/Caller.compact')).toBe(
      '25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6',
    );
    expect(digest('../../test-center/ts/composable/entry-point.ts')).toBe(
      '3b3e8995988e50121749ba5c59a593da326ec58786590e5a688d96ec7740785d',
    );
    expect('crossContractCall' in publicRuntime).toBe(false);
    expect('createInternalCallTraceContext' in publicRuntime).toBe(false);
    expect('queryInternalCallTraceContext' in publicRuntime).toBe(false);
  });

  test('copies all mutable containers and keeps per-address live cells aligned', async () => {
    const { calls, root } = setup();
    const copy = copyInternalCallTraceContext(root);
    expect(copy.callContext).not.toBe(root.callContext);
    expect(copy.queryContexts).not.toBe(root.queryContexts);
    expect(copy.gasCosts).not.toBe(root.gasCosts);
    expect(copy.zswapLocalStates).not.toBe(root.zswapLocalStates);
    expect(copy.callProofDataTrace).not.toBe(root.callProofDataTrace);
    expect(copy.events).not.toBe(root.events);
    expect(Object.getPrototypeOf(copy.queryContexts)).toBeNull();
    const b = await calls.begin(request(B));
    const entered = enterInternalCallTraceContext(root, b);
    expect(entered.callContext.currentQueryContext).toBe(entered.queryContexts[B]);
    expect(entered.callContext.currentGasCost).toBe(entered.gasCosts[B]);
    expect(entered.callContext.currentZswapLocalState).toBe(entered.zswapLocalStates[B]);
    expect(entered.zswapLocalStates[B]).not.toBe(root.zswapLocalStates[A]);
    expect(entered.zswapLocalStates[B].coinPublicKey).toEqual(root.zswapLocalStates[A].coinPublicKey);
    expect(root.queryContexts[B]).toBeUndefined();
    b.end();
  });

  test('finalizes nested calls depth first, then the root, with separate address and Zswap cells', async () => {
    const { calls, root } = setup();
    const b = await calls.begin(request(B));
    const enteredB = enterInternalCallTraceContext(root, b);
    const c = await calls.begin(request(C));
    const enteredC = enterInternalCallTraceContext(enteredB, c);
    const nextCQuery = nextQuery(enteredC.callContext.currentQueryContext);
    const nextCZswap = { ...enteredC.callContext.currentZswapLocalState, currentIndex: 1n };
    const advancedC = advanceInternalCallTraceContext(
      enteredC,
      nextCQuery,
      { ...emptyRunningCost(), computeTime: 3n },
      nextCZswap,
    );
    const completeC = finalizeInternalCallProofData(advancedC, proofData());
    expect(completeC.callProofDataTrace.map((entry) => entry.contractAddress)).toEqual([C]);
    expect(completeC.callProofDataTrace[0]).toMatchObject({
      circuitId: 'add',
      contractAddress: C,
      initialQueryContext: enteredC.callContext.initialQueryContext,
      finalQueryContext: nextCQuery,
      zswapLocalState: nextCZswap,
    });
    c.end();
    const resumedB = restoreInternalCallerTraceContext(enteredB, completeC, c);
    expect(resumedB.callContext.contractAddress).toBe(B);
    expect(resumedB.queryContexts[C]).toBe(nextCQuery);
    expect(resumedB.gasCosts[C].computeTime).toBe(3n);
    const completeB = finalizeInternalCallProofData(resumedB, proofData());
    b.end();
    const resumedRoot = restoreInternalCallerTraceContext(root, completeB, b);
    const completeRoot = finalizeInternalCallProofData(resumedRoot, proofData());
    expect(completeRoot.callProofDataTrace.map(({ contractAddress }) => contractAddress)).toEqual([C, B, A]);
    expect(completeRoot.callProofDataTrace.map(({ circuitId }) => circuitId)).toEqual(['add', 'add', 'pay']);
    expect(completeRoot.callProofDataTrace[0].zswapLocalState.currentIndex).toBe(1n);
    expect(root.callProofDataTrace).toEqual([]);
  });

  test('discarding a failed child copy does not commit its query, gas, Zswap or trace cells', async () => {
    const { calls, root } = setup();
    const b = await calls.begin(request(B));
    const entered = enterInternalCallTraceContext(root, b);
    const failed = finalizeInternalCallProofData(
      advanceInternalCallTraceContext(
        entered,
        nextQuery(entered.callContext.currentQueryContext),
        { ...emptyRunningCost(), readTime: 5n },
        { ...entered.callContext.currentZswapLocalState, currentIndex: 2n },
      ),
      proofData(),
    );
    expect(failed.callProofDataTrace).toHaveLength(1);
    expect(root.queryContexts[B]).toBeUndefined();
    expect(root.gasCosts[B]).toBeUndefined();
    expect(root.zswapLocalStates[B]).toBeUndefined();
    expect(root.callProofDataTrace).toEqual([]);
    expect(root.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(root.callContext.currentZswapLocalState.currentIndex).toBe(0n);
    b.end();
  });

  test('restoration requires a finalized matching callee and rejects changed parent block', async () => {
    const { calls, root } = setup();
    const b = await calls.begin(request(B));
    const entered = enterInternalCallTraceContext(root, b);
    expect(() => restoreInternalCallerTraceContext(root, entered, b)).toThrow(/finalized callee/);
    const complete = finalizeInternalCallProofData(entered, proofData());
    const changed = { ...complete, parentBlockHash: 'cd'.repeat(32) };
    expect(() => restoreInternalCallerTraceContext(root, changed, b)).toThrow(/does not match/);
    const wrongQuery = nextQuery(entered.callContext.currentQueryContext);
    wrongQuery.block = { ...wrongQuery.block, parentBlockHash: 'cd'.repeat(32) };
    expect(() =>
      advanceInternalCallTraceContext(entered, wrongQuery, emptyRunningCost(), entered.callContext.currentZswapLocalState),
    ).toThrow(/pinned parent block/);
    expect(root.callProofDataTrace).toEqual([]);
    b.end();
    const withB = restoreInternalCallerTraceContext(root, complete, b);
    const c = await calls.begin(request(C));
    const completeC = finalizeInternalCallProofData(enterInternalCallTraceContext(withB, c), proofData());
    completeC.callProofDataTrace[0] = { ...completeC.callProofDataTrace[0] };
    expect(() => restoreInternalCallerTraceContext(withB, completeC, c)).toThrow(/earlier call proof entry/);
    c.end();
  });

  test('a later call to one address resumes its Zswap state without changing the root state', async () => {
    const { calls, root } = setup();
    const first = await calls.begin(request(B));
    const entered = enterInternalCallTraceContext(root, first);
    const nextZswap = { ...entered.callContext.currentZswapLocalState, currentIndex: 4n };
    const advanced = advanceInternalCallTraceContext(
      entered,
      entered.callContext.currentQueryContext,
      emptyRunningCost(),
      nextZswap,
    );
    const complete = finalizeInternalCallProofData(advanced, proofData());
    first.end();
    const resumedRoot = restoreInternalCallerTraceContext(root, complete, first);
    const second = await calls.begin(request(B));
    const reentered = enterInternalCallTraceContext(resumedRoot, second);
    expect(reentered.callContext.currentZswapLocalState.currentIndex).toBe(4n);
    expect(reentered.callContext.currentZswapLocalState).toBe(resumedRoot.zswapLocalStates[B]);
    expect(reentered.zswapLocalStates[A].currentIndex).toBe(0n);
    second.end();
  });
});
