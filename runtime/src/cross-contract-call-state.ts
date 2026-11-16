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

/**
 * Internal call-state preparation for Ledger 9. The root context and the
 * address maps live here until the full generated invocation path can use
 * them. This module is deliberately absent from the Runtime 0.18 index.
 */

import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { emptyRunningCost } from './circuit-context.js';
import { CompactError } from './error.js';
import { InterfaceDescriptor } from './interface-descriptor.js';
import { Module } from './module.js';
import { ContractModuleProvider, ContractStateProvider } from './providers.js';
import { preflightCrossContractCall } from './cross-contract-preflight.js';
import { isContractAddress } from './utils.js';

export type ParentBlockSnapshot = Readonly<{
  parentBlockHash: string;
  time: number;
}>;

export type InternalCallStateOptions = {
  readonly entryAddress: ocrt.ContractAddress;
  readonly entryState: ocrt.ContractState;
  readonly parentBlockHash: string;
  readonly time: number;
  readonly stateProvider: ContractStateProvider;
  readonly moduleProvider: ContractModuleProvider;
};

export type InternalCallRequest = {
  readonly calleeAddress: ocrt.ContractAddress;
  readonly calleeCircuitId: string;
  readonly interfaceName: string;
  readonly declaration: InterfaceDescriptor;
};

export type ActiveCall = Readonly<{
  callerAddress: ocrt.ContractAddress;
  calleeAddress: ocrt.ContractAddress;
  calleeCircuitId: string;
  deployedState: ocrt.ContractState;
  module: Module;
  readonly queryContext: ocrt.QueryContext;
  readonly gasCost: ocrt.RunningCost;
  end(): void;
  /** Discard all state-class writes made since this call entered. */
  abort(): void;
}>;

/** The state callback cannot finish the handle or mutate provider-owned objects. */
export type ActiveCallView = Pick<ActiveCall, 'callerAddress' | 'calleeAddress' | 'calleeCircuitId' | 'queryContext' | 'gasCost'>;

/** Preserves the callback failure and every failed rollback while refusing reuse. */
export class InternalCallTransactionRollbackError extends CompactError {
  readonly cause: unknown;
  readonly rollbackFailures: readonly unknown[];

  constructor(cause: unknown, rollbackFailures: readonly unknown[]) {
    super(`Cross-contract transaction failed and rollback failed in ${rollbackFailures.length} frame(s); call state is poisoned`);
    this.cause = cause;
    this.rollbackFailures = Object.freeze([...rollbackFailures]);
  }
}

type RollbackSnapshot = {
  readonly queryContexts: Map<ocrt.ContractAddress, ocrt.QueryContext>;
  readonly gasCosts: Map<ocrt.ContractAddress, ocrt.RunningCost>;
  readonly deployedStates: Map<ocrt.ContractAddress, ocrt.ContractState>;
  /** `begin` changes these fields on a cached QueryContext in place. */
  readonly queryFields: ReadonlyArray<{
    readonly context: ocrt.QueryContext;
    readonly block: ocrt.CallContext;
    readonly effects: ocrt.Effects;
    readonly commitmentIndices: ocrt.QueryContext['comIndices'];
    readonly ownCommitmentIndices: PropertyDescriptor | undefined;
    readonly restoreBlock: () => void;
    readonly restoreEffects: () => void;
    readonly restoreCommitmentIndices: () => void;
  }>;
  readonly gasFields: ReadonlyArray<() => void>;
  readonly stateFields: ReadonlyArray<{
    readonly state: ocrt.ContractState;
    readonly balance: ocrt.ContractState['balance'];
    readonly data: ocrt.ContractState['data'];
    readonly maintenanceAuthority: ocrt.ContractState['maintenanceAuthority'];
    readonly restoreBalance: () => void;
  }>;
};

type Frame = {
  readonly address: ocrt.ContractAddress;
  readonly call?: ActiveCall;
  readonly rollback?: RollbackSnapshot;
};

