// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import { describe, expect, test } from 'vitest';
import { CompactTypeField } from '../src/compact-types.js';
import {
  assertPreparedCallProofBinding,
  frHexToAlignedValue,
  kernelClaimContractCallProgram,
  prepareCallProofBinding,
  withVerifiedCallProofBinding,
} from '../src/cross-contract-proof-binding.js';
import * as publicRuntime from '../src/index.js';

const fixtureHash = (path: string): string =>
  createHash('sha256')
    .update(readFileSync(new URL(path, import.meta.url)))
    .digest('hex');

const input: ocrt.AlignedValue = { value: [Uint8Array.of(3)], alignment: CompactTypeField.alignment() };
const output: ocrt.AlignedValue = { value: [Uint8Array.of(4)], alignment: CompactTypeField.alignment() };

const claimCell = (program: ocrt.Op<null>[]): ocrt.AlignedValue => {
  const op = program[4];
  if (typeof op !== 'object' || !('push' in op)) throw new Error('missing claim cell push');
  return ocrt.StateValue.decode(op.push.value).asCell();
};

describe('internal call proof binding', () => {
  test('the pinned EntryPoint identity is transfer, not its same-shape decoy', () => {
    expect(fixtureHash('../../test-center/composable/EntryPoint/Callee.compact')).toBe(
      '89263a08b98928b7b9c4b750588c6bc2226784ceaf62a660511042c2126a51c7',
    );
    expect(fixtureHash('../../test-center/composable/EntryPoint/Caller.compact')).toBe(
      '25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6',
    );
    expect(fixtureHash('../../test-center/ts/composable/entry-point.ts')).toBe(
      '3b3e8995988e50121749ba5c59a593da326ec58786590e5a688d96ec7740785d',
    );
    expect(ocrt.entryPointHash('transfer')).toBe('d8aa677d2e908c12e403303d22abc24e0bf75c275515216441f4d6074ddd079e');
    expect(ocrt.entryPointHash('decoy')).toBe('5194e27b6b904a5a62c7f747a4aafc3997882412a03986461e411ffc49c241b1');
  });

  test('claim cell binds the callee address, entry point and communication commitment in pinned order', () => {
    // A random address ending in zero is shortened by StateValue canonical decoding.
    const address = '11'.repeat(32);
    const randomness = ocrt.communicationCommitmentRandomness();
    const binding = prepareCallProofBinding(address, 'transfer', input, output, randomness);
    expect(binding.commCommData).toEqual({
      commComm: ocrt.communicationCommitment(input, output, randomness),
      commCommRand: randomness,
    });
    expect(binding.privateTranscriptOutputs[0]).toEqual(output);
    expect(binding.privateTranscriptOutputs[0]).not.toBe(output);
    expect(() => assertPreparedCallProofBinding(binding)).not.toThrow();
    expect(binding.privateTranscriptOutputs[1]).toEqual(frHexToAlignedValue(randomness));
    expect(binding.claimProgram.map((op) => (typeof op === 'string' ? op : Object.keys(op)[0]))).toEqual([
      'swap',
      'idx',
      'dup',
      'size',
      'push',
      'concat',
      'push',
      'ins',
      'swap',
    ]);
    expect(binding.claimProgram[5]).toEqual({ concat: { cached: true, n: 160 } });
    expect(binding.claimProgram[7]).toEqual({ ins: { cached: true, n: 2 } });
    const cell = claimCell(binding.claimProgram);
    expect(cell.value).toHaveLength(3);
    expect(cell.value[0]).toEqual(ocrt.encodeContractAddress(address));
    expect(cell.value[1]).toEqual(Uint8Array.from(Buffer.from(ocrt.entryPointHash('transfer'), 'hex')));
    expect(cell.value[2]).toEqual(Uint8Array.from(frHexToAlignedValue(binding.commCommData.commComm).value[0]));
    expect(cell.alignment).toEqual([
      ...new Array(2).fill({ tag: 'atom', value: { tag: 'bytes', length: 32 } }),
      ...CompactTypeField.alignment(),
    ]);
  });

  test('same-shape sibling changes only the claimed entry point', () => {
    const address = ocrt.sampleContractAddress();
    const commitment = ocrt.communicationCommitment(input, output, ocrt.communicationCommitmentRandomness());
    const transfer = claimCell(kernelClaimContractCallProgram(address, 'transfer', commitment));
    const decoy = claimCell(kernelClaimContractCallProgram(address, 'decoy', commitment));
    expect(transfer.value[0]).toEqual(decoy.value[0]);
    expect(transfer.value[1]).not.toEqual(decoy.value[1]);
    expect(transfer.value[2]).toEqual(decoy.value[2]);
  });

  test('output and randomness are separate transcript values and a changed output changes the commitment', () => {
    const address = ocrt.sampleContractAddress();
    const randomness = ocrt.communicationCommitmentRandomness();
    const binding = prepareCallProofBinding(address, 'transfer', input, output, randomness);
    const changed = { value: [Uint8Array.of(5)], alignment: CompactTypeField.alignment() };
    const changedBinding = prepareCallProofBinding(address, 'transfer', input, changed, randomness);
    expect(binding.privateTranscriptOutputs).toHaveLength(2);
    expect(binding.privateTranscriptOutputs[0]).toEqual(output);
    expect(binding.privateTranscriptOutputs[0]).not.toBe(output);
    expect(binding.privateTranscriptOutputs[1]).toEqual(frHexToAlignedValue(randomness));
    expect(changedBinding.commCommData.commComm).not.toBe(binding.commCommData.commComm);
    expect(input.value[0]).toEqual(Uint8Array.of(3));
    expect(output.value[0]).toEqual(Uint8Array.of(4));
  });

  test('malformed Fr and commitment inputs refuse before a claim is returned', () => {
    const address = ocrt.sampleContractAddress();
    expect(() => frHexToAlignedValue('')).toThrow(/Fr hex/);
    expect(() => frHexToAlignedValue('zz')).toThrow(/Fr hex/);
    expect(() => frHexToAlignedValue('1')).toThrow(/Fr hex/);
    expect(() => kernelClaimContractCallProgram(address, 'transfer', 'zz')).toThrow(/Fr hex/);
    expect(() => prepareCallProofBinding(address, 'transfer', input, output, 'zz')).toThrow();
  });

  test('mutating source values after preparation cannot change the committed snapshots', () => {
    const sourceInput = { value: [Uint8Array.of(3)], alignment: CompactTypeField.alignment() };
    const sourceOutput = { value: [Uint8Array.of(4)], alignment: CompactTypeField.alignment() };
    const randomness = ocrt.communicationCommitmentRandomness();
    const binding = prepareCallProofBinding(ocrt.sampleContractAddress(), 'transfer', sourceInput, sourceOutput, randomness);
    const committed = binding.commCommData.commComm;
    sourceInput.value[0][0] = 8;
    sourceOutput.value[0][0] = 9;
    sourceOutput.alignment[0] = { tag: 'atom', value: { tag: 'bytes', length: 1 } };
    expect(binding.callInput.value[0]).toEqual(Uint8Array.of(3));
    expect(binding.callOutput.value[0]).toEqual(Uint8Array.of(4));
    expect(binding.privateTranscriptOutputs[0]).toBe(binding.callOutput);
    expect(binding.callOutput.alignment).toEqual(CompactTypeField.alignment());
    expect(binding.commCommData.commComm).toBe(committed);
    expect(() => assertPreparedCallProofBinding(binding)).not.toThrow();
  });

  test('mutating returned output, randomness, identity, claim cell or opcode refuses before commit', () => {
    const fresh = () => prepareCallProofBinding(ocrt.sampleContractAddress(), 'transfer', input, output);
    const changedOutput = fresh();
    changedOutput.privateTranscriptOutputs[0].value[0][0] ^= 1;
    expect(() => assertPreparedCallProofBinding(changedOutput)).toThrow(/changed after commitment/);
    const changedRandomness = fresh();
    changedRandomness.privateTranscriptOutputs[1].value[0][0] ^= 1;
    expect(() => assertPreparedCallProofBinding(changedRandomness)).toThrow(/changed after commitment/);
    const changedIdentity = fresh();
    Object.assign(changedIdentity, { calleeCircuitId: 'decoy' });
    expect(() => assertPreparedCallProofBinding(changedIdentity)).toThrow(/changed after commitment/);
    const changedCell = fresh();
    changedCell.claimProgram[4] = { push: { storage: false, value: ocrt.StateValue.newNull().encode() } };
    expect(() => assertPreparedCallProofBinding(changedCell)).toThrow(/changed after commitment/);
    const changedOpcode = fresh();
    changedOpcode.claimProgram[5] = { concat: { cached: true, n: 159 } };
    expect(() => assertPreparedCallProofBinding(changedOpcode)).toThrow(/changed after commitment/);
  });

  test('deleted array entries cannot bypass the guard at any nested proof level', () => {
    const fresh = () => prepareCallProofBinding(ocrt.sampleContractAddress(), 'transfer', input, output);
    const deletedOpcode = fresh();
    Reflect.deleteProperty(deletedOpcode.claimProgram, '5');
    expect(() => assertPreparedCallProofBinding(deletedOpcode)).toThrow(/changed after commitment/);
    const deletedValue = fresh();
    Reflect.deleteProperty(deletedValue.callOutput.value, '0');
    expect(() => assertPreparedCallProofBinding(deletedValue)).toThrow(/changed after commitment/);
    const deletedAlignment = fresh();
    Reflect.deleteProperty(deletedAlignment.callOutput.alignment, '0');
    expect(() => assertPreparedCallProofBinding(deletedAlignment)).toThrow(/changed after commitment/);
    const deletedTranscript = fresh();
    Reflect.deleteProperty(deletedTranscript.privateTranscriptOutputs, '1');
    expect(() => assertPreparedCallProofBinding(deletedTranscript)).toThrow(/changed after commitment/);
  });

  test('accessor operands refuse without invoking the getter and a fresh sealed copy is consumed synchronously', () => {
    const fresh = () => prepareCallProofBinding(ocrt.sampleContractAddress(), 'transfer', input, output);
    const accessor = fresh();
    let getterCalls = 0;
    Object.defineProperty(accessor.claimProgram[5], 'concat', {
      enumerable: true,
      configurable: true,
      get: () => {
        getterCalls++;
        return { cached: true, n: 160 };
      },
    });
    expect(() => assertPreparedCallProofBinding(accessor)).toThrow(/changed after commitment/);
    expect(getterCalls).toBe(0);
    let committed = false;
    expect(() =>
      withVerifiedCallProofBinding(accessor, () => {
        committed = true;
      }),
    ).toThrow(/changed after commitment/);
    expect(committed).toBe(false);
    const valid = fresh();
    withVerifiedCallProofBinding(valid, (sealedCopy) => {
      committed = true;
      expect(sealedCopy).not.toBe(valid);
      expect(sealedCopy.callOutput).not.toBe(valid.callOutput);
      expect(sealedCopy.claimProgram).not.toBe(valid.claimProgram);
      valid.claimProgram[5] = { concat: { cached: true, n: 159 } };
      expect(sealedCopy.claimProgram[5]).toEqual({ concat: { cached: true, n: 160 } });
      expect(sealedCopy.privateTranscriptOutputs[0]).toEqual(output);
    });
    expect(committed).toBe(true);
  });

  test('the host Runtime public surface stays closed', () => {
    expect('crossContractCall' in publicRuntime).toBe(false);
    expect('prepareCallProofBinding' in publicRuntime).toBe(false);
    expect('withVerifiedCallProofBinding' in publicRuntime).toBe(false);
    expect('kernelClaimContractCallProgram' in publicRuntime).toBe(false);
  });
});
