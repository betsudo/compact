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

"""Focused pinned source inventory controls for the Vector loop gate."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_loops as gate


class LoopInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/loops'
        self.files = {}
        for name in gate.CASES:
            for filename in (f'vector_loop_{name}.compact', 'compile.pass.test.ts',
                             'runtime.pass.test.ts'):
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

    def test_accepts_all_nine_original_files(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_refuses_changed_original_runtime_assertion(self) -> None:
        (self.family / 'iteration/runtime.pass.test.ts').write_text('changed\n')
        with self.assertRaisesRegex(RuntimeError, 'original Vector loop file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_missing_original_fixture(self) -> None:
        self.files.pop('test-contracts/primitives/vector/loops/empty_iteration/compile.pass.test.ts')
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)


if __name__ == '__main__':
    unittest.main()