const emptyEffects = (): ocrt.Effects => ({
  claimedNullifiers: [],
  claimedShieldedReceives: [],
  claimedShieldedSpends: [],
  claimedContractCalls: [],
  shieldedMints: new Map(),
  unshieldedMints: new Map(),
  unshieldedInputs: new Map(),
  unshieldedOutputs: new Map(),
  claimedUnshieldedSpends: new Map(),
});

/** Keep the original mutable containers and restore their contents on abort. */
const snapshotMutable = (value: unknown, seen = new WeakSet<object>()): (() => void) => {
  if (value === null || typeof value !== 'object') return () => {};
  if (seen.has(value)) throw new CompactError('Cannot snapshot a cyclic call-state container');
  seen.add(value);
  if (value instanceof Uint8Array) {
    const bytes = Uint8Array.from(value);
    return () => Uint8Array.prototype.set.call(value, bytes);
  }
  if (value instanceof Map) {
    const entries = Array.from(value, ([key, item]) => ({
      key,
      item,
      restoreKey: snapshotMutable(key, seen),
      restoreItem: snapshotMutable(item, seen),
    }));
    return () => {
      for (const entry of entries) {
        entry.restoreKey();
        entry.restoreItem();
      }
      value.clear();
      for (const { key, item } of entries) value.set(key, item);
    };
  }
  if (value instanceof Set) {
    const entries = Array.from(value, (item) => ({ item, restore: snapshotMutable(item, seen) }));
    return () => {
      for (const entry of entries) entry.restore();
      value.clear();
      for (const { item } of entries) value.add(item);
    };
  }
  if (!Array.isArray(value) && Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null) {
    throw new CompactError('Cannot snapshot an unfamiliar call-state container');
  }
  const descriptors = Object.getOwnPropertyDescriptors(value);
  const entries = Reflect.ownKeys(descriptors).map((key) => {
    const descriptor = descriptors[key as keyof typeof descriptors];
    if (!('value' in descriptor)) throw new CompactError('Cannot snapshot an accessor call-state container');
    return { key, descriptor, restore: snapshotMutable(descriptor.value, seen) };
  });
  return () => {
    for (const entry of entries) entry.restore();
    for (const key of Reflect.ownKeys(value)) {
      if (!Object.prototype.hasOwnProperty.call(descriptors, key)) Reflect.deleteProperty(value, key);
    }
    // Restore array length before reintroducing any removed numeric indices.
    if (Array.isArray(value)) value.length = descriptors.length.value as number;
    for (const { key, descriptor } of entries) {
      if (key !== 'length') Object.defineProperty(value, key, descriptor);
    }
  };
};

const assertAddress = (address: ocrt.ContractAddress): void => {
  if (!isContractAddress(address)) {
    throw new CompactError(`Value ${address} is not a contract address`);
  }
};

const assertPinnedContext = (context: ocrt.QueryContext, address: ocrt.ContractAddress, snapshot: ParentBlockSnapshot): void => {
  if (
    context.address !== address ||
    context.block.ownAddress !== address ||
    context.block.parentBlockHash !== snapshot.parentBlockHash ||
    context.block.secondsSinceEpoch !== BigInt(snapshot.time)
  ) {
    throw new CompactError(`Query context for '${address}' changed the pinned parent block or call identity`);
  }
};

/**
 * A single root call tree. Preparing a callee cannot change the maps. Entering
 * one commits its separate query/gas/deployed-state cells; ending restores the
 * caller frame while retaining the callee's cells for later sequential calls.
 * No circuit or proof is executed here.
 * @internal
 */
export class InternalCrossContractCallState {
  readonly #snapshot: ParentBlockSnapshot;
  readonly #stateProvider: ContractStateProvider;
  readonly #moduleProvider: ContractModuleProvider;
  readonly #frames: Frame[];
  readonly #active: Set<ocrt.ContractAddress>;
  readonly #queryContexts = new Map<ocrt.ContractAddress, ocrt.QueryContext>();
  readonly #gasCosts = new Map<ocrt.ContractAddress, ocrt.RunningCost>();
  readonly #deployedStates = new Map<ocrt.ContractAddress, ocrt.ContractState>();
  #generation = 0;
  #poisoned: CompactError | undefined;

  #requireHealthy(): void {
    if (this.#poisoned) throw this.#poisoned;
  }

