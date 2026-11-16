// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

/**
 * The pure, internal proof binding made after a callee returns. The executing
 * call path must apply these values to its own copied context and transcript;
 * this module deliberately does not enter a callee or expose a public call API.
 */
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { alignedConcat } from './built-ins.js';
import { CompactTypeBytes, CompactTypeField, CompactTypeUnsignedInteger } from './compact-types.js';
import { CompactError } from './error.js';
import { fromHex } from './utils.js';

const Bytes32Descriptor = new CompactTypeBytes(32);
const KernelStateFieldIndexDescriptor = new CompactTypeUnsignedInteger(255n, 1);

const bytes32Value = (bytes: Uint8Array): ocrt.AlignedValue => ({
  value: Bytes32Descriptor.toValue(bytes),
  alignment: Bytes32Descriptor.alignment(),
});

/** The pinned wasm encodes an Fr as a SCALE marker followed by little-endian bytes. */
export const frHexToAlignedValue = (frHex: string): ocrt.AlignedValue => {
  if (!/^(?:[0-9a-fA-F]{2})+$/.test(frHex)) {
    throw new CompactError('invalid Fr hex encoding');
  }
  const leBytes = fromHex(frHex).slice(1);
  let end = leBytes.length;
  while (end > 0 && leBytes[end - 1] === 0) end -= 1;
  return { value: [leBytes.slice(0, end)], alignment: CompactTypeField.alignment() };
};

/** Exact pinned Kernel.claimContractCall program, still unexecuted here. */
export const kernelClaimContractCallProgram = (
  calleeAddress: ocrt.ContractAddress,
  calleeCircuitId: string,
  commComm: ocrt.CommunicationCommitment,
): ocrt.Op<null>[] => [
  { swap: { n: 0 } },
  {
    idx: {
      cached: true,
      pushPath: true,
      path: [
        {
          tag: 'value',
          value: {
            value: KernelStateFieldIndexDescriptor.toValue(3n),
            alignment: KernelStateFieldIndexDescriptor.alignment(),
          },
        },
      ],
    },
  },
  { dup: { n: 0 } },
  'size',
  {
    push: {
      storage: false,
      value: ocrt.StateValue.newCell(
        alignedConcat(
          bytes32Value(ocrt.encodeContractAddress(calleeAddress)),
          bytes32Value(fromHex(ocrt.entryPointHash(calleeCircuitId))),
          frHexToAlignedValue(commComm),
        ),
      ).encode(),
    },
  },
  { concat: { cached: true, n: 160 } },
  { push: { storage: false, value: ocrt.StateValue.newNull().encode() } },
  { ins: { cached: true, n: 2 } },
  { swap: { n: 0 } },
];

export interface PreparedCallProofBinding {
  readonly commCommData: {
    readonly commComm: ocrt.CommunicationCommitment;
    readonly commCommRand: ocrt.CommunicationCommitmentRand;
  };
  /** Append in this order to the caller's private transcript, after success. */
  readonly privateTranscriptOutputs: readonly [ocrt.AlignedValue, ocrt.AlignedValue];
  readonly claimProgram: ocrt.Op<null>[];
}

/** Compute, but do not apply, a returned call's communication and claim data. */
export const prepareCallProofBinding = (
  calleeAddress: ocrt.ContractAddress,
  calleeCircuitId: string,
  input: ocrt.AlignedValue,
  output: ocrt.AlignedValue,
  randomness: ocrt.CommunicationCommitmentRand = ocrt.communicationCommitmentRandomness(),
): PreparedCallProofBinding => {
  const commComm = ocrt.communicationCommitment(input, output, randomness);
  const commCommRand = randomness;
  return {
    commCommData: { commComm, commCommRand },
    privateTranscriptOutputs: [output, frHexToAlignedValue(commCommRand)],
    claimProgram: kernelClaimContractCallProgram(calleeAddress, calleeCircuitId, commComm),
  };
};
