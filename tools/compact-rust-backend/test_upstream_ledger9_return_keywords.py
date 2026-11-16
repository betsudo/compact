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

"""Focused source and refusal controls for the pinned return gate."""

import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_upstream_ledger9_return_keywords as gate  # noqa: E402


class ReturnKeywordGateControls(unittest.TestCase):
    def test_all_pinned_originals_and_diagnostics(self):
        originals, refusals = gate.original_inventory()
        self.assertEqual(len(originals), 35)
        self.assertEqual(set(refusals), set(gate.FAIL))
        self.assertEqual(len(gate.PASS) + len(gate.FAIL), 16)

    def test_missing_or_changed_manifest_entry_refuses(self):
        record = json.loads(gate.MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'manifest.json'
            record['files'].pop(next(iter(record['files'])))
            path.write_text(json.dumps(record))
            with patch.object(gate, 'MANIFEST', path):
                with self.assertRaisesRegex(Exception, 'pinned roster changed'):
                    gate.original_inventory()

    def test_proof_bearing_or_nonpure_result_refuses(self):
        valid = {'circuits':[{'name':'return_lambda_default', 'pure':True,
                              'proof':False}]}
        gate.check_capability(valid, 'lambda_default')
        for change in ({'proof':True}, {'pure':False}, {'name':'other'}):
            invalid = {'circuits':[dict(valid['circuits'][0], **change)]}
            with self.assertRaisesRegex(Exception, 'capability changed'):
                gate.check_capability(invalid, 'lambda_default')

    def test_generated_symlink_and_mutation_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact = root / 'index.js'
            artifact.write_text('export const value = 0;\n')
            snapshot = gate.generated_files(root)
            artifact.write_text('export const value = 1;\n')
            self.assertNotEqual(gate.generated_files(root), snapshot)
            artifact.write_text('export const value = 0;\n')
            (root / 'borrowed.js').symlink_to(artifact)
            with self.assertRaisesRegex(Exception, 'generated symlink'):
                gate.generated_files(root)


if __name__ == '__main__':
    unittest.main()
