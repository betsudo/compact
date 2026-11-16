// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, test, vi } from 'vitest';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import {
  applyVerifiedCallProofBinding,
  kernelClaimContractCallProgram,
  prepareCallProofBinding,
} from '../src/cross-contract-proof-binding.js';
import { createCircuitContext, emptyRunningCost, queryLedgerState, type CallProofData } from '../src/circuit-context.js';
import { InternalGeneratedContextTransaction } from '../src/internal-generated-context-transaction.js';
import * as publicRuntime from '../src/index.js';

const A = `0200${'11'.repeat(30)}`;
const B = `0200${'22'.repeat(30)}`;
const BLOCK = 'ab'.repeat(32);

const aligned = (byte: number): ocrt.AlignedValue => ({
  value: [Uint8Array.of(byte)],
  alignment: [{ tag: 'atom', value: { tag: 'field' } }],
});

const setup = () => {
  const state = new ocrt.ContractState();
  const caller = createCircuitContext({
    circuitId: 'caller',
    contractAddress: A,
    coinPublicKeyOrZswapState: '0'.repeat(64),
    contractState: state,
    privateState: {},
    parentBlockHash: BLOCK,
    time: 42,
  });
  const callerProof = {
    input: aligned(1),
    publicTranscript: [] as ocrt.Op<ocrt.AlignedValue>[],
    privateTranscriptOutputs: [] as ocrt.AlignedValue[],
  };
  const initial = caller.callContext.currentQueryContext;
  const afterRead = new ocrt.QueryContext(state.data, A);
  afterRead.block = { ...initial.block };
  vi.spyOn(initial, 'query').mockReturnValue({
    context: afterRead,
    gasCost: { ...emptyRunningCost(), readTime: 1n },
    events: [],
  } as ReturnType<typeof initial.query>);
  queryLedgerState(caller, callerProof, [{ dup: { n: 0 } }]);
  const calleeQuery = new ocrt.QueryContext(state.data, B);
  calleeQuery.block = {
    ...calleeQuery.block,
    ownAddress: B,
    parentBlockHash: BLOCK,
    secondsSinceEpoch: 42n,
    caller: { tag: 'contract', address: A },
  };
  const calleeProof: CallProofData = {
    circuitId: 'transfer',
    contractAddress: B,
    input: aligned(2),
    output: aligned(3),
    publicTranscript: [],
    privateTranscriptOutputs: [],
    initialQueryContext: calleeQuery,
    finalQueryContext: calleeQuery,
    zswapLocalState: caller.currentZswapLocalState,
  };
  caller.callProofDataTrace.push(calleeProof);
  const binding = prepareCallProofBinding(B, 'transfer', calleeProof.input, calleeProof.output);
  return { caller, callerProof, calleeProof, afterRead, state, binding };
};

describe('internal real Runtime Kernel claim commit', () => {
  test('appends after the genuine pre-call query and runs the exact claim program', async () => {
    const { caller, callerProof, calleeProof, afterRead, state, binding } = setup();
    const afterClaim = new ocrt.QueryContext(state.data, A);
    afterClaim.block = { ...afterRead.block };
    const claim = vi.spyOn(afterRead, 'query').mockReturnValue({
      context: afterClaim,
      gasCost: { ...emptyRunningCost(), computeTime: 3n },
      events: [],
    } as ReturnType<typeof afterRead.query>);
    const journal = new InternalGeneratedContextTransaction();
    await journal.transact(caller, callerProof, () =>
      applyVerifiedCallProofBinding(journal, caller, callerProof, calleeProof, binding),
    );

    expect(claim).toHaveBeenCalledExactlyOnceWith(
      kernelClaimContractCallProgram(B, 'transfer', binding.commCommData.commComm),
      caller.costModel,
      caller.gasLimit,
    );
    expect(callerProof.publicTranscript[0]).toEqual({ dup: { n: 0 } });
    expect(callerProof.publicTranscript.slice(1)).toEqual(binding.claimProgram);
    expect(callerProof.privateTranscriptOutputs).toEqual(binding.privateTranscriptOutputs);
    expect(calleeProof.commCommData).toEqual(binding.commCommData);
    expect(caller.callContext.currentQueryContext).toBe(afterClaim);
    expect(caller.gasCosts[A].computeTime).toBe(3n);
    expect('applyVerifiedCallProofBinding' in publicRuntime).toBe(false);
    expect('crossContractCall' in publicRuntime).toBe(false);
  });

  test('VM failure restores claim and randomness while retaining the pre-call read', async () => {
    const { caller, callerProof, calleeProof, afterRead, binding } = setup();
    vi.spyOn(afterRead, 'query').mockImplementation(() => {
      throw new Error('Kernel rejected the claim');
    });
    const journal = new InternalGeneratedContextTransaction();
    await expect(
      journal.transact(caller, callerProof, () =>
        applyVerifiedCallProofBinding(journal, caller, callerProof, calleeProof, binding),
      ),
    ).rejects.toThrow(/Kernel rejected the claim/);
    expect(callerProof.publicTranscript).toEqual([{ dup: { n: 0 } }]);
    expect(callerProof.privateTranscriptOutputs).toEqual([]);
    expect(calleeProof.commCommData).toBeUndefined();
    expect(caller.callContext.currentQueryContext).toBe(afterRead);
    expect(journal.isPoisoned).toBe(false);
  });

  test('an unjournaled call, decoy, or changed callee bytes refuse before a Kernel query', async () => {
    const { caller, callerProof, calleeProof, afterRead } = setup();
    const query = vi.spyOn(afterRead, 'query');
    const decoy = prepareCallProofBinding(B, 'decoy', calleeProof.input, calleeProof.output);
    const binding = prepareCallProofBinding(B, 'transfer', calleeProof.input, calleeProof.output);
    const journal = new InternalGeneratedContextTransaction();
    expect(() => applyVerifiedCallProofBinding(journal, caller, callerProof, calleeProof, binding)).toThrow(
      /requires its active transaction journal/,
    );
    await expect(
      journal.transact(caller, callerProof, () =>
        applyVerifiedCallProofBinding(journal, caller, callerProof, calleeProof, decoy),
      ),
    ).rejects.toThrow(/differs from the sealed/);
    calleeProof.output.value[0][0] ^= 1;
    await expect(
      journal.transact(caller, callerProof, () =>
        applyVerifiedCallProofBinding(journal, caller, callerProof, calleeProof, binding),
      ),
    ).rejects.toThrow(/differs from the sealed/);
    expect(query).not.toHaveBeenCalled();
    expect(callerProof.publicTranscript).toEqual([{ dup: { n: 0 } }]);
    expect(callerProof.privateTranscriptOutputs).toEqual([]);
  });
});
