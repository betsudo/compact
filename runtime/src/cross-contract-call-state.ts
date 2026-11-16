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
}>;

type Frame = {
  readonly address: ocrt.ContractAddress;
  readonly call?: ActiveCall;
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
      balance: entryState.balance,
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
    return this.#frames[this.#frames.length - 1].address;
  }

  isActive(address: ocrt.ContractAddress): boolean {
    return this.#active.has(address);
  }

  queryContextFor(address: ocrt.ContractAddress): ocrt.QueryContext | undefined {
    return this.#queryContexts.get(address);
  }

  gasCostFor(address: ocrt.ContractAddress): ocrt.RunningCost | undefined {
    return this.#gasCosts.get(address);
  }

  deployedStateFor(address: ocrt.ContractAddress): ocrt.ContractState | undefined {
    return this.#deployedStates.get(address);
  }

  /** Resolve first, then enter. Re-entry is rejected before touching providers. */
  async begin({ calleeAddress, calleeCircuitId, interfaceName, declaration }: InternalCallRequest): Promise<ActiveCall> {
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
    let queryContext = this.#queryContexts.get(calleeAddress);
    if (queryContext === undefined) {
      queryContext = new ocrt.QueryContext(prepared.deployedState.data, calleeAddress);
      queryContext.block = {
        ...queryContext.block,
        balance: prepared.deployedState.balance,
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
    const queryContextForCall = (): ocrt.QueryContext => this.#queryContexts.get(calleeAddress)!;
    const gasCostForCall = (): ocrt.RunningCost => this.#gasCosts.get(calleeAddress)!;
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
    });
    this.#frames.push({ address: calleeAddress, call });
    this.#active.add(calleeAddress);
    this.#generation++;
    return call;
  }

  /** Record an executed call's next context, without allowing its block to drift. */
  advance(call: ActiveCall, context: ocrt.QueryContext, gasCost: ocrt.RunningCost): void {
    this.#assertTop(call);
    assertPinnedContext(context, call.calleeAddress, this.#snapshot);
    this.#queryContexts.set(call.calleeAddress, context);
    this.#gasCosts.set(call.calleeAddress, { ...gasCost });
    this.#generation++;
  }

  private end(call: ActiveCall): void {
    this.#assertTop(call);
    this.#frames.pop();
    this.#active.delete(call.calleeAddress);
    this.#generation++;
  }

  #assertTop(call: ActiveCall): void {
    if (this.#frames[this.#frames.length - 1].call !== call) {
      throw new CompactError('Cross-contract call must finish at the top of its active stack');
    }
    assertPinnedContext(this.#queryContexts.get(call.calleeAddress)!, call.calleeAddress, this.#snapshot);
  }
}
