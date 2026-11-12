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

import { ContractAddress } from '@midnightntwrk/onchain-runtime-v4';
import type { Secp256k1Point } from './compact-types.js';
import { SECP256K1_BASE_MODULUS } from './constants.js';
import { EncodedContractAddress } from './zswap.js';

/**
 * Regex matching hex strings of even length.
 */
export const HEX_REGEX_NO_PREFIX = /^([0-9A-Fa-f]{2})*$/;

/**
 * The expected length (in bytes) of a contract address.
 */
export const CONTRACT_ADDRESS_BYTE_LENGTH = 32;

/**
 * Tests whether the input value is a {@link ContractAddress}, i.e., string.
 *
 * @param x The value that is tested to be a {@link ContractAddress}.
 */
export function isContractAddress(x: unknown): x is ContractAddress {
  return typeof x === 'string' && x.length === CONTRACT_ADDRESS_BYTE_LENGTH * 2 && HEX_REGEX_NO_PREFIX.test(x);
}

export function isEncodedContractAddress(x: unknown): x is EncodedContractAddress {
  return (
    typeof x === 'object' &&
    x !== null &&
    x !== undefined &&
    'bytes' in x &&
    x.bytes instanceof Uint8Array &&
    x.bytes.length == CONTRACT_ADDRESS_BYTE_LENGTH
  );
}

/** Checks a secp256k1 point supplied to a generated circuit. */
export function isValidSecp256k1Point(value: unknown): value is Secp256k1Point {
  if (typeof value !== 'object' || value === null || !('x' in value) || !('y' in value) || !('identity' in value)) {
    return false;
  }
  const { x, y, identity } = value;
  const modulus = SECP256K1_BASE_MODULUS;
  if (
    typeof x !== 'bigint' ||
    x < 0n ||
    x >= modulus ||
    typeof y !== 'bigint' ||
    y < 0n ||
    y >= modulus ||
    typeof identity !== 'boolean'
  ) {
    return false;
  }
  return identity || (y * y) % modulus === (((x * x) % modulus) * x + 7n) % modulus;
}

export const fromHex = (s: string): Uint8Array => Buffer.from(s, 'hex');

export const toHex = (s: Uint8Array): string => Buffer.from(s).toString('hex');
