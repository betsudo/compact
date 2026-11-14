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

"""Focused pinned Vector implicit-cast source and assertion controls."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_implicit_casts as gate


class ImplicitCastInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector/implicit_casts'
        self.files = {}
        for case in gate.CASES:
            for name in (f'vector_implicit_casts_{case}.compact',
                         'compile.pass.test.ts', 'runtime.pass.test.ts'):
                path = self.family / case / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f'{case} {name}\n')
                self.files[path.relative_to(self.root).as_posix()] = gate.sha256(path)
        self.pinned = dict(self.files)
        self.manifest = self.family / 'original-source-sha256.json'
        self.write_manifest()
        for name, value in [('ROOT', self.root),
                            ('pinned_sha256', lambda path: self.pinned[path])]:
            patcher = mock.patch.object(gate, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def write_manifest(self) -> None:
        self.manifest.write_text(json.dumps({'upstream_commit': gate.PINNED,
                                             'files': self.files}))

    def test_accepts_six_pinned_originals(self) -> None:
        self.assertEqual(gate.original_inventory(self.manifest), self.files)

    def test_refuses_forged_fixture_and_manifest(self) -> None:
        name = 'test-contracts/primitives/vector/implicit_casts/widening/runtime.pass.test.ts'
        source = self.root / name
        source.write_text('forged\n')
        self.files[name] = gate.sha256(source)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original Vector implicit-cast file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_symlink_and_wrong_pin(self) -> None:
        source = self.family / 'widening/runtime.pass.test.ts'
        backup = self.root / 'backup.test.ts'
        source.rename(backup)
        source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'original Vector implicit-cast file changed'):
            gate.original_inventory(self.manifest)
        source.unlink()
        backup.rename(source)
        self.manifest.write_text(json.dumps({'upstream_commit': 'wrong', 'files': self.files}))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_extra_file_or_malformed_digest(self) -> None:
        self.files['extra'] = '0' * 64
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)
        self.files.pop('extra')
        self.files['test-contracts/primitives/vector/implicit_casts/to_tuple/runtime.pass.test.ts'] = 'bad'
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original Vector implicit-cast file changed'):
            gate.original_inventory(self.manifest)

    def test_extracts_one_original_array_assertion(self) -> None:
        source = self.family / 'to_tuple/runtime.pass.test.ts'
        source.write_text('expect(Array.from(result)).toEqual([1n, 2n]);\n')
        self.assertEqual(gate.original_result(source), [1, 2])
        source.write_text('expect(Array.from(result)).toEqual([1n, 2n]);\n'
                          'expect(Array.from(result)).toEqual([3n]);\n')
        with self.assertRaisesRegex(RuntimeError, 'assertion changed'):
            gate.original_result(source)
        source.write_text('expect(Array.from(result)).toEqual([1, x]);\n')
        with self.assertRaisesRegex(RuntimeError, 'value changed'):
            gate.original_result(source)

    def test_requires_exact_named_rust_assertions(self) -> None:
        names = ''.join(f'test original_{name} ... ok\n' for name in gate.CASES)
        self.assertTrue(gate.rust_tests_passed(names + 'test result: ok. 2 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names + 'test result: ok. 12 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names.replace('original_widening', 'wrong') +
                                                'test result: ok. 2 passed; 0 failed;'))


if __name__ == '__main__':
    unittest.main()
