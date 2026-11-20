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
        self.pinned = dict(self.files)
        self.write_manifest()
        patcher = mock.patch.object(gate, 'ROOT', self.root)
        patcher.start()
        self.addCleanup(patcher.stop)
        pin = mock.patch.object(gate, 'pinned_original_sha256',
                                side_effect=lambda name: self.pinned[name])
        pin.start()
        self.addCleanup(pin.stop)

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

    def test_refuses_forged_original_and_manifest(self) -> None:
        name = 'test-contracts/primitives/vector/loops/iteration/runtime.pass.test.ts'
        source = self.root / name
        source.write_text('forged original and assertion\n')
        self.files[name] = gate.sha256(source)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original Vector loop file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_symlinked_original(self) -> None:
        source = self.family / 'iteration/runtime.pass.test.ts'
        backup = self.root / 'backup.test.ts'
        source.rename(backup)
        source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'original Vector loop file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_wrong_pin_and_extra_entry(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': 'wrong',
                                             'files': self.files}))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)
        self.files['extra'] = '0' * 64
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_malformed_digest(self) -> None:
        name = 'test-contracts/primitives/vector/loops/iteration/runtime.pass.test.ts'
        self.files[name] = 'bad'
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original Vector loop file changed'):
            gate.original_inventory(self.manifest)

    def test_original_result_is_single_exact_assertion(self) -> None:
        source = self.family / 'iteration/runtime.pass.test.ts'
        source.write_text('expect(result).toBe(150n);\n')
        self.assertEqual(gate.original_result(source), '150')
        source.write_text('expect(result).toBe(150n);\nexpect(result).toBe(151n);\n')
        with self.assertRaisesRegex(RuntimeError, 'result assertion changed'):
            gate.original_result(source)

    def test_rust_count_requires_exact_named_cases(self) -> None:
        names = ''.join(f'test original_{name} ... ok\n' for name in gate.CASES)
        self.assertTrue(gate.rust_tests_passed(names + 'test result: ok. 3 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names + 'test result: ok. 13 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names.replace('original_iteration', 'other') +
                                                'test result: ok. 3 passed; 0 failed;'))


if __name__ == '__main__':
    unittest.main()
