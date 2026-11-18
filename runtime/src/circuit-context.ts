// This file is part of Compact.
// Copyright (C) 2025 Midnight Foundation
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

import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import {
  emptyZswapLocalState,
  EncodedCoinPublicKey,
  EncodedZswapLocalState,
  ZswapLocalState,
  encodeZswapLocalState,
  copyEncodedZswapLocalState,
} from './zswap.js';
import { PartialProofData, ProofData } from './proof-data.js';
import { CompactError } from './error.js';
import { CompactTypeBytes } from './compact-types.js';
import type { ContractModuleProvider, ContractStateProvider } from './providers.js';
import { isContractAddress } from './utils.js';

/** External circuit name used in generated module metadata. */
export type CircuitId = string;

/** A Ledger VM log with the address of the emitting contract. */
export type LogEvent = Extract<ocrt.GatherResult, { tag: 'log' }>['content'] & {
  address: ocrt.ContractAddress;
};

/** The proof data for one completed call in depth-first completion order. */
export interface CallProofData extends ProofData {
  circuitId: CircuitId;
  contractAddress: ocrt.ContractAddress;
  initialQueryContext: ocrt.QueryContext;
  finalQueryContext: ocrt.QueryContext;
  zswapLocalState: EncodedZswapLocalState;
  commCommData?: {
    commComm: ocrt.CommunicationCommitment;
    commCommRand: ocrt.CommunicationCommitmentRand;
  };
}

export type CallProofDataTrace = CallProofData[];

/** The live per-contract frame used by generated Ledger 9 circuits. */
export interface CircuitCallContext<PS = any> {
  circuitId: CircuitId;
  contractAddress: ocrt.ContractAddress;
  initialQueryContext: ocrt.QueryContext;
  currentQueryContext: ocrt.QueryContext;
  currentGasCost: ocrt.RunningCost;
  currentPrivateState: PS | undefined;
  currentZswapLocalState: EncodedZswapLocalState;
  parentBlockHash?: string;
  time: number;
}

export type CallContext<PS = any> = CircuitCallContext<PS>;

export type CrossContractInputs = {
  readonly stateProvider: ContractStateProvider;
  readonly moduleProvider: ContractModuleProvider;
};

/**
 * The external information accessible from within a Compact circuit call
 */
export interface CircuitContext<PS = any> {
  /** Live call frame; its cells alias the entry in each per-address map. */
  callContext: CircuitCallContext<PS>;
  queryContexts: Record<ocrt.ContractAddress, ocrt.QueryContext>;
  gasCosts: Record<ocrt.ContractAddress, ocrt.RunningCost>;
  zswapLocalStates: Record<ocrt.ContractAddress, EncodedZswapLocalState>;
  contractStates: Record<ocrt.ContractAddress, ocrt.ContractState>;
  callProofDataTrace: CallProofDataTrace;
  stateProvider?: ContractStateProvider;
  moduleProvider?: ContractModuleProvider;
  /** Shared across copies so a future coordinator can refuse re-entry. */
  activeContracts: Set<ocrt.ContractAddress>;
  /**
   * The current private state for the contract.
   */
  currentPrivateState: PS;
  /**
   * The current Zswap local state. Tracks inputs and outputs produced during circuit execution.
   */
  currentZswapLocalState: EncodedZswapLocalState;
  /**
   * The current on-chain context the transaction is evolving.
   */
  currentQueryContext: ocrt.QueryContext;
  /**
   * The cost model to use for the execution.
   */
  costModel: ocrt.CostModel;
  /**
   * The gas limit for this circuit.
   */
  gasLimit?: ocrt.RunningCost;
  /** Events emitted by successful calls, in execution order. */
  events: LogEvent[];
}

type ContextProvenance = Readonly<{
  mode: 'ledger9' | 'legacy';
  circuitId: string;
  contractAddress: ocrt.ContractAddress;
  time: number;
  parentBlockHash?: string;
  /** The parent contract for a separately pinned callee frame. */
  callerAddress?: ocrt.ContractAddress;
  sourceContractState?: ocrt.ContractState;
  initialQueryContext: ocrt.QueryContext;
}>;

