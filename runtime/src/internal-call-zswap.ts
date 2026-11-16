// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

/** Replacement-cell Zswap operations for the internal per-address call trace. */
import { Bytes32Descriptor } from './compact-types.js';
import { CompactError } from './error.js';
import { createCoinCommitment } from './internal-coin-commitment.js';
import {
  advanceInternalCallTraceContext,
  assertInternalCallTraceLiveCells,
  type InternalCallTraceContext,
} from './internal-call-trace-context.js';
import type {
  EncodedCoinPublicKey,
  EncodedQualifiedShieldedCoinInfo,
  EncodedRecipient,
  EncodedShieldedCoinInfo,
  EncodedZswapLocalState,
} from './zswap.js';

const ownValue = <T>(object: object, key: string): T => {
  const descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (!descriptor || !('value' in descriptor)) {
    throw new CompactError(`Encoded Zswap ${key} must be an own data field`);
  }
  return descriptor.value as T;
};

const copyBytes = (bytes: Uint8Array): Uint8Array => {
  if (!(bytes instanceof Uint8Array)) throw new CompactError('Encoded Zswap bytes must be a Uint8Array');
  return Uint8Array.prototype.slice.call(bytes) as Uint8Array;
};

const copyDense = <T, R>(items: T[], copy: (item: T) => R): R[] => {
  if (!Array.isArray(items)) throw new CompactError('Encoded Zswap items must be an array');
  const result: R[] = [];
  for (let index = 0; index < items.length; index++) {
    result.push(copy(ownValue<T>(items, String(index))));
  }
  return result;
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

const copyZswap = (state: EncodedZswapLocalState): EncodedZswapLocalState => ({
  coinPublicKey: { bytes: copyBytes(ownValue(ownValue(state, 'coinPublicKey'), 'bytes')) },
  currentIndex: ownValue(state, 'currentIndex'),
  inputs: copyDense(ownValue<EncodedQualifiedShieldedCoinInfo[]>(state, 'inputs'), copyQualifiedCoin),
  outputs: copyDense(ownValue<EncodedZswapLocalState['outputs']>(state, 'outputs'), (output) => ({
    coinInfo: copyCoin(ownValue<EncodedShieldedCoinInfo>(output, 'coinInfo')),
    recipient: copyRecipient(ownValue<EncodedRecipient>(output, 'recipient')),
  })),
});

const currentState = (context: InternalCallTraceContext): EncodedZswapLocalState => {
  assertInternalCallTraceLiveCells(context);
  const current = context.callContext.currentZswapLocalState;
  if (current === undefined || current === null) {
    throw new CompactError(`Zswap local state is undefined for contract '${context.callContext.contractAddress}'`);
  }
  return current;
};

/** Record an encoded input on a copied call trace without aliasing caller-owned bytes. */
export const appendInternalZswapInput = (
  context: InternalCallTraceContext,
  coin: EncodedQualifiedShieldedCoinInfo,
): InternalCallTraceContext => {
  const previous = currentState(context);
  const next = copyZswap(previous);
  next.inputs.push(copyQualifiedCoin(coin));
  return advanceInternalCallTraceContext(
    context,
    context.callContext.currentQueryContext,
    context.callContext.currentGasCost,
    next,
  );
};

/** Insert the real SDK coin commitment and record one encoded output atomically. */
export const appendInternalZswapOutput = (
  context: InternalCallTraceContext,
  coin: EncodedShieldedCoinInfo,
  recipient: EncodedRecipient,
): InternalCallTraceContext => {
  const previous = currentState(context);
  const next = copyZswap(previous);
  const ownedCoin = copyCoin(coin);
  const ownedRecipient = copyRecipient(recipient);
  const commitment = createCoinCommitment(ownedCoin, ownedRecipient);
  const commitmentHex = Buffer.from(Bytes32Descriptor.fromValue(commitment.value)).toString('hex');
  const nextQuery = context.callContext.currentQueryContext.insertCommitment(commitmentHex, previous.currentIndex);
  next.currentIndex += 1n;
  next.outputs.push({ coinInfo: ownedCoin, recipient: ownedRecipient });
  return advanceInternalCallTraceContext(context, nextQuery, context.callContext.currentGasCost, next);
};

/** Return a defensive submitter-key copy for the currently executing address. */
export const ownInternalZswapPublicKey = (context: InternalCallTraceContext): EncodedCoinPublicKey => ({
  bytes: copyBytes(ownValue(ownValue(currentState(context), 'coinPublicKey'), 'bytes')),
});
