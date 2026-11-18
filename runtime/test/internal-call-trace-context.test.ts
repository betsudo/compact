// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { describe, expect, test, vi } from 'vitest';
import { InternalCrossContractCallState } from '../src/cross-contract-call-state.js';
import { emptyRunningCost } from '../src/circuit-context.js';
import { CompactTypeUnsignedInteger } from '../src/compact-types.js';
import { InterfaceDescriptor } from '../src/interface-descriptor.js';
import { appendInternalZswapInput, appendInternalZswapOutput, ownInternalZswapPublicKey } from '../src/internal-call-zswap.js';
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

const coinInfo = () =>
  ocrt.encodeShieldedCoinInfo({
    type: ocrt.sampleRawTokenType(),
    nonce: '2ab78b2272ec3489da60e6af54a87bfa53a7fa727602a040df782ebae7f5ab59',
    value: 5n,
  });
const recipient = (address: ocrt.ContractAddress) => ({
  is_left: false,
  left: { bytes: new Uint8Array(32) },
  right: { bytes: ocrt.encodeContractAddress(address) },
});

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
    expect('appendInternalZswapInput' in publicRuntime).toBe(false);
    expect('appendInternalZswapOutput' in publicRuntime).toBe(false);
  });

  test('internal input and output replace per-address coin/query cells without charging gas', () => {
    const root = entryPointReadContext();
    const queried = queryInternalCallTraceContext(
      root,
      proofData(),
      callerFirstRead(),
      ocrt.CostModel.initialCostModel(),
    ).context;
    const gas = queried.callContext.currentGasCost;
    const coin = coinInfo();
    const target = recipient(B);
    const input = appendInternalZswapInput(queried, { ...coin, mt_index: 0n });
    const output = appendInternalZswapOutput(input, coin, target);
    expect(input.callContext.currentZswapLocalState.inputs).toHaveLength(1);
    expect(input.callContext.currentZswapLocalState).toBe(input.zswapLocalStates[A]);
    expect(output.callContext.currentZswapLocalState.outputs).toHaveLength(1);
    expect(output.callContext.currentZswapLocalState.currentIndex).toBe(1n);
    expect(output.callContext.currentZswapLocalState).toBe(output.zswapLocalStates[A]);
    expect(output.callContext.currentQueryContext).toBe(output.queryContexts[A]);
    expect(output.callContext.currentQueryContext).not.toBe(input.callContext.currentQueryContext);
    expect(output.callContext.currentQueryContext.comIndices.size).toBe(1);
    expect(input.callContext.currentQueryContext.comIndices.size).toBe(0);
    const legacy = publicRuntime.createCircuitContext(A, '0'.repeat(64), new ocrt.ContractState(), null);
    publicRuntime.createZswapOutput(legacy, coin, target);
    expect([...output.callContext.currentQueryContext.comIndices.keys()]).toEqual([
      ...legacy.currentQueryContext.comIndices.keys(),
    ]);
    expect(input.callContext.currentGasCost).toEqual(gas);
    expect(output.callContext.currentGasCost).toEqual(gas);
    expect(output.callContext.currentGasCost).toBe(output.gasCosts[A]);
    expect(root.callContext.currentZswapLocalState.inputs).toEqual([]);
    expect(root.callContext.currentZswapLocalState.outputs).toEqual([]);
    expect(ownInternalZswapPublicKey(output)).toEqual({ bytes: ocrt.encodeCoinPublicKey('0'.repeat(64)) });

    // The completed copy owns every byte array, including prior entries.
    coin.nonce[0] ^= 1;
    target.right.bytes[0] ^= 1;
    input.callContext.currentZswapLocalState.inputs[0].color[0] ^= 1;
    expect(output.callContext.currentZswapLocalState.outputs[0].coinInfo.nonce).not.toEqual(coin.nonce);
    expect(output.callContext.currentZswapLocalState.outputs[0].recipient.right.bytes).not.toEqual(target.right.bytes);
    expect(output.callContext.currentZswapLocalState.inputs[0].color).not.toEqual(
      input.callContext.currentZswapLocalState.inputs[0].color,
    );
    const key = ownInternalZswapPublicKey(output);
    key.bytes[0] ^= 1;
    expect(ownInternalZswapPublicKey(output)).toEqual({ bytes: ocrt.encodeCoinPublicKey('0'.repeat(64)) });
  });

  test('callee coin state is separate from the caller and survives return to its address', async () => {
    const { calls, root } = setup();
    const active = await calls.begin(request(B));
    const entered = enterInternalCallTraceContext(root, active);
    const withCoin = appendInternalZswapOutput(entered, coinInfo(), recipient(B));
    expect(withCoin.callContext.currentZswapLocalState).toBe(withCoin.zswapLocalStates[B]);
    expect(withCoin.callContext.currentQueryContext).toBe(withCoin.queryContexts[B]);
    expect(withCoin.zswapLocalStates[B].coinPublicKey.bytes).toEqual(root.zswapLocalStates[A].coinPublicKey.bytes);
    expect(root.zswapLocalStates[B]).toBeUndefined();
    expect(root.queryContexts[B]).toBeUndefined();
    const finalized = finalizeInternalCallProofData(withCoin, proofData());
    const restored = restoreInternalCallerTraceContext(root, finalized, active);
    active.end();
    expect(restored.zswapLocalStates[B].outputs).toHaveLength(1);
    expect(restored.queryContexts[B].comIndices.size).toBe(1);
    expect(restored.callContext.currentZswapLocalState).toBe(root.zswapLocalStates[A]);
    expect(restored.gasCosts[A]).toEqual(emptyRunningCost());
    expect(restored.gasCosts[B]).toEqual(emptyRunningCost());
    expect(restored.callProofDataTrace[0].zswapLocalState).toBe(finalized.zswapLocalStates[B]);
  });

  test('malformed coin, stale map, and wrong returned query identity discard an internal output', () => {
    const validCoin = coinInfo();
    const target = recipient(B);
    const original = entryPointReadContext();
    const badCoin = { ...validCoin, value: -1n };
    expect(() => appendInternalZswapOutput(original, badCoin, target)).toThrow();
    expect(original.callContext.currentZswapLocalState.outputs).toEqual([]);
    expect(original.callContext.currentQueryContext.comIndices.size).toBe(0);
    for (const changed of ['block', 'address'] as const) {
      const root = entryPointReadContext();
      const next = new ocrt.QueryContext(root.callContext.currentQueryContext.state, changed === 'address' ? B : A);
      next.block = {
        ...root.callContext.currentQueryContext.block,
        ownAddress: changed === 'address' ? B : A,
        parentBlockHash: changed === 'block' ? 'cd'.repeat(32) : BLOCK,
      };
      const spy = vi.spyOn(root.callContext.currentQueryContext, 'insertCommitment').mockReturnValue(next);
      expect(() => appendInternalZswapOutput(root, validCoin, target)).toThrow('pinned parent block or call identity');
      expect(spy).toHaveBeenCalledOnce();
      expect(root.queryContexts[A]).toBe(root.callContext.currentQueryContext);
      expect(root.zswapLocalStates[A].outputs).toEqual([]);
      expect(root.callContext.currentQueryContext.comIndices.size).toBe(0);
    }
    const stale = entryPointReadContext();
    stale.zswapLocalStates[A] = { ...stale.zswapLocalStates[A] };
    expect(() => appendInternalZswapInput(stale, { ...validCoin, mt_index: 0n })).toThrow('live call cells differ');
    expect(() => appendInternalZswapOutput(stale, validCoin, target)).toThrow('live call cells differ');
    expect(stale.callContext.currentZswapLocalState.inputs).toEqual([]);
    const absent = entryPointReadContext();
    absent.callContext.currentZswapLocalState = undefined as unknown as typeof absent.callContext.currentZswapLocalState;
    absent.zswapLocalStates[A] = absent.callContext.currentZswapLocalState;
    expect(() => ownInternalZswapPublicKey(absent)).toThrow('Zswap local state is undefined');
  });

  test('encoded accessors and sparse previous coin arrays refuse without evaluating a getter', () => {
    const root = entryPointReadContext();
    const coin = coinInfo();
    let reads = 0;
    Object.defineProperty(coin, 'nonce', {
      enumerable: true,
      get: () => {
        reads++;
        return new Uint8Array(32);
      },
    });
    expect(() => appendInternalZswapOutput(root, coin, recipient(B))).toThrow('own data field');
    expect(reads).toBe(0);
    expect(root.callContext.currentQueryContext.comIndices.size).toBe(0);
    expect(root.callContext.currentZswapLocalState.outputs).toEqual([]);

    const withInput = appendInternalZswapInput(root, { ...coinInfo(), mt_index: 0n });
    delete withInput.callContext.currentZswapLocalState.inputs[0];
    expect(() => appendInternalZswapOutput(withInput, coinInfo(), recipient(B))).toThrow('own data field');
    expect(withInput.callContext.currentQueryContext.comIndices.size).toBe(0);
    expect(withInput.callContext.currentZswapLocalState.outputs).toEqual([]);
  });

  test('typed-array species cannot alias source coin, recipient, or submitter-key bytes', () => {
    class AliasingBytes extends Uint8Array {}
    const aliased = new AliasingBytes(32).fill(7);
    Object.defineProperty(AliasingBytes, Symbol.species, {
      value: function () {
        return aliased;
      },
    });
    expect(Uint8Array.prototype.slice.call(aliased)).toBe(aliased);
    const root = entryPointReadContext();
    root.callContext.currentZswapLocalState = {
      ...root.callContext.currentZswapLocalState,
      coinPublicKey: { bytes: aliased },
    };
    root.zswapLocalStates[A] = root.callContext.currentZswapLocalState;
    const coin = { ...coinInfo(), nonce: aliased, color: aliased };
    const target = { ...recipient(B), right: { bytes: aliased } };
    const input = appendInternalZswapInput(root, { ...coin, mt_index: 0n });
    const output = appendInternalZswapOutput(input, coin, target);
    const outputNonce = output.zswapLocalStates[A].outputs[0].coinInfo.nonce;
    const inputColor = output.zswapLocalStates[A].inputs[0].color;
    const recipientBytes = output.zswapLocalStates[A].outputs[0].recipient.right.bytes;
    const submitterBytes = ownInternalZswapPublicKey(output).bytes;
    for (const copied of [outputNonce, inputColor, recipientBytes, submitterBytes]) {
      expect(copied).not.toBe(aliased);
      expect(copied.constructor).toBe(Uint8Array);
      expect(copied[0]).toBe(7);
    }
    aliased[0] = 3;
    for (const copied of [outputNonce, inputColor, recipientBytes, submitterBytes]) expect(copied[0]).toBe(7);
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