// The symbol carries a nonforgeable token through generated shallow spreads;
// the actual pin stays in private WeakMaps. A removed or forged token refuses.
const provenanceSymbol = Symbol('compact circuit context provenance');
const tokenProvenance = new WeakMap<object, ContextProvenance>();
const contextProvenance = new WeakMap<object, ContextProvenance>();
const querySourceStates = new WeakMap<ocrt.QueryContext, ocrt.ContractState>();
const activeSetHas = Set.prototype.has;
const activeSetForEach = Set.prototype.forEach;
const activeSetAdd = Set.prototype.add;

const bindProvenance = <PS>(
  context: CircuitContext<PS>,
  pin: ContextProvenance,
  token: object = Object.freeze({}),
): CircuitContext<PS> => {
  tokenProvenance.set(token, pin);
  Object.defineProperty(context, provenanceSymbol, { value: token, enumerable: true });
  contextProvenance.set(context, pin);
  return context;
};

const readProvenance = (context: CircuitContext): ContextProvenance => {
  const direct = contextProvenance.get(context);
  const token = Object.getOwnPropertyDescriptor(context, provenanceSymbol)?.value;
  const carried = token !== null && typeof token === 'object' ? tokenProvenance.get(token) : undefined;
  if ((direct && carried !== direct) || (!direct && !carried)) {
    throw new CompactError('Circuit context provenance is missing or changed');
  }
  return (direct ?? carried)!;
};

/** Source identity is carried privately through generated shallow copies. @internal */
export const sourceContractStateOf = (context: CircuitContext): ocrt.ContractState | undefined => {
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  return pin.sourceContractState;
};

/** Bind an on-chain VM query to the deployed state that created its call. @internal */
export const bindQuerySourceState = (query: ocrt.QueryContext, state: ocrt.ContractState): void => {
  const existing = querySourceStates.get(query);
  if (existing !== undefined && existing !== state) throw new CompactError('Query source contract state changed');
  querySourceStates.set(query, state);
};

/** Keep the original deployed-state identity across VM query replacements. @internal */
export const inheritQuerySourceState = (previous: ocrt.QueryContext, next: ocrt.QueryContext): void => {
  const source = querySourceStates.get(previous);
  if (source !== undefined) bindQuerySourceState(next, source);
};

/** Refuse a fork whose query was not produced from this deployed state. @internal */
export const assertQuerySourceState = (query: ocrt.QueryContext, state: ocrt.ContractState): void => {
  if (querySourceStates.get(query) !== state) throw new CompactError('Callee query and deployed state have different sources');
};

const assertFrameProvenance = (context: CircuitContext, pin: ContextProvenance): void => {
  const frame = context.callContext;
  if (
    frame?.circuitId !== pin.circuitId ||
    frame.contractAddress !== pin.contractAddress ||
    frame.time !== pin.time ||
    frame.parentBlockHash !== pin.parentBlockHash ||
    frame.initialQueryContext !== pin.initialQueryContext
  ) {
    throw new CompactError('Circuit call frame changed its pinned provenance');
  }
};

/** The mode is authenticated independently of mutable call-frame fields. */
export const isLedger9CircuitContext = (context: CircuitContext): boolean => {
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  return pin.mode === 'ledger9';
};

/**
 * @internal
 */
const coerceToChargedState = (contractState: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState): ocrt.ChargedState => {
  let state;
  if (contractState instanceof ocrt.ChargedState) {
    state = contractState;
  } else if (contractState instanceof ocrt.ContractState) {
    state = contractState.data;
  } else if (contractState instanceof ocrt.StateValue) {
    state = new ocrt.ChargedState(contractState);
  } else {
    throw new CompactError(`'contractState' parameter ${contractState} has unexpected type`);
  }
  return state;
};

/**
 * @internal
 */