  constructor({ entryAddress, entryState, parentBlockHash, time, stateProvider, moduleProvider }: InternalCallStateOptions) {
    assertAddress(entryAddress);
    if (entryAddress === ocrt.dummyContractAddress()) {
      throw new CompactError('The entry contract cannot have the default contract address');
    }
    if (!/^[0-9a-f]{64}$/.test(parentBlockHash)) {
      throw new CompactError('A canonical 32-byte parent block hash is required for internal calls');
    }
    if (!Number.isSafeInteger(time) || time < 0) {
      throw new CompactError('A non-negative integer block time is required for internal calls');
    }
    this.#snapshot = Object.freeze({ parentBlockHash, time });
    this.#stateProvider = stateProvider;
    this.#moduleProvider = moduleProvider;
    const queryContext = new ocrt.QueryContext(entryState.data, entryAddress);
    queryContext.block = {
      ...queryContext.block,
      ownAddress: entryAddress,
      balance: new Map(entryState.balance),
      parentBlockHash,
      secondsSinceEpoch: BigInt(time),
    };
    this.#frames = [{ address: entryAddress }];
    this.#active = new Set([entryAddress]);
    this.#queryContexts.set(entryAddress, queryContext);
    this.#gasCosts.set(entryAddress, emptyRunningCost());
    this.#deployedStates.set(entryAddress, entryState);
  }

  get snapshot(): ParentBlockSnapshot {
    return this.#snapshot;
  }

  get currentAddress(): ocrt.ContractAddress {
    this.#requireHealthy();
    return this.#frames[this.#frames.length - 1].address;
  }

  isActive(address: ocrt.ContractAddress): boolean {
    this.#requireHealthy();
    return this.#active.has(address);
  }

  queryContextFor(address: ocrt.ContractAddress): ocrt.QueryContext | undefined {
    this.#requireHealthy();
    return this.#queryContexts.get(address);
  }

  gasCostFor(address: ocrt.ContractAddress): ocrt.RunningCost | undefined {
    this.#requireHealthy();
    return this.#gasCosts.get(address);
  }

  deployedStateFor(address: ocrt.ContractAddress): ocrt.ContractState | undefined {
    this.#requireHealthy();
    return this.#deployedStates.get(address);
  }

  /**
   * Own the call-state half of one execution. A thrown state callback unwinds
   * nested frames before restoring this call's query, gas, and balance cells.
   * The generated circuit context and transcript need their own owner before
   * this can be used as a public cross-contract call.
   */
  async transact<T>(request: InternalCallRequest, execute: (call: ActiveCallView) => Promise<T>): Promise<T> {
    const call = await this.begin(request);
    const view: ActiveCallView = Object.freeze({
      callerAddress: call.callerAddress,
      calleeAddress: call.calleeAddress,
      calleeCircuitId: call.calleeCircuitId,
      get queryContext() {
        return call.queryContext;
      },
      get gasCost() {
        return call.gasCost;
      },
    });
    try {
      const result = await execute(view);
      this.#assertTop(call);
      call.end();
      return result;
    } catch (error) {
      // A nested call may have entered before the failure. Its rollback must
      // run first, even if the user callback did not get a chance to do so.
      const rollbackFailures: unknown[] = [];
      while (this.#frames.some((frame) => frame.call === call) && this.#frames[this.#frames.length - 1].call !== call) {
        try {
          this.#frames[this.#frames.length - 1].call!.abort();
        } catch (failure) {
          rollbackFailures.push(failure);
        }
      }
      if (this.#frames[this.#frames.length - 1].call === call) {
        try {
          call.abort();
        } catch (failure) {
          rollbackFailures.push(failure);
        }
      }
      if (rollbackFailures.length) throw new InternalCallTransactionRollbackError(error, rollbackFailures);
      throw error;
    }
  }

