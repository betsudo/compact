#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Execute the actual workflow input/argv seams without cache or proof execution."""
import itertools
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest

WORKFLOW = Path(__file__).resolve().parents[2] / '.github/workflows/forge-local-ci.yml'


class SnapshotTransportTests(unittest.TestCase):
    def setUp(self):
        self.text = WORKFLOW.read_text()
        self.partition = 'a' * 64
        self.identity = 'b' * 64
        self.key = 'forge-v1-' + self.partition + '-' + 'c' * 64 + '-generation-1'

    def validate(self, values, mode='linux-owner', event='workflow_dispatch', publish=False, approval='', ref='refs/heads/codex/issue-679-trusted-native-seed'):
        block = self.text.split("python3 - <<'PY_SNAPSHOT'\n", 1)[1].split('          PY_SNAPSHOT', 1)[0]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'output'
            env = dict(os.environ, QUALIFICATION=mode, EVENT_NAME=event, GITHUB_OUTPUT=str(output),
                       SNAPSHOT_KEY=values[0], SNAPSHOT_ID=values[1], SNAPSHOT_PARTITION=values[2],
                       SEED_PUBLISH=str(publish).lower(),SEED_WINDOW=approval,ACTUAL_REF=ref)
            result = subprocess.run([sys.executable, '-c', textwrap.dedent(block)], env=env, capture_output=True, text=True)
            return result.returncode, output.read_text() if output.exists() else ''

    def test_complete_and_empty_routes(self):
        self.assertEqual(self.validate((self.key, self.identity, self.partition)), (0, 'restore-requested=true\n'))
        for mode, event in (('apple','workflow_dispatch'), ('','push'), ('','workflow_call')):
            self.assertEqual(self.validate(('', '', ''), mode, event), (0, 'restore-requested=false\n'))

    def test_every_partial_tuple_and_other_modes_refuse(self):
        complete = (self.key, self.identity, self.partition)
        for mask in itertools.product((False, True), repeat=3):
            if all(mask) or not any(mask):
                continue
            self.assertNotEqual(self.validate(tuple(value if enabled else '' for value, enabled in zip(complete, mask)))[0], 0)
        for mode in ('apple', 'linux-native', 'linux-candidate', ''):
            self.assertNotEqual(self.validate(complete, mode)[0], 0)
        self.assertNotEqual(self.validate(complete, event='push')[0], 0)

    def test_malformed_partition_mismatch_and_injection_refuse(self):
        for values in ((self.key, 'bad', self.partition), (self.key, self.identity, 'd' * 64),
                       (self.key + '\nother', self.identity, self.partition),
                       ('$(touch should-not-exist)', self.identity, self.partition),
                       ('forge-v1-' + self.partition, self.identity, self.partition)):
            self.assertNotEqual(self.validate(values)[0], 0)

    def test_explicit_publisher_requires_fixed_ref_window_and_no_restore(self):
        self.assertEqual(self.validate(('', '', ''),publish=True,approval='approved maintenance window')[0],0)
        for options in (dict(event='pull_request'),dict(event='push'),dict(mode='apple'),
                        dict(ref='refs/heads/codex/rust-backend-ast'),dict(approval=''),
                        dict(approval='bad\nlabel')):
            supplied=dict(publish=True,approval='approved window');supplied.update(options)
            self.assertNotEqual(self.validate(('', '', ''),**supplied)[0],0)
        self.assertNotEqual(self.validate((self.key,self.identity,self.partition),publish=True,approval='approved')[0],0)
        self.assertNotEqual(self.validate(('', '', ''),approval='not enabled')[0],0)
        self.assertIn('default: false',self.text.split('      native-seed-publish:',1)[1].split('      native-seed-window:',1)[0])

    def test_exact_readonly_restore_and_producer_dependencies(self):
        restore = self.text.split('      - name: Restore exact supplied stopped snapshot (read-only)', 1)[1].split('      - name:', 1)[0]
        self.assertIn('actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae', restore)
        self.assertIn('key: ${{ inputs.native-snapshot-key }}', restore)
        self.assertIn('fail-on-cache-miss: true', restore)
        self.assertNotIn('restore-keys:', restore)
        linux = self.text.split('  linux-candidate:', 1)[1]
        self.assertNotIn('actions/cache/save', linux)
        self.assertIn('test "$CACHE_HIT" = true', linux)
        self.assertIn('test "$MATCHED_KEY" = "$EXPECTED_KEY"', linux)
        for job in ('standalone-darwin-compiler', 'standalone-linux-compiler', 'forge-linux-material'):
            block = self.text.split('  ' + job + ':', 1)[1]
            # The direct job dependency is before each job condition and reusable call.
            self.assertTrue(block.startswith('\n    needs: snapshot-inputs'))
        self.assertIn("needs.snapshot-inputs.result == 'success'", linux)

    def test_publisher_is_pinned_after_gates_without_permission_increase(self):
        linux=self.text.split('  linux-candidate:',1)[1]
        self.assertIn('ref: 8e4efdeb2ac905fa73dac7ebd35d82a2119fb24f',linux)
        self.assertLess(linux.index('Install locked publisher dependencies'),linux.index('Qualify cold, warm'))
        self.assertLess(linux.index('Execute the single original native owner'),linux.index('Prepare official seed'))
        self.assertLess(linux.index('Prepare official seed'),linux.index('Admit and publish one measured'))
        self.assertIn("steps.native-owner.outcome == 'success'",linux)
        self.assertIn("steps.seed-prepare.outcome == 'success'",linux)
        self.assertIn('actions: read',linux)
        self.assertNotIn('actions: write',linux)
        for field in ('binary','cache-dir','baseline','compact-guard','compact-plan'):
            self.assertIn('          '+field+':',linux)
        self.assertIn('${{ runner.temp }}/native-seed-*.json',linux)
        self.assertNotIn('actions/cache/save',linux)

    def test_owner_shell_array_preserves_literal_arguments_and_cold(self):
        block = self.text.split('      - name: Execute the single original native owner after current finite qualification', 1)[1]
        script = textwrap.dedent(block.split('        run: |\n', 1)[1].split('      - name:', 1)[0])
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            executable = root / 'python3'
            executable.write_text('#!' + sys.executable + '\nimport json,os,sys\nopen(os.environ["CAPTURE"],"w").write(json.dumps(sys.argv[1:]))\n')
            executable.chmod(0o755)
            for requested in ('false', 'true'):
                env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ.get('PATH',''),
                           RUNNER_TEMP=str(root), COMPACTC='/compiler', LINUX_REPORT='/finite',
                           LINUX_PROOF_RUNNER='/proof', FORGE_HELPER='/helper', RESTORE_REQUESTED=requested,
                           EXPECTED_SNAPSHOT_ID='$(touch ' + str(root / 'unsafe') + ')',
                           EXPECTED_SNAPSHOT_PARTITION='literal;value', CAPTURE=str(root / 'argv.json'))
                subprocess.run(['bash', '-e', '-c', script], env=env, check=True)
                argv = json.loads((root / 'argv.json').read_text())
                self.assertEqual('--snapshot' in argv, requested == 'true')
                if requested == 'true':
                    self.assertEqual(argv[argv.index('--snapshot-id') + 1], env['EXPECTED_SNAPSHOT_ID'])
                    self.assertEqual(argv[argv.index('--snapshot-partition') + 1], 'literal;value')
                self.assertFalse((root / 'unsafe').exists())


if __name__ == '__main__':
    unittest.main()
