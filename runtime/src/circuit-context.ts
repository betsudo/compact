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
} from './zswap.js';
import { PartialProofData, ProofData } from './proof-data.js';
import { CompactError } from './error.js';
import { CompactTypeBytes } from './compact-types.js';

/** A Ledger VM log with the address of the emitting contract. */
export type LogEvent = Extract<ocrt.GatherResult, { tag: 'log' }>['content'] & {
  address: ocrt.ContractAddress;
};

/**
 * The external information accessible from within a Compact circuit call
 */
export interface CircuitContext<PS = any> {
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
  events?: readonly LogEvent[];
}

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
  time?: number,
): ocrt.QueryContext => {
  const initialQueryContext = new ocrt.QueryContext(coerceToChargedState(contractState), contractAddress);
  const balance = contractState instanceof ocrt.ContractState ? contractState.balance : new Map();
  initialQueryContext.block = {
    ...initialQueryContext.block,
    balance,
    ownAddress: contractAddress,
    secondsSinceEpoch: BigInt(time ?? Math.floor(Date.now() / 1_000)),
  };
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

export const createCircuitContext = <PS>(
  contractAddress: ocrt.ContractAddress,
  coinPublicKeyOrZswapState: ocrt.CoinPublicKey | EncodedCoinPublicKey | ZswapLocalState | EncodedZswapLocalState,
  contractState: ocrt.ContractState | ocrt.StateValue | ocrt.ChargedState,
  privateState: PS,
  gasLimit?: ocrt.RunningCost,
  costModel?: ocrt.CostModel,
  time?: number,
): CircuitContext<PS> => {
  const initialQueryContext = createInitialQueryContext(contractState, contractAddress, time);

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

  return {
    currentPrivateState: privateState,
    currentZswapLocalState: zswapLocalState,
    currentQueryContext: initialQueryContext,
    costModel: costModel ?? ocrt.CostModel.initialCostModel(),
    gasLimit,
    events: [],
  };
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
): ocrt.AlignedValue | ocrt.GatherResult[] => {
  try {
    const res = circuitContext.currentQueryContext.query(program, circuitContext.costModel, circuitContext.gasLimit);
    circuitContext.currentQueryContext = res.context;
    // Generated circuits move this internal accumulator to CircuitResults.
    const context = circuitContext as CircuitContext & { gasCost?: ocrt.RunningCost };
    const previous = context.gasCost ?? emptyRunningCost();
    context.gasCost = {
      readTime: previous.readTime + res.gasCost.readTime,
      computeTime: previous.computeTime + res.gasCost.computeTime,
      bytesWritten: previous.bytesWritten + res.gasCost.bytesWritten,
      bytesDeleted: previous.bytesDeleted + res.gasCost.bytesDeleted,
    };
    const logs = res.events
      .filter((event) => event.tag === 'log')
      .map((event) => ({ ...event.content, address: res.context.address }));
    if (logs.length !== 0) {
      circuitContext.events = [...(circuitContext.events ?? []), ...logs];
    }
    const reads = res.events.filter((e) => e.tag === 'read');
    let i = 0;
    partialProofData.publicTranscript = partialProofData.publicTranscript.concat(
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

const emitEvent = (context: CircuitContext, proofData: PartialProofData, tag: bigint, payload: Uint8Array, length: number): [] => {
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
