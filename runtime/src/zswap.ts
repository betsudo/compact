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
import { assertCircuitQueryIdentity, assertLiveCircuitCall, type CircuitContext } from './circuit-context.js';
import { Bytes32Descriptor, Recipient } from './compact-types.js';
import { createCoinCommitment } from './internal-coin-commitment.js';
import { CompactError } from './error.js';
import { toHex } from './utils.js';

/**
 * Tracks the coins consumed and produced throughout circuit execution.
 */
export interface ZswapLocalState {
  /**
   * The Zswap coin public key of the user executing the circuit.
   */
  coinPublicKey: ocrt.CoinPublicKey;
  /**
   * The Merkle tree index of the next coin produced.
   */
  currentIndex: bigint;
  /**
   * The coins consumed as inputs to the circuit.
   */
  inputs: ocrt.QualifiedShieldedCoinInfo[];
  /**
   * The coins produced as outputs from the circuit.
   */
  outputs: {
    coinInfo: ocrt.ShieldedCoinInfo;
    recipient: Recipient;
  }[];
}

/**
 * A {@link CoinPublicKey} encoded as a byte string. This representation is used internally by the contract executable.
 */
export interface EncodedCoinPublicKey {
  /**
   * The coin public key's bytes.
   */
  readonly bytes: Uint8Array;
}

/**
 * A {@link ContractAddress} encoded as a byte string. This representation is used internally by the contract executable.
 */
export interface EncodedContractAddress {
  /**
   * The contract address's bytes.
   */
  readonly bytes: Uint8Array;
}

/**
 * A {@link ShieldedCoinInfo} with its fields encoded as byte strings. This representation is used internally by
 * the contract executable.
 */
export interface EncodedShieldedCoinInfo {
  /**
   * The coin's randomness, preventing it from colliding with other coins.
   */
  readonly nonce: Uint8Array;
  /**
   * The coin's type, identifying the currency it represents.
   */
  readonly color: Uint8Array;
  /**
   * The coin's value, in atomic units dependent on the currency. Bounded to be a non-negative 128-bit integer.
   */
  readonly value: bigint;
}

/**
 * A {@link QualifiedCoinInfo} with its fields encoded as byte strings. This representation is used internally by
 * the contract executable.
 */
export interface EncodedQualifiedShieldedCoinInfo extends EncodedShieldedCoinInfo {
  /**
   * The coin's location in the chain's Merkle tree of coin commitments. Bounded to be a non-negative 64-bit integer.
   */
  readonly mt_index: bigint;
}

/**
 * A {@link Recipient} with its fields encoded as byte strings. This representation is used internally by the contract executable.
 */
export interface EncodedRecipient {
  /**
   * Whether the recipient is a user or a contract.
   */
  readonly is_left: boolean;
  /**
   * The recipient's public key, if the recipient is a user.
   */
  readonly left: EncodedCoinPublicKey;
  /**
   * The recipient's contract address, if the recipient is a contract.
   */
  readonly right: EncodedContractAddress;
}

/**
 * Tracks the coins consumed and produced throughout circuit execution.
 */
export interface EncodedZswapLocalState {
  /**
   * The Zswap coin public key of the user executing the circuit.
   */
  coinPublicKey: EncodedCoinPublicKey;
  /**
   * The Merkle tree index of the next coin produced.
   */
  currentIndex: bigint;
  /**
   * The coins consumed as inputs to the circuit.
   */
  inputs: EncodedQualifiedShieldedCoinInfo[];
  /**
   * The coins produced as outputs from the circuit.
   */
  outputs: {
    coinInfo: EncodedShieldedCoinInfo;
    recipient: EncodedRecipient;
  }[];
}

const ownValue = <T>(object: object, key: string): T => {
  const descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (!descriptor || !('value' in descriptor)) {
    throw new CompactError(`Encoded Zswap ${key} must be an own data field`);
  }
  return descriptor.value as T;
};

const typedArrayLength = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype), 'length')!.get!;
const typedArraySet = Uint8Array.prototype.set;

const copyBytes = (bytes: Uint8Array): Uint8Array => {
  if (!(bytes instanceof Uint8Array)) throw new CompactError('Encoded Zswap bytes must be a Uint8Array');
  let length: number;
  try {
    length = typedArrayLength.call(bytes) as number;
  } catch {
    throw new CompactError('Encoded Zswap bytes must have a valid typed-array buffer');
  }
  const owned = new Uint8Array(length);
  typedArraySet.call(owned, bytes);
  return owned;
};

const copyDense = <T, R>(items: T[], copy: (item: T) => R): R[] => {
  if (!Array.isArray(items)) throw new CompactError('Encoded Zswap items must be an array');
  const owned: R[] = [];
  for (let index = 0; index < items.length; index++) {
    owned.push(copy(ownValue<T>(items, String(index))));
  }
  return owned;
};

