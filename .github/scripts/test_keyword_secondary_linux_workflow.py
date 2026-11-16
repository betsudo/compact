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
"""Narrow static workflow and receipt tampering tests for secondary Linux."""

import ast
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-keyword-secondary-linux.yml'
VERIFIER = ROOT / '.github/scripts/verify_keyword_secondary_linux_receipts.py'
SPEC = importlib.util.spec_from_file_location('secondary_receipts', VERIFIER)
receipts = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(receipts)
sys.path.insert(0, str(ROOT / 'tools/compact-rust-backend'))
from check_upstream_keyword_vitest_host import original_host_counts_match


class WorkflowTest(unittest.TestCase):
    def test_host_accepts_plain_and_sgr_colored_exact_counts(self):
        cases = '      ✓ original case\n' * 263
        plain = cases + ' Test Files  1 passed (1)\n      Tests  263 passed (263)\n'
        colored = (cases + '\x1b[2m Test Files \x1b[22m '
                   '\x1b[1m\x1b[32m1 passed\x1b[39m\x1b[22m'
                   '\x1b[90m (1)\x1b[39m\n'
                   '\x1b[2m      Tests \x1b[22m '
                   '\x1b[1m\x1b[32m263 passed\x1b[39m\x1b[22m'
                   '\x1b[90m (263)\x1b[39m\n')
        self.assertTrue(original_host_counts_match(plain))
        self.assertTrue(original_host_counts_match(colored))
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / 'original-vitest.log'
            log.write_text(colored)
            receipts.host_vitest_log(log, receipts.sha(log))

    def test_host_refuses_wrong_counts_markers_and_non_sgr_control(self):
        cases = '      ✓ original case\n' * 263
        exact = cases + ' Test Files  1 passed (1)\n      Tests  263 passed (263)\n'
        self.assertFalse(original_host_counts_match(
            exact.replace('Tests  263 passed', 'Tests  262 passed')))
        self.assertFalse(original_host_counts_match(
            exact.replace('Test Files  1 passed', 'Test Files  2 passed')))
        self.assertFalse(original_host_counts_match(exact.replace('✓', '', 1)))
        self.assertFalse(original_host_counts_match(exact.replace('263 passed',
                                                                  '\x1b[K263 passed')))
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / 'original-vitest.log'
            for bad in (exact.replace('Tests  263 passed', 'Tests  262 passed'),
                        exact.replace('✓', '', 1)):
                log.write_text(bad)
                with self.assertRaisesRegex(ValueError, 'original host execution differs'):
                    receipts.host_vitest_log(log, receipts.sha(log))
            log.write_text(exact)
            with self.assertRaisesRegex(ValueError, 'original host execution differs'):
                receipts.host_vitest_log(log, '0' * 64)

    def test_exact_source_and_serial_gate_graph(self):
        workflow = WORKFLOW.read_text()
        self.assertIn('refs/heads/codex/ledger9-keyword-secondary-linux/$SOURCE_SHA',
                      workflow)
        self.assertIn('git verify-commit HEAD', workflow)
        self.assertIn('uses: ./.github/workflows/compiler-producer.yml', workflow)
        self.assertIn('system: x86_64-linux', workflow)
        self.assertIn('export-shell: true', workflow)
        self.assertIn('for gate in include prefix assert module export import_type',
                      workflow)
        sequence = ('check_upstream_ledger9_small_keywords.py',
                    'check_upstream_ledger9_bytes_indexed_access.py',
                    'check_static_bytes_fold_index_domain.py',
                    'test-rust-vector-indices.ss',
                    'stage_upstream_keyword_vitest_host.py',
                    'check_upstream_keyword_vitest_host.py',
                    'check_upstream_ledger9_import_ledger_field_proof.py',
                    'check_upstream_ledger9_bytes_indexed_access_proof.py',
                    'check_upstream_ledger9_keyword_constructor_parameters_proof.py',
                    'verify_keyword_secondary_linux_receipts.py')
        positions = [workflow.index(name) for name in sequence]
        self.assertEqual(positions, sorted(positions))
        self.assertIn('include-hidden-files: true', workflow)
        self.assertIn('needs: [source-preflight, compiler-linux, secondary-semantics]',
                      workflow)
        self.assertNotIn('pull_request:', workflow)

    def test_upload_excludes_rebuildable_dependencies_and_targets(self):
        workflow = WORKFLOW.read_text()
        root = '${{ runner.temp }}/keyword-secondary-semantic-evidence/'
        self.assertIn(root + '\n', workflow)
        for name in ('node_modules', '.yarn', 'target', 'rust-target'):
            self.assertIn('!' + root + '**/' + name + '/**', workflow)
        for gate in ('import_proof', 'bytes_proof'):
            self.assertNotIn('!' + root + gate + '/**', workflow)

    def test_family_and_host_roster(self):
        self.assertEqual(list(receipts.FAMILIES),
                         ['include', 'prefix', 'assert', 'module',
                          'export', 'import_type'])
        self.assertEqual(sum(row[0] for row in receipts.FAMILIES.values()), 68)
        self.assertEqual(sum(row[1] for row in receipts.FAMILIES.values()), 35)
        self.assertEqual(len(receipts.HOST_FAMILIES), 20)
        self.assertEqual(receipts.HOST_FAMILIES[-1], 'disclose')
        stage = ast.parse((ROOT / 'tools/compact-rust-backend/'
                           'stage_upstream_keyword_vitest_host.py').read_text())
        roster = next(ast.literal_eval(node.value) for node in stage.body
                      if isinstance(node, ast.Assign) and
                      any(isinstance(target, ast.Name) and
                          target.id == 'FAMILY_COUNTS' for target in node.targets))
        self.assertEqual(len(roster), 20)
        self.assertEqual(roster['disclose'], (13, 3, 2, 3, 0))
        self.assertEqual(16 + sum(row[0] for row in roster.values()), 489)
        self.assertEqual(sum(row[1] + row[2] for row in roster.values()), 196)
        self.assertEqual(sum(sum(row[1:]) for row in roster.values()), 263)

    def test_missing_and_tampered_artifact_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact = root / 'product'
            artifact.mkdir()
            path = artifact / 'contract.js'
            path.write_text('accepted')
            expected = receipts.inventory(artifact)
            receipts.files(artifact, expected)
            path.write_text('tampered')
            with self.assertRaisesRegex(ValueError, 'inventory differs'):
                receipts.files(artifact, expected)
            path.unlink()
            with self.assertRaisesRegex(ValueError, 'inventory differs'):
                receipts.files(artifact, expected)

    def test_linked_artifact_and_wrong_platform_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'real').write_text('payload')
            (root / 'alias').symlink_to(root / 'real')
            with self.assertRaisesRegex(ValueError, 'linked artifact'):
                receipts.inventory(root)
        row = {'status': 'passed', 'platform': 'Darwin arm64',
               'head': 'a' * 40, 'tree': 'b' * 40}
        with self.assertRaisesRegex(ValueError, 'Linux receipt'):
            receipts.identity(row, 'a' * 40, 'b' * 40)


if __name__ == '__main__':
    unittest.main()
