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

"""Focused pinned-fixture, refusal and lexical-fold admission checks."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_fold as gate


class FoldAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/fold'
        self.files = {}
        for name in gate.POSITIVES + tuple(gate.REFUSALS):
            filenames = (f'vector_fold_{name}.compact', 'compile.pass.test.ts',
                         'runtime.pass.test.ts') if name in gate.POSITIVES else (
                             f'vector_fold_{name}.compact', 'compile.fail.test.ts')
            for filename in filenames:
                file = self.family / name / filename
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text(f'{name} {filename}\n')
                self.files[file.relative_to(self.root).as_posix()] = gate.sha256(file)
        self.manifest = self.family / 'original-source-sha256.json'
        self.write_manifest()
        patcher = mock.patch.object(gate, 'ROOT', self.root)
        patcher.start()
        self.addCleanup(patcher.stop)

    def write_manifest(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': gate.PINNED,
                                             'files': self.files}))

    def test_accepts_all_ten_exact_originals(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_refuses_changed_original_result_assertion(self) -> None:
        (self.family / 'semantics/runtime.pass.test.ts').write_text('changed\n')
        with self.assertRaisesRegex(RuntimeError, 'original Vector fold file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_missing_original_refusal(self) -> None:
        self.files.pop('test-contracts/primitives/vector/fold/lambda_arity_mismatch/compile.fail.test.ts')
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_refusal_requires_diagnostics_and_no_output(self) -> None:
        destination = self.root / 'refused'
        diagnostic = gate.REFUSALS['lambda_arity_mismatch']
        failed = subprocess.CompletedProcess([], 255, '', 'line 18 char 10: incompatible arguments in call to anonymous circuit')
        gate.refusal(failed, destination, diagnostic)
        with self.assertRaisesRegex(RuntimeError, 'original Vector fold refusal changed'):
            gate.refusal(subprocess.CompletedProcess([], 0, '', failed.stderr), destination,
                         diagnostic)
        destination.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(failed, destination, diagnostic)

    def test_refuses_reversed_named_circuit_diagnostic_lines(self) -> None:
        destination = self.root / 'refused'
        diagnostic = gate.REFUSALS['circuit_argument_mismatch']
        ordered = subprocess.CompletedProcess(
            [], 255, '',
            'line 22 char 10:\n  no compatible function named enabled_power is in scope at this call\n'
            '  one function is incompatible with the supplied argument types')
        gate.refusal(ordered, destination, diagnostic)
        reversed_lines = subprocess.CompletedProcess(
            [], 255, '',
            'one function is incompatible with the supplied argument types\n'
            'no compatible function named enabled_power is in scope at this call\n'
            'line 22 char 10:')
        with self.assertRaisesRegex(RuntimeError, 'original Vector fold refusal changed'):
            gate.refusal(reversed_lines, destination, diagnostic)

    def test_lexical_fold_node_is_not_confused_with_named_fold(self) -> None:
        source = {'kind': 'vector_fold', 'length': 5,
                  'body': {'kind': 'value', 'value': {'kind': 'parameter'}}}
        self.assertEqual(gate.inline_fold_nodes({'body': source}), [source])
        self.assertEqual(gate.inline_fold_nodes({'kind': 'vector_fold_call'}), [])


if __name__ == '__main__':
    unittest.main()