const createInitialQueryContext = (
  contractState: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState,
  contractAddress: ocrt.ContractAddress,
  time: number,
  parentBlockHash?: string,
  caller?: ocrt.PublicAddress,
): ocrt.QueryContext => {
  const initialQueryContext = new ocrt.QueryContext(coerceToChargedState(contractState), contractAddress);
  const balance = contractState instanceof ocrt.ContractState ? contractState.balance : new Map();
  initialQueryContext.block = {
    ...initialQueryContext.block,
    balance,
    ownAddress: contractAddress,
    secondsSinceEpoch: BigInt(time),
  };
  if (parentBlockHash !== undefined) {
    initialQueryContext.block = { ...initialQueryContext.block, parentBlockHash };
  }
  if (caller !== undefined) {
    initialQueryContext.block = { ...initialQueryContext.block, caller };
  }
  if (contractState instanceof ocrt.ContractState) bindQuerySourceState(initialQueryContext, contractState);
  return initialQueryContext;
};

/**
 * @internal
 */
const isZswapLocalState = (value: any): value is ZswapLocalState => {
  return (
    typeof value === 'object' &&
    value !== null &&
    'coinPublicKey' in value &&
    typeof value.coinPublicKey === 'string' &&
    'currentIndex' in value &&
    'inputs' in value &&
    'outputs' in value
  );
};

/**
 * @internal
 */
const isEncodedZswapLocalState = (value: any): value is EncodedZswapLocalState => {
  return (
    typeof value === 'object' &&
    value !== null &&
    'coinPublicKey' in value &&
    typeof value.coinPublicKey === 'object' &&
    value.coinPublicKey !== null &&
    'bytes' in value.coinPublicKey &&
    'currentIndex' in value &&
    'inputs' in value &&
    'outputs' in value
  );
};

/**
 * The pinned Ledger 9 object form. Positional Runtime 0.18 calls remain valid
 * and have no circuit ID or cross-contract providers.
 */
export interface HostCircuitContextOptions<PS> {
  circuitId: string;
  contractAddress: ocrt.ContractAddress;
  coinPublicKeyOrZswapState: ocrt.CoinPublicKey | EncodedCoinPublicKey | ZswapLocalState | EncodedZswapLocalState;
  contractState: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState;
  privateState: PS;
  gasLimit?: ocrt.RunningCost;
  costModel?: ocrt.CostModel;
  time?: number;
  parentBlockHash?: string;
  crossContract?: CrossContractInputs;
}

