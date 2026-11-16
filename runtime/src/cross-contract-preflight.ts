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
 * Internal, read-only preparation for a future cross-contract call. This module
 * never enters the callee or writes to a CircuitContext. In particular, the
 * parent block hash is supplied explicitly until the full Ledger 9 CallContext
 * exists. Do not export this module from the package index.
 */

import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { CircuitId } from './circuit-context.js';
import { checkConformance } from './conformance.js';
import { CompactError } from './error.js';
import { InterfaceDescriptor } from './interface-descriptor.js';
import { Module } from './module.js';
import { ModuleResolutionContext, ModuleResolutionError, ModuleResolutionFailure } from './module-resolution.js';
import { ContractModuleProvider, ContractStateProvider, ModuleThunk } from './providers.js';
import { isContractAddress } from './utils.js';
import { VerifierKeyHash, isVerifierKeyHash, verifierKeyHashOf } from './verifier-key-hash.js';

export type CrossContractPreflightOptions = {
  readonly callerAddress: ocrt.ContractAddress;
  readonly calleeAddress: ocrt.ContractAddress;
  readonly calleeCircuitId: CircuitId;
  readonly interfaceName: string;
  readonly declaration: InterfaceDescriptor;
  /** The exact parent block whose post-evaluation contract state is requested. */
  readonly parentBlockHash: string;
  readonly stateProvider: ContractStateProvider;
  readonly moduleProvider?: ContractModuleProvider;
};

export type CrossContractPreflightResult = {
  readonly deployedState: ocrt.ContractState;
  readonly module: Module;
};

const failResolution: (context: ModuleResolutionContext, failure: ModuleResolutionFailure) => never = (context, failure) => {
  throw new ModuleResolutionError(context, failure);
};

/** The pinned module-loader classifications, before reading any generated export. */
const resolveModule = async (
  provider: ContractModuleProvider,
  address: ocrt.ContractAddress,
  context: ModuleResolutionContext,
): Promise<Module> => {
  let thunk: ModuleThunk | undefined;
  try {
    thunk = provider.resolve(address);
  } catch (cause) {
    failResolution(context, { kind: 'ProviderThrew', cause });
  }
  if (thunk === undefined) {
    failResolution(context, { kind: 'UnsupportedImplementation' });
  }
  if (typeof thunk !== 'function') {
    failResolution(context, { kind: 'ProviderThrew', cause: thunk });
  }
  let module: Module;
  try {
    module = await thunk();
  } catch (cause) {
    failResolution(context, { kind: 'ModuleLoadRejected', cause });
  }
  if (module === null || module === undefined) {
    failResolution(context, { kind: 'ProviderThrew', cause: module });
  }
  const required: readonly (keyof Module)[] = ['Contract', 'circuitSignatures', 'expectedVk'];
  const missing = required.filter(
    (name) => !Object.prototype.hasOwnProperty.call(module, name) || module[name] === null || module[name] === undefined,
  );
  if (missing.length !== 0) {
    failResolution(context, { kind: 'IncompleteModule', missing });
  }
  return module;
};

const checkModuleConformance = (declaration: InterfaceDescriptor, module: Module, context: ModuleResolutionContext): void => {
  const result = checkConformance(declaration, module.circuitSignatures);
  switch (result.outcome) {
    case 'Conformant':
      return;
    case 'Violation':
      failResolution(context, {
        kind: 'NonconformantImplementation',
        circuitId: result.circuitId,
        check: result.check,
        argumentIndex: result.argumentIndex,
      });
      break;
    case 'Unreadable':
      failResolution(context, {
        kind: 'UnreadableModule',
        circuitId: result.circuitId,
        unreadableTag: result.unreadableTag,
        argumentIndex: result.argumentIndex,
      });
      break;
    default: {
      const exhaustive: never = result;
      throw new CompactError(`unhandled conformance outcome ${JSON.stringify(exhaustive)}`);
    }
  }
};