const copyCoin = (coin: EncodedShieldedCoinInfo): EncodedShieldedCoinInfo => ({
  nonce: copyBytes(ownValue(coin, 'nonce')),
  color: copyBytes(ownValue(coin, 'color')),
  value: ownValue(coin, 'value'),
});

const copyQualifiedCoin = (coin: EncodedQualifiedShieldedCoinInfo): EncodedQualifiedShieldedCoinInfo => ({
  ...copyCoin(coin),
  mt_index: ownValue(coin, 'mt_index'),
});

const copyRecipient = (recipient: EncodedRecipient): EncodedRecipient => ({
  is_left: ownValue(recipient, 'is_left'),
  left: { bytes: copyBytes(ownValue(ownValue(recipient, 'left'), 'bytes')) },
  right: { bytes: copyBytes(ownValue(ownValue(recipient, 'right'), 'bytes')) },
});

/** Defensive byte and array copy for Ledger 9 context cells. */
export const copyEncodedZswapLocalState = (state: EncodedZswapLocalState): EncodedZswapLocalState => ({
  coinPublicKey: { bytes: copyBytes(ownValue(ownValue(state, 'coinPublicKey'), 'bytes')) },
  currentIndex: ownValue(state, 'currentIndex'),
  inputs: copyDense(ownValue<EncodedQualifiedShieldedCoinInfo[]>(state, 'inputs'), copyQualifiedCoin),
  outputs: copyDense(ownValue<EncodedZswapLocalState['outputs']>(state, 'outputs'), (output) => ({
    coinInfo: copyCoin(ownValue(output, 'coinInfo')),
    recipient: copyRecipient(ownValue(output, 'recipient')),
  })),
});

/**
 * Constructs a new {@link EncodedZswapLocalState} with the given coin public key. The result can be used to create a
 * {@link ConstructorContext}.
 *
 * @param coinPublicKey The Zswap coin public key of the user executing the circuit.
 */
export const emptyZswapLocalState = (coinPublicKey: ocrt.CoinPublicKey | EncodedCoinPublicKey): EncodedZswapLocalState => ({
  coinPublicKey: typeof coinPublicKey === 'string' ? { bytes: ocrt.encodeCoinPublicKey(coinPublicKey) } : coinPublicKey,
  currentIndex: 0n,
  inputs: [],
  outputs: [],
});

/**
 * Converts an {@link Recipient} to an {@link EncodedRecipient}. Useful for testing.
 */
export const encodeRecipient = ({ is_left, left, right }: Recipient): EncodedRecipient => ({
  is_left,
  left: { bytes: ocrt.encodeCoinPublicKey(left) },
  right: { bytes: ocrt.encodeContractAddress(right) },
});

/**
 * Converts an {@link EncodedRecipient} to a {@link Recipient}.
 */
export const decodeRecipient = ({ is_left, left, right }: EncodedRecipient): Recipient => ({
  is_left,
  left: ocrt.decodeCoinPublicKey(left.bytes),
  right: ocrt.decodeContractAddress(right.bytes),
});

/**
 * Converts a {@link ZswapLocalState} to an {@link EncodedZswapLocalState}. Useful for testing.
 *
 * @param state The decoded Zswap local state.
 */
export const encodeZswapLocalState = (state: ZswapLocalState): EncodedZswapLocalState => ({
  coinPublicKey: { bytes: ocrt.encodeCoinPublicKey(state.coinPublicKey) },
  currentIndex: state.currentIndex,
  inputs: state.inputs.map(ocrt.encodeQualifiedShieldedCoinInfo),
  outputs: state.outputs.map(({ coinInfo, recipient }) => ({
    coinInfo: ocrt.encodeShieldedCoinInfo(coinInfo),
    recipient: encodeRecipient(recipient),
  })),
});

/**
 * Converts an {@link EncodedZswapLocalState} to a {@link ZswapLocalState}. Used when we need to use data from contract
 * execution to construct transactions.
 *
 * @param state The encoded Zswap local state.
 */
export const decodeZswapLocalState = (state: EncodedZswapLocalState): ZswapLocalState => ({
  coinPublicKey: ocrt.decodeCoinPublicKey(state.coinPublicKey.bytes),
  currentIndex: state.currentIndex,
  inputs: state.inputs.map(ocrt.decodeQualifiedShieldedCoinInfo),
  outputs: state.outputs.map(({ coinInfo, recipient }) => ({
    coinInfo: ocrt.decodeShieldedCoinInfo(coinInfo),
    recipient: decodeRecipient(recipient),
  })),
});

/**
 * Adds a coin to the list of inputs consumed by the circuit.
 *
 * @param circuitContext The current circuit context.
 * @param qualifiedShieldedCoinInfo The input to consume.
 */