export function createCircuitContext<PS>(options: HostCircuitContextOptions<PS>): CircuitContext<PS>;
export function createCircuitContext<PS>(
  contractAddress: ocrt.ContractAddress,
  coinPublicKeyOrZswapState: ocrt.CoinPublicKey | EncodedCoinPublicKey | ZswapLocalState | EncodedZswapLocalState,
  contractState: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState,
  privateState: PS,
  gasLimit?: ocrt.RunningCost,
  costModel?: ocrt.CostModel,
  time?: number,
): CircuitContext<PS>;
export function createCircuitContext<PS>(
  contractAddressOrOptions: ocrt.ContractAddress | HostCircuitContextOptions<PS>,
  coinPublicKeyOrZswapState?: ocrt.CoinPublicKey | EncodedCoinPublicKey | ZswapLocalState | EncodedZswapLocalState,
  contractState?: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState,
  privateState?: PS,
  gasLimit?: ocrt.RunningCost,
  costModel?: ocrt.CostModel,
  time?: number,
): CircuitContext<PS> {
  let contractAddress: ocrt.ContractAddress;
  const objectForm = arguments.length === 1;
  let circuitId = '';
  let parentBlockHash: string | undefined;
  let crossContract: CrossContractInputs | undefined;
  if (objectForm) {
    const options = contractAddressOrOptions as HostCircuitContextOptions<PS>;
    if (
      typeof options !== 'object' ||
      options === null ||
      typeof options.circuitId !== 'string' ||
      options.circuitId.length === 0 ||
      !('contractAddress' in options) ||
      !('coinPublicKeyOrZswapState' in options) ||
      !('contractState' in options) ||
      !('privateState' in options)
    ) {
      throw new CompactError('createCircuitContext: invalid object-form context options');
    }
    circuitId = options.circuitId;
    parentBlockHash = options.parentBlockHash;
    crossContract = options.crossContract;
    if (parentBlockHash !== undefined && !/^[0-9a-f]{64}$/.test(parentBlockHash)) {
      throw new CompactError('createCircuitContext: parent block hash must be canonical Bytes<32>');
    }
    if (
      crossContract !== undefined &&
      (typeof crossContract !== 'object' ||
        crossContract === null ||
        parentBlockHash === undefined ||
        typeof crossContract.stateProvider?.getContractState !== 'function' ||
        typeof crossContract.moduleProvider?.resolve !== 'function')
    ) {
      throw new CompactError('createCircuitContext: cross-contract providers require a pinned parent block');
    }
    contractAddress = options.contractAddress;
    coinPublicKeyOrZswapState = options.coinPublicKeyOrZswapState;
    contractState = options.contractState;
    privateState = options.privateState;
    gasLimit = options.gasLimit;
    costModel = options.costModel;
    time = options.time;
  } else {
    contractAddress = contractAddressOrOptions as ocrt.ContractAddress;
  }
  if (coinPublicKeyOrZswapState === undefined || contractState === undefined) {
    throw new CompactError('createCircuitContext: missing coin key or contract state');
  }
  const blockTime = time ?? Math.floor(Date.now() / 1_000);
  if (!Number.isSafeInteger(blockTime) || blockTime < 0) {
    throw new CompactError('createCircuitContext: block time must be a non-negative integer');
  }
  const initialQueryContext = createInitialQueryContext(contractState, contractAddress, blockTime, parentBlockHash);

  let zswapLocalState: EncodedZswapLocalState;

  if (isZswapLocalState(coinPublicKeyOrZswapState)) {
    // Convert ZswapLocalState to EncodedZswapLocalState
    zswapLocalState = encodeZswapLocalState(coinPublicKeyOrZswapState);
  } else if (isEncodedZswapLocalState(coinPublicKeyOrZswapState)) {
    // Use EncodedZswapLocalState directly
    zswapLocalState = coinPublicKeyOrZswapState;
  } else {
    // It's a CoinPublicKey or EncodedCoinPublicKey, create empty state
    zswapLocalState = emptyZswapLocalState(coinPublicKeyOrZswapState);
  }
  if (circuitId) {
    zswapLocalState = copyEncodedZswapLocalState(zswapLocalState);
  }

  const currentGasCost = emptyRunningCost();
  const callContext: CircuitCallContext<PS> = {
    circuitId,
    contractAddress,
    initialQueryContext,
    currentQueryContext: initialQueryContext,
    currentGasCost,
    currentPrivateState: privateState,
    currentZswapLocalState: zswapLocalState,
    parentBlockHash,
    time: blockTime,
  };
  const context: CircuitContext<PS> = {
    callContext,
    queryContexts: { [contractAddress]: initialQueryContext },
    gasCosts: { [contractAddress]: currentGasCost },
    zswapLocalStates: { [contractAddress]: zswapLocalState },
    contractStates: Object.create(null) as Record<ocrt.ContractAddress, ocrt.ContractState>,
    callProofDataTrace: [],
    stateProvider: crossContract?.stateProvider,
    moduleProvider: crossContract?.moduleProvider,
    activeContracts: new Set([contractAddress]),
    currentPrivateState: privateState as PS,
    currentZswapLocalState: zswapLocalState,
    currentQueryContext: initialQueryContext,
    costModel: costModel ?? ocrt.CostModel.initialCostModel(),
    gasLimit,
    events: [],
  };
  return bindProvenance(
    context,
    Object.freeze({
      mode: objectForm ? 'ledger9' : 'legacy',
      circuitId,
      contractAddress,
      time: blockTime,
      parentBlockHash,
      sourceContractState: contractState instanceof ocrt.ContractState ? contractState : undefined,
      initialQueryContext,
    }),
  );
}

