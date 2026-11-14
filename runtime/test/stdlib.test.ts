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

import { describe, expect, test } from 'vitest';
import { keccak_256 } from '@noble/hashes/sha3.js';
import * as compactRuntime from '../src/index.js';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';

describe('createCoinCommitment', () => {
  test('Check for success', () => {
    const context = compactRuntime.createCircuitContext(
      ocrt.sampleContractAddress(),
      '0'.repeat(64),
      new ocrt.ContractState(),
      undefined,
    );
    const coinInfo = {
      tag: 'shielded',
      type: ocrt.sampleRawTokenType(),
      nonce: '2ab78b2272ec3489da60e6af54a87bfa53a7fa727602a040df782ebae7f5ab59',
      value: 572290297060094569n,
    };
    const recipient = {
      is_left: false,
      left: { bytes: new Uint8Array(32) },
      right: { bytes: ocrt.encodeContractAddress(ocrt.sampleContractAddress()) },
    };
    compactRuntime.createZswapOutput(context, ocrt.encodeShieldedCoinInfo(coinInfo), recipient);
    expect(context.currentZswapLocalState.outputs.length).toBe(1);
  });
});

describe('CompactError', () => {
  const msg = 'my message';

  const f = () => {
    throw new compactRuntime.CompactError(msg);
  };

  test('Check for error type resolution', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('Check for error message', () => {
    expect(f).toThrow(msg);
  });
});

describe('__compact.assert', () => {
  const msg = 'my message';

  const f = () => {
    compactRuntime.assert(false, msg);
  };

  test('Check for success', () => {
    expect(compactRuntime.assert(true, msg)).toEqual([]);
  });

  test('Check for error type', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('Check for error message', () => {
    expect(f).toThrow(msg);
  });
});

describe('__compact.typeError', () => {
  const f = () => {
    compactRuntime.typeError('who', 'what', 'where', 'type', 'x');
  };

  test('Check for error type', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('Check for error message', () => {
    expect(f).toThrow(/type error/);
  });
});

describe('__compact.convertFieldToBytes', () => {
  const x = 256n;

  test('Check for success', () => {
    const a = compactRuntime.convertFieldToBytes(2, x, 'source');
    expect(a).toEqual(new Uint8Array([0, 1]));
  });

  const f = () => {
    compactRuntime.convertFieldToBytes(1, x, 'source');
  };

  test('Check for error type', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('Check for error message', () => {
    expect(f).toThrow(/range error/);
  });
});

describe('__compact.convertBytesToField', () => {
  test('Check for success', () => {
    const a = new Uint8Array([0, 1]);
    const x = compactRuntime.convertBytesToField(a.length, a, 'source');
    expect(x).toBe(256n);
  });

  const f = () => {
    const a = new Uint8Array(57);
    a[56] = 1;
    compactRuntime.convertBytesToField(57, a, 'source');
  };

  test('check for error type', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('check for error message', () => {
    expect(f).toThrow(/range error/);
  });
});

describe('__compact.convertBytesToUint', () => {
  test('Check for success', () => {
    const a = new Uint8Array([0xff, 0]);
    const x = compactRuntime.convertBytesToUint(255n, a.length, a, 'source');
    expect(x).toBe(255n);
  });

  const f = () => {
    const a = new Uint8Array([0, 1]);
    compactRuntime.convertBytesToUint(255n, a.length, a, 'source');
  };

  test('check for error type', () => {
    expect(f).toThrow(compactRuntime.CompactError);
  });

  test('check for error message', () => {
    expect(f).toThrow(/range error/);
  });
});

describe('builtin hash functions', () => {
  test('transientHash', () => {
    expect(typeof compactRuntime.transientHash(compactRuntime.CompactTypeField, 5n)).toEqual('bigint');
  });

  test('persistentHash', () => {
    const res = compactRuntime.persistentHash(compactRuntime.CompactTypeField, 5n);
    expect(res).toBeInstanceOf(Uint8Array);
    expect(res.length).toBe(32);
  });

  test('keccak256', () => {
    const res = compactRuntime.keccak256(compactRuntime.CompactTypeField, 5n);
    expect(res).toBeInstanceOf(Uint8Array);
    expect(res.length).toBe(32);
    // keccak256 must be deterministic
    expect(compactRuntime.keccak256(compactRuntime.CompactTypeField, 5n)).toEqual(res);
    // distinct inputs must produce distinct outputs
    expect(compactRuntime.keccak256(compactRuntime.CompactTypeField, 6n)).not.toEqual(res);
  });

  // Cases from upstream test-contracts/stdlib/keccak256/trailing_zero at
  // 672bd0f226f47a7d036c5c074ce71fbe1af1925e.
  test.each([
    {
      label: 'no trailing zero',
      input: Uint8Array.from({ length: 12 }, (_, i) => i + 1),
      pinned: 'e232c64e7136218f5a7d8d41b16bf3f22e74085032aaca77baa5c6c50cc4ca0e',
    },
    {
      label: 'one trailing zero',
      input: Uint8Array.from({ length: 12 }, (_, i) => (i === 11 ? 0 : i + 1)),
      pinned: 'ed6e483c71bb1e667911203acc659782cc5ad0fb03f78a57febbe599b6a21e8d',
    },
    {
      label: 'four trailing zeros',
      input: Uint8Array.from({ length: 12 }, (_, i) => (i < 8 ? i + 1 : 0)),
    },
    {
      label: 'eleven trailing zeros',
      input: Uint8Array.from({ length: 12 }, (_, i) => (i === 0 ? 1 : 0)),
    },
    {
      label: 'all twelve bytes zero',
      input: new Uint8Array(12),
      pinned: '30e2bfdaad2f3c218a1a8cc54fa1c4e6182b6b7f3bca273390cf587b50b47311',
    },
    {
      label: 'interior zero only',
      input: Uint8Array.from({ length: 12 }, (_, i) => (i === 3 ? 0 : i + 1)),
    },
    {
      label: 'eleven leading zeros',
      input: Uint8Array.from({ length: 12 }, (_, i) => (i === 11 ? 0xff : 0)),
    },
  ])('keccak256Aligned retains $label', ({ input, pinned }) => {
    const rtType = new compactRuntime.CompactTypeBytes(12);
    const expected = keccak_256(input);
    if (pinned !== undefined) {
      expect(Buffer.from(expected).toString('hex')).toBe(pinned);
    }
    if (input[input.length - 1] === 0) {
      expect(compactRuntime.keccak256(rtType, input)).not.toEqual(expected);
    }
    expect(compactRuntime.keccak256Aligned(rtType, input)).toEqual(expected);
  });

  test('keccak256Aligned uses a full little-endian Field and refuses opaque atoms', () => {
    const input = new Uint8Array(32);
    input[1] = 1;
    expect(compactRuntime.keccak256Aligned(compactRuntime.CompactTypeField, 256n)).toEqual(keccak_256(input));
    expect(() => compactRuntime.keccak256Aligned(compactRuntime.CompactTypeOpaqueString, 'opaque')).toThrow(
      'invalid Compact hash input',
    );
  });

  test('transientCommit', () => {
    expect(
      typeof compactRuntime.transientCommit(
        new compactRuntime.CompactTypeVector(5, compactRuntime.CompactTypeField),
        [1n, 2n, 3n, 4n, 5n],
        42n,
      ),
    ).toEqual('bigint');
  });

  test('persistentCommit', () => {
    const res = compactRuntime.persistentCommit(
      new compactRuntime.CompactTypeVector(5, compactRuntime.CompactTypeField),
      [1n, 2n, 3n, 4n, 5n],
      new Uint8Array(32),
    );
    expect(res).toBeInstanceOf(Uint8Array);
    expect(res.length).toBe(32);
  });

  test('hashToCurve', () => {
    const res = compactRuntime.hashToCurve(new compactRuntime.CompactTypeVector(5, compactRuntime.CompactTypeField), [
      1n,
      2n,
      3n,
      4n,
      5n,
    ]);
    expect(typeof res.x).toEqual('bigint');
    expect(typeof res.y).toEqual('bigint');
  });

  test('elliptic curve arithmetic', () => {
    // testing that x * g + y * (g + g) == (x + 2y) * g
    // for x = 42, y = 12
    const g = compactRuntime.ecMulGenerator(1n);
    const lhs = compactRuntime.ecAdd(compactRuntime.ecMulGenerator(42n), compactRuntime.ecMul(compactRuntime.ecAdd(g, g), 12n));
    const rhs = compactRuntime.ecMulGenerator(42n + 2n * 12n);
    expect(lhs).toEqual(rhs);
    expect(typeof lhs.x).toEqual('bigint');
    expect(typeof lhs.y).toEqual('bigint');
  });

  test('elliptic curve negation', () => {
    const g = compactRuntime.ecMulGenerator(1n);
    const neg_g = compactRuntime.ecNeg(g);
    // neg(x, y) = (FIELD_MODULUS - x, y)
    expect(neg_g.x).toEqual(compactRuntime.MAX_FIELD + 1n - g.x);
    expect(neg_g.y).toEqual(g.y);
    // neg(neg(g)) == g
    expect(compactRuntime.ecNeg(neg_g)).toEqual(g);
    // g + neg(g) should equal the identity point (0, 1)
    const sum = compactRuntime.ecAdd(g, neg_g);
    expect(sum).toEqual({ x: 0n, y: 1n });
  });
});

test('sanity check for contract address utilities', () => {
  const address = ocrt.sampleContractAddress();
  expect(compactRuntime.fromHex(address).length).toEqual(compactRuntime.CONTRACT_ADDRESS_BYTE_LENGTH);
  expect(compactRuntime.isContractAddress(address)).toBe(true);
  const encodedAddress = { bytes: compactRuntime.fromHex(address) };
  expect(compactRuntime.isEncodedContractAddress(encodedAddress)).toBe(true);

  const bogusAddress = '098230498';
  expect(compactRuntime.isContractAddress(bogusAddress)).toBe(false);
  const encodedBogusAddress = { bytes: compactRuntime.fromHex(bogusAddress) };
  expect(compactRuntime.isEncodedContractAddress(encodedBogusAddress)).toBe(false);
});
