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

"""Focused pinned Vector comparison source and assertion controls."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_vector_comparison as gate


class ComparisonInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.family = self.root / 'test-contracts/primitives/vector'
        self.files = {}
        for case in gate.CASES:
            for name in (f'vector_{case}_semantics.compact',
                         'compile.pass.test.ts', 'runtime.pass.test.ts'):
                path = self.family / case / 'semantics' / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f'{case} {name}\n')
                self.files[path.relative_to(self.root).as_posix()] = gate.sha256(path)
        self.pinned = dict(self.files)
        self.manifest = self.family / 'comparison-original-source-sha256.json'
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
        name = 'test-contracts/primitives/vector/equality/semantics/runtime.pass.test.ts'
        source = self.root / name
        source.write_text('forged\n')
        self.files[name] = gate.sha256(source)
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'original Vector comparison file changed'):
            gate.original_inventory(self.manifest)

    def test_refuses_symlink_wrong_pin_and_extra_entry(self) -> None:
        source = self.family / 'inequality/semantics/runtime.pass.test.ts'
        backup = self.root / 'backup.test.ts'
        source.rename(backup)
        source.symlink_to(backup)
        with self.assertRaisesRegex(RuntimeError, 'original Vector comparison file changed'):
            gate.original_inventory(self.manifest)
        source.unlink()
        backup.rename(source)
        self.manifest.write_text(json.dumps({'upstream_commit': 'wrong', 'files': self.files}))
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)
        self.files['extra'] = '0' * 64
        self.write_manifest()
        with self.assertRaisesRegex(RuntimeError, 'inventory changed'):
            gate.original_inventory(self.manifest)

    def test_original_boolean_assertion_requires_one_match(self) -> None:
        source = self.family / 'equality/semantics/runtime.pass.test.ts'
        source.write_text('expect(result).toBe(true);\n')
        self.assertTrue(gate.original_result(source))
        source.write_text('expect(result).toBe(false);\n')
        self.assertFalse(gate.original_result(source))
        source.write_text('expect(result).toBe(true);\nexpect(result).toBe(false);\n')
        with self.assertRaisesRegex(RuntimeError, 'assertion changed'):
            gate.original_result(source)

    def test_rust_expectations_match_original_boolean_values(self) -> None:
        source = self.root / 'consumer.rs'
        source.write_text('const EQUALITY_EXPECTED: bool = true;\n'
                          'const INEQUALITY_EXPECTED: bool = true;\n')
        self.assertEqual(gate.consumer_expected(source), gate.EXPECTED)
        source.write_text(source.read_text().replace('INEQUALITY_EXPECTED: bool = true',
                                                       'INEQUALITY_EXPECTED: bool = false'))
        self.assertNotEqual(gate.consumer_expected(source), gate.EXPECTED)
        source.write_text(source.read_text().replace('INEQUALITY_EXPECTED: bool = false',
                                                       'INEQUALITY_EXPECTED: bool = 0'))
        with self.assertRaisesRegex(RuntimeError, 'Rust expected value changed'):
            gate.consumer_expected(source)

    def test_requires_exact_named_rust_assertions(self) -> None:
        names = ''.join(f'test original_{name} ... ok\n' for name in gate.CASES)
        self.assertTrue(gate.rust_tests_passed(names + 'test result: ok. 2 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names + 'test result: ok. 12 passed; 0 failed;'))
        self.assertFalse(gate.rust_tests_passed(names.replace('original_inequality', 'wrong') +
                                                'test result: ok. 2 passed; 0 failed;'))


if __name__ == '__main__':
    unittest.main()
