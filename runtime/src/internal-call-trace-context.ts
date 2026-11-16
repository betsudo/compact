// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

/**
 * Internal copy-on-write call trace state. This is not the public Runtime 0.18
 * CircuitContext and does not execute a callee, a kernel claim, or a proof.
 * The state/provider preflight has a separate transactional boundary.
 */
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import type { ActiveCall } from './cross-contract-call-state.js';
import { emptyRunningCost } from './circuit-context.js';
import type { LogEvent } from './circuit-context.js';
import { CompactError } from './error.js';
import type { ProofData } from './proof-data.js';
import { emptyZswapLocalState } from './zswap.js';
import type { EncodedZswapLocalState } from './zswap.js';

export interface InternalCallProofData extends ProofData {
  readonly circuitId: string;
  readonly contractAddress: ocrt.ContractAddress;
  readonly initialQueryContext: ocrt.QueryContext;
  readonly finalQueryContext: ocrt.QueryContext;
  readonly zswapLocalState: EncodedZswapLocalState;
  commCommData?: {
    commComm: ocrt.CommunicationCommitment;
    commCommRand: ocrt.CommunicationCommitmentRand;
  };
}

export interface InternalCallFrame<PS = unknown> {
  circuitId: string;
  contractAddress: ocrt.ContractAddress;
  initialQueryContext: ocrt.QueryContext;
  currentQueryContext: ocrt.QueryContext;
  currentGasCost: ocrt.RunningCost;
  currentPrivateState: PS | undefined;
  currentZswapLocalState: EncodedZswapLocalState;
}

export interface InternalCallTraceContext<PS = unknown> {
  readonly parentBlockHash: string;
  readonly time: number;
  callContext: InternalCallFrame<PS>;
  queryContexts: Record<ocrt.ContractAddress, ocrt.QueryContext>;
  gasCosts: Record<ocrt.ContractAddress, ocrt.RunningCost>;
  zswapLocalStates: Record<ocrt.ContractAddress, EncodedZswapLocalState>;
  callProofDataTrace: InternalCallProofData[];
  events: LogEvent[];
}

export interface InternalRootTraceOptions<PS> {
  readonly circuitId: string;
  readonly contractAddress: ocrt.ContractAddress;
  readonly queryContext: ocrt.QueryContext;
  readonly zswapLocalState: EncodedZswapLocalState;
  readonly privateState: PS;
  readonly parentBlockHash: string;
  readonly time: number;
  readonly gasCost?: ocrt.RunningCost;
}

const ownMap = <T>(entries: Record<string, T> = {}): Record<string, T> =>
  Object.assign(Object.create(null) as Record<string, T>, entries);

const assertPinned = (
  queryContext: ocrt.QueryContext,
  address: ocrt.ContractAddress,
  parentBlockHash: string,
  time: number,
): void => {
  if (
    queryContext.address !== address ||
    queryContext.block.ownAddress !== address ||
    queryContext.block.parentBlockHash !== parentBlockHash ||
    queryContext.block.secondsSinceEpoch !== BigInt(time)
  ) {
    throw new CompactError(`Trace query context for '${address}' changed the pinned parent block or call identity`);
  }
};

/** Root seed from the already pinned state model; no provider access here. */
export const createInternalCallTraceContext = <PS>({
  circuitId,
  contractAddress,
  queryContext,
  zswapLocalState,
  privateState,
  parentBlockHash,
  time,
  gasCost = emptyRunningCost(),
}: InternalRootTraceOptions<PS>): InternalCallTraceContext<PS> => {
  assertPinned(queryContext, contractAddress, parentBlockHash, time);
  const frame: InternalCallFrame<PS> = {
    circuitId,
    contractAddress,
    initialQueryContext: queryContext,
    currentQueryContext: queryContext,
    currentGasCost: gasCost,
    currentPrivateState: privateState,
    currentZswapLocalState: zswapLocalState,
  };
  const queryContexts = ownMap<ocrt.QueryContext>();
  const gasCosts = ownMap<ocrt.RunningCost>();
  const zswapLocalStates = ownMap<EncodedZswapLocalState>();
  queryContexts[contractAddress] = queryContext;
  gasCosts[contractAddress] = gasCost;
  zswapLocalStates[contractAddress] = zswapLocalState;
  return {
    parentBlockHash,
    time,
    callContext: frame,
    queryContexts,
    gasCosts,
    zswapLocalStates,
    callProofDataTrace: [],
    events: [],
  };
};

/** Like pinned copyCircuitContext: clone containers and frame; callers replace shared cells on advancement. */
export const copyInternalCallTraceContext = <PS>(context: InternalCallTraceContext<PS>): InternalCallTraceContext<PS> => ({
  ...context,
  callContext: { ...context.callContext },
  queryContexts: ownMap(context.queryContexts),
  gasCosts: ownMap(context.gasCosts),
  zswapLocalStates: ownMap(context.zswapLocalStates),
  callProofDataTrace: [...context.callProofDataTrace],
  events: [...context.events],
});

