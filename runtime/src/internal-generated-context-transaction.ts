// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0

/**
 * Internal transaction journal for generated Ledger 9 circuit cells. The
 * journal begins after the caller's already-recorded pre-call queries. It is
 * not the public cross-contract coordinator and never invokes a provider.
 */

import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { assertLiveCircuitCall, type CircuitContext } from './circuit-context.js';
import { snapshotInternalMutable } from './cross-contract-call-state.js';
import { CompactError } from './error.js';
import type { PartialProofData } from './proof-data.js';

type Restore = () => void;

type JournalFrame = {
  readonly context: CircuitContext;
  readonly proofData: PartialProofData;
  readonly restores: readonly Restore[];
};

export class InternalGeneratedContextRollbackError extends CompactError {
  readonly cause: unknown;
  readonly rollbackFailures: readonly unknown[];

  constructor(cause: unknown, rollbackFailures: readonly unknown[]) {
    super(`Generated circuit rollback failed to restore ${rollbackFailures.length} cell(s); transaction is poisoned`);
    this.cause = cause;
    this.rollbackFailures = Object.freeze([...rollbackFailures]);
  }
}

/** Restore own cells without trusting prototype setters added during a call. */
const snapshotOwn = (value: object): Restore => {
  if (Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null && !Array.isArray(value)) {
    throw new CompactError('Cannot snapshot a non-plain generated context container');
  }
  const descriptors = Object.getOwnPropertyDescriptors(value);
  for (const descriptor of Object.values(descriptors)) {
    if (!('value' in descriptor)) throw new CompactError('Cannot snapshot an accessor generated context container');
  }
  return () => {
    for (const key of Reflect.ownKeys(value)) {
      if (!Object.prototype.hasOwnProperty.call(descriptors, key) && !Reflect.deleteProperty(value, key)) {
        throw new CompactError('Could not remove an added generated context cell');
      }
    }
    if (Array.isArray(value)) value.length = descriptors.length.value as number;
    for (const key of Reflect.ownKeys(descriptors)) {
      if (key !== 'length') Object.defineProperty(value, key, descriptors[key as keyof typeof descriptors]);
    }
  };
};

const captureQuery = (query: ocrt.QueryContext, restores: Restore[]): void => {
  const block = query.block;
  const effects = query.effects;
  const commitmentIndices = query.comIndices;
  const ownCommitmentIndices = Object.getOwnPropertyDescriptor(query, 'comIndices');
  restores.push(snapshotInternalMutable(block));
  restores.push(snapshotInternalMutable(effects));
  restores.push(snapshotInternalMutable(commitmentIndices));
  restores.push(() => {
    query.block = block;
    query.effects = effects;
    if (ownCommitmentIndices) {
      Object.defineProperty(query, 'comIndices', ownCommitmentIndices);
    } else if (Object.prototype.hasOwnProperty.call(query, 'comIndices') && !Reflect.deleteProperty(query, 'comIndices')) {
      throw new CompactError('Could not restore generated commitment indices');
    }
    if (ownCommitmentIndices && 'value' in ownCommitmentIndices && query.comIndices !== commitmentIndices) {
      throw new CompactError('Generated commitment-index map did not restore');
    }
  });
};

const captureContractState = (state: ocrt.ContractState, restores: Restore[]): void => {
  const balance = state.balance;
  const data = state.data;
  const maintenanceAuthority = state.maintenanceAuthority;
  restores.push(snapshotInternalMutable(balance));
  restores.push(() => {
    state.balance = balance;
    state.data = data;
    state.maintenanceAuthority = maintenanceAuthority;
  });
};