/** Copy the call containers; the active set is intentionally shared across a call tree. */
export const copyCircuitContext = <PS>(context: CircuitContext<PS>): CircuitContext<PS> => {
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  const copy: CircuitContext<PS> = {
    ...context,
    callContext: { ...context.callContext },
    queryContexts: { ...context.queryContexts },
    gasCosts: { ...context.gasCosts },
    zswapLocalStates: { ...context.zswapLocalStates },
    contractStates: { ...context.contractStates },
    callProofDataTrace: [...context.callProofDataTrace],
    events: [...context.events],
  };
  return bindProvenance(copy, pin, Object.getOwnPropertyDescriptor(context, provenanceSymbol)!.value as object);
};

/**
 * An internal bridge from a pinned generated caller to a separately pinned
 * callee. A callee cannot be made by changing the caller's callContext: that
 * would invalidate the immutable source identity of both generated modules.
 * The later coordinator owns entry, return, and the global transaction.
 * @internal
 */
export const forkCalleeCircuitContext = (
  caller: CircuitContext,
  circuitId: CircuitId,
  contractAddress: ocrt.ContractAddress,
  initialQueryContext: ocrt.QueryContext,
  currentGasCost: ocrt.RunningCost,
  deployedState: ocrt.ContractState,
): CircuitContext => {
  assertLiveCircuitCall(caller);
  const parent = readProvenance(caller);
  if (!isContractAddress(contractAddress) || contractAddress === ocrt.dummyContractAddress()) {
    throw new CompactError('Callee context requires a non-default contract address');
  }
  if (
    !circuitId ||
    !activeSetHas.call(caller.activeContracts, parent.contractAddress) ||
    activeSetHas.call(caller.activeContracts, contractAddress)
  ) {
    throw new CompactError('Callee context requires a declared non-reentrant circuit');
  }
  if (
    initialQueryContext.address !== contractAddress ||
    initialQueryContext.block.ownAddress !== contractAddress ||
    initialQueryContext.block.parentBlockHash !== parent.parentBlockHash ||
    initialQueryContext.block.secondsSinceEpoch !== BigInt(parent.time) ||
    initialQueryContext.block.caller?.tag !== 'contract' ||
    initialQueryContext.block.caller.address !== parent.contractAddress
  ) {
    throw new CompactError('Callee query differs from the caller parent-block snapshot');
  }
  assertQuerySourceState(initialQueryContext, deployedState);
  if (caller.stateProvider === undefined || caller.moduleProvider === undefined) {
    throw new CompactError('Callee context requires both cross-contract providers');
  }
  const gasCost = { ...currentGasCost };
  const zswapLocalState = copyEncodedZswapLocalState(
    caller.zswapLocalStates[contractAddress] ??
      emptyZswapLocalState(copyEncodedZswapLocalState(caller.callContext.currentZswapLocalState).coinPublicKey),
  );
  const callContext: CircuitCallContext = {
    circuitId,
    contractAddress,
    initialQueryContext,
    currentQueryContext: initialQueryContext,
    currentGasCost: gasCost,
    currentPrivateState: undefined,
    currentZswapLocalState: zswapLocalState,
    parentBlockHash: parent.parentBlockHash,
    time: parent.time,
  };
  const activeContracts = new Set<ocrt.ContractAddress>();
  activeSetForEach.call(caller.activeContracts, (address) => activeSetAdd.call(activeContracts, address));
  activeSetAdd.call(activeContracts, contractAddress);
  const child: CircuitContext = {
    ...caller,
    callContext,
    queryContexts: { ...caller.queryContexts, [contractAddress]: initialQueryContext },
    gasCosts: { ...caller.gasCosts, [contractAddress]: gasCost },
    zswapLocalStates: { ...caller.zswapLocalStates, [contractAddress]: zswapLocalState },
    contractStates: { ...caller.contractStates, [contractAddress]: deployedState },
    callProofDataTrace: [...caller.callProofDataTrace],
    activeContracts,
    currentPrivateState: undefined,
    currentZswapLocalState: zswapLocalState,
    currentQueryContext: initialQueryContext,
    events: [...caller.events],
  };
  return bindProvenance(
    child,
    Object.freeze({
      mode: 'ledger9',
      circuitId,
      contractAddress,
      time: parent.time,
      parentBlockHash: parent.parentBlockHash,
      callerAddress: parent.contractAddress,
      sourceContractState: deployedState,
      initialQueryContext,
    }),
  );
};

