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

"""Exercise actual proof-smoke build-script admission for four Bytes exports."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'tools/compact-rust-proof-smoke/build.rs'
FEATURES = ('CARGO_FEATURE_BYTES_FOLD_CONDITIONAL_PROOF',
            'CARGO_FEATURE_BYTES_FOLD_SUMMATION_PROOF',
            'CARGO_FEATURE_BYTES_LOOP_SPREAD_PROOF',
            'CARGO_FEATURE_BYTES_LOOP_ITERATION_PROOF')
GENERATED = 'COMPACT_BYTES_NEXT_REMAINING_GENERATED'
CFG = 'cargo::rustc-cfg=compact_bytes_next_remaining_proof_generated'


class RemainingBytesAdmission(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix='bytes-next-admission-')
        cls.directory = Path(cls.temp.name)
        cls.binary = cls.directory / 'build-script'
        subprocess.run(['rustc', '+1.99.0', '--edition', '2024', str(SOURCE),
                        '-o', str(cls.binary)], check=True, cwd=ROOT,
                       capture_output=True, text=True)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def run_script(self, features=(), root=None):
        env = os.environ.copy()
        for key in FEATURES + (GENERATED,):
            env.pop(key, None)
        for key in features:
            env[key] = '1'
        if root is not None:
            env[GENERATED] = str(root)
        return subprocess.run([str(self.binary)], cwd=ROOT, env=env,
                              capture_output=True, text=True)

    def complete_root(self):
        root = self.directory / f'generated-{self._testMethodName}'
        contract = root / 'contract'
        contract.mkdir(parents=True)
        for name in ('lib.rs', 'compact-rust-ir.json', 'rust-capabilities.json'):
            (contract / name).write_text('bound input\n')
        return root

    def test_no_selected_feature_skips_generated_input(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn(CFG, result.stdout)

    def test_one_feature_requires_complete_absolute_root(self):
        for feature in FEATURES:
            with self.subTest(feature=feature):
                absent = self.run_script((feature,))
                self.assertNotEqual(absent.returncode, 0)
                self.assertIn('complete remaining Bytes generated root required', absent.stderr)
        root = self.complete_root()
        result = self.run_script((FEATURES[0],), root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(CFG, result.stdout)
        (root / 'contract/rust-capabilities.json').unlink()
        incomplete = self.run_script((FEATURES[0],), root)
        self.assertNotEqual(incomplete.returncode, 0)
        self.assertIn('complete remaining Bytes generated member required', incomplete.stderr)

    def test_multiple_features_skip_without_root_and_refuse_with_root(self):
        skipped = self.run_script(FEATURES)
        self.assertEqual(skipped.returncode, 0, skipped.stderr)
        self.assertNotIn(CFG, skipped.stdout)
        refused = self.run_script(FEATURES, self.complete_root())
        self.assertNotEqual(refused.returncode, 0)
        self.assertIn('one remaining Bytes proof crate per generated runner', refused.stderr)


if __name__ == '__main__':
    unittest.main()
