// This file is part of Compact.
// Copyright (C) 2026 contributors to Minokawa Compact
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { performance } from 'node:perf_hooks';
import { setImmediate } from 'node:timers';
import { describe, expect, test, vi } from 'vitest';
import { flushProofChecks, registerProofCheck } from './util.js';

const message = 'proof check timed out after 10000ms';
const turn = () => new Promise<void>((resolve) => setImmediate(resolve));
const deferred = () => {
  let resolve!: () => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};

// Real deadline checks use the unmodified default and real time; no checker is called.
describe.sequential('proof-check timeout retention', () => {
  test('the actual default ten-second timeout is retained and a late underlying rejection is handled', async () => {
    const work = deferred();
    const unhandled: unknown[] = [];
    const observe = (reason: unknown) => { unhandled.push(reason); };
    process.on('unhandledRejection', observe);
    try {
      registerProofCheck(work.promise);
      const started = performance.now();
      await expect(flushProofChecks()).rejects.toMatchObject({ message });
      expect(performance.now() - started).toBeGreaterThanOrEqual(9_900);
      work.reject(new Error('underlying rejection after timeout'));
      await turn();
      expect(unhandled).toEqual([]);
      await flushProofChecks();
    } finally {
      work.resolve();
      process.off('unhandledRejection', observe);
    }
  }, 30_000);

  test('a real timeout settled before a late flush remains a failure', async () => {
    const work = deferred();
    const unhandled: unknown[] = [];
    const observe = (reason: unknown) => { unhandled.push(reason); };
    process.on('unhandledRejection', observe);
    try {
      const started = performance.now();
      registerProofCheck(work.promise);
      await new Promise<void>((resolve) => setTimeout(resolve, 10_100));
      expect(performance.now() - started).toBeGreaterThanOrEqual(9_900);
      await expect(flushProofChecks()).rejects.toMatchObject({ message });
      await turn();
      expect(unhandled).toEqual([]);
      await flushProofChecks();
    } finally {
      work.resolve();
      process.off('unhandledRejection', observe);
    }
  }, 30_000);

  // Fake timers establish timer cleanup only, not the real default deadline.
  test('success and an early error clear their deadline timers and preserve the error', async () => {
    vi.useFakeTimers();
    try {
      const before = vi.getTimerCount();
      registerProofCheck(Promise.resolve());
      expect(vi.getTimerCount()).toBe(before + 1);
      await flushProofChecks();
      expect(vi.getTimerCount()).toBe(before);
      const failure = new Error('checker rejection before timeout');
      registerProofCheck(Promise.reject(failure));
      await expect(flushProofChecks()).rejects.toBe(failure);
      expect(vi.getTimerCount()).toBe(before);
      await flushProofChecks();
    } finally {
      vi.useRealTimers();
    }
  });

  test('an undefined rejection stays distinct and clears its deadline timer', async () => {
    vi.useFakeTimers();
    try {
      const before = vi.getTimerCount();
      registerProofCheck(Promise.reject(undefined));
      let rejected = false;
      try {
        await flushProofChecks();
      } catch (error) {
        rejected = true;
        expect(error).toBeUndefined();
      }
      expect(rejected).toBe(true);
      expect(vi.getTimerCount()).toBe(before);
      await flushProofChecks();
    } finally {
      vi.useRealTimers();
    }
  });
});