export const assertCircuitQueryIdentity = (context: CircuitContext, query: ocrt.QueryContext): void => {
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  if (pin.mode !== 'ledger9') throw new CompactError('Ledger 9 query requires an object-form context');
  if (
    query.address !== pin.contractAddress ||
    query.block.ownAddress !== pin.contractAddress ||
    query.block.secondsSinceEpoch !== BigInt(pin.time) ||
    (pin.parentBlockHash !== undefined && query.block.parentBlockHash !== pin.parentBlockHash) ||
    (pin.callerAddress !== undefined &&
      (query.block.caller?.tag !== 'contract' || query.block.caller.address !== pin.callerAddress))
  ) {
    throw new CompactError('Ledger 9 query changed the pinned block or call identity');
  }
};

/** Refuse stale live cells before a Ledger 9 query or proof entry. */
export const assertLiveCircuitCall = (context: CircuitContext): void => {
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  if (pin.mode !== 'ledger9') throw new CompactError('Ledger 9 live call requires an object-form context');
  const frame = context.callContext;
  const address = pin.contractAddress;
  if (
    !frame.circuitId ||
    context.queryContexts[address] !== frame.currentQueryContext ||
    context.gasCosts[address] !== frame.currentGasCost ||
    context.zswapLocalStates[address] !== frame.currentZswapLocalState
  ) {
    throw new CompactError('Ledger 9 live call cells are incomplete or stale');
  }
  assertCircuitQueryIdentity(context, frame.initialQueryContext);
  assertCircuitQueryIdentity(context, frame.currentQueryContext);
};

const queryContextMode = (context: CircuitContext): ContextProvenance['mode'] | 'synthetic' => {
  // Generated ledger readers use a minimal synthetic context and no factory
  // provenance. They cannot finalize a call or perform Zswap operations.
  const unbranded = !contextProvenance.has(context) && Object.getOwnPropertyDescriptor(context, provenanceSymbol) === undefined;
  const frame = context.callContext as Partial<CircuitCallContext> | undefined;
  if (
    unbranded &&
    frame?.circuitId === undefined &&
    context.queryContexts === undefined &&
    context.gasCosts === undefined &&
    context.zswapLocalStates === undefined &&
    context.stateProvider === undefined &&
    context.moduleProvider === undefined
  ) {
    return 'synthetic';
  }
  const pin = readProvenance(context);
  assertFrameProvenance(context, pin);
  return pin.mode;
};

/** Append the current completed call after a generated circuit returns proof data. */
export const finalizeCallProofData = (context: CircuitContext, proofData: ProofData): void => {
  const frame = context.callContext;
  const address = frame.contractAddress;
  assertLiveCircuitCall(context);
  context.callProofDataTrace = [
    ...context.callProofDataTrace,
    {
      ...proofData,
      circuitId: frame.circuitId,
      contractAddress: address,
      initialQueryContext: frame.initialQueryContext,
      finalQueryContext: frame.currentQueryContext,
      zswapLocalState: frame.currentZswapLocalState,
    },
  ];
};

/**
 * Function for creating an initial running cost of zero.
 *
 * @internal
 */
export const emptyRunningCost = (): ocrt.RunningCost => ({
  readTime: 0n,
  computeTime: 0n,
  bytesWritten: 0n,
  bytesDeleted: 0n,
});

/**
 * The results of the call to a Compact circuit
 */
export interface CircuitResults<PS = any, R = any> {
  /**
   * The primary result, as returned from Compact
   */
  result: R;
  /**
   * The data required to prove this circuit run
   */
  proofData: ProofData;
  /**
   * The updated context after the circuit execution, that can be used to
   * inform further runs
   */
  context: CircuitContext<PS>;
  /**
   * The gas consumption of the circuit execution
   */
  gasCost: ocrt.RunningCost;
}