/** Capture each original alias once; container cells are rebound after values. */
const capture = (context: CircuitContext, proofData: PartialProofData): JournalFrame => {
  assertLiveCircuitCall(context);
  const restores: Restore[] = [];
  const queries = new Set<ocrt.QueryContext>([
    context.callContext.initialQueryContext,
    context.callContext.currentQueryContext,
    context.currentQueryContext,
    ...Object.values(context.queryContexts),
    ...context.callProofDataTrace.flatMap((call) => [call.initialQueryContext, call.finalQueryContext]),
  ]);
  for (const query of queries) captureQuery(query, restores);

  const costs = new Set<ocrt.RunningCost>([context.callContext.currentGasCost, ...Object.values(context.gasCosts)]);
  for (const cost of costs) restores.push(snapshotInternalMutable(cost));
  restores.push(snapshotInternalMutable(context.gasLimit));

  const zswapStates = new Set([
    context.callContext.currentZswapLocalState,
    context.currentZswapLocalState,
    ...Object.values(context.zswapLocalStates),
    ...context.callProofDataTrace.map((call) => call.zswapLocalState),
  ]);
  for (const state of zswapStates) restores.push(snapshotInternalMutable(state));

  const privateStates = new Set([context.callContext.currentPrivateState, context.currentPrivateState]);
  for (const state of privateStates) restores.push(snapshotInternalMutable(state));

  for (const state of new Set(Object.values(context.contractStates))) captureContractState(state, restores);
  restores.push(snapshotInternalMutable(context.activeContracts));
  restores.push(snapshotInternalMutable(context.events));
  restores.push(snapshotInternalMutable(proofData.input));
  restores.push(snapshotInternalMutable(proofData.publicTranscript));
  restores.push(snapshotInternalMutable(proofData.privateTranscriptOutputs));

  for (const call of context.callProofDataTrace) {
    restores.push(snapshotInternalMutable(call.input));
    restores.push(snapshotInternalMutable(call.output));
    restores.push(snapshotInternalMutable(call.publicTranscript));
    restores.push(snapshotInternalMutable(call.privateTranscriptOutputs));
    if (call.commCommData !== undefined) restores.push(snapshotInternalMutable(call.commCommData));
    restores.push(snapshotOwn(call));
  }
  restores.push(snapshotOwn(context.callProofDataTrace));
  restores.push(snapshotOwn(context.callContext));
  restores.push(snapshotOwn(context.queryContexts));
  restores.push(snapshotOwn(context.gasCosts));
  restores.push(snapshotOwn(context.zswapLocalStates));
  restores.push(snapshotOwn(context.contractStates));
  restores.push(snapshotOwn(proofData));
  restores.push(snapshotOwn(context));
  return { context, proofData, restores };
};

/**
 * Owns only generated in-memory cells. The opaque cost model and provider
 * operations are external inputs, and native VM internals, call-state, and
 * actual Kernel claims require the later public coordinator.
 */
export class InternalGeneratedContextTransaction {
  readonly #frames: JournalFrame[] = [];
  #poisoned: CompactError | undefined;

  get activeDepth(): number {
    return this.#frames.length;
  }

  get isPoisoned(): boolean {
    return this.#poisoned !== undefined;
  }

  /** Refuse an internal commit unless this exact context and proof are journaled now. */
  withActiveFrame<T>(context: CircuitContext, proofData: PartialProofData, commit: () => T): T {
    if (this.#poisoned) throw this.#poisoned;
    const frame = this.#frames[this.#frames.length - 1];
    if (frame?.context !== context || frame.proofData !== proofData) {
      throw new CompactError('Generated context commit requires its active transaction journal');
    }
    return commit();
  }

  async transact<T>(context: CircuitContext, proofData: PartialProofData, callback: () => T | Promise<T>): Promise<T> {
    if (this.#poisoned) throw this.#poisoned;
    const frame = capture(context, proofData);
    this.#frames.push(frame);
    try {
      const result = await callback();
      if (this.#poisoned) throw this.#poisoned;
      if (this.#frames[this.#frames.length - 1] !== frame) {
        throw new CompactError('Generated context transactions completed out of order');
      }
      assertLiveCircuitCall(context);
      this.#frames.pop();
      return result;
    } catch (cause) {
      if (this.#frames[this.#frames.length - 1] !== frame) {
        // The overlapping transaction still owns the live cells. Restoring
        // this older snapshot now would erase its writes, so refuse reuse.
        this.#poisoned = new CompactError('Generated context transactions overlapped; transaction is poisoned');
        const index = this.#frames.lastIndexOf(frame);
        if (index >= 0) this.#frames.splice(index, 1);
        throw new InternalGeneratedContextRollbackError(cause, [this.#poisoned]);
      }
      const failures: unknown[] = [];
      try {
        for (const restore of frame.restores) {
          try {
            restore();
          } catch (error) {
            failures.push(error);
          }
        }
        try {
          assertLiveCircuitCall(context);
        } catch (error) {
          failures.push(error);
        }
      } finally {
        this.#frames.pop();
      }
      if (failures.length) {
        this.#poisoned = new CompactError('Generated circuit rollback failed; transaction is poisoned');
        throw new InternalGeneratedContextRollbackError(cause, failures);
      }
      throw cause;
    }
  }
}