  /** Resolve first, then enter. Re-entry is rejected before touching providers. */
  async begin({ calleeAddress, calleeCircuitId, interfaceName, declaration }: InternalCallRequest): Promise<ActiveCall> {
    this.#requireHealthy();
    if (this.#active.has(calleeAddress)) {
      throw new CompactError(
        `Contract re-entrancy detected: '${calleeAddress}' is already executing on the call stack; ` +
          're-entrant cross-contract calls are not yet supported',
      );
    }
    const callerAddress = this.currentAddress;
    const generation = this.#generation;
    const snapshot = this.#snapshot;
    assertPinnedContext(this.#queryContexts.get(callerAddress)!, callerAddress, snapshot);
    const stateProvider: ContractStateProvider = {
      getContractState: async (blockHash, address) => {
        if (blockHash !== snapshot.parentBlockHash || address !== calleeAddress) {
          throw new CompactError('Cross-contract state lookup escaped the parent-block snapshot');
        }
        const cached = this.#deployedStates.get(address);
        return cached ?? this.#stateProvider.getContractState(blockHash, address);
      },
    };
    const prepared = await preflightCrossContractCall({
      callerAddress,
      calleeAddress,
      calleeCircuitId,
      interfaceName,
      declaration,
      parentBlockHash: snapshot.parentBlockHash,
      stateProvider,
      moduleProvider: this.#moduleProvider,
    });
    if (generation !== this.#generation || callerAddress !== this.currentAddress || this.#active.has(calleeAddress)) {
      throw new CompactError('Cross-contract call stack changed during preflight');
    }
    // Capture the whole tree before the first assignment. A parent abort must
    // also undo a successful nested callee, including a sequential callee's
    // in-place block/effects reset. Generation stays monotonic on rollback.
    const rollback: RollbackSnapshot = {
      queryContexts: new Map(this.#queryContexts),
      gasCosts: new Map(this.#gasCosts),
      deployedStates: new Map(this.#deployedStates),
      queryFields: Array.from(new Set(this.#queryContexts.values()), (context) => {
        const block = context.block;
        const effects = context.effects;
        const commitmentIndices = context.comIndices;
        return {
          context,
          block,
          effects,
          commitmentIndices,
          ownCommitmentIndices: Object.getOwnPropertyDescriptor(context, 'comIndices'),
          restoreBlock: snapshotMutable(block),
          restoreEffects: snapshotMutable(effects),
          restoreCommitmentIndices: snapshotMutable(commitmentIndices),
        };
      }),
      gasFields: Array.from(new Set(this.#gasCosts.values()), (cost) => snapshotMutable(cost)),
      stateFields: Array.from(new Set([...this.#deployedStates.values(), prepared.deployedState]), (state) => ({
        state,
        balance: state.balance,
        data: state.data,
        maintenanceAuthority: state.maintenanceAuthority,
        restoreBalance: snapshotMutable(state.balance),
      })),
    };
    let queryContext = this.#queryContexts.get(calleeAddress);
    if (queryContext === undefined) {
      queryContext = new ocrt.QueryContext(prepared.deployedState.data, calleeAddress);
      queryContext.block = {
        ...queryContext.block,
        balance: new Map(prepared.deployedState.balance),
        ownAddress: calleeAddress,
        secondsSinceEpoch: BigInt(snapshot.time),
        parentBlockHash: snapshot.parentBlockHash,
        caller: { tag: 'contract', address: callerAddress },
      };
    } else {
      assertPinnedContext(queryContext, calleeAddress, snapshot);
      queryContext.block = {
        ...queryContext.block,
        caller: { tag: 'contract', address: callerAddress },
      };
      queryContext.effects = emptyEffects();
    }
    this.#queryContexts.set(calleeAddress, queryContext);
    this.#gasCosts.set(calleeAddress, this.#gasCosts.get(calleeAddress) ?? emptyRunningCost());
    this.#deployedStates.set(calleeAddress, prepared.deployedState);
    const queryContextForCall = (): ocrt.QueryContext => {
      this.#requireHealthy();
      return this.#queryContexts.get(calleeAddress)!;
    };
    const gasCostForCall = (): ocrt.RunningCost => {
      this.#requireHealthy();
      return this.#gasCosts.get(calleeAddress)!;
    };
    const call: ActiveCall = Object.freeze({
      callerAddress,
      calleeAddress,
      calleeCircuitId,
      deployedState: prepared.deployedState,
      module: prepared.module,
      get queryContext() {
        return queryContextForCall();
      },
      get gasCost() {
        return gasCostForCall();
      },
      end: () => this.end(call),
      abort: () => this.abort(call),
    });
    this.#frames.push({ address: calleeAddress, call, rollback });
    this.#active.add(calleeAddress);
    this.#generation++;
    return call;
  }

  /** Record an executed call's next context, without allowing its block to drift. */
  advance(call: ActiveCall, context: ocrt.QueryContext, gasCost: ocrt.RunningCost): void {
    this.#requireHealthy();
    this.#assertTop(call);
    assertPinnedContext(context, call.calleeAddress, this.#snapshot);
    this.#queryContexts.set(call.calleeAddress, context);
    this.#gasCosts.set(call.calleeAddress, { ...gasCost });
    this.#generation++;
  }

  private end(call: ActiveCall): void {
    this.#requireHealthy();
    this.#assertTop(call);
    this.#frames.pop();
    this.#active.delete(call.calleeAddress);
    this.#generation++;
  }

  private abort(call: ActiveCall): void {
    // A corrupted callee block is precisely when rollback is needed, so do
    // not run assertPinnedContext before restoring the saved fields.
    this.#assertTopHandle(call);
    const rollback = this.#frames[this.#frames.length - 1].rollback!;
    const failures: unknown[] = [];
    const attempt = (restore: () => void): void => {
      try {
        restore();
      } catch (error) {
        failures.push(error);
      }
    };
    try {
      for (const {
        context,
        block,
        effects,
        commitmentIndices,
        ownCommitmentIndices,
        restoreBlock,
        restoreEffects,
        restoreCommitmentIndices,
      } of rollback.queryFields) {
        attempt(restoreBlock);
        attempt(restoreEffects);
        attempt(restoreCommitmentIndices);
        attempt(() => {
          context.block = block;
        });
        attempt(() => {
          context.effects = effects;
        });
        attempt(() => {
          if (ownCommitmentIndices) {
            Object.defineProperty(context, 'comIndices', ownCommitmentIndices);
          } else if (Object.prototype.hasOwnProperty.call(context, 'comIndices')) {
            if (!Reflect.deleteProperty(context, 'comIndices')) {
              throw new CompactError('Could not restore the commitment-index property');
            }
          }
          if (ownCommitmentIndices && 'value' in ownCommitmentIndices && context.comIndices !== commitmentIndices) {
            throw new CompactError('Could not restore the commitment-index map');
          }
        });
      }
      for (const restore of rollback.gasFields) attempt(restore);
      for (const { state, balance, data, maintenanceAuthority, restoreBalance } of rollback.stateFields) {
        attempt(restoreBalance);
        attempt(() => {
          state.balance = balance;
          state.data = data;
          state.maintenanceAuthority = maintenanceAuthority;
        });
      }
      attempt(() => {
        this.#queryContexts.clear();
        for (const [address, context] of rollback.queryContexts) this.#queryContexts.set(address, context);
      });
      attempt(() => {
        this.#gasCosts.clear();
        for (const [address, cost] of rollback.gasCosts) this.#gasCosts.set(address, cost);
      });
      attempt(() => {
        this.#deployedStates.clear();
        for (const [address, state] of rollback.deployedStates) this.#deployedStates.set(address, state);
      });
    } finally {
      this.#frames.pop();
      this.#active.delete(call.calleeAddress);
      this.#generation++;
    }
    if (failures.length) {
      this.#poisoned = new CompactError(
        `Cross-contract rollback failed to restore ${failures.length} mutable cell(s); call state is poisoned`,
      );
      throw this.#poisoned;
    }
  }

  #assertTopHandle(call: ActiveCall): void {
    if (this.#frames[this.#frames.length - 1].call !== call) {
      throw new CompactError('Cross-contract call must finish at the top of its active stack');
    }
  }

  #assertTop(call: ActiveCall): void {
    this.#assertTopHandle(call);
    assertPinnedContext(this.#queryContexts.get(call.calleeAddress)!, call.calleeAddress, this.#snapshot);
  }
}