/**
 * Runs a program (query) against the current ledger state in the given circuit context. Records the transcript in the
 * given partial proof data.
 *
 * @param circuitContext The context for the currently executing circuit.
 * @param partialProofData The partial proof data to insert the query results into.
 * @param program The query to run.
 */
export const queryLedgerState = (
  circuitContext: CircuitContext,
  partialProofData: PartialProofData,
  program: ocrt.Op<null>[],
): ocrt.AlignedValue | ocrt.GatherResult[] | undefined => {
  try {
    // The Runtime 0.18 generated reader has only the top-level cell. Ledger 9
    // circuits use the call frame, which is also the source for per-address maps.
    const mode = queryContextMode(circuitContext);
    const ledger9 = mode === 'ledger9';
    if (ledger9) assertLiveCircuitCall(circuitContext);
    const currentQueryContext = ledger9
      ? circuitContext.callContext.currentQueryContext
      : mode === 'synthetic'
        ? (circuitContext.callContext?.currentQueryContext ?? circuitContext.currentQueryContext)
        : circuitContext.currentQueryContext;
    const res = currentQueryContext.query(program, circuitContext.costModel, circuitContext.gasLimit);
    if (ledger9) assertCircuitQueryIdentity(circuitContext, res.context);
    const frame = circuitContext.callContext;
    const nextGasCost: ocrt.RunningCost | undefined =
      frame && mode !== 'synthetic'
        ? {
            readTime: frame.currentGasCost.readTime + res.gasCost.readTime,
            computeTime: frame.currentGasCost.computeTime + res.gasCost.computeTime,
            bytesWritten: frame.currentGasCost.bytesWritten + res.gasCost.bytesWritten,
            bytesDeleted: frame.currentGasCost.bytesDeleted + res.gasCost.bytesDeleted,
          }
        : undefined;
    const nextFrame =
      frame && mode === 'synthetic'
        ? { ...frame, currentQueryContext: res.context }
        : frame && nextGasCost
          ? { ...frame, currentQueryContext: res.context, currentGasCost: nextGasCost }
          : undefined;
    const nextQueryContexts =
      frame && circuitContext.queryContexts !== undefined
        ? { ...circuitContext.queryContexts, [frame.contractAddress]: res.context }
        : undefined;
    const nextGasCosts =
      frame && nextGasCost && circuitContext.gasCosts !== undefined
        ? { ...circuitContext.gasCosts, [frame.contractAddress]: nextGasCost }
        : undefined;
    // Generated circuits move this internal accumulator to CircuitResults.
    const context = circuitContext as CircuitContext & { gasCost?: ocrt.RunningCost };
    const previous = context.gasCost ?? emptyRunningCost();
    const nextLegacyGasCost = {
      readTime: previous.readTime + res.gasCost.readTime,
      computeTime: previous.computeTime + res.gasCost.computeTime,
      bytesWritten: previous.bytesWritten + res.gasCost.bytesWritten,
      bytesDeleted: previous.bytesDeleted + res.gasCost.bytesDeleted,
    };
    const logs = res.events
      .filter((event) => event.tag === 'log')
      .map((event) => ({ ...event.content, address: frame?.contractAddress ?? res.context.address }));
    const nextEvents = logs.length === 0 ? circuitContext.events : [...(circuitContext.events ?? []), ...logs];
    const reads = res.events.filter((e) => e.tag === 'read');
    let i = 0;
    const nextTranscript = partialProofData.publicTranscript.concat(
      program.map((op) =>
        typeof op === 'object' && 'popeq' in op
          ? {
              popeq: {
                ...op.popeq,
                result: reads[i++].content,
              },
            }
          : op,
      ) as ocrt.Op<ocrt.AlignedValue>[],
    );
    if (ledger9) inheritQuerySourceState(currentQueryContext, res.context);
    // Commit after all identity, transcript and copy checks have succeeded.
    circuitContext.currentQueryContext = res.context;
    if (nextFrame) circuitContext.callContext = nextFrame;
    if (nextQueryContexts) circuitContext.queryContexts = nextQueryContexts;
    if (nextGasCosts) circuitContext.gasCosts = nextGasCosts;
    context.gasCost = nextLegacyGasCost;
    if (logs.length !== 0) circuitContext.events = nextEvents;
    partialProofData.publicTranscript = nextTranscript;
    if (res.events.length === 1) {
      const event = res.events[0];
      if (event.tag === 'read') {
        return event.content;
      }
    }
    return res.events;
  } catch (err) {
    if (err instanceof Error) {
      throw new CompactError(err.toString());
    }
    throw err;
  }
};