export function createZswapInput(
  circuitContext: CircuitContext,
  qualifiedShieldedCoinInfo: EncodedQualifiedShieldedCoinInfo,
): [] {
  const current = circuitContext.callContext?.circuitId
    ? circuitContext.callContext.currentZswapLocalState
    : circuitContext.currentZswapLocalState;
  const ledger9 = Boolean(circuitContext.callContext?.circuitId);
  if (ledger9) assertLiveCircuitCall(circuitContext);
  const next = ledger9 ? copyEncodedZswapLocalState(current) : { ...current, inputs: [...current.inputs] };
  next.inputs.push(ledger9 ? copyQualifiedCoin(qualifiedShieldedCoinInfo) : qualifiedShieldedCoinInfo);
  circuitContext.currentZswapLocalState = next;
  if (circuitContext.callContext !== undefined) {
    circuitContext.callContext = { ...circuitContext.callContext, currentZswapLocalState: next };
    circuitContext.zswapLocalStates = {
      ...circuitContext.zswapLocalStates,
      [circuitContext.callContext.contractAddress]: next,
    };
  }
  return [];
}

/**
 * Adds a coin to the list of outputs produced by the circuit.
 *
 * @param circuitContext The current circuit context.
 * @param coinInfo The coin to produce.
 * @param recipient The coin recipient - either a coin public key representing an end user or a contract address
 *                  representing a contract.
 */
export function createZswapOutput(
  circuitContext: CircuitContext<unknown>,
  coinInfo: EncodedShieldedCoinInfo,
  recipient: EncodedRecipient,
): [] {
  const ledger9 = Boolean(circuitContext.callContext?.circuitId);
  if (ledger9) assertLiveCircuitCall(circuitContext);
  const currentQueryContext = ledger9 ? circuitContext.callContext.currentQueryContext : circuitContext.currentQueryContext;
  const currentZswapLocalState = ledger9
    ? circuitContext.callContext.currentZswapLocalState
    : circuitContext.currentZswapLocalState;
  const ownedCoin = ledger9 ? copyCoin(coinInfo) : coinInfo;
  const ownedRecipient = ledger9 ? copyRecipient(recipient) : recipient;
  const nextZswapLocalState = ledger9
    ? copyEncodedZswapLocalState(currentZswapLocalState)
    : { ...currentZswapLocalState, outputs: [...currentZswapLocalState.outputs] };
  const nextQueryContext = currentQueryContext.insertCommitment(
    Buffer.from(Bytes32Descriptor.fromValue(createCoinCommitment(ownedCoin, ownedRecipient).value)).toString('hex'),
    currentZswapLocalState.currentIndex,
  );
  if (ledger9) assertCircuitQueryIdentity(circuitContext.callContext, nextQueryContext);
  nextZswapLocalState.currentIndex += 1n;
  nextZswapLocalState.outputs.push({ coinInfo: ownedCoin, recipient: ownedRecipient });
  circuitContext.currentQueryContext = nextQueryContext;
  circuitContext.currentZswapLocalState = nextZswapLocalState;
  // Replacement cells keep a shallow-copied input context as a snapshot.
  if (circuitContext.callContext !== undefined) {
    circuitContext.callContext = {
      ...circuitContext.callContext,
      currentQueryContext: nextQueryContext,
      currentZswapLocalState: nextZswapLocalState,
    };
    const address = circuitContext.callContext.contractAddress;
    circuitContext.queryContexts = { ...circuitContext.queryContexts, [address]: nextQueryContext };
    circuitContext.zswapLocalStates = { ...circuitContext.zswapLocalStates, [address]: nextZswapLocalState };
  }
  return [];
}

/**
 * Retrieves the Zswap coin public key of the user executing the circuit.
 *
 * @param circuitContext The current circuit context.
 */
export function ownPublicKey(circuitContext: CircuitContext<unknown>): EncodedCoinPublicKey {
  if (circuitContext.callContext?.circuitId) {
    assertLiveCircuitCall(circuitContext);
    return { bytes: copyBytes(circuitContext.callContext.currentZswapLocalState.coinPublicKey.bytes) };
  }
  return circuitContext.currentZswapLocalState.coinPublicKey;
}

/**
 * Checks whether a coin commitment has already been added to the current query context.
 *
 * @param context The current circuit context.
 * @param coinInfo The coin information to check.
 * @param recipient The coin recipient to check.
 */
export const hasCoinCommitment = (
  context: CircuitContext,
  coinInfo: EncodedShieldedCoinInfo,
  recipient: EncodedRecipient,
): boolean => {
  if (context.callContext?.circuitId) assertLiveCircuitCall(context);
  return (context.callContext?.circuitId ? context.callContext.currentQueryContext : context.currentQueryContext).comIndices.has(
    toHex(Bytes32Descriptor.fromValue(createCoinCommitment(coinInfo, recipient).value)),
  );
};
