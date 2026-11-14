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

"""Focused source, refusal and artifact admission controls."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_indexing as gate


class IndexingAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/indexing'
        self.files = {}
        for name in gate.POSITIVES + tuple(gate.REFUSALS):
            filenames = (f'vector_indexing_{name}.compact',
                         'compile.pass.test.ts', 'runtime.pass.test.ts') if name in gate.POSITIVES else (
                             f'vector_indexing_{name}.compact', 'compile.fail.test.ts')
            for filename in filenames:
                path = self.family / name / filename
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f'{name} {filename}\n')
                self.files[path.relative_to(self.root).as_posix()] = gate.sha256(path)
        self.manifest = self.family / 'original-source-sha256.json'
        self.write_manifest()
        patcher = mock.patch.object(gate, 'ROOT', self.root)
        patcher.start()
        self.addCleanup(patcher.stop)

    def write_manifest(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': gate.PINNED,
                                             'files': self.files}))

    def test_accepts_exact_original_inventory(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_refuses_changed_original_even_with_unchanged_manifest(self) -> None:
        (self.family / 'basic/vector_indexing_basic.compact').write_text('changed\n')
        with self.assertRaisesRegex(RuntimeError, 'pinned original indexing fixture changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_dropped_original_refusal(self) -> None:
        self.files.pop('test-contracts/primitives/vector/indexing/out_of_bounds/compile.fail.test.ts')
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_refusal_requires_diagnostic_and_no_published_output(self) -> None:
        destination = self.root / 'refused'
        passed = subprocess.CompletedProcess([], 255, '', 'index 10 is out-of-bounds for a vector of length 4')
        gate.refusal(passed, destination, gate.REFUSALS['out_of_bounds'])
        with self.assertRaisesRegex(RuntimeError, 'original indexing refusal changed'):
            gate.refusal(subprocess.CompletedProcess([], 0, '', passed.stderr), destination,
                         gate.REFUSALS['out_of_bounds'])
        destination.mkdir()
        with self.assertRaisesRegex(RuntimeError, 'published output'):
            gate.refusal(passed, destination, gate.REFUSALS['out_of_bounds'])


if __name__ == '__main__':
    unittest.main()
