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

import { describe, expect, test, vi } from 'vitest';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { createCircuitContext, finalizeCallProofData, queryLedgerState } from '../src/circuit-context.js';
import {
  InternalGeneratedContextRollbackError,
  InternalGeneratedContextTransaction,
} from '../src/internal-generated-context-transaction.js';

const address = ocrt.sampleContractAddress();
const otherAddress = ocrt.sampleContractAddress();
const parentBlockHash = 'ab'.repeat(32);

const context = () =>
  createCircuitContext({
    circuitId: 'caller',
    contractAddress: address,
    coinPublicKeyOrZswapState: '0'.repeat(64),
    contractState: new ocrt.ContractState(),
    privateState: { counter: 7 },
    parentBlockHash,
    time: 123,
    crossContract: {
      stateProvider: { getContractState: vi.fn(async () => undefined) },
      moduleProvider: { resolve: vi.fn(() => undefined) },
    },
  });

const proofData = () => ({
  input: { value: [], alignment: [] },
  output: { value: [], alignment: [] },
  publicTranscript: [] as ocrt.Op<ocrt.AlignedValue>[],
  privateTranscriptOutputs: [] as ocrt.AlignedValue[],
});

describe('internal generated context transaction', () => {
  test('keeps a caller pre-call query while undoing generated cells and proof data', async () => {
    const caller = context();
    const proof = proofData();
    const initialQuery = caller.callContext.currentQueryContext;
    const preCallOp = { preCallRead: 'address' } as unknown as ocrt.Op<null>;
    vi.spyOn(initialQuery, 'query').mockReturnValue({
      context: initialQuery,
      gasCost: { readTime: 1n, computeTime: 0n, bytesWritten: 0n, bytesDeleted: 0n },
      events: [],
    } as ReturnType<typeof initialQuery.query>);
    queryLedgerState(caller, proof, [preCallOp]);
    expect(proof.publicTranscript).toEqual([preCallOp]);
    const transcript = proof.publicTranscript;
    const query = caller.callContext.currentQueryContext;
    Object.defineProperty(query, 'block', { value: query.block, configurable: true, writable: true });
    Object.defineProperty(query, 'effects', { value: query.effects, configurable: true, writable: true });
    Object.defineProperty(query, 'comIndices', { value: query.comIndices, configurable: true, writable: true });
    const initialIndices = query.comIndices;
    const state = new ocrt.ContractState();
    caller.contractStates[address] = state;
    finalizeCallProofData(caller, proof);
    const trace = caller.callProofDataTrace;
    const balance = query.block.balance;
    const initialBalanceSize = balance.size;
    const beforeGas = caller.callContext.currentGasCost.readTime;
    const transaction = new InternalGeneratedContextTransaction();
    const failure = new Error('callee failed');

    await expect(
      transaction.transact(caller, proof, async () => {
        caller.currentPrivateState.counter = 99;
        caller.callContext.currentPrivateState!.counter = 99;
        caller.callContext.currentGasCost.readTime = 88n;
        caller.callContext.currentZswapLocalState.currentIndex = 5n;
        caller.callContext.currentZswapLocalState.inputs.push({
          nonce: new Uint8Array(32),
          color: new Uint8Array(32),
          value: 1n,
          mt_index: 0n,
        });
        balance.set('cd'.repeat(32) as unknown as ocrt.TokenType, 4n);
        query.effects.claimedNullifiers.push('ee'.repeat(32));
        Object.defineProperty(query, 'comIndices', { value: new Map([['new', 1n]]), configurable: true, writable: true });
        state.balance.set('ff'.repeat(32) as unknown as ocrt.TokenType, 9n);
        caller.queryContexts[otherAddress] = new ocrt.QueryContext(new ocrt.ContractState().data, otherAddress);
        caller.gasCosts[otherAddress] = { readTime: 2n, computeTime: 0n, bytesWritten: 0n, bytesDeleted: 0n };
        caller.zswapLocalStates[otherAddress] = caller.callContext.currentZswapLocalState;
        caller.contractStates[otherAddress] = new ocrt.ContractState();
        caller.callProofDataTrace.push({ ...trace[0] });
        trace[0].privateTranscriptOutputs.push({ value: [], alignment: [] });
        caller.events.push({ address, data: new Uint8Array([1]) } as never);
        caller.activeContracts.add(otherAddress);
        proof.publicTranscript[0] = { changed: true } as never;
        proof.publicTranscript.push({ later: true } as never);
        proof.privateTranscriptOutputs.push({ value: [], alignment: [] });
        caller.callContext = { ...caller.callContext, currentPrivateState: { counter: 100 } };
        caller.queryContexts = { ...caller.queryContexts };
        caller.callProofDataTrace = [...caller.callProofDataTrace];
        throw failure;
      }),
    ).rejects.toBe(failure);

    expect(transaction.activeDepth).toBe(0);
    expect(transaction.isPoisoned).toBe(false);
    expect(caller.currentPrivateState.counter).toBe(7);
    expect(caller.callContext.currentPrivateState!.counter).toBe(7);
    expect(caller.callContext.currentGasCost.readTime).toBe(beforeGas);
    expect(caller.callContext.currentZswapLocalState.currentIndex).toBe(0n);
    expect(caller.callContext.currentZswapLocalState.inputs).toEqual([]);
    expect(caller.queryContexts[address]).toBe(query);
    expect(caller.queryContexts[otherAddress]).toBeUndefined();
    expect(caller.gasCosts[otherAddress]).toBeUndefined();
    expect(caller.zswapLocalStates[otherAddress]).toBeUndefined();
    expect(caller.contractStates[otherAddress]).toBeUndefined();
    expect(balance.size).toBe(initialBalanceSize);
    expect(query.effects.claimedNullifiers).toEqual([]);
    expect(query.comIndices).toBe(initialIndices);
    expect(state.balance.size).toBe(0);
    expect(caller.callProofDataTrace).toBe(trace);
    expect(trace).toHaveLength(1);
    expect(trace[0].privateTranscriptOutputs).toEqual([]);
    expect(caller.events).toEqual([]);
    expect(caller.activeContracts).toEqual(new Set([address]));
    expect(proof.publicTranscript).toBe(transcript);
    expect(proof.publicTranscript).toEqual([preCallOp]);
    expect(proof.privateTranscriptOutputs).toEqual([]);
  });

  test('a nested abort preserves the outer caller prefix and may be caught', async () => {
    const caller = context();
    const proof = proofData();
    proof.publicTranscript.push({ alreadyRead: true } as never);
    const transaction = new InternalGeneratedContextTransaction();
    await transaction.transact(caller, proof, async () => {
      caller.currentPrivateState.counter = 8;
      proof.publicTranscript.push({ outer: true } as never);
      await expect(
        transaction.transact(caller, proof, () => {
          caller.currentPrivateState.counter = 9;
          proof.publicTranscript.push({ inner: true } as never);
          throw new Error('inner');
        }),
      ).rejects.toThrow('inner');
      expect(caller.currentPrivateState.counter).toBe(8);
      expect(proof.publicTranscript).toEqual([{ alreadyRead: true }, { outer: true }]);
    });
    expect(transaction.activeDepth).toBe(0);
    expect(transaction.isPoisoned).toBe(false);
    expect(caller.currentPrivateState.counter).toBe(8);
    expect(proof.publicTranscript).toEqual([{ alreadyRead: true }, { outer: true }]);
  });

  test('a caught poisoned inner rollback still refuses the outer commit', async () => {
    const caller = context();
    const proof = proofData();
    const transaction = new InternalGeneratedContextTransaction();
    await expect(
      transaction.transact(caller, proof, async () => {
        const fresh = { counter: 8 };
        caller.currentPrivateState = fresh;
        caller.callContext.currentPrivateState = fresh;
        try {
          await transaction.transact(caller, proof, () => {
            fresh.counter = 9;
            Object.freeze(fresh);
            throw new Error('inner');
          });
        } catch (error) {
          expect(error).toBeInstanceOf(InternalGeneratedContextRollbackError);
        }
        return 'must not commit';
      }),
    ).rejects.toThrow(/poisoned/);
    expect(transaction.activeDepth).toBe(0);
    expect(transaction.isPoisoned).toBe(true);
    expect(caller.currentPrivateState.counter).toBe(7);
  });

  test('overlapping async calls refuse out-of-order completion without claiming a commit', async () => {
    const caller = context();
    const proof = proofData();
    const transaction = new InternalGeneratedContextTransaction();
    let releaseFirst!: () => void;
    let releaseSecond!: () => void;
    const firstGate = new Promise<void>((resolve) => {
      releaseFirst = resolve;
    });
    const secondGate = new Promise<void>((resolve) => {
      releaseSecond = resolve;
    });
    const first = transaction.transact(caller, proof, async () => {
      caller.currentPrivateState.counter = 8;
      await firstGate;
      return 'first';
    });
    const second = transaction.transact(caller, proof, async () => {
      caller.currentPrivateState.counter = 9;
      await secondGate;
      return 'second';
    });
    releaseFirst();
    await expect(first).rejects.toBeInstanceOf(InternalGeneratedContextRollbackError);
    releaseSecond();
    await expect(second).rejects.toThrow(/poisoned/);
    expect(transaction.activeDepth).toBe(0);
    expect(transaction.isPoisoned).toBe(true);
  });

  test('restores actual bytes and map/set entries despite poisoned instance iterators and methods', async () => {
    const caller = context();
    const proof = proofData();
    const bytes = caller.currentZswapLocalState.coinPublicKey.bytes;
    bytes[0] = 7;
    Object.defineProperty(bytes, Symbol.iterator, {
      value: function* () {
        yield 255;
      },
      configurable: true,
    });
    const token = 'cc'.repeat(32) as unknown as ocrt.TokenType;
    const added = 'dd'.repeat(32) as unknown as ocrt.TokenType;
    const state = new ocrt.ContractState();
    Object.defineProperty(state, 'balance', { value: state.balance, configurable: true, writable: true });
    caller.contractStates[address] = state;
    Map.prototype.set.call(state.balance, token, 2n);
    Object.defineProperty(state.balance, Symbol.iterator, { value: function* () {}, configurable: true });
    Object.defineProperty(state.balance, 'clear', { value: () => undefined, configurable: true });
    Object.defineProperty(caller.activeContracts, Symbol.iterator, { value: function* () {}, configurable: true });
    Object.defineProperty(caller.activeContracts, 'clear', { value: () => undefined, configurable: true });
    const transaction = new InternalGeneratedContextTransaction();
    await expect(
      transaction.transact(caller, proof, () => {
        bytes[0] = 9;
        Map.prototype.set.call(state.balance, added, 3n);
        Set.prototype.add.call(caller.activeContracts, otherAddress);
        Object.defineProperty(caller.activeContracts, 'add', { value: () => undefined, configurable: true });
        throw new Error('abort');
      }),
    ).rejects.toThrow('abort');
    expect(transaction.isPoisoned).toBe(false);
    expect(bytes[0]).toBe(7);
    expect(Map.prototype.get.call(state.balance, token)).toBe(2n);
    expect(Map.prototype.has.call(state.balance, added)).toBe(false);
    expect(Set.prototype.has.call(caller.activeContracts, address)).toBe(true);
    expect(Set.prototype.has.call(caller.activeContracts, otherAddress)).toBe(false);
    expect(Object.prototype.hasOwnProperty.call(caller.activeContracts, 'add')).toBe(false);
  });

  test('a nonconfigurable added private-state cell makes abort fail closed', async () => {
    const caller = context();
    const proof = proofData();
    const transaction = new InternalGeneratedContextTransaction();
    let thrown: unknown;
    try {
      await transaction.transact(caller, proof, () => {
        Object.defineProperty(caller.currentPrivateState, 'permanent', { value: 1, configurable: false });
        throw new Error('abort');
      });
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(InternalGeneratedContextRollbackError);
    expect((thrown as InternalGeneratedContextRollbackError).rollbackFailures.length).toBeGreaterThan(0);
    expect(transaction.isPoisoned).toBe(true);
    expect(transaction.activeDepth).toBe(0);
  });

  test('an un-restorable frozen alias poisons the journal and refuses reuse', async () => {
    const caller = context();
    const proof = proofData();
    const transaction = new InternalGeneratedContextTransaction();
    const query = caller.callContext.currentQueryContext;
    Object.defineProperty(query, 'block', { value: query.block, configurable: true, writable: true });
    const block = query.block;
    const failure = new Error('callback failed');
    let thrown: unknown;
    try {
      await transaction.transact(caller, proof, () => {
        block.balance.set('cd'.repeat(32) as unknown as ocrt.TokenType, 2n);
        Object.freeze(block);
        throw failure;
      });
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(InternalGeneratedContextRollbackError);
    expect((thrown as InternalGeneratedContextRollbackError).cause).toBe(failure);
    expect((thrown as InternalGeneratedContextRollbackError).rollbackFailures.length).toBeGreaterThan(0);
    expect(transaction.activeDepth).toBe(0);
    expect(transaction.isPoisoned).toBe(true);
    await expect(transaction.transact(caller, proof, () => undefined)).rejects.toThrow(/poisoned/);
  });
});
