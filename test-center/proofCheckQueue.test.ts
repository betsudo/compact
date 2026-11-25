// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { setImmediate } from 'node:timers';
import { describe, expect, test } from 'vitest';
import { flushProofChecks, registerProofCheck } from './util.js';

const turn = () => new Promise<void>((resolve) => setImmediate(resolve));
const deferred = () => {
  let resolve!: () => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};

// This module owns no checker work: each test drains only promises it registered.
describe.sequential('proof-check queue retention', () => {
  test('fulfilled checks drain and the next flush is empty', async () => {
    registerProofCheck(Promise.resolve());
    registerProofCheck(Promise.resolve());
    await turn();
    await flushProofChecks();
    await flushProofChecks();
  });

  test('a rejection settled before flush retains its identity without an unhandled rejection', async () => {
    const unhandled: unknown[] = [];
    const observe = (reason: unknown) => { unhandled.push(reason); };
    process.on('unhandledRejection', observe);
    try {
      const failure = new Error('early checker rejection');
      registerProofCheck(Promise.reject(failure));
      await turn();
      await expect(flushProofChecks()).rejects.toBe(failure);
      await turn();
      expect(unhandled).toEqual([]);
      await flushProofChecks();
    } finally {
      process.off('unhandledRejection', observe);
    }
  });

  test('a rejection arriving during a flush remains its failure', async () => {
    const work = deferred();
    const failure = new Error('pending checker rejection');
    registerProofCheck(work.promise);
    const observed = expect(flushProofChecks()).rejects.toBe(failure);
    work.reject(failure);
    await observed;
    await flushProofChecks();
  });

  test('rejection with undefined remains a rejection', async () => {
    registerProofCheck(Promise.reject(undefined));
    await turn();
    let rejected = false;
    try {
      await flushProofChecks();
    } catch (error) {
      rejected = true;
      expect(error).toBeUndefined();
    }
    expect(rejected).toBe(true);
    await flushProofChecks();
  });

  test('registration during a flush is awaited for both success and rejection', async () => {
    for (const shouldReject of [false, true]) {
      const first = deferred();
      const late = deferred();
      const failure = new Error('late checker rejection');
      registerProofCheck(first.promise);
      let finished = false;
      const observed = flushProofChecks().then(
        () => { finished = true; return { rejected: false as const }; },
        (error: unknown) => { finished = true; return { rejected: true as const, error }; },
      );
      registerProofCheck(late.promise);
      first.resolve();
      try {
        await turn();
        expect(finished).toBe(false);
      } finally {
        // Always settle owned work even if an assertion fails.
        if (shouldReject) late.reject(failure); else late.resolve();
      }
      const outcome = await observed;
      expect(outcome.rejected).toBe(shouldReject);
      if (outcome.rejected) expect(outcome.error).toBe(failure);
      await flushProofChecks();
    }
  });

  test('the first failure is reported after consuming every check and does not leak into the next flush', async () => {
    const first = new Error('first checker rejection');
    const second = new Error('second checker rejection');
    registerProofCheck(Promise.reject(first));
    registerProofCheck(Promise.reject(second));
    registerProofCheck(Promise.resolve());
    await expect(flushProofChecks()).rejects.toBe(first);
    await flushProofChecks();
    registerProofCheck(Promise.resolve());
    await flushProofChecks();
  });
});