/** @internal */
export const emitShieldedSpend = (context: CircuitContext, proofData: PartialProofData, nullifier: Uint8Array): [] => {
  if (nullifier.length !== 32) {
    throw new CompactError('ShieldedSpend nullifier must be Bytes<32>');
  }
  return emitEvent(context, proofData, 0n, nullifier, 32);
};

/** @internal */
export const emitShieldedBurn = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 49) {
    throw new CompactError('ShieldedBurn payload must be Bytes<49>');
  }
  return emitEvent(context, proofData, 3n, payload, 49);
};

/** @internal */
export const emitShieldedMint = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 81) {
    throw new CompactError('ShieldedMint payload must be Bytes<81>');
  }
  return emitEvent(context, proofData, 2n, payload, 81);
};

/** @internal */
export const emitShieldedReceive = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 578) {
    throw new CompactError('ShieldedReceive payload must be Bytes<578>');
  }
  return emitEvent(context, proofData, 1n, payload, 578);
};

/** @internal */
export const emitUnshieldedSpend = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 145) {
    throw new CompactError('UnshieldedSpend payload must be Bytes<145>');
  }
  return emitEvent(context, proofData, 4n, payload, 145);
};

/** @internal */
export const emitUnshieldedReceive = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 145) {
    throw new CompactError('UnshieldedReceive payload must be Bytes<145>');
  }
  return emitEvent(context, proofData, 5n, payload, 145);
};

/** @internal */
export const emitUnshieldedMint = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 80) {
    throw new CompactError('UnshieldedMint payload must be Bytes<80>');
  }
  return emitEvent(context, proofData, 6n, payload, 80);
};

/** @internal */
export const emitUnshieldedBurn = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 113) {
    throw new CompactError('UnshieldedBurn payload must be Bytes<113>');
  }
  return emitEvent(context, proofData, 7n, payload, 113);
};

/** @internal */
export const emitMisc = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 288) {
    throw new CompactError('Misc payload must be Bytes<288>');
  }
  return emitEvent(context, proofData, 10n, payload, 288);
};

/** @internal */
export const emitPaused = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 0) {
    throw new CompactError('Paused payload must be Bytes<0>');
  }
  return emitEvent(context, proofData, 8n, payload, 0);
};

/** @internal */
export const emitUnpaused = (context: CircuitContext, proofData: PartialProofData, payload: Uint8Array): [] => {
  if (payload.length !== 0) {
    throw new CompactError('Unpaused payload must be Bytes<0>');
  }
  return emitEvent(context, proofData, 9n, payload, 0);
};

const emitEvent = (
  context: CircuitContext,
  proofData: PartialProofData,
  tag: bigint,
  payload: Uint8Array,
  length: number,
): [] => {
  const cell = (value: ocrt.Value, length: number): ocrt.EncodedStateValue => ({
    tag: 'cell',
    content: { value, alignment: [{ tag: 'atom', value: { tag: 'bytes', length } }] },
  });
  queryLedgerState(context, proofData, [
    {
      push: {
        storage: false,
        value: {
          tag: 'array',
          content: [
            cell(ocrt.bigIntToValue(1n), 4),
            cell(ocrt.bigIntToValue(tag), 1),
            cell(new CompactTypeBytes(length).toValue(payload), length),
          ],
        },
      },
    },
    'log',
  ]);
  return [];
};
