#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""Fail-closed job graph and dispatch tests for the focused DID Linux gate."""

import itertools
import os
from pathlib import Path
import re
import subprocess
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = (ROOT / '.github/workflows/did-primitive-reducers.yml').read_text()


def jobs():
    body = WORKFLOW.split('\njobs:\n', 1)[1]
    return {match.group(1): body[match.end():next_match.start() if next_match else None]
            for match, next_match in zip(
                list(re.finditer(r'^  ([a-z][a-z0-9-]+):\n', body, re.M)),
                list(re.finditer(r'^  ([a-z][a-z0-9-]+):\n', body, re.M))[1:] + [None])}


def run_block(job, step):
    section = job.split(f'      - name: {step}\n', 1)[1]
    match = re.search(r'^        run: \|\n((?:^          .*\n)+)', section, re.M)
    return textwrap.dedent(match.group(1))


class ManualDidWorkflowTests(unittest.TestCase):
    def test_dispatch_requires_exact_sha_before_any_expensive_job(self):
        header = WORKFLOW.split('\njobs:\n', 1)[0]
        self.assertIn('on:\n  workflow_dispatch:', header)
        self.assertNotIn('pull_request:', header)
        self.assertNotIn('schedule:', header)
        self.assertRegex(header, r'expected_sha:\n(?:        .*\n)*?        required: true')
        configured = jobs()
        self.assertEqual(set(configured), {'source-preflight', 'compiler-linux',
                                           'did-primitive-reducers', 'complete'})
        preflight = configured['source-preflight']
        check = run_block(preflight, 'Require explicitly selected exact commit')
        valid = 'a' * 40
        for expected, source, accepted in ((valid, valid, True),
                                           (valid, 'b' * 40, False),
                                           ('A' * 40, 'A' * 40, False),
                                           ('a' * 39, 'a' * 39, False),
                                           ('', '', False)):
            with self.subTest(expected=expected, source=source):
                env = dict(os.environ, EXPECTED_SHA=expected, SOURCE_SHA=source)
                result = subprocess.run(['bash', '-e', '-c', check], env=env,
                                        capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, accepted)
        self.assertIn('needs: source-preflight', configured['compiler-linux'])
        self.assertIn('uses: ./.github/workflows/compiler-producer.yml', configured['compiler-linux'])
        self.assertIn('system: x86_64-linux', configured['compiler-linux'])
        self.assertIn('export-shell: true', configured['compiler-linux'])

    def test_consumer_and_final_gate_fail_closed_for_every_dependency_state(self):
        configured = jobs()
        did = configured['did-primitive-reducers']
        self.assertIn('needs: [source-preflight, compiler-linux]', did)
        condition = re.search(r'^    if: \$\{\{ (.*) \}\}$', did, re.M).group(1)
        self.assertEqual(set(re.findall(r'needs\.([\w-]+)\.result', condition)),
                         {'source-preflight', 'compiler-linux'})
        expression = re.sub(r'needs\.([\w-]+)\.result',
                            lambda match: 'states[' + repr(match[1]) + ']', condition)
        expression = expression.replace('!cancelled()', 'not cancelled').replace('&&', ' and ')
        states = ('success', 'failure', 'cancelled', 'skipped', '')
        for preflight, producer, cancelled in itertools.product(states, states, (False, True)):
            actual = eval(expression, {'__builtins__': {}},
                          dict(states={'source-preflight': preflight,
                                       'compiler-linux': producer}, cancelled=cancelled))
            self.assertEqual(actual, not cancelled and preflight == producer == 'success')
        complete = configured['complete']
        self.assertIn('needs: [source-preflight, compiler-linux, did-primitive-reducers]', complete)
        self.assertIn('if: ${{ always() }}', complete)
        final = run_block(complete, 'Require every DID qualification owner')
        for values in itertools.product(states, repeat=3):
            env = dict(os.environ)
            env.update(zip(('PREFLIGHT_RESULT', 'PRODUCER_RESULT', 'DID_RESULT'), values))
            result = subprocess.run(['bash', '-e', '-c', final], env=env,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode == 0, values == ('success',) * 3)

    def test_authenticated_proof_material_and_locked_matrix_remain_bound(self):
        configured = jobs()
        preflight = configured['source-preflight']
        did = configured['did-primitive-reducers']
        self.assertIn('ref: ${{ github.sha }}', preflight)
        self.assertIn('ref: ${{ github.sha }}', did)
        self.assertIn('test "$(git rev-parse HEAD)" = "$SOURCE_SHA"', preflight)
        self.assertIn('python3 -B .github/scripts/test_compiler_closure.py', preflight)
        for required in ('--source-sha "$SOURCE_SHA" --system x86_64-linux',
                         '--run-id "$GITHUB_RUN_ID" --run-attempt "$GITHUB_RUN_ATTEMPT"',
                         '--producer-attempt "$COMPILER_PRODUCER_ATTEMPT"',
                         '--repository "$REPOSITORY" --artifact-id "$ARTIFACT_ID"',
                         'needs.compiler-linux.outputs.shell-artifact-id',
                         'needs.compiler-linux.outputs.producer-attempt',
                         'toolchain: 1.99.0',
                         "hashFiles('Cargo.lock')",
                         "record['ledger_version'] != 'ledger-9.1.0.1'",
                         'cargo metadata --locked --offline',
                         'resolve_ledger_test_static.py',
                         'nix build --no-link .#runtime.package',
                         'nix develop .#compiler --command',
                         'did_primitive_reducer_gate.py',
                         "receipt['git_head'] == sys.argv[2]",
                         "len(receipt['reducers']) == len(receipt['expected_reducers']) == 9",
                         "for row in receipt['reducers']) == 21"):
            with self.subTest(required=required):
                self.assertIn(required, did)
        self.assertNotIn('--only', did)
        self.assertIn('compact-did-primitive-reducer-proof/', did)


if __name__ == '__main__':
    unittest.main()
