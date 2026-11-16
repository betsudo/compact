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

import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-keyword-secondary-linux.yml'
VERIFIER = ROOT / '.github/scripts/verify_keyword_secondary_linux_receipts.py'
SPEC = importlib.util.spec_from_file_location('secondary_receipts', VERIFIER)
receipts = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(receipts)


class WorkflowTest(unittest.TestCase):
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
        sequence = ('check_upstream_ledger9_bytes_indexed_access.py',
                    'check_static_bytes_fold_index_domain.py',
                    'test-rust-vector-indices.ss',
                    'stage_upstream_keyword_vitest_host.py',
                    'check_upstream_keyword_vitest_host.py',
                    'check_upstream_ledger9_import_ledger_field_proof.py',
                    'check_upstream_ledger9_bytes_indexed_access_proof.py',
                    'verify_keyword_secondary_linux_receipts.py')
        positions = [workflow.index(name) for name in sequence]
        self.assertEqual(positions, sorted(positions))
        self.assertIn('include-hidden-files: true', workflow)
        self.assertIn('needs: [source-preflight, compiler-linux, secondary-semantics]',
                      workflow)
        self.assertNotIn('pull_request:', workflow)

    def test_family_and_host_roster(self):
        self.assertEqual(list(receipts.FAMILIES),
                         ['include', 'prefix', 'assert', 'module',
                          'export', 'import_type'])
        self.assertEqual(sum(row[0] for row in receipts.FAMILIES.values()), 68)
        self.assertEqual(sum(row[1] for row in receipts.FAMILIES.values()), 35)
        self.assertEqual(len(receipts.HOST_FAMILIES), 12)

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
