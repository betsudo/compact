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
import {
  copyCircuitContext,
  createCircuitContext,
  createZswapInput,
  createZswapOutput,
  emptyRunningCost,
  finalizeCallProofData,
  ownPublicKey,
  queryLedgerState,
} from '../src/index.js';

const address = ocrt.sampleContractAddress();
const parentBlockHash = 'ab'.repeat(32);
const options = () => ({
  circuitId: 'transfer',
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
  publicTranscript: [],
  privateTranscriptOutputs: [],
});

describe('Ledger 9 public context cells', () => {
  test('pins the object-form root and aliases each initial per-address cell', () => {
    const inputs = options();
    const context = createCircuitContext(inputs);
    expect(context.callContext.circuitId).toBe('transfer');
    expect(context.callContext.contractAddress).toBe(address);
    expect(context.callContext.initialQueryContext).toBe(context.callContext.currentQueryContext);
    expect(context.queryContexts[address]).toBe(context.callContext.currentQueryContext);
    expect(context.gasCosts[address]).toBe(context.callContext.currentGasCost);
    expect(context.zswapLocalStates[address]).toBe(context.callContext.currentZswapLocalState);
    expect(context.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(context.callContext.currentPrivateState).toBe(inputs.privateState);
    expect(context.callContext.currentQueryContext.block.parentBlockHash).toBe(parentBlockHash);
    expect(context.callContext.currentQueryContext.block.secondsSinceEpoch).toBe(123n);
    expect(context.stateProvider).toBe(inputs.crossContract.stateProvider);
    expect(context.moduleProvider).toBe(inputs.crossContract.moduleProvider);
    expect(context.activeContracts).toEqual(new Set([address]));
    expect(context.callProofDataTrace).toEqual([]);
  });

  test('copies containers and leaves earlier query, gas and event cells intact', () => {
    const original = createCircuitContext(options());
    const next = copyCircuitContext(original);
    expect(next.activeContracts).toBe(original.activeContracts);
    expect(next.callContext).not.toBe(original.callContext);
    expect(next.queryContexts).not.toBe(original.queryContexts);
    expect(next.gasCosts).not.toBe(original.gasCosts);
    expect(next.zswapLocalStates).not.toBe(original.zswapLocalStates);
    const query = original.callContext.currentQueryContext;
    const result = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    result.block = { ...query.block };
    vi.spyOn(query, 'query').mockReturnValue({
      context: result,
      gasCost: { readTime: 1n, computeTime: 2n, bytesWritten: 3n, bytesDeleted: 4n },
      events: [],
    } as ReturnType<typeof query.query>);
    const partial = proofData();
    expect(queryLedgerState(next, partial, [])).toEqual([]);
    expect(next.callContext.currentQueryContext).toBe(result);
    expect(next.queryContexts[address]).toBe(result);
    expect(next.callContext.currentGasCost).toBe(next.gasCosts[address]);
    expect(next.callContext.currentGasCost.readTime).toBe(1n);
    expect(original.callContext.currentQueryContext).toBe(query);
    expect(original.queryContexts[address]).toBe(query);
    expect(original.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(original.events).toEqual([]);
  });

  test('writes input and output coins through per-address cells without changing a copied parent', () => {
    const original = createCircuitContext(options());
    const copied = copyCircuitContext(original);
    const coin = {
      nonce: new Uint8Array(32),
      color: new Uint8Array(32),
      value: 5n,
    };
    createZswapInput(copied, { ...coin, mt_index: 0n });
    expect(copied.zswapLocalStates[address]).toBe(copied.callContext.currentZswapLocalState);
    expect(copied.callContext.currentZswapLocalState.inputs).toHaveLength(1);
    expect(original.callContext.currentZswapLocalState.inputs).toHaveLength(0);
    const recipient = {
      is_left: false,
      left: { bytes: new Uint8Array(32) },
      right: { bytes: ocrt.encodeContractAddress(ocrt.sampleContractAddress()) },
    };
    createZswapOutput(copied, coin, recipient);
    expect(copied.queryContexts[address]).toBe(copied.callContext.currentQueryContext);
    expect(copied.zswapLocalStates[address]).toBe(copied.callContext.currentZswapLocalState);
    expect(copied.callContext.currentZswapLocalState.outputs).toHaveLength(1);
    expect(copied.callContext.currentZswapLocalState.currentIndex).toBe(1n);
    expect(original.callContext.currentZswapLocalState.outputs).toHaveLength(0);
    expect(original.callContext.currentQueryContext.comIndices.size).toBe(0);
    const addressByte = copied.callContext.currentZswapLocalState.outputs[0].recipient.right.bytes[0];
    coin.nonce[0] = 1;
    recipient.right.bytes[0] ^= 1;
    expect(copied.callContext.currentZswapLocalState.inputs[0].nonce[0]).toBe(0);
    expect(copied.callContext.currentZswapLocalState.outputs[0].coinInfo.nonce[0]).toBe(0);
    expect(copied.callContext.currentZswapLocalState.outputs[0].recipient.right.bytes[0]).toBe(addressByte);
  });

  test('owns encoded initial bytes and returns a defensive public key copy', () => {
    const encoded = {
      coinPublicKey: { bytes: new Uint8Array(32) },
      currentIndex: 0n,
      inputs: [],
      outputs: [],
    };
    const context = createCircuitContext({ ...options(), coinPublicKeyOrZswapState: encoded });
    encoded.coinPublicKey.bytes[0] = 1;
    expect(context.callContext.currentZswapLocalState.coinPublicKey.bytes[0]).toBe(0);
    const key = ownPublicKey(context);
    key.bytes[0] = 2;
    expect(context.callContext.currentZswapLocalState.coinPublicKey.bytes[0]).toBe(0);
  });

  test('finalizes a copy once its live cells match, and refuses stale or legacy frames', () => {
    const root = createCircuitContext(options());
    const completed = copyCircuitContext(root);
    finalizeCallProofData(completed, proofData());
    expect(completed.callProofDataTrace).toHaveLength(1);
    expect(completed.callProofDataTrace[0].contractAddress).toBe(address);
    expect(completed.callProofDataTrace[0].initialQueryContext).toBe(root.callContext.currentQueryContext);
    expect(root.callProofDataTrace).toEqual([]);
    const stale = copyCircuitContext(root);
    stale.queryContexts[address] = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    expect(() => finalizeCallProofData(stale, proofData())).toThrow(/live call cells are incomplete or stale/);
    const legacy = createCircuitContext(address, '0'.repeat(64), new ocrt.ContractState(), {});
    expect(() => finalizeCallProofData(legacy, proofData())).toThrow(/requires an object-form context/);
  });

  test('refuses a VM query that changes the parent-block identity without writing cells', () => {
    const context = createCircuitContext(options());
    const query = context.callContext.currentQueryContext;
    const changed = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    changed.block = { ...query.block, parentBlockHash: 'cd'.repeat(32) };
    vi.spyOn(query, 'query').mockReturnValue({
      context: changed,
      gasCost: emptyRunningCost(),
      events: [],
    } as ReturnType<typeof query.query>);
    const partial = proofData();
    expect(() => queryLedgerState(context, partial, [])).toThrow(/changed the pinned block or call identity/);
    expect(context.callContext.currentQueryContext).toBe(query);
    expect(context.queryContexts[address]).toBe(query);
    expect(partial.publicTranscript).toEqual([]);
  });

  test('refuses a cleared visible parent hash in query, coin write and finalization', () => {
    const context = createCircuitContext(options());
    const query = vi.spyOn(context.callContext.currentQueryContext, 'query');
    context.callContext = { ...context.callContext, parentBlockHash: undefined };
    expect(() => queryLedgerState(context, proofData(), [])).toThrow(/changed its pinned provenance/);
    expect(query).not.toHaveBeenCalled();
    const coin = { nonce: new Uint8Array(32), color: new Uint8Array(32), value: 5n };
    const recipient = {
      is_left: false,
      left: { bytes: new Uint8Array(32) },
      right: { bytes: ocrt.encodeContractAddress(ocrt.sampleContractAddress()) },
    };
    expect(() => createZswapOutput(context, coin, recipient)).toThrow(/changed its pinned provenance/);
    expect(() => finalizeCallProofData(context, proofData())).toThrow(/changed its pinned provenance/);
    expect(context.callProofDataTrace).toEqual([]);
  });

  test('refuses a blank circuit ID in query, coin input and finalization', () => {
    const context = createCircuitContext(options());
    const query = vi.spyOn(context.callContext.currentQueryContext, 'query');
    context.callContext = { ...context.callContext, circuitId: '' };
    expect(() => queryLedgerState(context, proofData(), [])).toThrow(/changed its pinned provenance/);
    expect(query).not.toHaveBeenCalled();
    expect(() =>
      createZswapInput(context, {
        nonce: new Uint8Array(32),
        color: new Uint8Array(32),
        value: 5n,
        mt_index: 0n,
      }),
    ).toThrow(/changed its pinned provenance/);
    expect(() => finalizeCallProofData(context, proofData())).toThrow(/changed its pinned provenance/);
    expect(context.callContext.currentZswapLocalState.inputs).toEqual([]);
  });

  test('carries a private pin through shallow spreads and rejects a stripped token', () => {
    const root = createCircuitContext(options());
    const shallow = { ...root, callContext: { ...root.callContext, circuitId: '' } };
    expect(() => queryLedgerState(shallow, proofData(), [])).toThrow(/changed its pinned provenance/);
    const stripped = { ...root };
    for (const symbol of Object.getOwnPropertySymbols(stripped)) {
      delete (stripped as unknown as Record<symbol, unknown>)[symbol];
    }
    expect(() => queryLedgerState(stripped, proofData(), [])).toThrow(/provenance is missing or changed/);
  });

  test('retains positional Runtime 0.18 calls through shallow spreads', () => {
    const legacy = createCircuitContext(address, '0'.repeat(64), new ocrt.ContractState(), {});
    const shallow = { ...legacy };
    const query = shallow.currentQueryContext;
    const result = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    vi.spyOn(query, 'query').mockReturnValue({
      context: result,
      gasCost: emptyRunningCost(),
      events: [],
    } as ReturnType<typeof query.query>);
    expect(queryLedgerState(shallow, proofData(), [])).toEqual([]);
    expect(shallow.currentQueryContext).toBe(result);
    expect(legacy.currentQueryContext).toBe(query);
  });

  test('does not publish query cells when a VM read result is missing', () => {
    const context = createCircuitContext(options());
    const query = context.callContext.currentQueryContext;
    const next = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    next.block = { ...query.block };
    vi.spyOn(query, 'query').mockReturnValue({
      context: next,
      gasCost: { readTime: 1n, computeTime: 0n, bytesWritten: 0n, bytesDeleted: 0n },
      events: [],
    } as ReturnType<typeof query.query>);
    const partial = proofData();
    const malformedRead = [{ popeq: {} }] as unknown as ocrt.Op<null>[];
    expect(() => queryLedgerState(context, partial, malformedRead)).toThrow();
    expect(context.callContext.currentQueryContext).toBe(query);
    expect(context.queryContexts[address]).toBe(query);
    expect(context.callContext.currentGasCost).toEqual(emptyRunningCost());
    expect(partial.publicTranscript).toEqual([]);
  });

  test('refuses a coin write that changes the parent block before publishing cells', () => {
    const context = createCircuitContext(options());
    const query = context.callContext.currentQueryContext;
    const changed = new ocrt.QueryContext(new ocrt.ContractState().data, address);
    changed.block = { ...query.block, parentBlockHash: 'cd'.repeat(32) };
    vi.spyOn(query, 'insertCommitment').mockReturnValue(changed);
    const coin = { nonce: new Uint8Array(32), color: new Uint8Array(32), value: 5n };
    const recipient = {
      is_left: false,
      left: { bytes: new Uint8Array(32) },
      right: { bytes: ocrt.encodeContractAddress(ocrt.sampleContractAddress()) },
    };
    expect(() => createZswapOutput(context, coin, recipient)).toThrow(/changed the pinned block or call identity/);
    expect(context.callContext.currentQueryContext).toBe(query);
    expect(context.queryContexts[address]).toBe(query);
    expect(context.callContext.currentZswapLocalState.outputs).toEqual([]);
  });

  test('refuses sparse encoded coin lists rather than copying an incomplete entry', () => {
    const inputs = options();
    const malformed = {
      coinPublicKey: { bytes: new Uint8Array(32) },
      currentIndex: 0n,
      inputs: new Array(1),
      outputs: [],
    };
    expect(() => createCircuitContext({ ...inputs, coinPublicKeyOrZswapState: malformed })).toThrow(
      /Encoded Zswap 0 must be an own data field/,
    );
  });

  test('refuses unpinned provider forms and invalid block time before exposing a context', () => {
    const inputs = options();
    expect(() => createCircuitContext({ ...inputs, parentBlockHash: undefined })).toThrow(
      /providers require a pinned parent block/,
    );
    expect(() => createCircuitContext({ ...inputs, crossContract: null as never })).toThrow(
      /providers require a pinned parent block/,
    );
    expect(() =>
      createCircuitContext({ ...inputs, crossContract: { ...inputs.crossContract, moduleProvider: {} as never } }),
    ).toThrow(/providers require a pinned parent block/);
    expect(() => createCircuitContext({ ...inputs, time: 0.5 })).toThrow(/block time must be a non-negative integer/);
  });
});