/** Compare called and overlapping verifier keys, after conformance succeeds. */
const checkImplementation = (
  state: ocrt.ContractState,
  module: Module,
  circuitId: CircuitId,
  context: ModuleResolutionContext,
): void => {
  const deployedHash = (name: CircuitId): VerifierKeyHash | undefined => {
    const key = state.operation(name)?.verifierKey;
    return key === undefined || key.length === 0 ? undefined : verifierKeyHashOf(key);
  };
  const recordedHash = (name: CircuitId, recorded: string): VerifierKeyHash => {
    if (!isVerifierKeyHash(recorded)) {
      failResolution(context, { kind: 'MalformedVerifierKeyHash', circuitId: name, recorded });
    }
    return recorded;
  };
  const actual = deployedHash(circuitId);
  if (actual === undefined) {
    failResolution(context, { kind: 'OperationAbsent' });
  }
  const table = module.expectedVk;
  const recorded = Object.prototype.hasOwnProperty.call(table, circuitId) ? table[circuitId] : undefined;
  if (recorded === undefined) {
    failResolution(context, { kind: 'ImplementationMismatch', circuitId, actual });
  }
  const expected = recordedHash(circuitId, recorded);
  if (expected !== actual) {
    failResolution(context, { kind: 'ImplementationMismatch', circuitId, expected, actual });
  }
  for (const otherId of Object.keys(table)) {
    if (otherId === circuitId) continue;
    const otherActual = deployedHash(otherId);
    if (otherActual === undefined) continue;
    const otherExpected = recordedHash(otherId, table[otherId]);
    if (otherExpected !== otherActual) {
      failResolution(context, {
        kind: 'ImplementationMismatch',
        circuitId: otherId,
        expected: otherExpected,
        actual: otherActual,
      });
    }
  }
};

/**
 * Resolve a callee without entering it. All resolution failures happen before
 * any caller/callee execution state is changed; provider reads can still have
 * effects outside the runtime and remain the application's responsibility.
 * @internal
 */
export const preflightCrossContractCall = async ({
  callerAddress,
  calleeAddress,
  calleeCircuitId,
  interfaceName,
  declaration,
  parentBlockHash,
  stateProvider,
  moduleProvider,
}: CrossContractPreflightOptions): Promise<CrossContractPreflightResult> => {
  const context: ModuleResolutionContext = { callerAddress, calleeAddress, calleeCircuitId, interfaceName };
  if (!Object.prototype.hasOwnProperty.call(declaration, calleeCircuitId) || declaration[calleeCircuitId] === undefined) {
    throw new CompactError(
      `Expected declaration of circuit '${calleeCircuitId}' on contract type '${interfaceName}' to be defined`,
    );
  }
  if (!isContractAddress(calleeAddress)) {
    throw new CompactError(`Value ${calleeAddress} is not a contract address`);
  }
  if (calleeAddress === ocrt.dummyContractAddress()) {
    throw new CompactError('Cannot perform cross-contract call to default contract address');
  }
  if (moduleProvider === undefined) {
    failResolution(context, { kind: 'ModuleProviderAbsent' });
  }
  if (typeof parentBlockHash !== 'string' || parentBlockHash.length === 0) {
    throw new CompactError(`Expected parent block hash to fetch state for callee '${calleeAddress}' to be defined`);
  }
  if (stateProvider === undefined) {
    throw new CompactError(`Expected state provider for call to '${calleeAddress}' to be defined`);
  }
  const deployedState = await stateProvider.getContractState(parentBlockHash, calleeAddress);
  if (deployedState === undefined || deployedState === null) {
    throw new CompactError(`Expected contract state for callee '${calleeAddress}' to be defined`);
  }
  const module = await resolveModule(moduleProvider, calleeAddress, context);
  checkModuleConformance(declaration, module, context);
  checkImplementation(deployedState, module, calleeCircuitId, context);
  return { deployedState, module };
};
