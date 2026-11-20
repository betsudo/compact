// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

export const upstreamCallbacks = {
  "decode"(test, expect, startContract, contractCode, runtime) {
    test('Bytes to secp256r1 fields casts', async () => {
      const [contract, context] = await startContract(contractCode, {}, 0);
      // Random values in range.
      var base = 0x6c8a8c3f071f4d6dbe937aaf1a1d457dc5ab2e9e188e6a5c3190ea1072df9a97n;
      var scalar = 0x78db8cdf5ff2bc906d349cd5b11384283b39a67a9ed2739a2428dfd814c6e43dn;
      var baseBytes = new Uint8Array([
        0x97, 0x9a, 0xdf, 0x72, 0x10, 0xea, 0x90, 0x31,
        0x5c, 0x6a, 0x8e, 0x18, 0x9e, 0x2e, 0xab, 0xc5,
        0x7d, 0x45, 0x1d, 0x1a, 0xaf, 0x7a, 0x93, 0xbe,
        0x6d, 0x4d, 0x1f, 0x07, 0x3f, 0x8c, 0x8a, 0x6c,
      ]);
      var scalarBytes = new Uint8Array([
        0x3d, 0xe4, 0xc6, 0x14, 0xd8, 0xdf, 0x28, 0x24,
        0x9a, 0x73, 0xd2, 0x9e, 0x7a, 0xa6, 0x39, 0x3b,
        0x28, 0x84, 0x13, 0xb1, 0xd5, 0x9c, 0x34, 0x6d,
        0x90, 0xbc, 0xf2, 0x5f, 0xdf, 0x8c, 0xdb, 0x78,
      ]);
      var r = await contract.circuits.test(context, baseBytes, scalarBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(base);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalar);
      base = 0n;
      scalar = runtime.MAX_SECP256R1_SCALAR;
      baseBytes = new Uint8Array(32);  // Initialized to zeros.
      scalarBytes = new Uint8Array([
        0x50, 0x25, 0x63, 0xfc, 0xc2, 0xca, 0xb9, 0xf3,
        0x84, 0x9e, 0x17, 0xa7, 0xad, 0xfa, 0xe6, 0xbc,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
      ]);
      r = await contract.circuits.test(context, baseBytes, scalarBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(base);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalar);
      base = runtime.MAX_SECP256R1_BASE;
      scalar = 0n;
      baseBytes = new Uint8Array([
        0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
      ]);
      scalarBytes = new Uint8Array(32);
      r = await contract.circuits.test(context, baseBytes, scalarBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(base);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalar);
    });
  },
  "encode"(test, expect, startContract, contractCode, runtime) {
    test('secp256r1 fields to Bytes casts', async () => {
      const [contract, context] = await startContract(contractCode, {}, 0);
      // Random values in range.
      var base = 0x6c8a8c3f071f4d6dbe937aaf1a1d457dc5ab2e9e188e6a5c3190ea1072df9a97n;
      var scalar = 0x78db8cdf5ff2bc906d349cd5b11384283b39a67a9ed2739a2428dfd814c6e43dn;
      var baseBytes = new Uint8Array([
        0x97, 0x9a, 0xdf, 0x72, 0x10, 0xea, 0x90, 0x31,
        0x5c, 0x6a, 0x8e, 0x18, 0x9e, 0x2e, 0xab, 0xc5,
        0x7d, 0x45, 0x1d, 0x1a, 0xaf, 0x7a, 0x93, 0xbe,
        0x6d, 0x4d, 0x1f, 0x07, 0x3f, 0x8c, 0x8a, 0x6c,
      ]);
      var scalarBytes = new Uint8Array([
        0x3d, 0xe4, 0xc6, 0x14, 0xd8, 0xdf, 0x28, 0x24,
        0x9a, 0x73, 0xd2, 0x9e, 0x7a, 0xa6, 0x39, 0x3b,
        0x28, 0x84, 0x13, 0xb1, 0xd5, 0x9c, 0x34, 0x6d,
        0x90, 0xbc, 0xf2, 0x5f, 0xdf, 0x8c, 0xdb, 0x78,
      ]);
      var r = await contract.circuits.test(context, base, scalar);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(baseBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalarBytes);
      base = 0n;
      scalar = runtime.MAX_SECP256R1_SCALAR;
      baseBytes = new Uint8Array(32);  // Initialized to zeros.
      scalarBytes = new Uint8Array([
        0x50, 0x25, 0x63, 0xfc, 0xc2, 0xca, 0xb9, 0xf3,
        0x84, 0x9e, 0x17, 0xa7, 0xad, 0xfa, 0xe6, 0xbc,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
      ]);
      r = await contract.circuits.test(context, base, scalar);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(baseBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalarBytes);
      base = runtime.MAX_SECP256R1_BASE;
      scalar = 0n;
      baseBytes = new Uint8Array([
        0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
      ]);
      scalarBytes = new Uint8Array(32);
      r = await contract.circuits.test(context, base, scalar);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(baseBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalarBytes);
    });
  },
  "guarded-decode"(test, expect, startContract, contractCode, runtime) {
    test('Bytes to secp256r1 field casts inside an if', async () => {
      const [contract, context] = await startContract(contractCode, {}, 0);
      // Random values in range.
      var base = 0x6c8a8c3f071f4d6dbe937aaf1a1d457dc5ab2e9e188e6a5c3190ea1072df9a97n;
      var scalar = 0x78db8cdf5ff2bc906d349cd5b11384283b39a67a9ed2739a2428dfd814c6e43dn;
      var baseBytes = new Uint8Array([
        0x97, 0x9a, 0xdf, 0x72, 0x10, 0xea, 0x90, 0x31,
        0x5c, 0x6a, 0x8e, 0x18, 0x9e, 0x2e, 0xab, 0xc5,
        0x7d, 0x45, 0x1d, 0x1a, 0xaf, 0x7a, 0x93, 0xbe,
        0x6d, 0x4d, 0x1f, 0x07, 0x3f, 0x8c, 0x8a, 0x6c,
      ]);
      var scalarBytes = new Uint8Array([
        0x3d, 0xe4, 0xc6, 0x14, 0xd8, 0xdf, 0x28, 0x24,
        0x9a, 0x73, 0xd2, 0x9e, 0x7a, 0xa6, 0x39, 0x3b,
        0x28, 0x84, 0x13, 0xb1, 0xd5, 0x9c, 0x34, 0x6d,
        0x90, 0xbc, 0xf2, 0x5f, 0xdf, 0x8c, 0xdb, 0x78,
      ]);
      // c is true, so the casts run and the ledger gets the field values.
      var r = await contract.circuits.test(context, true, baseBytes, scalarBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(base);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalar);
      // c is false, so the casts are skipped and the ledger keeps its default value.
      r = await contract.circuits.test(context, false, new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff));
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(0n);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(0n);
    });
  },
  "guarded-encode"(test, expect, startContract, contractCode, runtime) {
    test('secp256r1 field to bytes casts inside an if', async () => {
      const [contract, context] = await startContract(contractCode, {}, 0);
      // Random values in range.
      var base = 0x6c8a8c3f071f4d6dbe937aaf1a1d457dc5ab2e9e188e6a5c3190ea1072df9a97n;
      var scalar = 0x78db8cdf5ff2bc906d349cd5b11384283b39a67a9ed2739a2428dfd814c6e43dn;
      var baseBytes = new Uint8Array([
        0x97, 0x9a, 0xdf, 0x72, 0x10, 0xea, 0x90, 0x31,
        0x5c, 0x6a, 0x8e, 0x18, 0x9e, 0x2e, 0xab, 0xc5,
        0x7d, 0x45, 0x1d, 0x1a, 0xaf, 0x7a, 0x93, 0xbe,
        0x6d, 0x4d, 0x1f, 0x07, 0x3f, 0x8c, 0x8a, 0x6c,
      ]);
      var scalarBytes = new Uint8Array([
        0x3d, 0xe4, 0xc6, 0x14, 0xd8, 0xdf, 0x28, 0x24,
        0x9a, 0x73, 0xd2, 0x9e, 0x7a, 0xa6, 0x39, 0x3b,
        0x28, 0x84, 0x13, 0xb1, 0xd5, 0x9c, 0x34, 0x6d,
        0x90, 0xbc, 0xf2, 0x5f, 0xdf, 0x8c, 0xdb, 0x78,
      ]);
      // c is true, so the casts run and the ledger gets the bytes.
      var r = await contract.circuits.test(context, true, base, scalar);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(baseBytes);
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(scalarBytes);
      // c is false, so the casts are skipped and the ledger keeps its default value.
      r = await contract.circuits.test(context, false, runtime.MAX_SECP256R1_BASE, runtime.MAX_SECP256R1_SCALAR);
      expect(contractCode.ledger(r.context.currentQueryContext.state).base).toEqual(new Uint8Array(32));
      expect(contractCode.ledger(r.context.currentQueryContext.state).scalar).toEqual(new Uint8Array(32));
    });
  },
};