/** Enter a preflighted callee on a child copy, leaving the caller copy intact. */
export const enterInternalCallTraceContext = (caller: InternalCallTraceContext, call: ActiveCall): InternalCallTraceContext => {
  if (call.callerAddress !== caller.callContext.contractAddress) {
    throw new CompactError('Trace callee does not belong to the current caller');
  }
  assertPinned(call.queryContext, call.calleeAddress, caller.parentBlockHash, caller.time);
  const child = copyInternalCallTraceContext(caller);
  const zswap =
    child.zswapLocalStates[call.calleeAddress] ?? emptyZswapLocalState(caller.callContext.currentZswapLocalState.coinPublicKey);
  const gasCost = { ...call.gasCost };
  child.callContext = {
    circuitId: call.calleeCircuitId,
    contractAddress: call.calleeAddress,
    initialQueryContext: call.queryContext,
    currentQueryContext: call.queryContext,
    currentGasCost: gasCost,
    currentPrivateState: undefined,
    currentZswapLocalState: zswap,
  };
  child.queryContexts[call.calleeAddress] = call.queryContext;
  child.gasCosts[call.calleeAddress] = gasCost;
  child.zswapLocalStates[call.calleeAddress] = zswap;
  return child;
};

/** Record the VM-returned next cells; input objects are treated as values. */
export const advanceInternalCallTraceContext = (
  context: InternalCallTraceContext,
  queryContext: ocrt.QueryContext,
  gasCost: ocrt.RunningCost,
  zswapLocalState: EncodedZswapLocalState,
): InternalCallTraceContext => {
  const address = context.callContext.contractAddress;
  assertPinned(queryContext, address, context.parentBlockHash, context.time);
  const next = copyInternalCallTraceContext(context);
  const nextGasCost = { ...gasCost };
  next.callContext.currentQueryContext = queryContext;
  next.callContext.currentGasCost = nextGasCost;
  next.callContext.currentZswapLocalState = zswapLocalState;
  next.queryContexts[address] = queryContext;
  next.gasCosts[address] = nextGasCost;
  next.zswapLocalStates[address] = zswapLocalState;
  return next;
};

/** Append one completed call, after the generated circuit has produced proof data. */
export const finalizeInternalCallProofData = (
  context: InternalCallTraceContext,
  proofData: ProofData,
): InternalCallTraceContext => {
  const next = copyInternalCallTraceContext(context);
  const { circuitId, contractAddress, initialQueryContext, currentQueryContext, currentZswapLocalState } = next.callContext;
  assertPinned(initialQueryContext, contractAddress, next.parentBlockHash, next.time);
  assertPinned(currentQueryContext, contractAddress, next.parentBlockHash, next.time);
  if (
    next.queryContexts[contractAddress] !== currentQueryContext ||
    next.zswapLocalStates[contractAddress] !== currentZswapLocalState
  ) {
    throw new CompactError('Trace live call cells differ from their per-address cells');
  }
  next.callProofDataTrace.push({
    ...proofData,
    circuitId,
    contractAddress,
    initialQueryContext,
    finalQueryContext: currentQueryContext,
    zswapLocalState: currentZswapLocalState,
  });
  return next;
};

/** Commit a completed child; discarding it instead leaves every caller container untouched. */
export const restoreInternalCallerTraceContext = (
  caller: InternalCallTraceContext,
  child: InternalCallTraceContext,
  call: ActiveCall,
): InternalCallTraceContext => {
  if (
    call.callerAddress !== caller.callContext.contractAddress ||
    call.calleeAddress !== child.callContext.contractAddress ||
    call.calleeCircuitId !== child.callContext.circuitId ||
    caller.parentBlockHash !== child.parentBlockHash ||
    caller.time !== child.time
  ) {
    throw new CompactError('Trace restoration does not match the completed call');
  }
  const last = child.callProofDataTrace[child.callProofDataTrace.length - 1];
  if (
    child.callProofDataTrace.length <= caller.callProofDataTrace.length ||
    last?.contractAddress !== call.calleeAddress ||
    last?.circuitId !== call.calleeCircuitId
  ) {
    throw new CompactError('Trace restoration requires a finalized callee entry');
  }
  if (caller.callProofDataTrace.some((entry, index) => child.callProofDataTrace[index] !== entry)) {
    throw new CompactError('Callee changed an earlier call proof entry');
  }
  const callerAddress = caller.callContext.contractAddress;
  if (
    child.queryContexts[callerAddress] !== caller.callContext.currentQueryContext ||
    child.gasCosts[callerAddress] !== caller.callContext.currentGasCost ||
    child.zswapLocalStates[callerAddress] !== caller.callContext.currentZswapLocalState
  ) {
    throw new CompactError('Callee changed the suspended caller cells');
  }
  const restored = copyInternalCallTraceContext(caller);
  restored.queryContexts = ownMap(child.queryContexts);
  restored.gasCosts = ownMap(child.gasCosts);
  restored.zswapLocalStates = ownMap(child.zswapLocalStates);
  restored.callProofDataTrace = [...child.callProofDataTrace];
  restored.events = [...child.events];
  return restored;
};
